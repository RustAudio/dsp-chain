//! 
//! The `Graph` type constructs a directed, acyclic graph of DSP `Node` types.
//! It supports multiple input and multiple output nodes.
//! `Graph` uses bluss's petgraph crate. See more [here](https://crates.io/crates/petgraph).
//! The `Graph` type requires its nodes to have implemented the `Node` trait.
//!


use node::Node;
use petgraph as pg;
use sound_stream::{Sample, Settings};


/// An alias for our Graph's Node Index.
pub type NodeIndex = pg::graph::NodeIndex<u32>;
/// An alias for our Graph's Edge Index.
pub type EdgeIndex = pg::graph::EdgeIndex<u32>;

/// An alias for the petgraph Graph used within our DSP Graph.
pub type PetGraph<S, N> = pg::Graph<Slot<N>, Connection<S>, pg::Directed>;

/// An alias representing neighboring nodes.
pub type PgNeighbors<'a, S> = pg::graph::Neighbors<'a, Connection<S>, u32>;

/// A directed, acyclic DSP graph.
#[derive(Clone, Debug)]
pub struct Graph<S, N> {
    graph: PetGraph<S, N>,
    visit_order: Vec<NodeIndex>,
    maybe_master: Option<NodeIndex>,
}

/// A Dsp object and its sample buffer.
#[derive(Clone, Debug)]
struct Slot<N> {
    /// User defined DspNode type.
    node: N,
}

/// Describes a connection between two Nodes within the Graph.
#[derive(Clone, Debug)]
struct Connection<S> {
    buffer: Vec<S>,
}

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
    graph: &'a PetGraph<S, N>,
    neighbors: PgNeighbors<'a, S>,
}

/// An iterator over references to the neighbors of a Graph node.
pub struct NeighborsWithIndices<'a, S: 'a, N: 'a> {
    graph: &'a PetGraph<S, N>,
    neighbors: PgNeighbors<'a, S>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMut<'a, S: 'a, N: 'a> {
    graph: &'a mut PetGraph<S, N>,
    neighbors: PgNeighbors<'a, S>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMutWithIndices<'a, S: 'a, N: 'a> {
    graph: &'a mut PetGraph<S, N>,
    neighbors: PgNeighbors<'a, S>,
}

impl<S, N> Graph<S, N> where S: Sample, N: Node<S> {

    /// Constructor for a new dsp Graph.
    pub fn new() -> Graph<S, N> {
        let graph = pg::Graph::new();
        Graph {
            graph: graph,
            visit_order: Vec::new(),
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
        self.prepare_visit_order();
    }

    /// Return the master index if there is one.
    pub fn master_index(&self) -> Option<NodeIndex> {
        self.maybe_master
    }

    /// Add a node to the dsp graph.
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        let idx = self.graph.add_node(Slot { node: node, });
        self.prepare_visit_order();
        idx
    }

    /// Prepare the visit order for the graph in its current state.
    ///
    /// When audio is requested from the graph, we need to iterate through all nodes so that all
    /// child nodes are visited before their parents. To do this, we can use petgraph's toposort
    /// algorithm to return the topological order of our graph.
    fn prepare_visit_order(&mut self) {
        self.visit_order = pg::algo::toposort(&self.graph);
    }

    /// Remove a node from the dsp graph.
    /// Reset maybe_master to None if the index matches the current master index.
    pub fn remove_node(&mut self, idx: NodeIndex) -> Option<N> {
        if let Some(master_idx) = self.maybe_master {
            if idx == master_idx {
                self.maybe_master = None;
            }
        }
        let maybe_removed = self.graph.remove_node(idx).map(|slot| {
            let Slot { node, .. } = slot;
            node
        });
        self.prepare_visit_order();
        maybe_removed
    }

    /// Adds a connection from `a` to `b`. That is, `a` is now an input to `b`.
    /// Returns an error instead if the input would create a cycle in the graph.
    pub fn add_input(&mut self, a: NodeIndex, b: NodeIndex) -> Result<(), WouldCycle> {

        // Add the input connection between the two nodes with a Buffer the size of the output's.
        let edge = self.graph.add_edge(a, b, Connection { buffer: Vec::new() });


        // If the connection would create a cycle, remove the node and return an error.
        if pg::algo::is_cyclic_directed(&self.graph) {
            self.graph.remove_edge(edge);
            Err(WouldCycle)
        }
        // Otherwise the method was successful.
        else {
            self.prepare_visit_order();
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
        self.prepare_visit_order();
    }

    /// Returns an iterator over references to each neighboring node in the given direction.
    fn neighbors<'a>(&'a self,
                     idx: NodeIndex,
                     direction: pg::EdgeDirection) -> Neighbors<'a, S, N> {
        Neighbors {
            graph: &self.graph,
            neighbors: self.graph.neighbors_directed(idx, direction),
        }
    }

    /// Returns an iterator over mutable references to each neighboring node in the given direction.
    fn neighbors_mut<'a>(&'a mut self,
                         idx: NodeIndex,
                         direction: pg::EdgeDirection) -> NeighborsMut<'a, S, N> {
        let graph = &mut self.graph as *mut PetGraph<S, N>;
        // Here we use `unsafe` to allow for aliasing references to the Graph.
        // We allow aliasing in this case because we know that it is impossible
        // for a user to use NeighborsMut unsafely as its fields are private and
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
        self.visit_order.clear();
        self.maybe_master = None;
    }

    /// Prepare the buffers for all nodes within the Graph.
    pub fn prepare_buffers(&mut self, settings: Settings) {
        let target_len = settings.buffer_size();

        // Initialise all connection buffers.
        for connection in self.graph.edge_weights_mut() {
            let len = connection.buffer.len();
            if len < target_len {
                connection.buffer.extend((len..target_len).map(|_| Sample::zero()));
            } else if len > target_len {
                connection.buffer.truncate(target_len);
            }
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

        let Graph { ref visit_order, ref mut graph, .. } = *self;

        for &node_idx in visit_order.iter() {

            // Zero the buffer, ready to sum the inputs.
            for sample in output.iter_mut() {
                *sample = Sample::zero();
            }

            // Walk over each of the incoming connections to sum their buffers to the output.
            let mut incoming_edges = graph.walk_edges_directed(node_idx, pg::Incoming);
            while let Some(edge) = incoming_edges.next(graph) {
                let connection = &graph[edge];
                for (sample, in_sample) in output.iter_mut().zip(connection.buffer.iter()) {
                    *sample = *sample + *in_sample;
                }
            }

            // Render our `output` buffer with the current node.
            graph[node_idx].node.audio_requested(output, settings);

            // Walk over each of the outgoing connections and write the rendered output to them.
            let mut outgoing_edges = graph.walk_edges_directed(node_idx, pg::Outgoing);
            while let Some(edge) = outgoing_edges.next(graph) {
                let connection = &mut graph[edge];

                // Ensure the buffer matches the target length.
                let len = connection.buffer.len();
                let target_len = output.len();
                if len < target_len {
                    connection.buffer.extend((len..target_len).map(|_| Sample::zero()));
                } else if len > target_len {
                    connection.buffer.truncate(target_len);
                }

                for (out_sample, rendered) in connection.buffer.iter_mut().zip(output.iter()) {
                    *out_sample = *rendered;
                }
            }

        }

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


