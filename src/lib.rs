#![crate_name = "dsp"]
#![deny(missing_docs)]
#![feature(macro_rules)]

//! A pure Rust audio digital signal processing library for Piston.

extern crate serialize;
extern crate sound_stream;

pub use buffer::DspBuffer;
pub use node::Node;
pub use node::Volume;
pub use node::Panning;
pub use sound_stream::{AudioBuffer, Event, Sample, Settings, SoundStream};
pub use sound_stream::Error as SoundStreamError;

mod buffer;
mod node;

