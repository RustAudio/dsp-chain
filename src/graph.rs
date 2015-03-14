
use dsp::Dsp;
use petgraph as pg;
use petgraph::graph::NodeIndex;
use sound_stream::{Sample, Settings};

/// A directed, acyclic DSP graph.
#[derive(Clone, Debug)]
pub struct Graph<S, D> {
    graph: pg::Graph<D, ()>,
    maybe_master: Option<Index>,
    phantom_data: ::std::marker::PhantomData<S>,
}

/// Represents a graph node index.
pub type Index = NodeIndex<u32>;

/// A type for representing an error on the occasion
/// that a connection would create a cyclic graph.
#[derive(Copy, Clone, Debug)]
pub struct WouldCycle;

/// An iterator over references to the inputs of a Graph node.
pub type Inputs<'a, D> = Neighbors<'a, D>;
/// An iterator over mutable references to the inputs of a Graph node.
pub type InputsMut<'a, D> = NeighborsMut<'a, D>;
/// An iterator over references to the outputs of a Graph node.
pub type Outputs<'a, D> = Neighbors<'a, D>;
/// An iterator over mutable references to the outputs of a Graph node.
pub type OutputsMut<'a, D> = NeighborsMut<'a, D>;

/// An iterator over references to the neighbors of a Graph node.
pub struct Neighbors<'a, D: 'a> {
    graph: &'a pg::Graph<D, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMut<'a, D: 'a> {
    graph: &'a mut pg::Graph<D, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

impl<S, D> Graph<S, D> where S: Sample, D: Dsp<S> {

    /// Constructor for a new dsp Graph.
    pub fn new() -> Graph<S, D> {
        let graph = pg::Graph::new();
        Graph {
            graph: graph,
            maybe_master: None,
            phantom_data: ::std::marker::PhantomData,
        }
    }

    /// Set the master node for the Graph.
    /// Graph will check to see if a node exists for the given index before assigning.
    /// Graph's Dsp implementation will use the dsp at the master node when the audio_requested
    /// method is called.
    pub fn set_master(&mut self, maybe_index: Option<Index>) {
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
    pub fn master_index(&self) -> Option<Index> {
        self.maybe_master
    }

    /// Add a node to the dsp graph.
    pub fn add_node(&mut self, dsp: D) -> Index {
        self.graph.add_node(dsp)
    }

    /// Remove a node from the dsp graph.
    /// Reset maybe_master to None if the index matches the current master index.
    pub fn remove_node(&mut self, idx: Index) -> Option<D> {
        if let Some(master_idx) = self.maybe_master {
            if idx == master_idx {
                self.maybe_master = None;
            }
        }
        self.graph.remove_node(idx)
    }

    /// Adds a connection from `a` to `b`. That is, `a` is now an input to `b`.
    /// Returns an error instead if the input would create a cycle in the graph.
    pub fn add_input(&mut self, a: Index, b: Index) -> Result<(), WouldCycle> {
        let edge = self.graph.add_edge(a, b, ());
        if pg::algo::is_cyclic(&self.graph) {
            self.graph.remove_edge(edge);
            Err(WouldCycle)
        } else {
            Ok(())
        }
    }

    /// Remove the input between the nodes at the given indices if there is one.
    pub fn remove_input(&mut self, a: Index, b: Index) {
        if let Some(edge) = self.graph.find_edge(a, b) {
            self.graph.remove_edge(edge);
        } else if let Some(edge) = self.graph.find_edge(b, a) {
            self.graph.remove_edge(edge);
        }
    }

    /// Returns an iterator over references to each neighboring node in the given direction.
    fn neighbors<'a>(&'a self, idx: Index,
                     direction: pg::EdgeDirection) -> Neighbors<'a, D> {
        Neighbors {
            graph: &self.graph,
            neighbors: self.graph.neighbors_directed(idx, direction),
        }
    }

    /// Returns an iterator over mutable references to each neighboring node in the given direction.
    fn neighbors_mut<'a>(&'a mut self, idx: Index,
                         direction: pg::EdgeDirection) -> NeighborsMut<'a, D> {
        let graph = &mut self.graph as *mut pg::Graph<D, ()>;
        // Here we use `unsafe` to allow for aliasing references to the Graph.
        // We allow aliasing in this case because we know that it is impossible
        // for a user to use InputsMut unsafely as it's fields are private and
        // it only exposes its Iterator implementation, which is safe.
        NeighborsMut {
            graph: unsafe { ::std::mem::transmute(graph) },
            neighbors: unsafe { (*graph).neighbors_directed(idx, direction) },
        }
    }

    /// Returns an iterator over references to each input node.
    pub fn inputs<'a>(&'a self, idx: Index) -> Inputs<'a, D> {
        self.neighbors(idx, pg::Incoming)
    }

    /// Returns an iterator over mutable references to each input node.
    pub fn inputs_mut<'a>(&'a mut self, idx: Index) -> InputsMut<'a, D> {
        self.neighbors_mut(idx, pg::Incoming)
    }

    /// Returns an iterator over references to each output node.
    pub fn outputs<'a>(&'a self, idx: Index) -> Outputs<'a, D> {
        self.neighbors(idx, pg::Outgoing)
    }

    /// Returns an iterator over mutable references to each output node.
    pub fn outputs_mut<'a>(&'a mut self, idx: Index) -> OutputsMut<'a, D> {
        self.neighbors_mut(idx, pg::Outgoing)
    }

    /// Request audio from the node at the given index.
    pub fn audio_requested_from_node(&mut self, idx: Index, output: &mut[S], settings: Settings) {
        audio_requested(&mut self.graph, idx, output, settings);
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

}

impl<'a, D> Iterator for Neighbors<'a, D> {
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        match self.neighbors.next() {
            Some(idx) => Some(&self.graph[idx]),
            None => None,
        }
    }
}

impl<'a, D> Iterator for NeighborsMut <'a, D> {
    type Item = &'a mut D;
    fn next(&mut self) -> Option<&'a mut D> {
        let NeighborsMut { ref mut graph, ref mut neighbors } = *self;
        match neighbors.next() {
            Some(idx) => {
                let input_ref_mut: &mut D = &mut graph[idx];
                // Without the following unsafe block, rustc complains about
                // input_ref_mut not having a suitable life time. This is because
                // it is concerned about creating aliasing mutable references,
                // however we know that only one mutable reference will be returned
                // at a time and that they will never alias. Thus, we transmute to
                // silence the lifetime warning!
                Some(unsafe { ::std::mem::transmute(input_ref_mut) })
            },
            None => None,
        }
    }
}

impl<S, D> ::std::ops::Index<Index> for Graph<S, D> {
    type Output = D;
    fn index<'a>(&'a self, index: &Index) -> &'a D {
        &self.graph[*index]
    }
}

impl<S, D> ::std::ops::IndexMut<Index> for Graph<S, D> {
    fn index_mut(&mut self, index: &Index) -> &mut D {
        &mut self.graph[*index]
    }
}

impl<S, D> Dsp<S> for Graph<S, D>
    where
        S: Sample,
        D: Dsp<S>,
{
    fn audio_requested(&mut self, output: &mut [S], settings: Settings) {
        if let Some(idx) = self.maybe_master {
            audio_requested(&mut self.graph, idx, output, settings);
        }
    }
}

/// Request audio from the node at the given index. If that node has incoming neighbors,
/// they will also be requested and summed prior.
#[inline]
fn audio_requested<S, D>(graph: &mut pg::Graph<D, ()>,
                         idx: NodeIndex,
                         output: &mut [S],
                         settings: Settings)
    where
        S: Sample,
        D: Dsp<S>,
{
    {
        let graph = graph as *mut pg::Graph<D, ()>;
        for neighbor_idx in unsafe { (*graph).neighbors_directed(idx, pg::Incoming) } {
            let graph: &mut pg::Graph<D, ()> = unsafe { ::std::mem::transmute(graph) };
            let mut working = vec![Sample::zero(); settings.buffer_size()];
            audio_requested(graph, neighbor_idx, &mut working, settings);
            let vol_per_channel = graph[neighbor_idx].vol_per_channel();
            Sample::add_buffers(output, &working[..], &vol_per_channel[..]);
        }
    }
    let node = &mut graph[idx];
    node.audio_requested(output, settings);
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

