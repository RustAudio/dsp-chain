#![deny(missing_docs)]
#![feature(rustc_private)]

//! A pure Rust audio digital signal processing library for Piston.

extern crate serialize;
extern crate sound_stream;

pub use node::{
    Node,
    Volume, 
    Panning,
};
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

mod node;

