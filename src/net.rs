//! Network of audio units connected together.

use super::audionode::*;
use super::audiounit::*;
use super::buffer::*;
use super::combinator::*;
use super::math::*;
use super::realnet::*;
use super::signal::*;
use super::*;
use duplicate::duplicate_item;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thingbuf::mpsc::blocking::{channel, Receiver, Sender};

pub type NodeIndex = usize;
pub type PortIndex = usize;

/// Globally unique node ID for a node in a network.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct NodeId(u64);

/// This atomic supplies globally unique IDs.
static GLOBAL_NODE_ID: AtomicU64 = AtomicU64::new(0);

impl NodeId {
    /// Create a new, globally unique node ID.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        NodeId(GLOBAL_NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

const ID: u64 = 63;

/// Input or output port.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Port {
    /// Node input or output.
    Local(NodeIndex, PortIndex),
    /// Network input or output.
    Global(PortIndex),
    /// Unconnected input. Unconnected output ports are not marked anywhere.
    #[default]
    Zero,
}

#[derive(Clone, Copy, Debug, Default)]
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
struct Vertex48 {
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
    /// Stable, globally unique ID for this vertex.
    pub id: NodeId,
    /// This is set if all vertex inputs are sourced from matching outputs of the indicated node.
    /// We can then omit copying and use the node outputs directly.
    pub source_vertex: Option<NodeIndex>,
    /// Network revision in which this vertex was changed last.
    pub changed: u64,
}

#[duplicate_item(
    f48       Vertex48       AudioUnit48;
    [ f64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Vertex48 {
    pub fn new(id: NodeId, index: NodeIndex, unit: Box<dyn AudioUnit48>) -> Self {
        let inputs = unit.inputs();
        let outputs = unit.outputs();
        let mut vertex = Self {
            unit,
            source: vec![],
            input: Buffer::with_channels(inputs),
            output: Buffer::with_channels(outputs),
            tick_input: vec![0.0; inputs],
            tick_output: vec![0.0; outputs],
            id,
            source_vertex: None,
            changed: 0,
        };
        for i in 0..vertex.inputs() {
            vertex.source.push(edge(Port::Zero, Port::Local(index, i)));
        }
        vertex
    }

    /// Number of input channels.
    #[inline]
    pub fn inputs(&self) -> usize {
        self.tick_input.len()
    }

    /// Number of output channels.
    #[inline]
    pub fn outputs(&self) -> usize {
        self.tick_output.len()
    }

    /// Update source vertex shortcut.
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

    /// Preallocate everything.
    pub fn allocate(&mut self) {
        self.unit.allocate();
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
/// Network unit. It can contain other units and maintain connections between them.
/// Outputs of the network are sourced from user specified unit outputs or global inputs.
#[derive(Default)]
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
    /// Translation map from node ID to vertex index.
    node_index: HashMap<NodeId, NodeIndex>,
    /// Current sample rate.
    sample_rate: f64,
    /// Optional frontend.
    front: Option<(Sender<Net48>, Receiver<Net48>)>,
    /// Number of inputs in the backend. This is for checking consistency during commits.
    backend_inputs: usize,
    /// Number of outputs in the backend. This is for checking consistency during commits.
    backend_outputs: usize,
    /// Revision number. This is used by frontends and backends only.
    /// The revision is incremented after each commit.
    revision: u64,
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
            order: self.order.clone(),
            node_index: self.node_index.clone(),
            sample_rate: self.sample_rate,
            // Frontend is never cloned.
            front: None,
            backend_inputs: self.backend_inputs,
            backend_outputs: self.backend_outputs,
            revision: self.revision,
        }
    }
}

#[duplicate_item(
    f48       Net48       NetBackend48       Vertex48       AudioUnit48;
    [ f64 ]   [ Net64 ]   [ NetBackend64 ]   [ Vertex64 ]   [ AudioUnit64 ];
    [ f32 ]   [ Net32 ]   [ NetBackend32 ]   [ Vertex32 ]   [ AudioUnit32 ];
)]
impl Net48 {
    /// Create a new network with the given number of inputs and outputs.
    /// The number of inputs and outputs is fixed after construction.
    /// Network global outputs are initialized to zero.
    ///
    /// ### Example (Sine Oscillator)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// net.chain(Box::new(sine()));
    /// net.check();
    /// ```
    pub fn new(inputs: usize, outputs: usize) -> Self {
        let mut net = Self {
            input: Buffer::with_channels(inputs),
            output: Buffer::with_channels(outputs),
            output_edge: vec![],
            vertex: vec![],
            order: None,
            node_index: HashMap::new(),
            sample_rate: DEFAULT_SR,
            front: None,
            backend_inputs: inputs,
            backend_outputs: outputs,
            revision: 0,
        };
        for channel in 0..outputs {
            net.output_edge
                .push(edge(Port::Zero, Port::Global(channel)));
        }
        net
    }

    /// Add a new unit to the network. Return its ID handle.
    /// Unit inputs are initially set to zero.
    ///
    /// ### Example (Sine Oscillator)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id = net.push(Box::new(sine()));
    /// net.pipe_input(id);
    /// net.pipe_output(id);
    /// net.check();
    /// ```
    pub fn push(&mut self, mut unit: Box<dyn AudioUnit48>) -> NodeId {
        unit.set_sample_rate(self.sample_rate);
        let index = self.vertex.len();
        let id = NodeId::new();
        let vertex = Vertex48::new(id, index, unit);
        self.vertex.push(vertex);
        self.node_index.insert(id, index);
        // Note. We have designed the hash to depend on vertices but not edges.
        let hash = self.ping(true, AttoHash::new(ID));
        self.ping(false, hash);
        self.invalidate_order();
        id
    }

    /// Whether we have calculated the order vector.
    fn is_ordered(&self) -> bool {
        self.order.is_some()
    }

    /// Invalidate any precalculated order.
    fn invalidate_order(&mut self) {
        self.order = None;
    }

    /// Remove `node` from network. Returns the unit that was removed.
    /// All connections from the unit are replaced with zeros.
    ///
    /// ### Example (Sine Oscillator)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id1 = net.push(Box::new(sine()));
    /// let id2 = net.push(Box::new(sine()));
    /// net.connect_input(0, id2, 0);
    /// net.connect_output(id2, 0, 0);
    /// net.remove(id1);
    /// assert!(net.size() == 1);
    /// net.check();
    /// ```
    pub fn remove(&mut self, node: NodeId) -> Box<dyn AudioUnit48> {
        self.remove_2(node, false)
    }

    /// Remove `node` from network. Returns the unit that was removed.
    /// Connections from the unit are replaced with pass-through connections.
    /// The unit must have an equal number of inputs and outputs.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id1 = net.chain(Box::new(add(1.0)));
    /// let id2 = net.chain(Box::new(add(2.0)));
    /// assert!(net.size() == 2);
    /// assert!(net.filter_mono(1.0) == 4.0);
    /// net.remove_link(id2);
    /// assert!(net.size() == 1);
    /// assert!(net.filter_mono(1.0) == 2.0);
    /// net.check();
    /// ```
    pub fn remove_link(&mut self, node: NodeId) -> Box<dyn AudioUnit48> {
        self.remove_2(node, true)
    }

    /// Remove `node` from network. If `link` is false then connections from the unit
    /// are replaced with zeros; if `link` is true then connections are replaced
    /// by matching inputs of the unit, and the number of inputs must be equal to the number of outputs.
    fn remove_2(&mut self, node: NodeId, link: bool) -> Box<dyn AudioUnit48> {
        let node_index = self.node_index[&node];
        assert!(!link || self.vertex[node_index].inputs() == self.vertex[node_index].outputs());
        // Replace all global ports that use an output of the node.
        for channel in 0..self.outputs() {
            if let Port::Local(index, port) = self.output_edge[channel].source {
                if index == node_index {
                    self.output_edge[channel].source = if link {
                        self.vertex[node_index].source[port].source
                    } else {
                        Port::Zero
                    };
                }
            }
        }
        // Replace all local ports that use an output of the node.
        for vertex in 0..self.size() {
            for channel in 0..self.vertex[vertex].inputs() {
                if let Port::Local(index, port) = self.vertex[vertex].source[channel].source {
                    if index == node_index {
                        self.vertex[vertex].source[channel].source = if link {
                            self.vertex[node_index].source[port].source
                        } else {
                            Port::Zero
                        };
                    }
                }
            }
        }
        self.node_index.remove(&self.vertex[node_index].id);
        let last_index = self.size() - 1;
        if last_index != node_index {
            // Move node from `last_index` to `node_index`.
            self.vertex.swap(node_index, last_index);
            self.node_index
                .insert(self.vertex[node_index].id, node_index);
            for channel in 0..self.outputs() {
                if let Port::Local(index, port) = self.output_edge[channel].source {
                    if index == last_index {
                        self.output_edge[channel].source = Port::Local(node_index, port);
                    }
                }
            }
            for vertex in 0..self.size() - 1 {
                for channel in 0..self.vertex[vertex].inputs() {
                    if let Port::Local(index, port) = self.vertex[vertex].source[channel].source {
                        if index == last_index {
                            self.vertex[vertex].source[channel].source =
                                Port::Local(node_index, port);
                        }
                    }
                }
            }
            for channel in 0..self.vertex[node_index].inputs() {
                self.vertex[node_index].source[channel].target = Port::Local(node_index, channel);
            }
        }
        self.invalidate_order();

        self.vertex.pop().unwrap().unit
    }

    /// Replaces the given node in the network.
    /// All connections are retained.
    /// The replacement must have the same number of inputs and outputs
    /// as the node it is replacing.
    /// Returns the unit that was replaced.
    ///
    /// ### Example (Replace Saw Wave With Square Wave)
    /// ```
    /// use fundsp::hacker32::*;
    /// let mut net = Net32::new(0, 1);
    /// let id = net.push(Box::new(saw_hz(220.0)));
    /// net.pipe_output(id);
    /// net.replace(id, Box::new(square_hz(220.0)));
    /// net.check();
    /// ```
    pub fn replace(
        &mut self,
        node: NodeId,
        mut unit: Box<dyn AudioUnit48>,
    ) -> Box<dyn AudioUnit48> {
        let node_index = self.node_index[&node];
        assert_eq!(unit.inputs(), self.vertex[node_index].inputs());
        assert_eq!(unit.outputs(), self.vertex[node_index].outputs());
        std::mem::swap(&mut self.vertex[node_index].unit, &mut unit);
        self.vertex[node_index].changed = self.revision;
        unit
    }

    /// Connect the given unit output (`source`, `source_port`)
    /// to the given unit input (`target`, `target_port`).
    /// There is one connection for each unit input.
    ///
    /// ### Example (Filtered Saw Oscillator)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id1 = net.push(Box::new(saw()));
    /// let id2 = net.push(Box::new(lowpass_hz(1000.0, 1.0)));
    /// net.connect(id1, 0, id2, 0);
    /// net.pipe_input(id1);
    /// net.pipe_output(id2);
    /// net.check();
    /// ```
    pub fn connect(
        &mut self,
        source: NodeId,
        source_port: PortIndex,
        target: NodeId,
        target_port: PortIndex,
    ) {
        assert!(source != target);
        let source_index = self.node_index[&source];
        let target_index = self.node_index[&target];
        self.connect_index(source_index, source_port, target_index, target_port);
    }

    /// Disconnect `node` input `port`, replacing it with zero input.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id = net.chain(Box::new(pass()));
    /// assert!(net.filter_mono(1.0) == 1.0);
    /// net.disconnect(id, 0);
    /// assert!(net.filter_mono(1.0) == 0.0);
    /// net.check();
    /// ```
    pub fn disconnect(&mut self, node: NodeId, port: PortIndex) {
        let node_index = self.node_index[&node];
        self.vertex[node_index].source[port].source = Port::Zero;
        self.invalidate_order();
    }

    /// Connect the given unit output (`source`, `source_port`)
    /// to the given unit input (`target`, `target_port`).
    fn connect_index(
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
    ///
    /// ### Example (Saw Wave)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(1, 1);
    /// let id = net.push(Box::new(saw()));
    /// net.connect_input(0, id, 0);
    /// net.connect_output(id, 0, 0);
    /// net.check();
    /// ```
    pub fn connect_input(
        &mut self,
        global_input: PortIndex,
        target: NodeId,
        target_port: PortIndex,
    ) {
        let target_index = self.node_index[&target];
        self.connect_input_index(global_input, target_index, target_port);
    }

    /// Connect the node input (`target`, `target_port`)
    /// to the network input `global_input`.
    fn connect_input_index(
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
    ///
    /// ### Example (Stereo Filter)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(2, 2);
    /// let id = net.push(Box::new(peak_hz(1000.0, 1.0) | peak_hz(1000.0, 1.0)));
    /// net.pipe_input(id);
    /// net.pipe_output(id);
    /// net.check();
    /// ```
    pub fn pipe_input(&mut self, target: NodeId) {
        let target_index = self.node_index[&target];
        assert_eq!(self.vertex[target_index].inputs(), self.inputs());
        for i in 0..self.inputs() {
            self.vertex[target_index].source[i] =
                edge(Port::Global(i), Port::Local(target_index, i));
        }
        self.invalidate_order();
    }

    /// Connect node output (`source`, `source_port`) to network output `global_output`.
    /// There is one connection for each global output.
    pub fn connect_output(
        &mut self,
        source: NodeId,
        source_port: PortIndex,
        global_output: PortIndex,
    ) {
        let source_index = self.node_index[&source];
        self.connect_output_index(source_index, source_port, global_output);
    }

    /// Disconnect global `output`. Replaces output with zero signal.
    pub fn disconnect_output(&mut self, output: PortIndex) {
        self.output_edge[output] = edge(Port::Zero, Port::Global(output));
        self.invalidate_order();
    }

    /// Connect node output (`source`, `source_port`) to network output `global_output`.
    fn connect_output_index(
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
    ///
    /// ### Example (Stereo Reverb)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(2, 2);
    /// let id = net.push(Box::new(multipass() & reverb_stereo(10.0, 1.0, 0.5)));
    /// net.pipe_input(id);
    /// net.pipe_output(id);
    /// net.check();
    /// ```
    pub fn pipe_output(&mut self, source: NodeId) {
        let source_index = self.node_index[&source];
        assert!(self.vertex[source_index].outputs() == self.outputs());
        for channel in 0..self.outputs() {
            self.output_edge[channel] =
                edge(Port::Local(source_index, channel), Port::Global(channel));
        }
        self.invalidate_order();
    }

    /// Pass through global `input` to global `output`.
    ///
    /// ### Example (Stereo Pass-Through)
    /// ```
    /// use fundsp::hacker32::*;
    /// let mut net = Net32::new(2, 2);
    /// net.pass_through(0, 0);
    /// net.pass_through(1, 1);
    /// net.check();
    /// ```
    pub fn pass_through(&mut self, input: PortIndex, output: PortIndex) {
        self.output_edge[output] = edge(Port::Global(input), Port::Global(output));
        self.invalidate_order();
    }

    /// Connect `source` node outputs to `target` node inputs.
    /// The number of outputs in `source` and number of inputs in `target` must match.
    ///
    /// ### Example (Panned Sine Wave)
    /// ```
    /// use fundsp::hacker32::*;
    /// let mut net = Net32::new(0, 2);
    /// let id1 = net.push(Box::new(sine_hz(440.0)));
    /// let id2 = net.push(Box::new(pan(0.0)));
    /// net.pipe(id1, id2);
    /// net.pipe_output(id2);
    /// net.check();
    /// ```
    pub fn pipe(&mut self, source: NodeId, target: NodeId) {
        let source_index = self.node_index[&source];
        let target_index = self.node_index[&target];
        assert_eq!(
            self.vertex[source_index].outputs(),
            self.vertex[target_index].inputs()
        );
        for channel in 0..self.vertex[target_index].inputs() {
            self.vertex[target_index].source[channel] = edge(
                Port::Local(source_index, channel),
                Port::Local(target_index, channel),
            );
        }
        self.invalidate_order();
    }

    /// Number of nodes in the network.
    pub fn size(&self) -> usize {
        self.vertex.len()
    }

    /// Assuming this network is a chain of processing units ordered by insertion order,
    /// add a new unit to the chain. Global outputs will be assigned to the outputs of the unit
    /// if possible. The number of inputs to the unit must match the number of outputs of the
    /// previous unit, or the number of network inputs if there is no previous unit.
    /// Returns the ID of the new unit.
    ///
    /// ### Example (Lowpass And Highpass Filters In Series)
    /// ```
    /// use fundsp::hacker32::*;
    /// let mut net = Net32::new(1, 1);
    /// net.chain(Box::new(lowpass_hz(2000.0, 1.0)));
    /// net.chain(Box::new(highpass_hz(1000.0, 1.0)));
    /// net.check();
    /// ```
    pub fn chain(&mut self, unit: Box<dyn AudioUnit48>) -> NodeId {
        let unit_inputs = unit.inputs();
        let unit_outputs = unit.outputs();
        let id = self.push(unit);
        let index = self.node_index[&id];
        if self.outputs() == unit_outputs {
            self.pipe_output(id);
        }
        if unit_inputs > 0 {
            if self.size() > 1 {
                self.pipe(self.vertex[index - 1].id, id);
            } else {
                self.pipe_input(id);
            }
        }
        id
    }

    /// Access node.
    pub fn node(&self, node: NodeId) -> &dyn AudioUnit48 {
        &*self.vertex[self.node_index[&node]].unit
    }

    /// Access mutable node. Note that any changes made via this method
    /// are not accounted in the backend. This can be used to, e.g.,
    /// query for frequency responses.
    pub fn node_mut(&mut self, node: NodeId) -> &mut dyn AudioUnit48 {
        &mut *self.vertex[self.node_index[&node]].unit
    }

    /// Compute and store node order for this network.
    fn determine_order(&mut self) {
        for vertex in self.vertex.iter_mut() {
            vertex.update_source_vertex();
        }
        let mut order = Vec::new();
        if !self.determine_order_in(&mut order) {
            panic!("Cycle detected");
        }
        self.order = Some(order);
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
        for edge in all_edges.iter() {
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
            for edge in all_edges.iter() {
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

    /// Wrap arbitrary unit in a network.
    ///
    /// ### Example (Conditional Processing)
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::wrap(Box::new(square_hz(440.0)));
    /// let add_filter = true;
    /// if add_filter {
    ///     net = net >> lowpass_hz(880.0, 1.0);
    /// }
    /// ```
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

    /// Create a network that outputs a scalar value on all channels.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker32::*;
    /// let mut net = Net32::scalar(2, 1.0);
    /// assert!(net.get_stereo() == (1.0, 1.0));
    /// ```
    pub fn scalar(channels: usize, scalar: f48) -> Net48 {
        let mut net = Net48::new(0, channels);
        let id = net.push(Box::new(super::prelude::dc(scalar)));
        for i in 0..channels {
            net.connect_output(id, 0, i);
        }
        net
    }

    /// Check internal consistency of the network. Panic if something is wrong.
    pub fn check(&self) {
        assert_eq!(self.input.channels(), self.inputs());
        assert_eq!(self.output.channels(), self.outputs());
        assert_eq!(self.output_edge.len(), self.outputs());
        assert_eq!(self.node_index.len(), self.size());
        for channel in 0..self.outputs() {
            assert_eq!(self.output_edge[channel].target, Port::Global(channel));
            match self.output_edge[channel].source {
                Port::Local(node, port) => {
                    assert!(node < self.size());
                    assert!(port < self.vertex[node].outputs());
                }
                Port::Global(port) => {
                    assert!(port < self.inputs());
                }
                _ => (),
            }
        }
        for index in 0..self.size() {
            assert_eq!(self.node_index[&self.vertex[index].id], index);
            assert_eq!(self.vertex[index].source.len(), self.vertex[index].inputs());
            assert_eq!(
                self.vertex[index].input.channels(),
                self.vertex[index].inputs()
            );
            assert_eq!(
                self.vertex[index].output.channels(),
                self.vertex[index].outputs()
            );
            assert_eq!(
                self.vertex[index].tick_input.len(),
                self.vertex[index].inputs()
            );
            assert_eq!(
                self.vertex[index].tick_output.len(),
                self.vertex[index].outputs()
            );
            for channel in 0..self.vertex[index].inputs() {
                assert_eq!(
                    self.vertex[index].source[channel].target,
                    Port::Local(index, channel)
                );
                match self.vertex[index].source[channel].source {
                    Port::Local(node, port) => {
                        assert!(node < self.size());
                        assert!(node != index);
                        assert!(port < self.vertex[node].outputs());
                    }
                    Port::Global(port) => {
                        assert!(port < self.inputs());
                    }
                    _ => (),
                }
            }
        }
    }

    /// Disambiguate IDs in this network so they don't conflict with those in `other` network.
    /// Conflict is possible as a result of cloning and recombination.
    fn disambiguate_ids(&mut self, other: &Net48) {
        for i in 0..self.size() {
            let id = self.vertex[i].id;
            if other.node_index.contains_key(&id) {
                self.node_index.remove(&id);
                let new_id = NodeId::new();
                self.vertex[i].id = new_id;
                self.node_index.insert(new_id, i);
            }
        }
    }

    /// Migrate existing units to the new network. This is an internal function.
    pub(crate) fn migrate(&mut self, new: &mut Net48) {
        for (id, &index) in self.node_index.iter() {
            if let Some(&new_index) = new.node_index.get(id) {
                // We may use the existing unit if no changes have been made since our last update.
                if new.vertex[new_index].changed <= self.revision {
                    std::mem::swap(
                        &mut self.vertex[index].unit,
                        &mut new.vertex[new_index].unit,
                    );
                }
            }
        }
    }

    /// Create a real-time friendly backend for this network.
    /// This network is then the frontend and any changes made can be committed to the backend.
    /// The backend is initialized with the current state of the network.
    /// This can be called only once for a network.
    ///
    /// ### Example
    /// ```
    /// use fundsp::hacker::*;
    /// let mut net = Net64::new(0, 1);
    /// net.chain(Box::new(dc(1.0)));
    /// let mut backend = net.backend();
    /// net.chain(Box::new(mul(2.0)));
    /// assert!(backend.get_mono() == 1.0);
    /// net.commit();
    /// assert!(backend.get_mono() == 2.0);
    /// ```
    pub fn backend(&mut self) -> NetBackend48 {
        assert!(!self.has_backend());
        // Create huge channel buffers to make sure we don't run out of space easily.
        let (sender_a, receiver_a) = channel(1024);
        let (sender_b, receiver_b) = channel(1024);
        self.front = Some((sender_a, receiver_b));
        self.backend_inputs = self.inputs();
        self.backend_outputs = self.outputs();
        if !self.is_ordered() {
            self.determine_order();
        }
        let mut net = self.clone();
        // Send over the original nodes to the backend.
        // This is necessary if the nodes contain any backends, which cannot be cloned effectively.
        std::mem::swap(&mut net.vertex, &mut self.vertex);
        net.allocate();
        self.revision += 1;
        NetBackend48::new(sender_b, receiver_a, net)
    }

    /// Returns whether this network has a backend.
    pub fn has_backend(&self) -> bool {
        self.front.is_some()
    }

    /// Commit changes made to this frontend to the backend.
    /// This may be called only if the network has a backend.
    pub fn commit(&mut self) {
        assert!(self.has_backend());
        if self.inputs() != self.backend_inputs {
            panic!("The number of inputs has changed since last commit. The number of inputs must stay the same.");
        }
        if self.outputs() != self.backend_outputs {
            panic!("The number of outputs has changed since last commit. The number of outputs must stay the same.");
        }
        if !self.is_ordered() {
            self.determine_order();
        }
        let mut net = self.clone();
        // Send over the original nodes to the backend.
        // This is necessary if the nodes contain any backends, which cannot be cloned effectively.
        std::mem::swap(&mut net.vertex, &mut self.vertex);
        // Preallocate all necessary memory.
        net.allocate();
        if let Some((sender, receiver)) = &mut self.front {
            // Deallocate all previous versions.
            while receiver.try_recv().is_ok() {}
            // Send the new version over.
            if sender.try_send(net).is_ok() {}
        }
        self.revision += 1;
    }

    /// Resolve new frontend for a binary combination.
    fn resolve_frontend(&mut self, other: &mut Net48) {
        if self.has_backend() && other.has_backend() {
            panic!("Cannot combine two frontends.");
        }
        if other.has_backend() {
            std::mem::swap(&mut self.front, &mut other.front);
            self.backend_inputs = other.backend_inputs;
            self.backend_outputs = other.backend_outputs;
            self.revision = other.revision;
        }
    }
}

#[duplicate_item(
    f48       Net48       Vertex48       AudioUnit48         Callback48;
    [ f64 ]   [ Net64 ]   [ Vertex64 ]   [ AudioUnit64 ]   [ Callback64 ];
    [ f32 ]   [ Net32 ]   [ Vertex32 ]   [ AudioUnit32 ]   [ Callback32 ];
)]
impl AudioUnit48 for Net48 {
    fn inputs(&self) -> usize {
        self.input.channels()
    }

    fn outputs(&self) -> usize {
        self.output.channels()
    }

    fn set_sample_rate(&mut self, sample_rate: f64) {
        self.sample_rate = sample_rate;
        for vertex in &mut self.vertex {
            vertex.unit.set_sample_rate(sample_rate);
        }
        // Take the opportunity to unload some calculations.
        if !self.is_ordered() {
            self.determine_order();
        }
    }

    fn reset(&mut self) {
        for vertex in &mut self.vertex {
            vertex.unit.reset();
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
        // Iterate units in network order.
        for &node_index in self.order.get_or_insert(Vec::new()).iter() {
            if let Some(source_node) = self.vertex[node_index].source_vertex {
                // We can source inputs directly from a source vertex.
                let ptr = &mut self.vertex[source_node].output as *mut Buffer<f48>;
                let vertex = &mut self.vertex[node_index];
                // Safety: we know there is no aliasing, as self connections are prohibited.
                unsafe {
                    vertex
                        .unit
                        .process(size, (*ptr).self_ref(), vertex.output.self_mut());
                }
            } else {
                let ptr = &mut self.vertex[node_index].input as *mut Buffer<f48>;
                // Gather inputs for this vertex.
                for channel in 0..self.vertex[node_index].inputs() {
                    // Safety: we know there is no aliasing, as self connections are prohibited.
                    unsafe {
                        match self.vertex[node_index].source[channel].source {
                            Port::Zero => (*ptr).mut_at(channel)[..size].fill(0.0),
                            Port::Global(port) => {
                                (*ptr).mut_at(channel)[..size].copy_from_slice(&input[port][..size])
                            }
                            Port::Local(source, port) => {
                                (*ptr).mut_at(channel)[..size]
                                    .copy_from_slice(&self.vertex[source].output.at(port)[..size]);
                            }
                        }
                    }
                }
                let vertex = &mut self.vertex[node_index];
                // Safety: we know there is no aliasing, as self connections are prohibited.
                unsafe {
                    vertex
                        .unit
                        .process(size, (*ptr).self_ref(), vertex.output.self_mut());
                }
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

    fn ping(&mut self, probe: bool, hash: AttoHash) -> AttoHash {
        let mut hash = hash.hash(ID);
        for x in self.vertex.iter_mut() {
            hash = x.unit.ping(probe, hash);
        }
        hash
    }

    fn route(&mut self, input: &SignalFrame, frequency: f64) -> SignalFrame {
        let mut inner_signal: Vec<SignalFrame> = vec![];
        for vertex in self.vertex.iter() {
            inner_signal.push(new_signal_frame(vertex.unit.outputs()));
        }
        if !self.is_ordered() {
            self.determine_order();
        }
        for &unit_index in self.order.as_mut().unwrap().iter() {
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

    fn footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    fn allocate(&mut self) {
        if !self.is_ordered() {
            self.determine_order();
        }
        for vertex in self.vertex.iter_mut() {
            vertex.allocate();
        }
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
        net2.disambiguate_ids(&net1);
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
            net1.node_index.insert(net1.vertex[node].id, node);
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
        net1.invalidate_order();
        net1.resolve_frontend(&mut net2);
        net1
    }

    /// Given nets A and B, create and return net A | B.
    pub fn stack_op(mut net1: Net48, mut net2: Net48) -> Net48 {
        net2.disambiguate_ids(&net1);
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
            net1.node_index.insert(net1.vertex[node].id, node);
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
        net1.invalidate_order();
        net1.resolve_frontend(&mut net2);
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
        }
        net2.disambiguate_ids(&net1);
        let output1 = net1.output_edge.clone();
        let output2 = net2.output_edge.clone();
        let input_offset = net1.inputs();
        let inputs = net1.inputs() + net2.inputs();
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        net1.input.resize(inputs);
        for node in offset..net1.vertex.len() {
            net1.node_index.insert(net1.vertex[node].id, node);
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
            net1.connect_output_index(add_offset + i, 0, i);
        }
        for i in 0..output1.len() {
            match output1[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect_index(source_node, source_port, add_offset + i, 0);
                }
                Port::Global(source_port) => {
                    net1.connect_input_index(source_port, add_offset + i, 0);
                }
                _ => (),
            }
        }
        for i in 0..output2.len() {
            match output2[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect_index(source_node + offset, source_port, add_offset + i, 1);
                }
                Port::Global(source_port) => {
                    net1.connect_input_index(source_port + input_offset, add_offset + i, 1);
                }
                _ => (),
            }
        }
        net1.invalidate_order();
        net1.resolve_frontend(&mut net2);
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
        net2.disambiguate_ids(&net1);
        let output1 = net1.output_edge.clone();
        let output2 = net2.output_edge.clone();
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        for node in offset..net1.vertex.len() {
            net1.node_index.insert(net1.vertex[node].id, node);
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
            net1.connect_output_index(add_offset + i, 0, i);
        }
        for i in 0..output1.len() {
            match output1[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect_index(source_node, source_port, add_offset + i, 0);
                }
                Port::Global(source_port) => {
                    net1.connect_input_index(source_port, add_offset + i, 0);
                }
                _ => (),
            }
        }
        for i in 0..output2.len() {
            match output2[i].source {
                Port::Local(source_node, source_port) => {
                    net1.connect_index(source_node + offset, source_port, add_offset + i, 1);
                }
                Port::Global(source_port) => {
                    net1.connect_input_index(source_port, add_offset + i, 1);
                }
                _ => (),
            }
        }
        net1.invalidate_order();
        net1.resolve_frontend(&mut net2);
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
        net2.disambiguate_ids(&net1);
        let offset = net1.vertex.len();
        net1.vertex.append(&mut net2.vertex);
        // Adjust local ports.
        for node in offset..net1.vertex.len() {
            net1.node_index.insert(net1.vertex[node].id, node);
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
        net1.output_edge = net2.output_edge.clone();
        net1.output = net2.output.clone();
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
        net1.resolve_frontend(&mut net2);
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
