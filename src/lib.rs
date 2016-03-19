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

extern crate daggy as daggy_lib;
extern crate sample as sample_lib;

pub use daggy_lib::{Walker, self as daggy};
pub use graph::{
    Connection,
    Dag,
    EdgeIndex,
    Graph,
    Inputs,
    NodeIndex,
    NodesMut,
    Outputs,
    PetGraph,
    RawEdges,
    RawNodes,
    VisitOrder,
    VisitOrderReverse,
    WouldCycle,
};
pub use node::Node;
pub use sample_lib::{
    self as sample,
    conv,
    rate,
    slice,
    signal,
    Duplex as DuplexSample,
    Frame,
    FromSample,
    ToSample,
    Sample,
    Signal,
};
pub use settings::Settings;

mod graph;
mod node;
mod settings;

/// The amplitude multiplier.
pub type Volume = f32;

/// The spacial positioning of the node. Currently only supports Stereo or Mono.
/// -1.0 = Left.
///  0.0 = Center.
///  1.0 = Right.
pub type Panning = f32;
