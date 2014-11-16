#![crate_name = "dsp"]
#![deny(missing_docs)]
#![feature(macro_rules, phase, globs, linkage)]

//! A pure Rust audio digital signal processing library for Piston.

extern crate portaudio;
extern crate time;
extern crate serialize;

pub use buffer::DspBuffer;
pub use node::Node as Node;
pub use node::Data as NodeData;
pub use sound_stream::SoundStream as SoundStream;
pub use sound_stream_settings::SoundStreamSettings as SoundStreamSettings;

pub mod macros;

mod buffer;
mod node;
mod port_audio_back_end;
mod sound_stream;
mod sound_stream_settings;
