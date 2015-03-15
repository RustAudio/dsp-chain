
use dsp::Dsp;
use petgraph as pg;
use sound_stream::{Sample, Settings};

/// A directed, acyclic DSP graph.
#[derive(Clone, Debug)]
pub struct Graph<S, D> {
    graph: pg::Graph<Node<S, D>, ()>,
    maybe_master: Option<NodeIndex>,
    phantom_data: ::std::marker::PhantomData<S>,
}

/// A Dsp object and it's sample buffer.
#[derive(Clone, Debug)]
struct Node<S, D>(D, Option<Vec<S>>);

/// Represents a graph node index.
pub type NodeIndex = pg::graph::NodeIndex<u32>;

/// A type for representing an error on the occasion
/// that a connection would create a cyclic graph.
#[derive(Copy, Clone, Debug)]
pub struct WouldCycle;

/// An iterator over references to the inputs of a Graph node.
pub type Inputs<'a, S, D> = Neighbors<'a, S, D>;
/// An iterator over mutable references to the inputs of a Graph node.
pub type InputsMut<'a, S, D> = NeighborsMut<'a, S, D>;
/// An iterator over references to the outputs of a Graph node.
pub type Outputs<'a, S, D> = Neighbors<'a, S, D>;
/// An iterator over mutable references to the outputs of a Graph node.
pub type OutputsMut<'a, S, D> = NeighborsMut<'a, S, D>;

/// An iterator over references to the neighbors of a Graph node.
pub struct Neighbors<'a, S: 'a, D: 'a> {
    graph: &'a pg::Graph<Node<S, D>, ()>,
    neighbors: pg::graph::Neighbors<'a, (), u32>,
}

/// An iterator over mutable references to the neighbors of a Graph node.
pub struct NeighborsMut<'a, S: 'a, D: 'a> {
    graph: &'a mut pg::Graph<Node<S, D>, ()>,
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
    pub fn add_node(&mut self, dsp: D) -> NodeIndex {
        self.graph.add_node(Node(dsp, None))
    }

    /// Remove a node from the dsp graph.
    /// Reset maybe_master to None if the index matches the current master index.
    pub fn remove_node(&mut self, idx: NodeIndex) -> Option<D> {
        if let Some(master_idx) = self.maybe_master {
            if idx == master_idx {
                self.maybe_master = None;
            }
        }
        self.graph.remove_node(idx).map(|node| {
            let Node(dsp, _) = node;
            dsp
        })
    }

    /// Adds a connection from `a` to `b`. That is, `a` is now an input to `b`.
    /// Returns an error instead if the input would create a cycle in the graph.
    pub fn add_input(&mut self, a: NodeIndex, b: NodeIndex) -> Result<(), WouldCycle> {
        let edge = self.graph.add_edge(a, b, ());
        if pg::algo::is_cyclic(&self.graph) {
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
                     direction: pg::EdgeDirection) -> Neighbors<'a, S, D> {
        Neighbors {
            graph: &self.graph,
            neighbors: self.graph.neighbors_directed(idx, direction),
        }
    }

    /// Returns an iterator over mutable references to each neighboring node in the given direction.
    fn neighbors_mut<'a>(&'a mut self, idx: NodeIndex,
                         direction: pg::EdgeDirection) -> NeighborsMut<'a, S, D> {
        let graph = &mut self.graph as *mut pg::Graph<Node<S, D>, ()>;
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
    pub fn inputs<'a>(&'a self, idx: NodeIndex) -> Inputs<'a, S, D> {
        self.neighbors(idx, pg::Incoming)
    }

    /// Returns an iterator over mutable references to each input node.
    pub fn inputs_mut<'a>(&'a mut self, idx: NodeIndex) -> InputsMut<'a, S, D> {
        self.neighbors_mut(idx, pg::Incoming)
    }

    /// Returns an iterator over references to each output node.
    pub fn outputs<'a>(&'a self, idx: NodeIndex) -> Outputs<'a, S, D> {
        self.neighbors(idx, pg::Outgoing)
    }

    /// Returns an iterator over mutable references to each output node.
    pub fn outputs_mut<'a>(&'a mut self, idx: NodeIndex) -> OutputsMut<'a, S, D> {
        self.neighbors_mut(idx, pg::Outgoing)
    }

    /// Request audio from the node at the given index.
    pub fn audio_requested_from_node(&mut self,
                                     idx: NodeIndex,
                                     output: &mut[S],
                                     settings: Settings) {
        request_audio_from_graph(&mut self.graph, idx, output, settings);
        reset_graph_buffers(&mut self.graph, idx);
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


impl<'a, S, D> Iterator for Neighbors<'a, S, D> {
    type Item = &'a D;
    fn next(&mut self) -> Option<&'a D> {
        match self.neighbors.next() {
            Some(idx) => {
                let &Node(ref dsp, _) = &self.graph[idx];
                Some(dsp)
            },
            None => None,
        }
    }
}

impl<'a, S, D> Iterator for NeighborsMut<'a, S, D> {
    type Item = &'a mut D;
    fn next(&mut self) -> Option<&'a mut D> {
        let NeighborsMut { ref mut graph, ref mut neighbors } = *self;
        match neighbors.next() {
            Some(idx) => {
                let &mut Node(ref mut dsp, _) = &mut graph[idx];
                let dsp: &mut D = dsp;
                // Without the following unsafe block, rustc complains about
                // input_ref_mut not having a suitable life time. This is because
                // it is concerned about creating aliasing mutable references,
                // however we know that only one mutable reference will be returned
                // at a time and that they will never alias. Thus, we transmute to
                // silence the lifetime warning!
                Some(unsafe { ::std::mem::transmute(dsp) })
            },
            None => None,
        }
    }
}


impl<S, D> ::std::ops::Index<NodeIndex> for Graph<S, D> {
    type Output = D;
    fn index<'a>(&'a self, index: &NodeIndex) -> &'a D {
        let &Node(ref dsp, _) = &self.graph[*index];
        dsp
    }
}

impl<S, D> ::std::ops::IndexMut<NodeIndex> for Graph<S, D> {
    fn index_mut(&mut self, index: &NodeIndex) -> &mut D {
        let &mut Node(ref mut dsp, _) = &mut self.graph[*index];
        dsp
    }
}


impl<S, D> Dsp<S> for Graph<S, D>
    where
        S: Sample,
        D: Dsp<S>,
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
fn request_audio_from_graph<S, D>(graph: &mut pg::Graph<Node<S, D>, ()>,
                                  idx: pg::graph::NodeIndex,
                                  output: &mut [S],
                                  settings: Settings)
    where
        S: Sample,
        D: Dsp<S>,
{
    {
        let graph = graph as *mut pg::Graph<Node<S, D>, ()>;
        for neighbor_idx in unsafe { (*graph).neighbors_directed(idx, pg::Incoming) } {
            let graph: &mut pg::Graph<Node<S, D>, ()> = unsafe { ::std::mem::transmute(graph) };
            let mut working = vec![Sample::zero(); settings.buffer_size()];
            request_audio_from_graph(graph, neighbor_idx, &mut working, settings);
            let Node(ref dsp, _) = graph[neighbor_idx];
            let vol_per_channel = dsp.vol_per_channel();
            Sample::add_buffers(output, &working[..], &vol_per_channel[..]);
        }
    }
    request_audio_from_node(graph, idx, output, settings);
}


/// If the node at the given index already has a prepared buffer, clone that.
/// Otherwise, request a new buffer from the dsp object.
fn request_audio_from_node<S, D>(graph: &mut pg::Graph<Node<S, D>, ()>,
                                 idx: pg::graph::NodeIndex,
                                 output: &mut [S],
                                 settings: Settings)
    where
        S: Sample,
        D: Dsp<S>,
{
    if graph.neighbors_directed(idx, pg::Outgoing).take(2).count() > 1 {
        let &mut Node(ref mut dsp, ref mut maybe_buffer) = &mut graph[idx];
        let buffer = match *maybe_buffer {
            Some(ref buffer) => {
                output.clone_from_slice(&buffer[..]);
                return;
            },
            None => {
                let mut buffer = vec![Sample::zero(); settings.buffer_size()];
                dsp.audio_requested(&mut buffer[..], settings);
                output.clone_from_slice(&buffer[..]);
                buffer
            },
        };
        *maybe_buffer = Some(buffer);
    } else {
        let &mut Node(ref mut dsp, _) = &mut graph[idx];
        dsp.audio_requested(output, settings);
    }
}


/// Reset all buffers within all nodes that have incoming connections towards the node at the
/// given index.
fn reset_graph_buffers<S, D>(graph: &mut pg::Graph<Node<S, D>, ()>,
                             idx: pg::graph::NodeIndex,)
    where
        S: Sample,
        D: Dsp<S>,
{
    {
        let graph = graph as *mut pg::Graph<Node<S, D>, ()>;
        for neighbor_idx in unsafe { (*graph).neighbors_directed(idx, pg::Incoming) } {
            let graph: &mut pg::Graph<Node<S, D>, ()> = unsafe { ::std::mem::transmute(graph) };
            reset_graph_buffers(graph, neighbor_idx);
        }
    }
    reset_node_buffer(graph, idx);
}


/// Reset the buffer owned by the node at the given index.
fn reset_node_buffer<S, D>(graph: &mut pg::Graph<Node<S, D>, ()>,
                           idx: pg::graph::NodeIndex)
    where
        S: Sample,
        D: Dsp<S>,
{
    let &mut Node(_, ref mut maybe_buffer) = &mut graph[idx];
    if maybe_buffer.is_some() {
        *maybe_buffer = None;
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

