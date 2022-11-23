//! Network of AudioUnits connected together.

use super::audionode::*;
use super::audiounit::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;

pub type NodeIndex = usize;
pub type PortIndex = usize;

const ID: u64 = 63;

/// Input or output port.
#[derive(Clone, Copy, Debug)]
pub enum Port {
    /// Node input or output.
    Local(NodeIndex, PortIndex),
    /// Network input or output.
    Global(PortIndex),
    /// Unconnected input. Unconnected output ports are not marked anywhere.
    Zero,
}

#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub source: Port,
    pub target: Port,
}

/// Create an edge from source to target.
pub fn edge(source: Port, target: Port) -> Edge {
    Edge { source, target }
}

#[duplicate_item(
    f48       Vertex48       AudioUnit48;
    [ f64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
#[derive(Clone)]
/// Individual AudioUnits are vertices in the graph.
pub struct Vertex48 {
    /// The unit.
    pub unit: Box<dyn AudioUnit48>,
    /// Edges connecting into this vertex. The length is equal to the number of inputs.
    pub source: Vec<Edge>,
    /// Input buffers. The length is equal to the number of inputs.
    pub input: Buffer<f48>,
    /// Output buffers. The length is equal to the number of outputs.
    pub output: Buffer<f48>,
    /// Input for tick iteration. The length is equal to the number of inputs.
    pub tick_input: Vec<f48>,
    /// Output for tick iteration. The length is equal to the number of outputs.
    pub tick_output: Vec<f48>,
    /// Index or ID of this unit. This equals unit index in graph.
    pub id: NodeIndex,
    /// This is set if all vertex inputs are sourced from matching outputs of the indicated node.
    /// We can then omit copying and use the node outputs directly.
    pub source_vertex: Option<NodeIndex>,
}

#[duplicate_item(
    f48       Vertex48       AudioUnit48;
    [ f64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Vertex48 {
    pub fn new(id: NodeIndex, inputs: usize, outputs: usize) -> Self {
        Self {
            unit: Box::new(super::prelude::pass()),
            source: vec![],
            input: Buffer::with_size(inputs),
            output: Buffer::with_size(outputs),
            tick_input: vec![0.0; inputs],
            tick_output: vec![0.0; outputs],
            id,
            source_vertex: None,
        }
    }

    pub fn inputs(&self) -> usize {
        self.input.buffers()
    }

    pub fn outputs(&self) -> usize {
        self.output.buffers()
    }

    pub fn update_source_vertex(&mut self) {
        self.source_vertex = None;
        if self.inputs() == 0 {
            return;
        }
        let mut source_node = 0;
        for i in 0..self.inputs() {
            match self.source[i].source {
                Port::Local(node, port) => {
                    if port != i {
                        return;
                    }
                    if i == 0 {
                        source_node = node;
                    } else if source_node != node {
                        return;
                    }
                }
                _ => {
                    return;
                }
            }
        }
        self.source_vertex = Some(source_node);
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
/// Network unit. It can contain other units and maintain connections between them.
/// Outputs of the network are sourced from user specified unit outputs or global inputs.
pub struct Net48 {
    /// Global input buffers.
    input: Buffer<f48>,
    /// Global output buffers.
    output: Buffer<f48>,
    /// Sources of global outputs.
    output_edge: Vec<Edge>,
    /// Vertices of the graph.
    vertex: Vec<Vertex48>,
    /// Ordering of vertex evaluation.
    order: Option<Vec<NodeIndex>>,
    sample_rate: f64,
    tmp_buffer: Buffer<f48>,
    /// This cache is used by the `route` method,
    /// which does not have mutable access to the network.
    order_cache: std::sync::Mutex<Option<Vec<NodeIndex>>>,
    /// Accumulated errors.
    error_msg: Vec<String>,
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Clone for Net48 {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            output: self.output.clone(),
            output_edge: self.output_edge.clone(),
            vertex: self.vertex.clone(),
            order: None,
            sample_rate: self.sample_rate,
            tmp_buffer: self.tmp_buffer.clone(),
            order_cache: std::sync::Mutex::new(None),
            error_msg: self.error_msg.clone(),
        }
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Net48 {
    /// Create a new network with the given number of inputs and outputs.
    /// The number of inputs and outputs is fixed after construction.
    pub fn new(inputs: usize, outputs: usize) -> Self {
        let mut net = Self {
            input: Buffer::with_size(inputs),
            output: Buffer::with_size(outputs),
            output_edge: vec![],
            vertex: vec![],
            order: None,
            sample_rate: DEFAULT_SR,
            tmp_buffer: Buffer::new(),
            order_cache: std::sync::Mutex::new(None),
            error_msg: Vec::new(),
        };
        for channel in 0..outputs {
            net.output_edge
                .push(edge(Port::Zero, Port::Global(channel)));
        }
        net
    }

    /// Add a new unit to the network. Return its ID handle.
    /// ID handles are always consecutive numbers starting from zero.
    /// The unit is reset with the sample rate of the network.
    pub fn push(&mut self, mut unit: Box<dyn AudioUnit48>) -> NodeIndex {
        unit.reset(Some(self.sample_rate));
        let id = self.vertex.len();
        let inputs = unit.inputs();
        let outputs = unit.outputs();
        let mut vertex = Vertex48 {
            unit,
            source: vec![],
            input: Buffer::with_size(inputs),
            output: Buffer::with_size(outputs),
            tick_input: vec![0.0; inputs],
            tick_output: vec![0.0; outputs],
            id,
            source_vertex: None,
        };
        for i in 0..vertex.inputs() {
            vertex.source.push(edge(Port::Zero, Port::Local(id, i)));
        }
        self.vertex.push(vertex);
        // Note. We have designed the hash to depend on vertices but not edges.
        let hash = self.ping(true, AttoRand::new(ID));
        self.ping(false, hash);
        self.invalidate_order();
        id
    }

    /// Number of accumulated errors.
    pub fn errors(&self) -> usize {
        self.error_msg.len()
    }

    /// Access error message.
    pub fn error(&self, i: usize) -> &String {
        &self.error_msg[i]
    }

    /// Whether we have calculated the order vector.
    fn is_ordered(&self) -> bool {
        self.order.is_some()
    }

    /// Invalidate any precalculated order.
    fn invalidate_order(&mut self) {
        self.order = None;
        if let Ok(mut lock) = self.order_cache.lock() {
            lock.take();
        }
    }

    /// Replaces the given node in the network.
    /// The replacement must have the same number of inputs and outputs
    /// as the node it is replacing.
    pub fn replace(&mut self, node: NodeIndex, replacement: Box<dyn AudioUnit48>) {
        assert!(replacement.inputs() == self.node(node).inputs());
        assert!(replacement.outputs() == self.node(node).outputs());
        self.vertex[node].unit = replacement;
    }

    /// Connect the given unit output (`source`, `source_port`)
    /// to the given unit input (`target`, `target_port`).
    pub fn connect(
        &mut self,
        source: NodeIndex,
        source_port: PortIndex,
        target: NodeIndex,
        target_port: PortIndex,
    ) {
        self.vertex[target].source[target_port] = edge(
            Port::Local(source, source_port),
            Port::Local(target, target_port),
        );
        self.invalidate_order();
    }

    /// Connect the node input (`target`, `target_port`)
    /// to the network input `global_input`.
    pub fn connect_input(
        &mut self,
        global_input: PortIndex,
        target: NodeIndex,
        target_port: PortIndex,
    ) {
        self.vertex[target].source[target_port] =
            edge(Port::Global(global_input), Port::Local(target, target_port));
        self.invalidate_order();
    }

    /// Pipe global input to node `target`.
    /// Number of node inputs must match the number of network inputs.
    pub fn pipe_input(&mut self, target: NodeIndex) {
        assert!(self.vertex[target].inputs() == self.inputs());
        for i in 0..self.inputs() {
            self.vertex[target].source[i] = edge(Port::Global(i), Port::Local(target, i));
        }
        self.invalidate_order();
    }

    /// Connect node output (`source`, `source_port`) to network output `global_output`.
    pub fn connect_output(
        &mut self,
        source: NodeIndex,
        source_port: PortIndex,
        global_output: PortIndex,
    ) {
        self.output_edge[global_output] = edge(
            Port::Local(source, source_port),
            Port::Global(global_output),
        );
        self.invalidate_order();
    }

    /// Pipe node outputs to global outputs.
    /// Number of outputs must match the number of network outputs.
    pub fn pipe_output(&mut self, source: NodeIndex) {
        assert!(self.vertex[source].outputs() == self.outputs());
        for i in 0..self.outputs() {
            self.output_edge[i] = edge(Port::Local(source, i), Port::Global(i));
        }
        self.invalidate_order();
    }

    /// Add an arbitrary edge to the network.
    pub fn join(&mut self, edge: Edge) {
        match edge.target {
            Port::Global(global_output) => self.output_edge[global_output] = edge,
            Port::Local(target, target_port) => self.vertex[target].source[target_port] = edge,
            _ => (),
        }
        self.invalidate_order();
    }

    /// Connect `source` to `target`.
    /// The number of outputs in `source` and number of inputs in `target` must match.
    pub fn pipe(&mut self, source: NodeIndex, target: NodeIndex) {
        assert!(self.vertex[source].outputs() == self.vertex[target].inputs());
        for i in 0..self.vertex[target].inputs() {
            self.vertex[target].source[i] = edge(Port::Local(source, i), Port::Local(target, i));
        }
        self.invalidate_order();
    }

    /// Assuming this network is a chain of processing units ordered by increasing node ID,
    /// add a new unit to the chain. Global outputs will be assigned to the outputs of the unit
    /// if possible. The number of inputs to the unit must match the number of outputs of the
    /// previous unit, or the number of network inputs if there is no previous unit.
    /// Returns the ID of the new unit.
    pub fn chain(&mut self, unit: Box<dyn AudioUnit48>) -> NodeIndex {
        let unit_inputs = unit.inputs();
        let unit_outputs = unit.outputs();
        let id = self.push(unit);
        if self.outputs() == unit_outputs {
            self.pipe_output(id);
        }
        if unit_inputs > 0 {
            if id > 0 {
                self.pipe(id - 1, id);
            } else {
                self.pipe_input(id);
            }
        }
        id
    }

    /// Access node.
    pub fn node(&self, node: NodeIndex) -> &dyn AudioUnit48 {
        &*self.vertex[node].unit
    }

    /// Access mutable node.
    pub fn node_mut(&mut self, node: NodeIndex) -> &mut dyn AudioUnit48 {
        &mut *self.vertex[node].unit
    }

    /// Indicate to callback handler that time is about to elapse.
    fn elapse(&mut self, _dt: f48) {
        // TODO. Not implemented.
    }

    /// Compute and store node order for this network.
    fn determine_order(&mut self) {
        for vertex in self.vertex.iter_mut() {
            vertex.update_source_vertex();
        }
        let mut order = Vec::new();
        if !self.determine_order_in(&mut order) {
            panic!("Cycle detected");
            //self.error_msg.push(String::from("Cycle detected."));
        }
        self.order = Some(order.clone());
        if let Ok(mut lock) = self.order_cache.lock() {
            lock.replace(order);
        }
    }

    /// Determine node order in the supplied vector. Returns true if successful, false
    /// if a cycle was detected.
    fn determine_order_in(&self, order: &mut Vec<NodeIndex>) -> bool {
        let mut vertices_left = self.vertex.len();
        let mut vertex_left = vec![true; self.vertex.len()];
        // Note about contents of the edge vector.
        // Each node input appears there exactly once.
        // Sources, however, are not unique or guaranteed to appear.
        let mut all_edges: Vec<Edge> = Vec::new();
        for vertex in self.vertex.iter() {
            for edge in &vertex.source {
                all_edges.push(*edge);
            }
        }

        let mut inputs_left = vec![0; self.vertex.len()];
        for i in 0..inputs_left.len() {
            inputs_left[i] = self.vertex[i].unit.inputs();
            if inputs_left[i] == 0 {
                vertex_left[i] = false;
                order.push(i);
                vertices_left -= 1;
            }
        }

        // Start from network inputs.
        for (_, edge) in all_edges.iter().enumerate() {
            if let (Port::Global(_) | Port::Zero, Port::Local(vertex, _)) =
                (edge.source, edge.target)
            {
                if vertex_left[vertex] {
                    inputs_left[vertex] -= 1;
                    if inputs_left[vertex] == 0 {
                        vertex_left[vertex] = false;
                        order.push(vertex);
                        vertices_left -= 1;
                    }
                }
            }
        }
        while vertices_left > 0 {
            let mut progress = false;
            for (_i, edge) in all_edges.iter().enumerate() {
                if let (Port::Local(source, _), Port::Local(target, _)) = (edge.source, edge.target)
                {
                    if !vertex_left[source] && vertex_left[target] {
                        progress = true;
                        inputs_left[target] -= 1;
                        if inputs_left[target] == 0 {
                            vertex_left[target] = false;
                            order.push(target);
                            vertices_left -= 1;
                        }
                    }
                }
            }
            if !progress {
                return false;
            }
        }
        true
    }

    /// Returns an iterator over edges of the graph.
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        let mut node = 0;
        let mut port = 0;
        std::iter::from_fn(move || {
            while node < self.vertex.len() {
                if port >= self.vertex[node].inputs() {
                    node += 1;
                    port = 0;
                } else {
                    port += 1;
                    return Some(self.vertex[node].source[port - 1]);
                }
            }
            while node == self.vertex.len() {
                if port >= self.outputs() {
                    node += 1;
                    port = 0;
                } else {
                    port += 1;
                    return Some(self.output_edge[port - 1]);
                }
            }
            None
        })
    }

    /// Wrap arbitrary unit in a network.
    pub fn wrap(unit: Box<dyn AudioUnit48>) -> Net48 {
        let mut net = Net48::new(unit.inputs(), unit.outputs());
        let id = net.push(unit);
        if net.inputs() > 0 {
            net.pipe_input(id);
        }
        if net.outputs() > 0 {
            net.pipe_output(id);
        }
        net
    }

    /// Create a network that outputs a scalar value.
    pub fn scalar(channels: usize, scalar: f48) -> Net48 {
        let mut net = Net48::new(0, channels);
        let id = net.push(Box::new(super::prelude::dc(scalar)));
        for i in 0..channels {
            net.connect_output(id, 0, i);
        }
        net
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48         Callback48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ]   [ Callback64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ]   [ Callback32 ];
)]
impl AudioUnit48 for Net48 {
    fn inputs(&self) -> usize {
        self.input.buffers()
    }

    fn outputs(&self) -> usize {
        self.output.buffers()
    }

    fn reset(&mut self, sample_rate: Option<f64>) {
        if let Some(sr) = sample_rate {
            self.sample_rate = sr;
        }
        for vertex in &mut self.vertex {
            vertex.unit.reset(sample_rate);
        }
        // Take the opportunity to unload some calculations.
        if !self.is_ordered() {
            self.determine_order();
        }
    }

    fn tick(&mut self, input: &[f48], output: &mut [f48]) {
        if !self.is_ordered() {
            self.determine_order();
        }
        self.elapse(1.0 / self.sample_rate as f48);
        // Iterate units in network order.
        for &node_index in self.order.get_or_insert(Vec::new()).iter() {
            for channel in 0..self.vertex[node_index].inputs() {
                match self.vertex[node_index].source[channel].source {
                    Port::Zero => self.vertex[node_index].tick_input[channel] = 0.0,
                    Port::Global(port) => self.vertex[node_index].tick_input[channel] = input[port],
                    Port::Local(source, port) => {
                        self.vertex[node_index].tick_input[channel] =
                            self.vertex[source].tick_output[port]
                    }
                }
            }
            let vertex = &mut self.vertex[node_index];
            vertex
                .unit
                .tick(&vertex.tick_input, &mut vertex.tick_output);
        }

        // Then we set the global outputs.
        for channel in 0..output.len() {
            match self.output_edge[channel].source {
                Port::Global(port) => output[channel] = input[port],
                Port::Local(node, port) => output[channel] = self.vertex[node].tick_output[port],
                Port::Zero => output[channel] = 0.0,
            }
        }
    }

    fn process(&mut self, size: usize, input: &[&[f48]], output: &mut [&mut [f48]]) {
        if !self.is_ordered() {
            self.determine_order();
        }
        self.elapse(size as f48 / self.sample_rate as f48);
        // Iterate units in network order.
        for &node_index in self.order.get_or_insert(Vec::new()).iter() {
            if let Some(source_node) = self.vertex[node_index].source_vertex {
                // We can source inputs directly from a source vertex.
                std::mem::swap(&mut self.tmp_buffer, &mut self.vertex[source_node].output);
                let vertex = &mut self.vertex[node_index];
                vertex
                    .unit
                    .process(size, self.tmp_buffer.self_ref(), vertex.output.self_mut());
                std::mem::swap(&mut self.tmp_buffer, &mut self.vertex[source_node].output);
            } else {
                std::mem::swap(&mut self.tmp_buffer, &mut self.vertex[node_index].input);
                // Gather inputs for this vertex.
                for channel in 0..self.vertex[node_index].inputs() {
                    match self.vertex[node_index].source[channel].source {
                        Port::Zero => self.tmp_buffer.mut_at(channel)[..size].fill(0.0),
                        Port::Global(port) => self.tmp_buffer.mut_at(channel)[..size]
                            .copy_from_slice(&input[port][..size]),
                        Port::Local(source, port) => {
                            self.tmp_buffer.mut_at(channel)[..size]
                                .copy_from_slice(&self.vertex[source].output.at(port)[..size]);
                        }
                    }
                }
                let vertex = &mut self.vertex[node_index];
                vertex
                    .unit
                    .process(size, self.tmp_buffer.self_ref(), vertex.output.self_mut());
                std::mem::swap(&mut self.tmp_buffer, &mut self.vertex[node_index].input);
            }
        }

        // Then we set the global outputs.
        for channel in 0..output.len() {
            match self.output_edge[channel].source {
                Port::Global(port) => output[channel][..size].copy_from_slice(&input[port][..size]),
                Port::Local(node, port) => output[channel][..size]
                    .copy_from_slice(&self.vertex[node].output.at(port)[..size]),
                Port::Zero => output[channel][..size].fill(0.0),
            }
        }
    }

    fn get_id(&self) -> u64 {
        ID
    }

    fn set_hash(&mut self, hash: u64) {
        let mut hash = AttoRand::new(hash);
        for x in self.vertex.iter_mut() {
            x.unit.set_hash(hash.get());
        }
    }
    fn ping(&mut self, probe: bool, hash: AttoRand) -> AttoRand {
        if !probe {
            self.set_hash(hash.value());
        }
        let mut hash = hash.hash(ID);
        for x in self.vertex.iter_mut() {
            hash = x.unit.ping(probe, hash);
        }
        hash
    }

    fn route(&self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut inner_signal: Vec<SignalFrame> = vec![];
        for vertex in self.vertex.iter() {
            inner_signal.push(new_signal_frame(vertex.unit.outputs()));
        }
        let mut lock = self.order_cache.lock().unwrap();
        if lock.is_none() {
            let mut tmp_order = Vec::new();
            self.determine_order_in(&mut tmp_order);
            lock.get_or_insert(tmp_order);
        }
        for &unit_index in lock.get_or_insert(Vec::new()).iter() {
            let mut input_signal = new_signal_frame(self.vertex[unit_index].unit.inputs());
            for channel in 0..self.vertex[unit_index].unit.inputs() {
                match self.vertex[unit_index].source[channel].source {
                    Port::Local(j, port) => input_signal[channel] = inner_signal[j][port],
                    Port::Global(j) => input_signal[channel] = input[j],
                    Port::Zero => input_signal[channel] = Signal::Value(0.0),
                }
            }
            inner_signal[unit_index] = self.vertex[unit_index].unit.route(&input_signal, frequency);
        }

        // Then we set the global outputs.
        let mut output_signal = new_signal_frame(self.outputs());
        for channel in 0..self.outputs() {
            match self.output_edge[channel].source {
                Port::Global(port) => output_signal[channel] = input[port],
                Port::Local(node, port) => {
                    output_signal[channel] = inner_signal[node][port];
                }
                Port::Zero => output_signal[channel] = Signal::Value(0.0),
            }
        }
        output_signal
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Net48 {
    /// Given net A, create and return net !A.
    pub fn thru_op(mut net: Net48) -> Net48 {
        let outputs = net.outputs();
        net.output.resize(net.inputs());
        net.output_edge
            .resize(net.inputs(), edge(Port::Zero, Port::Zero));
        for i in outputs..net.inputs() {
            net.output_edge[i] = edge(Port::Global(i), Port::Global(i));
        }
        net
    }

    /// Given nets A and B, create and return net A ^ B.
    pub fn branch_op(mut net1: Net48, mut net2: Net48) -> Net48 {
        if net1.inputs() != net2.inputs() {
            panic!(
                "Branch: mismatched inputs ({} versus {}).",
                net1.inputs(),
                net2.inputs()
            );
        }
        /*
        if net1.inputs() != net2.inputs() {
            let mut error_net = Net48::new(net1.inputs(), net1.outputs() + net2.outputs());
            error_net.error_msg.append(&mut net1.error_msg);
            error_net.error_msg.append(&mut net2.error_msg);
            error_net.error_msg.push(format!(
                "Branch: mismatched inputs ({} versus {}).",
                net1.inputs(),
                net2.inputs()
            ));
            return error_net;
        }
        */
        let offset = net1.vertex.len();
        let output_offset = net1.outputs();
        let outputs = net1.outputs() + net2.outputs();
        net1.vertex.append(&mut net2.vertex);
        net1.output_edge.append(&mut net2.output_edge);
        net1.output.resize(outputs);
        for i in output_offset..net1.output_edge.len() {
            match net1.output_edge[i].source {
                Port::Local(source_node, source_port) => {
                    net1.output_edge[i] = edge(
                        Port::Local(source_node + offset, source_port),
                        Port::Global(i),
                    );
                }
                Port::Global(source_port) => {
                    net1.output_edge[i] = edge(Port::Global(source_port), Port::Global(i));
                }
                Port::Zero => {
                    net1.output_edge[i] = edge(Port::Zero, Port::Global(i));
                }
            }
        }
        for node in offset..net1.vertex.len() {
            net1.vertex[node].id = node;
            for port in 0..net1.vertex[node].inputs() {
                match net1.vertex[node].source[port].source {
                    Port::Local(source_node, source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Local(source_node + offset, source_port),
                            Port::Local(node, port),
                        );
                    }
                    Port::Global(source_port) => {
                        net1.vertex[node].source[port] =
                            edge(Port::Global(source_port), Port::Local(node, port));
                    }
                    Port::Zero => {
                        net1.vertex[node].source[port] = edge(Port::Zero, Port::Local(node, port));
                    }
                }
            }
        }
        net1
    }

    /// Given nets A and B, create and return net A | B.
    pub fn stack_op(mut net1: Net48, mut net2: Net48) -> Net48 {
        let offset = net1.vertex.len();
        let output_offset = net1.outputs();
        let input_offset = net1.inputs();
        let inputs = net1.inputs() + net2.inputs();
        let outputs = net1.outputs() + net2.outputs();
        net1.vertex.append(&mut net2.vertex);
        net1.output_edge.append(&mut net2.output_edge);
        net1.output.resize(outputs);
        net1.input.resize(inputs);
        for i in output_offset..net1.output_edge.len() {
            match net1.output_edge[i].source {
                Port::Local(source_node, source_port) => {
                    net1.output_edge[i] = edge(
                        Port::Local(source_node + offset, source_port),
                        Port::Global(i),
                    );
                }
                Port::Global(source_port) => {
                    net1.output_edge[i] =
                        edge(Port::Global(source_port + input_offset), Port::Global(i));
                }
                Port::Zero => {
                    net1.output_edge[i] = edge(Port::Zero, Port::Global(i));
                }
            }
        }
        for node in offset..net1.vertex.len() {
            net1.vertex[node].id = node;
            for port in 0..net1.vertex[node].inputs() {
                match net1.vertex[node].source[port].source {
                    Port::Local(source_node, source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Local(source_node + offset, source_port),
                            Port::Local(node, port),
                        );
                    }
                    Port::Global(source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Global(source_port + input_offset),
                            Port::Local(node, port),
                        );
                    }
                    Port::Zero => {
                        net1.vertex[node].source[port] = edge(Port::Zero, Port::Local(node, port));
                    }
                }
            }
        }
        net1.error_msg.append(&mut net2.error_msg);
        net1
    }

    /// Given nets A and B and binary operator op, create and return net A op B.
    pub fn bin_op<B: FrameBinop<super::prelude::U1, f48> + Sync + Send + 'static>(
        mut net1: Net48,
        mut net2: Net48,
        op: B,
    ) -> Net48 {
        if net1.outputs() != net2.outputs() {
            panic!(
                "Binary operation: mismatched outputs ({} versus {}).",
                net1.outputs(),
                net2.outputs()
            );
            /*
                let mut error_net = Net48::new(net1.inputs() + net2.inputs(), net1.outputs());
                error_net.error_msg.append(&mut net1.error_msg);
                error_net.error_msg.append(&mut net2.error_msg);
                error_net.error_msg.push(format!(
                    "Binary operation: mismatched outputs ({} versus {}).",
                    net1.outputs(),
                    net2.outputs()
                ));
                return error_net;
            */
        }
        let output1 = net1.output_edge.clone();
        let output2 = net2.output_edge.clone();
        let input_offset = net1.inputs();
        let inputs = net1.inputs() + net2.inputs();
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        net1.input.resize(inputs);
        for node in offset..net1.vertex.len() {
            net1.vertex[node].id = node;
            for port in 0..net1.vertex[node].inputs() {
                match net1.vertex[node].source[port].source {
                    Port::Local(source_node, source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Local(source_node + offset, source_port),
                            Port::Local(node, port),
                        );
                    }
                    Port::Global(source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Global(source_port + input_offset),
                            Port::Local(node, port),
                        );
                    }
                    Port::Zero => {
                        net1.vertex[node].source[port] = edge(Port::Zero, Port::Local(node, port));
                    }
                }
            }
        }
        let add_offset = net1.vertex.len();
        for i in 0..net1.outputs() {
            net1.push(Box::new(An(Binop::<f48, _, _, _>::new(
                Pass::<f48>::new(),
                Pass::<f48>::new(),
                op.clone(),
            ))));
            net1.connect_output(add_offset + i, 0, i);
        }
        for i in 0..output1.len() {
            match output1[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect(source_node, source_port, add_offset + i, 0);
                }
                Port::Global(source_port) => {
                    net1.connect_input(source_port, add_offset + i, 0);
                }
                _ => (),
            }
        }
        for i in 0..output2.len() {
            match output2[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect(source_node + offset, source_port, add_offset + i, 1);
                }
                Port::Global(source_port) => {
                    net1.connect_input(source_port + input_offset, add_offset + i, 1);
                }
                _ => (),
            }
        }
        net1.invalidate_order();
        net1
    }

    /// Given nets A and B, create and return net A & B.
    pub fn bus_op(mut net1: Net48, mut net2: Net48) -> Net48 {
        if net1.inputs() != net2.inputs() {
            panic!(
                "Bus: mismatched inputs ({} versus {}).",
                net1.outputs(),
                net2.outputs()
            );
        }
        if net1.outputs() != net2.outputs() {
            panic!(
                "Bus: mismatched outputs ({} versus {}).",
                net1.outputs(),
                net2.outputs()
            );
        }
        /*
        if net1.inputs() != net2.inputs() || net1.outputs() != net2.outputs() {
            let mut error_net = Net48::new(net1.inputs(), net1.outputs());
            error_net.error_msg.append(&mut net1.error_msg);
            error_net.error_msg.append(&mut net2.error_msg);
            if net1.inputs() != net2.inputs() {
                error_net.error_msg.push(format!(
                    "Bus: mismatched inputs ({} versus {}).",
                    net1.inputs(),
                    net2.inputs()
                ));
            }
            if net1.outputs() != net2.outputs() {
                error_net.error_msg.push(format!(
                    "Bus: mismatched outputs ({} versus {}).",
                    net1.outputs(),
                    net2.outputs()
                ));
            }
            return error_net;
        }
        */
        let output1 = net1.output_edge.clone();
        let output2 = net2.output_edge.clone();
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        for node in offset..net1.vertex.len() {
            net1.vertex[node].id = node;
            for port in 0..net1.vertex[node].inputs() {
                match net1.vertex[node].source[port].source {
                    Port::Local(source_node, source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Local(source_node + offset, source_port),
                            Port::Local(node, port),
                        );
                    }
                    Port::Global(source_port) => {
                        net1.vertex[node].source[port] =
                            edge(Port::Global(source_port), Port::Local(node, port));
                    }
                    Port::Zero => {
                        net1.vertex[node].source[port] = edge(Port::Zero, Port::Local(node, port));
                    }
                }
            }
        }
        let add_offset = net1.vertex.len();
        for i in 0..net1.outputs() {
            net1.push(Box::new(An(Binop::<f48, _, _, _>::new(
                Pass::<f48>::new(),
                Pass::<f48>::new(),
                FrameAdd::new(),
            ))));
            net1.connect_output(add_offset + i, 0, i);
        }
        for i in 0..output1.len() {
            match output1[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect(source_node, source_port, add_offset + i, 0);
                }
                Port::Global(source_port) => {
                    net1.connect_input(source_port, add_offset + i, 0);
                }
                _ => (),
            }
        }
        for i in 0..output2.len() {
            match output2[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect(source_node + offset, source_port, add_offset + i, 1);
                }
                Port::Global(source_port) => {
                    net1.connect_input(source_port, add_offset + i, 1);
                }
                _ => (),
            }
        }
        net1.invalidate_order();
        net1
    }

    /// Given nets A and B, create and return net A >> B.
    pub fn pipe_op(mut net1: Net48, mut net2: Net48) -> Net48 {
        if net1.outputs() != net2.inputs() {
            panic!(
                "Pipe: mismatched connectivity ({} outputs versus {} inputs).",
                net1.outputs(),
                net2.inputs()
            );
        }
        /*
        if net1.outputs() != net2.inputs() {
            let mut error_net = Net48::new(net1.inputs(), net2.outputs());
            error_net.error_msg.append(&mut net1.error_msg);
            error_net.error_msg.append(&mut net2.error_msg);
            error_net.error_msg.push(format!(
                "Pipe: mismatched connectivity ({} outputs versus {} inputs).",
                net1.outputs(),
                net2.inputs()
            ));
            return error_net;
        }
        */
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        // Adjust local ports.
        for node in offset..net1.vertex.len() {
            net1.vertex[node].id = node;
            for port in 0..net1.vertex[node].inputs() {
                match net1.vertex[node].source[port].source {
                    Port::Local(source_node, source_port) => {
                        net1.vertex[node].source[port] = edge(
                            Port::Local(source_node + offset, source_port),
                            Port::Local(node, port),
                        );
                    }
                    Port::Global(source_port) => {
                        net1.vertex[node].source[port] = edge(
                            net1.output_edge[source_port].source,
                            Port::Local(node, port),
                        );
                    }
                    Port::Zero => {
                        net1.vertex[node].source[port] = edge(Port::Zero, Port::Local(node, port));
                    }
                }
            }
        }
        // Adjust output ports.
        let output_edge1 = net1.output_edge;
        net1.output_edge = net2.output_edge;
        net1.output = net2.output;
        for output_port in 0..net1.outputs() {
            match net1.output_edge[output_port].source {
                Port::Local(source_node, source_port) => {
                    net1.output_edge[output_port] = edge(
                        Port::Local(source_node + offset, source_port),
                        Port::Global(output_port),
                    );
                }
                Port::Global(source_port) => {
                    net1.output_edge[output_port] =
                        edge(output_edge1[source_port].source, Port::Global(output_port));
                }
                _ => (),
            }
        }
        net1.invalidate_order();
        net1
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Not for Net48 {
    type Output = Net48;
    #[inline]
    fn not(self) -> Self::Output {
        Net48::thru_op(self)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Neg for Net48 {
    type Output = Net48;
    #[inline]
    fn neg(self) -> Self::Output {
        // TODO. Optimize this.
        let n = self.outputs();
        Net48::scalar(n, f48::zero()) - self
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Shr<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn shr(self, y: Net48) -> Self::Output {
        Net48::pipe_op(self, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Shr<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn shr(self, y: An<X>) -> Self::Output {
        Net48::pipe_op(self, Net48::wrap(Box::new(y)))
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Shr<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn shr(self, y: Net48) -> Self::Output {
        Net48::pipe_op(Net48::wrap(Box::new(self)), y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::BitAnd<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn bitand(self, y: Net48) -> Self::Output {
        Net48::bus_op(self, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitAnd<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitand(self, y: An<X>) -> Self::Output {
        Net48::bus_op(self, Net48::wrap(Box::new(y)))
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitAnd<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitand(self, y: Net48) -> Self::Output {
        Net48::bus_op(Net48::wrap(Box::new(self)), y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::BitOr<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn bitor(self, y: Net48) -> Self::Output {
        Net48::stack_op(self, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitOr<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitor(self, y: An<X>) -> Self::Output {
        Net48::stack_op(self, Net48::wrap(Box::new(y)))
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitOr<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitor(self, y: Net48) -> Self::Output {
        Net48::stack_op(Net48::wrap(Box::new(self)), y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::BitXor<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn bitxor(self, y: Net48) -> Self::Output {
        Net48::branch_op(self, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitXor<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitxor(self, y: An<X>) -> Self::Output {
        Net48::branch_op(self, Net48::wrap(Box::new(y)))
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::BitXor<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn bitxor(self, y: Net48) -> Self::Output {
        Net48::branch_op(Net48::wrap(Box::new(self)), y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Add<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn add(self, y: Net48) -> Self::Output {
        Net48::bin_op(self, y, FrameAdd::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Add<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn add(self, y: An<X>) -> Self::Output {
        Net48::bin_op(self, Net48::wrap(Box::new(y)), FrameAdd::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Add<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn add(self, y: Net48) -> Self::Output {
        Net48::bin_op(Net48::wrap(Box::new(self)), y, FrameAdd::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Sub<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn sub(self, y: Net48) -> Self::Output {
        Net48::bin_op(self, y, FrameSub::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Sub<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn sub(self, y: An<X>) -> Self::Output {
        Net48::bin_op(self, Net48::wrap(Box::new(y)), FrameSub::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Sub<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn sub(self, y: Net48) -> Self::Output {
        Net48::bin_op(Net48::wrap(Box::new(self)), y, FrameSub::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Mul<Net48> for Net48 {
    type Output = Net48;
    #[inline]
    fn mul(self, y: Net48) -> Self::Output {
        Net48::bin_op(self, y, FrameMul::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Mul<An<X>> for Net48
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn mul(self, y: An<X>) -> Self::Output {
        Net48::bin_op(self, Net48::wrap(Box::new(y)), FrameMul::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl<X> std::ops::Mul<Net48> for An<X>
where
    X: AudioNode<Sample = f48> + std::marker::Send + Sync + 'static,
{
    type Output = Net48;
    #[inline]
    fn mul(self, y: Net48) -> Self::Output {
        Net48::bin_op(Net48::wrap(Box::new(self)), y, FrameMul::new())
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Add<f48> for Net48 {
    type Output = Net48;
    #[inline]
    fn add(self, y: f48) -> Self::Output {
        let n = self.outputs();
        self + Net48::scalar(n, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Add<Net48> for f48 {
    type Output = Net48;
    #[inline]
    fn add(self, y: Net48) -> Self::Output {
        let n = y.outputs();
        Net48::scalar(n, self) + y
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Sub<f48> for Net48 {
    type Output = Net48;
    #[inline]
    fn sub(self, y: f48) -> Self::Output {
        let n = self.outputs();
        self - Net48::scalar(n, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Sub<Net48> for f48 {
    type Output = Net48;
    #[inline]
    fn sub(self, y: Net48) -> Self::Output {
        let n = y.outputs();
        Net48::scalar(n, self) - y
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Mul<f48> for Net48 {
    type Output = Net48;
    #[inline]
    fn mul(self, y: f48) -> Self::Output {
        let n = self.outputs();
        self * Net48::scalar(n, y)
    }
}

#[duplicate_item(
    f48       Net48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ AudioUnit32 ];
)]
impl std::ops::Mul<Net48> for f48 {
    type Output = Net48;
    #[inline]
    fn mul(self, y: Net48) -> Self::Output {
        let n = y.outputs();
        Net48::scalar(n, self) * y
    }
}
