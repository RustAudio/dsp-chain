//! 
//! The `Graph` type constructs a directed, acyclic graph of DSP `Node` types.
//! It supports multiple input and multiple output nodes.
//! `Graph` uses bluss's petgraph crate. See more [here](https://crates.io/crates/petgraph).
//! The `Graph` type requires its nodes to have implemented the `Node` trait.
//!


use node::Node;
use petgraph as pg;
use sound_stream::{Sample, Settings};

/// A directed, acyclic DSP graph.
#[derive(Clone, Debug)]
pub struct Graph<S, N> {
    graph: pg::Graph<Slot<S, N>, ()>,
    maybe_master: Option<NodeIndex>,
}

/// A Dsp object and it's sample buffer.
#[derive(Clone, Debug)]
struct Slot<S, N> {
    /// User defined DspNode type.
    node: N,
    /// The Node's sample buffer.
    buffer: Vec<S>,
    /// Indicates whether or not the buffer has already been rendered
    /// for the current audio_requested duration. This saves us from re-rendering
    /// the buffer in the case that the Node has multiple output connections.
    is_rendered: bool,
}

/// Represents a graph node index.
pub type NodeIndex = pg::graph::NodeIndex<u32>;

/// A type for representing an error on the occasion
/// that a connection would create a cyclic graph.
#[derive(Copy, Clone, Debug)]
pub struct WouldCycle;

/// An iterator over references to the inputs of a Graph node.
pub type Inputs<'a, S, N> = Neighbors<'a, S, N>;
/// An iterator over references to the inputs of a Graph node along with their indices.
pub type InputsWithIndices<'a, S, N> = NeighborsWithIndices<'a, S, N>;
/// An iterator over mutable references to the inputs of a Graph node.
pub type InputsMut<'a, S, N> = NeighborsMut<'a, S, N>;
/// An iterator over mutable references to the inputs of a Graph node along with their indices.
pub type InputsMutWithIndices<'a, S, N> = NeighborsMutWithIndices<'a, S, N>;
/// An iterator over references to the outputs of a Graph node.
pub type Outputs<'a, S, N> = Neighbors<'a, S, N>;
/// An iterator over references to the outputs of a Graph node along with their indices.
pub type OutputsWithIndices<'a, S, N> = NeighborsWithIndices<'a, S, N>;
/// An iterator over mutable references to the outputs of a Graph node.
pub type OutputsMut<'a, S, N> = NeighborsMut<'a, S, N>;
/// An iterator over mutable references to the outputs of a Graph node alonw with their indices.
pub type OutputsMutWithIndices<'a, S, N> = NeighborsMutWithIndices<'a, S, N>;

/// An iterator over references to the neighbors of a Graph node.
pub struct Neighbors<'a, S: 'a, N: 'a> {
    graph: &'a pg::Graph<Slot<S, N>, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

/// An iterator over references to the neighbors of a Graph node.
pub struct NeighborsWithIndices<'a, S: 'a, N: 'a> {
    graph: &'a pg::Graph<Slot<S, N>, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMut<'a, S: 'a, N: 'a> {
    graph: &'a mut pg::Graph<Slot<S, N>, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMutWithIndices<'a, S: 'a, N: 'a> {
    graph: &'a mut pg::Graph<Slot<S, N>, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

impl<S, N> Graph<S, N> where S: Sample, N: Node<S> {

    /// Constructor for a new dsp Graph.
    pub fn new() -> Graph<S, N> {
        let graph = pg::Graph::new();
        Graph {
            graph: graph,
            maybe_master: None,
        }
    }

    /// Set the master node for the Graph.
    /// Graph will check to see if a node exists for the given index before assigning.
    /// Graph's Dsp implementation will use the dsp at the master node when the audio_requested
    /// method is called.
    pub fn set_master(&mut self, maybe_index: Option<NodeIndex>) {
        let maybe_index = match maybe_index {
            Some(index) => match self.graph.node_weight(index) {
                Some(_) => Some(index),
                None => None,
            },
            None => None,
        };
        self.maybe_master = maybe_index;
    }

    /// Return the master index if there is one.
    pub fn master_index(&self) -> Option<NodeIndex> {
        self.maybe_master
    }

    /// Add a node to the dsp graph.
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        self.graph.add_node(Slot {
            node: node,
            buffer: Vec::new(),
            is_rendered: false,
        })
    }

    /// Remove a node from the dsp graph.
    /// Reset maybe_master to None if the index matches the current master index.
    pub fn remove_node(&mut self, idx: NodeIndex) -> Option<N> {
        if let Some(master_idx) = self.maybe_master {
            if idx == master_idx {
                self.maybe_master = None;
            }
        }
        self.graph.remove_node(idx).map(|slot| {
            let Slot { node, .. } = slot;
            node
        })
    }

    /// Adds a connection from `a` to `b`. That is, `a` is now an input to `b`.
    /// Returns an error instead if the input would create a cycle in the graph.
    pub fn add_input(&mut self, a: NodeIndex, b: NodeIndex) -> Result<(), WouldCycle> {
        let edge = self.graph.add_edge(a, b, ());
        if pg::algo::is_cyclic_directed(&self.graph) {
            self.graph.remove_edge(edge);
            Err(WouldCycle)
        } else {
            Ok(())
        }
    }

    /// Remove the input between the nodes at the given indices if there is one.
    pub fn remove_input(&mut self, a: NodeIndex, b: NodeIndex) {
        if let Some(edge) = self.graph.find_edge(a, b) {
            self.graph.remove_edge(edge);
        } else if let Some(edge) = self.graph.find_edge(b, a) {
            self.graph.remove_edge(edge);
        }
    }

    /// Returns an iterator over references to each neighboring node in the given direction.
    fn neighbors<'a>(&'a self, idx: NodeIndex,
                     direction: pg::EdgeDirection) -> Neighbors<'a, S, N> {
        Neighbors {
            graph: &self.graph,
            neighbors: self.graph.neighbors_directed(idx, direction),
        }
    }

    /// Returns an iterator over mutable references to each neighboring node in the given direction.
    fn neighbors_mut<'a>(&'a mut self, idx: NodeIndex,
                         direction: pg::EdgeDirection) -> NeighborsMut<'a, S, N> {
        let graph = &mut self.graph as *mut pg::Graph<Slot<S, N>, ()>;
        // Here we use `unsafe` to allow for aliasing references to the Graph.
        // We allow aliasing in this case because we know that it is impossible
        // for a user to use InputsMut unsafely as it's fields are private and
        // it only exposes its Iterator implementation, which we know is safe.
        // (see the Iterator implementation below the Graph implementation).
        NeighborsMut {
            graph: unsafe { ::std::mem::transmute(graph) },
            neighbors: unsafe { (*graph).neighbors_directed(idx, direction) },
        }
    }

    /// Returns an iterator over references to each input node.
    pub fn inputs<'a>(&'a self, idx: NodeIndex) -> Inputs<'a, S, N> {
        self.neighbors(idx, pg::Incoming)
    }

    /// Returns an iterator over mutable references to each input node.
    pub fn inputs_mut<'a>(&'a mut self, idx: NodeIndex) -> InputsMut<'a, S, N> {
        self.neighbors_mut(idx, pg::Incoming)
    }

    /// Returns an iterator over references to each output node.
    pub fn outputs<'a>(&'a self, idx: NodeIndex) -> Outputs<'a, S, N> {
        self.neighbors(idx, pg::Outgoing)
    }

    /// Returns an iterator over mutable references to each output node.
    pub fn outputs_mut<'a>(&'a mut self, idx: NodeIndex) -> OutputsMut<'a, S, N> {
        self.neighbors_mut(idx, pg::Outgoing)
    }

    /// Request audio from the node at the given index.
    pub fn audio_requested_from_node(&mut self,
                                     idx: NodeIndex,
                                     output: &mut[S],
                                     settings: Settings) {
        request_audio_from_graph(&mut self.graph, idx, output, settings);
        self.reset_buffers();
    }

    /// Remove all incoming connections to the node at the given index.
    /// Return the number of connections removed.
    pub fn remove_all_inputs(&mut self, idx: NodeIndex) -> usize {
        let input_indices: Vec<_> = self.graph.neighbors_directed(idx, pg::Incoming).collect();
        let num = input_indices.len();
        for input_idx in input_indices {
            self.remove_input(input_idx, idx);
        }
        num
    }

    /// Remove all outgoing connections from the node at the given index.
    /// Return the number of connections removed.
    pub fn remove_all_outputs(&mut self, idx: NodeIndex) -> usize {
        let output_indices: Vec<_> = self.graph.neighbors_directed(idx, pg::Outgoing).collect();
        let num = output_indices.len();
        for output_idx in output_indices {
            self.remove_input(output_idx, idx);
        }
        num
    }

    /// Clear all dsp nodes that have no inputs and that are not inputs to any other nodes.
    pub fn clear_disconnected(&mut self) {
        let no_incoming: Vec<_> = self.graph.without_edges(pg::Incoming).collect();
        let no_outgoing: Vec<_> = self.graph.without_edges(pg::Outgoing).collect();
        let indices_for_removal = no_incoming.into_iter()
            .filter(|incoming| no_outgoing.iter().any(|outgoing| outgoing == incoming));
        for idx in indices_for_removal {
            if let Some(master_idx) = self.maybe_master {
                if master_idx == idx {
                    self.maybe_master = None;
                }
            }
            self.graph.remove_node(idx);
        }
    }

    /// Clear all dsp nodes.
    pub fn clear(&mut self) {
        self.graph.clear();
        self.maybe_master = None;
    }

    /// Prepare the buffers for all nodes within the Graph.
    pub fn prepare_buffers(&mut self, settings: Settings) {
        let target_len = settings.buffer_size();
        for node in self.graph.all_node_weights_mut() {
            let len = node.buffer.len();
            if len < target_len {
                node.buffer.extend((len..target_len).map(|_| Sample::zero()));
            } else if len > target_len {
                node.buffer.truncate(target_len);
            }
        }
    }

    /// Reset all buffers within all nodes that have incoming connections towards the node at the
    /// given index.
    fn reset_buffers(&mut self) {
        for node in self.graph.all_node_weights_mut() {
            node.is_rendered = false;
        }
    }

}


impl<S, N> ::std::ops::Index<NodeIndex> for Graph<S, N> {
    type Output = N;
    #[inline]
    fn index<'a>(&'a self, index: NodeIndex) -> &'a N {
        &self.graph[index].node
    }
}

impl<S, N> ::std::ops::IndexMut<NodeIndex> for Graph<S, N> {
    #[inline]
    fn index_mut(&mut self, index: NodeIndex) -> &mut N {
        &mut self.graph[index].node
    }
}


impl<S, N> Node<S> for Graph<S, N>
    where
        S: Sample,
        N: Node<S>,
{
    fn audio_requested(&mut self, output: &mut [S], settings: Settings) {
        if let Some(idx) = self.maybe_master {
            self.audio_requested_from_node(idx, output, settings);
        }
    }
}


/// Request audio from the node at the given index and all incoming nodes.
/// If the node does have incoming neighbors, they will be requested and summed first.
/// This process will continue recursively until all incoming connections have been visited.
#[inline]
fn request_audio_from_graph<S, N>(graph: &mut pg::Graph<Slot<S, N>, ()>,
                                  idx: pg::graph::NodeIndex,
                                  output: &mut [S],
                                  settings: Settings)
    where
        S: Sample,
        N: Node<S>,
{

    let graph = graph as *mut pg::Graph<Slot<S, N>, ()>;

    let &mut Slot { ref mut node, ref mut buffer, ref mut is_rendered } = unsafe {
        &mut(*graph)[idx]
    };

    // If the node at the current index hasn't already been rendered then:
    // - Initialise the buffer, ensuring it is the correct length and zeroed.
    // - If the Node has inputs, request audio from them and have it summed upon `buffer`.
    // - Request audio from this node using `buffer`.
    if !*is_rendered {

        // If the buffer's size does not match the output's, we need to change to the right size.
        if buffer.len() != output.len() {
            let len = buffer.len();
            let target_len = output.len();
            if len < target_len {
                buffer.extend((len..target_len).map(|_| Sample::zero()));
            } else if len > target_len {
                buffer.truncate(target_len);
            }
        }

        // Zero the buffer, ready to sum the inputs.
        for sample in buffer.iter_mut() {
            *sample = Sample::zero();
        }

        // Iterate over all of our inputs.
        let inputs = unsafe { (*graph).neighbors_directed(idx, pg::Incoming) };
        for neighbor_idx in inputs {
            let graph: &mut pg::Graph<Slot<S, N>, ()> = unsafe { ::std::mem::transmute(graph) };
            request_audio_from_graph(graph, neighbor_idx, buffer, settings);
        }

        // Request audio from the node.
        node.audio_requested(buffer, settings);

        *is_rendered = true;
    }

    // Some the rendered buffer onto the output buffer.
    for (output_sample, sample) in output.iter_mut().zip(buffer.iter()) {
        *output_sample = *output_sample + *sample;
    }

}



impl ::std::fmt::Display for WouldCycle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        writeln!(f, "{:?}", self)
    }
}

impl ::std::error::Error for WouldCycle {
    fn description(&self) -> &str {
        "Adding this input would have caused the graph to cycle!"
    }
}


impl<'a, S, N> Neighbors<'a, S, N> {
    /// Return an adaptor that will also return the neighor's NodeIndex.
    #[inline]
    pub fn with_indices(self) -> NeighborsWithIndices<'a, S, N> {
        let Neighbors { graph, neighbors } = self;
        NeighborsWithIndices {
            graph: graph,
            neighbors: neighbors,
        }
    }
}

impl<'a, S, N> NeighborsMut <'a, S, N> {
    /// Return an adaptor that will also return the neighor's NodeIndex.
    #[inline]
    pub fn with_indices(self) -> NeighborsMutWithIndices<'a, S, N> {
        let NeighborsMut { graph, neighbors } = self;
        NeighborsMutWithIndices {
            graph: graph,
            neighbors: neighbors,
        }
    }
}


impl<'a, S, N> Iterator for Neighbors<'a, S, N> {
    type Item = &'a N;
    #[inline]
    fn next(&mut self) -> Option<&'a N> {
        match self.neighbors.next() {
            Some(idx) => Some(&self.graph[idx].node),
            None => None,
        }
    }
}

impl<'a, S, N> Iterator for NeighborsWithIndices<'a, S, N> {
    type Item = (&'a N, NodeIndex);
    #[inline]
    fn next(&mut self) -> Option<(&'a N, NodeIndex)> {
        match self.neighbors.next() {
            Some(idx) => Some((&self.graph[idx].node, idx)),
            None => None,
        }
    }
}


impl<'a, S, N> Iterator for NeighborsMut<'a, S, N> {
    type Item = &'a mut N;
    #[inline]
    fn next(&mut self) -> Option<&'a mut N> {
        let NeighborsMut { ref mut graph, ref mut neighbors } = *self;
        match neighbors.next() {
            Some(idx) => {
                let node: &mut N = &mut graph[idx].node;
                // Without the following unsafe block, rustc complains about
                // input_ref_mut not having a suitable life time. This is because
                // it is concerned about creating aliasing mutable references,
                // however we know that only one mutable reference will be returned
                // at a time and that they will never alias. Thus, we transmute to
                // silence the lifetime warning!
                Some(unsafe { ::std::mem::transmute(node) })
            },
            None => None,
        }
    }
}

impl<'a, S, N> Iterator for NeighborsMutWithIndices<'a, S, N> {
    type Item = (&'a mut N, NodeIndex);
    #[inline]
    fn next(&mut self) -> Option<(&'a mut N, NodeIndex)> {
        let NeighborsMutWithIndices { ref mut graph, ref mut neighbors } = *self;
        match neighbors.next() {
            Some(idx) => {
                let node: &mut N = &mut graph[idx].node;
                // Without the following unsafe block, rustc complains about
                // input_ref_mut not having a suitable life time. This is because
                // it is concerned about creating aliasing mutable references,
                // however we know that only one mutable reference will be returned
                // at a time and that they will never alias. Thus, we transmute to
                // silence the lifetime warning!
                Some((unsafe { ::std::mem::transmute(node) }, idx))
            },
            None => None,
        }
    }
}


