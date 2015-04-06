#![deny(missing_docs)]
#![feature(collections)]

//! 
//! A pure Rust audio digital signal processing library for Piston.
//!
//! There are two primary modules of interest within this library, both of which
//! are unrelated and are designed to be used separately.
//! 1. node.rs and the `Node` trait.
//! 2. graph.rs and the `Graph` type.
//!
//! The `Node` trait offers a DSP chaining design via its `inputs` method. It is
//! slightly simpler to use than the `Graph` type however also slightly more limited.
//! Using the `Node` trait, it is impossible for two nodes to reference the same
//! input `Node` making it difficult to perform "bussing" and "side-chaining".
//!
//! The `Graph` type constructs a directed, acyclic graph of DSP nodes. It is
//! the recommended approach for more advanced DSP chains that involve things like
//! "bussing", "side-chaining" or more DAW-esque behaviour. The `Graph` type requires
//! its nodes to have implemented the `Dsp` trait (a slightly simplified version of the
//! `Node` trait, though entirely unrelated). Internally, `Graph` uses bluss's petgraph
//! crate. See more [here](https://crates.io/crates/petgraph).
//!

extern crate num;
extern crate petgraph;
extern crate sound_stream;

pub use dsp::Dsp;
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
    Amplitude,
    Event,
    PaSample,
    Sample,
    Settings,
    SoundStream,
    Wave
};
pub use sound_stream::Error as SoundStreamError;

mod dsp;
mod graph;
pub mod iter;
mod node;

/// The amplitude multiplier.
pub type Volume = Amplitude;

/// The spacial positioning of the node. Currently only supports Stereo or Mono.
/// -1.0 = Left.
///  0.0 = Center.
///  1.0 = Right.
pub type Panning = f32;

