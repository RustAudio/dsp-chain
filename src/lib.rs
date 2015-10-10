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
extern crate num;
extern crate sound_stream as sound_stream_lib;

pub use daggy_lib as daggy;
pub use sound_stream_lib as sound_stream;

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
    WalkInputs,
    WalkOutputs,
    WalkVisitOrder,
    WalkVisitOrderReverse,
    WouldCycle,
};
pub use node::Node;
pub use sound_stream::{
    portaudio,
    input,
    output,
    duplex,
    Amplitude,
    CallbackFlags,
    CallbackResult,
    DeltaTimeSeconds,
    Latency,
    PaSample,
    PaStream,
    Sample,
    Settings,
    SoundStream,
    StreamFlags,
    StreamParams,
    Wave
};
pub use sound_stream::Error as SoundStreamError;

mod graph;
mod node;

/// The amplitude multiplier.
pub type Volume = Amplitude;

/// The spacial positioning of the node. Currently only supports Stereo or Mono.
/// -1.0 = Left.
///  0.0 = Center.
///  1.0 = Right.
pub type Panning = f32;

