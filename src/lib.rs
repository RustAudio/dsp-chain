#![crate_name = "dsp"]
#![deny(missing_doc)]
#![feature(macro_rules, phase, globs, struct_variant)]

//! A pure Rust audio digital signal processing library for Piston.

extern crate portaudio;
extern crate time;
extern crate serialize;

//pub use Node = node_enum::Node;

pub use Node = node::Node;
pub use NodeData = node::NodeData;
pub use SoundStream = sound_stream::SoundStream;
pub use SoundStreamSettings = sound_stream_settings::SoundStreamSettings;

mod node;
mod port_audio_back_end;
mod sound_stream;
mod sound_stream_settings;

