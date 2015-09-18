//! 
//! A pure Rust audio digital signal processing library for Piston.
//!
//! There are two primary modules of interest within this library.
//! 1. graph.rs and the `Graph` type.
//! 2. node.rs and the `Node` trait.
//!

#![deny(missing_docs)]

extern crate num;
extern crate petgraph;
extern crate sound_stream;

pub use graph::{
    Graph,
    NodeIndex,
    Inputs,
    InputsWithIndices,
    InputsMut,
    InputsMutWithIndices,
    Outputs,
    OutputsWithIndices,
    OutputsMut,
    OutputsMutWithIndices,
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

