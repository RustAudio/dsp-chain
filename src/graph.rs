//! The [`Graph`](./struct.Graph.html) type constructs a directed, acyclic graph of DSP `Node`
//! types.
//!
//! It supports multiple input and multiple output nodes per node.
//!
//! `Graph` uses the daggy crate. See more [here](https://crates.io/crates/daggy).
//!
//! The `Graph` type requires that its nodes implement the [`Node`](../node/trait.Node.html) trait.

use crate::node::Node;
use daggy::{self, Walker};
use dasp::{self, Frame, Sample};

/// An alias for our Graph's Node Index.
pub type NodeIndex = daggy::NodeIndex<usize>;
/// An alias for our Graph's Edge Index.
pub type EdgeIndex = daggy::EdgeIndex<usize>;

/// An alias for the iterator yielding mutable access to all node weights.
pub type NodesMut<'a, N> = daggy::NodeWeightsMut<'a, N, usize>;

/// Read only access to a **Graph**'s internal node array.
pub type RawNodes<'a, N> = daggy::RawNodes<'a, N, usize>;
/// Read only access to a **Graph**'s internal edge array.
pub type RawEdges<'a, F> = daggy::RawEdges<'a, Connection<F>, usize>;

/// An iterator yielding indices to recently added connections.
pub type EdgeIndices = daggy::EdgeIndices<usize>;

/// An alias for the **Dag** used within our **Graph**.
pub type Dag<F, N> = daggy::Dag<N, Connection<F>, usize>;

/// An alias for the **PetGraph** used by our **Graph**'s internal **Dag**.
pub type PetGraph<F, N> = daggy::PetGraph<N, Connection<F>, usize>;

/// A directed, acyclic DSP graph.
///
/// Designed for easily and safely setting up high performance audio signal generating, processing
/// and mixing. Useful for both simple and highly complex signal graphs.
///
/// There are a variety of use cases for `Graph`:
///
/// - Designing effects.
/// - Creating an audio mixer.
/// - Making a sampler.
/// - Writing a DSP backend for a DAW.
/// - Any kind of modular audio synthesis or processing.
///
/// `Graph` is a wrapper around [daggy](http://mitchmindtree.github.io/daggy/daggy/)'s
/// [`Dag`](http://mitchmindtree.github.io/daggy/daggy/struct.Dag.html) type - an abstraction for
/// working with directed acyclic graph's where high performance node adding and accessing is
/// required.
///
/// An input -> output connection in this `Graph` is represented as a parent -> child connection
/// within the internal `Dag`. The following terms are all equivalent:
///
/// - *input -> output*
/// - *src -> dest*
/// - *parent -> child*
///
/// Audio can be requested from any node in the **Graph** using the
/// [`audio_requested_from`](./struct.Graph.html#method.audio_requested_from) method.
///
/// When [`audio_requested`](../node/trait.Node.html#method.audio_requested) is called on the
/// **Graph**, audio will be requested from the node specified by the index at `maybe_master`. If
/// `maybe_master` is `None`, audio will be requested from the first, input-only node found - that
/// is, the first node that is found with only input connections and no outputs.
///
/// **NodeIndex** is a type that acts as a reference to a node, while **EdgeIndex** is a type that
/// acts as a reference to an edge (which in this case describes a *src -> dest* **Connection**
/// between two nodes). It should be noted that these are only stable across certain operations.
/// **Removing indices may shift other indices of the same type!** Adding nodes or edges to the
/// **Graph** keeps all indices stable, but removing a node or edge will force the last node/edge
/// to shift its index to take its place.
///
/// **Graph** also offers methods for accessing its underlying **Dag** or **PetGraph**.
#[derive(Clone, Debug)]
pub struct Graph<F, N> {
    dag: Dag<F, N>,
    /// The order in which audio will be requested from each node.
    visit_order: Vec<NodeIndex>,
    /// The node from which audio will be requested upon a call to `Node::audio_requested`.
    maybe_master: Option<NodeIndex>,
    /// A buffer to re-use when mixing the dry and wet signals when audio is requested.
    dry_buffer: Vec<F>,
}

/// Describes a connection between two Nodes within the Graph: *input -> connection -> output*.
///
/// **Graph**'s API only allows for read-only access to **Connection**s, so you can be sure that
/// their buffers always represent the last frames rendered by their input node.
#[derive(Clone, Debug)]
pub struct Connection<F> {
    /// The buffer used to pass audio between nodes.
    ///
    /// After `Graph::audio_requested_from` is called, this buffer will contain the audio rendered
    /// by the **Connection**'s input node.
    pub buffer: Vec<F>,
}

/// The error returned when adding an edge that would create a cycle.
#[derive(Copy, Clone, Debug)]
pub struct WouldCycle;

/// A walker object for walking over nodes that are inputs to some node.
pub struct Inputs<F, N> {
    parents: daggy::Parents<N, Connection<F>, usize>,
}

/// A walker object for walking over nodes that are outputs to some node.
pub struct Outputs<F, N> {
    children: daggy::Children<N, Connection<F>, usize>,
}

/// A walker type for walking over a **Graph**'s nodes in the order in which they will visited when
/// audio is requested from the **Graph**.
pub struct VisitOrder {
    current_visit_order_idx: usize,
}

/// A walker type for walking over a **Graph**'s nodes in the order in which they will visited when
/// audio is requested from the **Graph**.
pub struct VisitOrderReverse {
    current_visit_order_idx: usize,
}

impl<F, N> Graph<F, N>
where
    F: Frame,
    N: Node<F>,
{
    /// Constructor for a new dsp Graph.
    ///
    /// [`with_capacity`](./struct.Graph.html#method.with_capacity) is recommended if you have a
    /// rough idea of the number of nodes, connections and frames per buffer upon the **Graph**'s
    /// instantiation.
    pub fn new() -> Self {
        let dag = daggy::Dag::new();
        Graph {
            dag: dag,
            visit_order: Vec::new(),
            dry_buffer: Vec::new(),
            maybe_master: None,
        }
    }

    /// Constructor for a new dsp Graph with some minimum capacity.
    ///
    /// - **nodes** is the capacity for the underlying **Dag**'s node `Vec`.
    /// - **connections** is the capacity for the underlying **Dag**'s edge `Vec`.
    /// - **frames_per_buffer** is the capacity for the **Graph**'s `dry_buffer`, which is used
    /// for mixing the dry and wet signals when `Node::audio_requested` is called.
    pub fn with_capacity(nodes: usize, connections: usize, frames_per_buffer: usize) -> Self {
        Graph {
            dag: daggy::Dag::with_capacity(nodes, connections),
            visit_order: Vec::with_capacity(nodes),
            dry_buffer: Vec::with_capacity(frames_per_buffer),
            maybe_master: None,
        }
    }

    /// A reference to the underlying **Dag**.
    pub fn dag(&self) -> &Dag<F, N> {
        &self.dag
    }

    /// Takes ownership of the **Graph** and returns the underlying **Dag**.
    pub fn into_dag(self) -> Dag<F, N> {
        let Graph { dag, .. } = self;
        dag
    }

    /// A reference to the internal **Dag**'s underlying **PetGraph**.
    pub fn pet_graph(&self) -> &PetGraph<F, N> {
        self.dag.graph()
    }

    /// Takes ownership of the **Graph** and returns the internal **Dag**'s underlying **PetGraph**.
    pub fn into_pet_graph(self) -> PetGraph<F, N> {
        self.into_dag().into_graph()
    }

    /// The total number of nodes in the **Graph**.
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    /// The total number of connections in the **Graph**.
    pub fn connection_count(&self) -> usize {
        self.dag.edge_count()
    }

    /// Return the **Graph**'s master index if there is one.
    ///
    /// **Graph**'s **Node** implementation will request audio from the node at `maybe_master`
    /// when the `Node::audio_requested` method is called.
    pub fn master_index(&self) -> Option<NodeIndex> {
        self.maybe_master
    }

    /// Set the master node for the **Graph**.
    ///
    /// **Graph** will check to see if a node exists for the given index before assigning.
    ///
    /// **Graph**'s **Node** implementation will request audio from the node at `maybe_master`
    /// when the `Node::audio_requested` method is called.
    pub fn set_master(&mut self, maybe_index: Option<NodeIndex>) {
        let maybe_index = maybe_index.and_then(|index| {
            if self.dag.node_weight(index).is_some() {
                Some(index)
            } else {
                None
            }
        });
        self.maybe_master = maybe_index;
        self.prepare_visit_order();
    }

    /// Add a node to the dsp graph.
    ///
    /// This computes in **O(1)** time.
    pub fn add_node(&mut self, node: N) -> NodeIndex {
        let idx = self.dag.add_node(node);
        idx
    }

    /// A reference to the node at the given index (or `None` if it doesn't exist).
    pub fn node(&self, node: NodeIndex) -> Option<&N> {
        self.dag.node_weight(node)
    }

    /// A mutable reference to the node at the given index (or `None` if it doesn't exist).
    pub fn node_mut(&mut self, node: NodeIndex) -> Option<&mut N> {
        self.dag.node_weight_mut(node)
    }

    /// Read only access to the internal node array.
    pub fn raw_nodes(&self) -> RawNodes<N> {
        self.dag.raw_nodes()
    }

    /// An iterator yielding mutable access to all nodes.
    ///
    /// The order in which nodes are yielded matches the order of their indices.
    pub fn nodes_mut(&mut self) -> NodesMut<N> {
        self.dag.node_weights_mut()
    }

    /// A reference to the connection at the given index (or `None` if it doesn't exist).
    pub fn connection(&self, edge: EdgeIndex) -> Option<&Connection<F>> {
        self.dag.edge_weight(edge)
    }

    /// Read only access to the internal edge array.
    pub fn raw_edges(&self) -> RawEdges<F> {
        self.dag.raw_edges()
    }

    /// Index the **Graph** by two `NodeIndex`s at once.
    ///
    /// **Panics** if the indices are equal or if they are out of bounds.
    pub fn index_twice_mut(&mut self, a: NodeIndex, b: NodeIndex) -> (&mut N, &mut N) {
        self.dag.index_twice_mut(a, b)
    }

    /// Remove a node from the dsp graph.
    ///
    /// Resets the master to None if the index matches the current master index.
    ///
    /// **Note:** This method may shift (and in turn invalidate) previously returned node indices!
    ///
    /// **Graph** will re-prepare its visit order if some node was removed.
    pub fn remove_node(&mut self, idx: NodeIndex) -> Option<N> {
        if self.maybe_master == Some(idx) {
            self.maybe_master = None;
        }
        self.dag.remove_node(idx).map(|node| {
            self.prepare_visit_order();
            node
        })
    }

    /// Adds an edge from `src` to `dest`. That is, `src` is now an input to `dest`.
    ///
    /// Returns an error instead if the input would create a cycle in the graph.
    ///
    /// **Graph** will re-prepare its visit order if some connection was successfully added.
    ///
    /// If you're using `add_node` followed by this method, consider using
    /// [`add_input`](./struct.Graph.html#method.add_input) or
    /// [`add_output`](./struct.Graph.html#method.add_output) instead for greater performance.
    /// This is because when adding a new node and edge simultaneously, we don't have to check
    /// whether adding the edge would create a cycle.
    ///
    /// **Panics** if there is no node for either `src` or `dest`.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index.
    pub fn add_connection(
        &mut self,
        src: NodeIndex,
        dest: NodeIndex,
    ) -> Result<EdgeIndex, WouldCycle> {
        self.dag
            .add_edge(src, dest, Connection { buffer: Vec::new() })
            .map(|edge| {
                self.prepare_visit_order();
                edge
            })
            .map_err(|_| WouldCycle)
    }

    /// The same as [`add_connection`](./struct.Graph.html#method.add_connection) but adds
    /// multiple connections to the **Graph**. Rather than checking for introduced cycles and
    /// re-preparing the visit order after adding each edge, we only do so after **all** edges are
    /// added. Thus, this is a far more efficient alternative to repeatedly calling the
    /// `add_connection` method.
    ///
    /// Returns an error instead if any of the connections would create a cycle in the graph.
    ///
    /// **Graph** will re-prepare its visit order if the connections were successfully added.
    ///
    /// If you're using `add_node` followed by this method, consider using
    /// [`add_input`](./struct.Graph.html#method.add_input) or
    /// [`add_output`](./struct.Graph.html#method.add_output) instead for greater performance.
    /// This is because when adding a new node and edge simultaneously, we don't have to check
    /// whether adding the edge would create a cycle.
    ///
    /// **Panics** if there is no node for either `src` or `dest`.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index.
    pub fn add_connections<I>(&mut self, connections: I) -> Result<EdgeIndices, WouldCycle>
    where
        I: ::std::iter::IntoIterator<Item = (NodeIndex, NodeIndex)>,
    {
        fn new_connection<F>() -> Connection<F> {
            Connection { buffer: Vec::new() }
        }
        self.dag
            .add_edges(
                connections
                    .into_iter()
                    .map(|(src, dest)| (src, dest, new_connection())),
            )
            .map(|edges| {
                self.prepare_visit_order();
                edges
            })
            .map_err(|_| WouldCycle)
    }

    /// Find and return the index to the edge that describes the connection where `src` is an input
    /// to `dest`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges connected to the nodes `a`
    /// and `b`.
    pub fn find_connection(&self, src: NodeIndex, dest: NodeIndex) -> Option<EdgeIndex> {
        self.dag.find_edge(src, dest)
    }

    /// Remove the connection described by the edge at the given index.
    ///
    /// Returns true if an edge was removed, returns false if there was no edge at the given index.
    ///
    /// Re-prepares the visit order if some edge was removed.
    pub fn remove_edge(&mut self, edge: EdgeIndex) -> bool {
        if self.dag.remove_edge(edge).is_some() {
            self.prepare_visit_order();
            true
        } else {
            false
        }
    }

    /// Find and remove any connection between a and b if there is one, whether it is *a -> b* or
    /// *b -> a*. We know that their may only be one edge as our API does not allow for creating a
    /// cyclic graph.
    ///
    /// Returns true if an edge was removed, returns false if there was no edge at the given index.
    ///
    /// Graph will re-prepare its visit order if some edge was removed.
    ///
    /// Note: If you have an index to the edge you want to remove,
    /// [`remove_edge`](./struct.Graph.html#method.remove_edge) is a more performant option.
    pub fn remove_connection(&mut self, a: NodeIndex, b: NodeIndex) -> bool {
        match self
            .dag
            .find_edge(a, b)
            .or_else(|| self.dag.find_edge(b, a))
        {
            Some(edge) => self.remove_edge(edge),
            None => false,
        }
    }

    /// Add a new node weight to the graph as an input to the wait at the given `dest` node index.
    ///
    /// *src -> new edge -> dest*
    ///
    /// Returns an index to both the new `src` node and the edge that represents the new connection
    /// between it and the node at `dest`.
    ///
    /// Computes in **O(n)** time where n is the number of nodes. This is because must update the
    /// visit order after adding the new connection.
    ///
    /// **Panics** if there is no node for the given `dest` index.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index.
    pub fn add_input(&mut self, src: N, dest: NodeIndex) -> (EdgeIndex, NodeIndex) {
        let indices = self
            .dag
            .add_parent(dest, Connection { buffer: Vec::new() }, src);
        self.prepare_visit_order();
        indices
    }

    /// Add a new node weight to the graph as an output to the wait at the given `src` node index.
    ///
    /// *src -> new edge -> dest*
    ///
    /// Returns an index to both the new `dest` node and the edge that represents the new connection
    /// between it and the node at `src`.
    ///
    /// Computes in **O(n)** time where n is the number of nodes. This is because must update the
    /// visit order after adding the new connection.
    ///
    /// **Panics** if there is no node for the given `dest` index.
    ///
    /// **Panics** if the Graph is at the maximum number of edges for its index.
    pub fn add_output(&mut self, src: NodeIndex, dest: N) -> (EdgeIndex, NodeIndex) {
        let indices = self
            .dag
            .add_child(src, Connection { buffer: Vec::new() }, dest);
        self.prepare_visit_order();
        indices
    }

    /// A "walker" object that may be used to step through the inputs of the given node.
    ///
    /// Unlike the `Inputs` type, `WalkInputs` does not borrow the `Graph`.
    ///
    /// Can be converted to an iterator using `.iter()`.
    pub fn inputs(&self, idx: NodeIndex) -> Inputs<F, N> {
        Inputs {
            parents: self.dag.parents(idx),
        }
    }

    /// A "walker" object that may be used to step through the outputs of the given node.
    ///
    /// Unlike the `Outputs` type, `WalkOutputs` does not borrow the **Graph**.
    ///
    /// Can be converted to an iterator using `.iter()`.
    pub fn outputs(&self, idx: NodeIndex) -> Outputs<F, N> {
        Outputs {
            children: self.dag.children(idx),
        }
    }

    /// A "walker" type that may be used to step through all node indices in the order in which
    /// they will be visited when audio is requested from the **Graph**.
    pub fn visit_order(&self) -> VisitOrder {
        VisitOrder {
            current_visit_order_idx: 0,
        }
    }

    /// A "walker" type that may be used to step through all node indices in the order in which
    /// they will be visited when audio is requested from the **Graph**.
    ///
    /// Unlike the VisitOrder type, VisitOrder does not borrow the **Graph**.
    pub fn visit_order_rev(&self) -> VisitOrderReverse {
        VisitOrderReverse {
            current_visit_order_idx: self.visit_order.len(),
        }
    }

    /// Remove all incoming connections to the node at the given index.
    ///
    /// Return the number of connections removed.
    pub fn remove_all_input_connections(&mut self, idx: NodeIndex) -> usize {
        let mut inputs = self.inputs(idx);
        let mut num = 0;
        while let Some(connection) = inputs.next_edge(&self) {
            self.remove_edge(connection);
            num += 1;
        }
        num
    }

    /// Remove all outgoing connections from the node at the given index.
    ///
    /// Return the number of connections removed.
    pub fn remove_all_output_connections(&mut self, idx: NodeIndex) -> usize {
        let mut outputs = self.outputs(idx);
        let mut num = 0;
        while let Some(connection) = outputs.next_edge(&self) {
            self.remove_edge(connection);
            num += 1;
        }
        num
    }

    /// Clear all dsp nodes that have no inputs or outputs.
    ///
    /// Returns the number of nodes removed.
    ///
    /// Note: this may shift (and in turn invalidate) previously returned node and edge indices!
    pub fn clear_disconnected(&mut self) -> usize {
        let mut num_removed = 0;
        for i in 0..self.dag.node_count() {
            let idx = NodeIndex::new(i);
            let num_inputs = self.inputs(idx).count(self);
            let num_outputs = self.outputs(idx).count(self);
            if num_inputs == 0 && num_outputs == 0 {
                if self.maybe_master == Some(idx) {
                    self.maybe_master = None;
                }
                self.dag.remove_node(idx);
                num_removed += 1;
            }
        }
        num_removed
    }

    /// Clear all dsp nodes.
    pub fn clear(&mut self) {
        self.dag.clear();
        self.visit_order.clear();
        self.maybe_master = None;
    }

    /// Prepare the buffers for all nodes within the Graph.
    pub fn prepare_buffers(&mut self, buffer_size: usize) {
        // Initialise the dry signal buffer.
        resize_buffer_to(&mut self.dry_buffer, buffer_size);

        // Initialise all connection buffers.
        for connection in self.dag.edge_weights_mut() {
            resize_buffer_to(&mut connection.buffer, buffer_size);
        }
    }

    /// Request audio from the node at the given index.
    ///
    /// **Panics** if there is no node for the given index.
    pub fn audio_requested_from(&mut self, out_node: NodeIndex, output: &mut [F], sample_hz: f64) {
        // We can only go on if a node actually exists for the given index.
        if self.node(out_node).is_none() {
            panic!("No node for the given index");
        }

        let buffer_size = output.len();

        // Ensure the dry_buffer is the same length as the output buffer.
        if self.dry_buffer.len() != buffer_size {
            resize_buffer_to(&mut self.dry_buffer, buffer_size);
        }

        let mut visit_order = self.visit_order();
        while let Some(node_idx) = visit_order.next(self) {
            // Set the buffers to equilibrium, ready to sum the inputs of the current node.
            for i in 0..buffer_size {
                output[i] = F::EQUILIBRIUM;
                self.dry_buffer[i] = F::EQUILIBRIUM;
            }

            // Walk over each of the input connections to sum their buffers to the output.
            let mut inputs = self.inputs(node_idx);
            while let Some(connection_idx) = inputs.next_edge(self) {
                let connection = &self[connection_idx];
                // Sum the connection's buffer onto the output.
                //
                // We can be certain that `connection`'s buffer is the same size as the
                // `output` buffer as all connections are visited from their input nodes
                // (towards the end of the visit_order while loop) before being visited here
                // by their output nodes.
                dasp::slice::zip_map_in_place(
                    output,
                    &connection.buffer,
                    |out_frame, con_frame| {
                        out_frame.zip_map(con_frame, |out_sample, con_sample| {
                            let out_signed =
                                out_sample.to_sample::<<F::Sample as Sample>::Signed>();
                            let con_signed =
                                con_sample.to_sample::<<F::Sample as Sample>::Signed>();
                            (out_signed + con_signed).to_sample::<F::Sample>()
                        })
                    },
                );
            }

            // Store the dry signal in the dry buffer for later summing.
            dasp::slice::write(&mut self.dry_buffer, output);

            // Render the audio with the current node and sum the dry and wet signals.
            let (dry, wet) = {
                let node = &mut self[node_idx];

                // Render our `output` buffer with the current node.
                // The `output` buffer is now representative of a fully wet signal.
                node.audio_requested(output, sample_hz);

                let dry = node.dry();
                let wet = node.wet();
                (dry, wet)
            };

            // Combine the dry and wet signals.
            dasp::slice::zip_map_in_place(output, &self.dry_buffer, |f_wet, f_dry| {
                f_wet.zip_map(f_dry, |s_wet, s_dry| {
                    let wet = s_wet.mul_amp(wet);
                    let dry = s_dry.mul_amp(dry);
                    wet.add_amp(dry.to_sample())
                })
            });

            // If we've reached our output node, we're done!
            if node_idx == out_node {
                return;
            }

            // Walk over each of the outgoing connections and write the rendered output to them.
            let mut outputs = self.outputs(node_idx);
            while let Some(connection_idx) = outputs.next_edge(self) {
                let connection = &mut self.dag[connection_idx];

                // Ensure the buffer matches the target length.
                if connection.buffer.len() != output.len() {
                    resize_buffer_to(&mut connection.buffer, output.len());
                }

                // Write the rendered audio to the outgoing connection buffers.
                dasp::slice::write(&mut connection.buffer, output);
            }
        }
    }

    /// Prepare the visit order for the graph in its current state.
    ///
    /// This is called whenever the **Graph** is mutated in some way that may change the flow of
    /// its edges.
    ///
    /// When audio is requested from the graph, we need to iterate through all nodes so that all
    /// child nodes are visited before their parents. To do this, we can use petgraph's toposort
    /// algorithm to return the topological order of our graph.
    ///
    /// The user should never have to worry about this, thus the method is private.
    fn prepare_visit_order(&mut self) {
        self.visit_order = daggy::petgraph::algo::toposort(self.dag.graph());
    }
}

impl<F, N> ::std::ops::Index<NodeIndex> for Graph<F, N> {
    type Output = N;
    #[inline]
    fn index<'a>(&'a self, index: NodeIndex) -> &'a N {
        &self.dag[index]
    }
}

impl<F, N> ::std::ops::IndexMut<NodeIndex> for Graph<F, N> {
    #[inline]
    fn index_mut(&mut self, index: NodeIndex) -> &mut N {
        &mut self.dag[index]
    }
}

impl<F, N> ::std::ops::Index<EdgeIndex> for Graph<F, N> {
    type Output = Connection<F>;
    #[inline]
    fn index<'a>(&'a self, index: EdgeIndex) -> &'a Connection<F> {
        &self.dag[index]
    }
}

impl<F, N> Node<F> for Graph<F, N>
where
    F: Frame,
    N: Node<F>,
{
    fn audio_requested(&mut self, output: &mut [F], sample_hz: f64) {
        match self.maybe_master {
            Some(master) => self.audio_requested_from(master, output, sample_hz),
            None => {
                // If there is no set master node, we'll start from the back of the visit_order and
                // use the first node that has no output connections.
                let mut visit_order_rev = self.visit_order_rev();
                while let Some(node) = visit_order_rev.next(self) {
                    if self.inputs(node).count(self) == 0 {
                        self.audio_requested_from(node, output, sample_hz);
                        return;
                    }
                }
            }
        }
    }
}

impl<F, N> Walker<Graph<F, N>> for Inputs<F, N> {
    type Index = usize;

    /// The next (connection, node) input pair to some node in our walk for the given **Graph**.
    #[inline]
    fn next(&mut self, graph: &Graph<F, N>) -> Option<(EdgeIndex, NodeIndex)> {
        self.parents.next(&graph.dag)
    }

    /// The next input connection to some node in our walk for the given **Graph**.
    #[inline]
    fn next_edge(&mut self, graph: &Graph<F, N>) -> Option<EdgeIndex> {
        self.parents.next_edge(&graph.dag)
    }

    /// The next input node to some node in our walk for the given **Graph**.
    #[inline]
    fn next_node(&mut self, graph: &Graph<F, N>) -> Option<NodeIndex> {
        self.parents.next_node(&graph.dag)
    }
}

impl<F, N> Walker<Graph<F, N>> for Outputs<F, N> {
    type Index = usize;

    /// The next (connection, node) output pair from some node in our walk for the given **Graph**.
    #[inline]
    fn next(&mut self, graph: &Graph<F, N>) -> Option<(EdgeIndex, NodeIndex)> {
        self.children.next(&graph.dag)
    }

    /// The next output connection from some node in our walk for the given **Graph**.
    #[inline]
    fn next_edge(&mut self, graph: &Graph<F, N>) -> Option<EdgeIndex> {
        self.children.next_edge(&graph.dag)
    }

    /// The next output node from some node in our walk for the given **Graph**.
    #[inline]
    fn next_node(&mut self, graph: &Graph<F, N>) -> Option<NodeIndex> {
        self.children.next_node(&graph.dag)
    }
}

impl VisitOrder {
    /// The index of the next node that would be visited during audio requested in our walk of the
    /// given **Graph**'s visit order.
    #[inline]
    pub fn next<F, N>(&mut self, graph: &Graph<F, N>) -> Option<NodeIndex> {
        graph
            .visit_order
            .get(self.current_visit_order_idx)
            .map(|&idx| {
                self.current_visit_order_idx += 1;
                idx
            })
    }
}

impl VisitOrderReverse {
    /// The index of the next node that would be visited during audio requested in our walk of the
    /// given **Graph**'s visit order.
    #[inline]
    pub fn next<F, N>(&mut self, graph: &Graph<F, N>) -> Option<NodeIndex> {
        if self.current_visit_order_idx > 0 {
            self.current_visit_order_idx -= 1;
            graph
                .visit_order
                .get(self.current_visit_order_idx)
                .map(|&idx| idx)
        } else {
            None
        }
    }
}

/// Resize the given buffer to the given target length.
fn resize_buffer_to<F>(buffer: &mut Vec<F>, target_len: usize)
where
    F: Frame,
{
    let len = buffer.len();
    if len < target_len {
        buffer.extend((len..target_len).map(|_| F::EQUILIBRIUM))
    } else if len > target_len {
        buffer.truncate(target_len);
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
