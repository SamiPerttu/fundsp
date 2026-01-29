//! Conversion of `AudioNode` trees into directed acyclic graphs
//! that reveal their inner structure.
extern crate alloc;
use alloc::vec::Vec;

/// Path in an `AudioNode` tree.
#[derive(Clone, Eq, PartialEq)]
pub struct Path {
    /// The path, from root, of a node in the tree.
    path: Vec<u32>,
    /// The index indicates an output (if this is a source path),
    /// an input (if this is a target path), or is arbitrary
    /// if this path points to a node.
    index: usize,
}

/// Path identifies a node in a generic tree of `AudioNode`s including the input or output index.
impl Path {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            index: 0,
        }
    }
    /// Length of the path.
    pub fn len(&self) -> usize {
        self.path.len()
    }
    /// Length of the path.
    pub fn length(&self) -> usize {
        self.path.len()
    }
    /// Get node number at depth `index`.
    pub fn at(&self, index: usize) -> u32 {
        self.path[index]
    }
    /// Add `suffix` to path.
    pub fn push(&mut self, suffix: u32) {
        self.path.push(suffix);
    }
    /// Set path suffix.
    pub fn set_suffix(&mut self, suffix: u32) {
        let i = self.path.len() - 1;
        self.path[i] = suffix;
    }
    /// Get source or target input or output index.
    pub fn index(&self) -> usize {
        self.index
    }
    /// Pop the last suffix in the path.
    pub fn pop(&mut self) {
        self.path.pop();
    }
    /// Push `suffix` to path and return the path.
    pub fn with_suffix(mut self, suffix: u32) -> Path {
        self.push(suffix);
        self
    }
    /// Set source or target `index` and return the path.
    pub fn with_index(mut self, index: usize) -> Path {
        self.index = index;
        self
    }
}

/// Connection from input source to output target.
#[derive(Clone, Eq, PartialEq)]
pub struct Edge {
    source: Path,
    target: Path,
}

impl Edge {
    pub fn new(source: Path, target: Path) -> Self {
        Self { source, target }
    }
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn target(&self) -> &Path {
        &self.target
    }
}

/// An `AudioNode` inside a tree of nodes.
#[derive(Clone)]
pub struct Node {
    path: Path,
    id: u64,
    inputs: usize,
    outputs: usize,
}

impl Node {
    pub fn new(path: Path, id: u64, inputs: usize, outputs: usize) -> Self {
        Self {
            path,
            id,
            inputs,
            outputs,
        }
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn inputs(&self) -> usize {
        self.inputs
    }
    pub fn outputs(&self) -> usize {
        self.outputs
    }
}

/// A tree of `AudioNode`s converted into a directed acyclic graph.
#[derive(Clone)]
pub struct Graph {
    edges: Vec<Edge>,
    nodes: Vec<Node>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            nodes: Vec::new(),
        }
    }

    pub fn edges(&self) -> usize {
        self.edges.len()
    }
    pub fn nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn push_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    pub fn push_edges(&mut self, source: Path, mut targets: Vec<Path>) {
        while let Some(target) = targets.pop() {
            self.edges.push(Edge::new(source.clone(), target));
        }
    }

    pub fn push_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn edge(&self, i: usize) -> &Edge {
        &self.edges[i]
    }
    pub fn node(&self, i: usize) -> &Node {
        &self.nodes[i]
    }

    pub fn find_node(&self, path: &Path) -> Option<&Node> {
        for i in 0..self.nodes() {
            if self.node(i).path() == path {
                return Some(self.node(i));
            }
        }
        None
    }
}
