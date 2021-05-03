//!
//! A generic, fast, audio digital signal processing library.
//!
//! There are two primary points of interest:
//!
//! 1. The [**Graph** type](./graph/struct.Graph.html) - a directed, acyclic audio DSP graph.
//!
//! 2. The [**Node** trait](./node/trait.Node.html) - to be implemented for types used within the
//!    **Graph**.
//!

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub use daggy::{self, Walker};
pub use dasp::{
    self, interpolate,
    sample::{conv, Duplex as DuplexSample, FromSample, Sample, ToSample},
    signal, slice, Frame, Signal,
};
pub use graph::{
    Connection, Dag, EdgeIndex, Graph, Inputs, NodeIndex, NodesMut, Outputs, PetGraph, RawEdges,
    RawNodes, VisitOrder, VisitOrderReverse, WouldCycle,
};
pub use node::Node;

mod graph;
mod node;

/// The amplitude multiplier.
pub type Volume = f32;

/// The spacial positioning of the node. Currently only supports Stereo or Mono.
/// -1.0 = Left.
///  0.0 = Center.
///  1.0 = Right.
pub type Panning = f32;
