#![crate_id = "dsp"]
#![deny(missing_doc)]
#![feature(macro_rules)]
#![feature(phase)]
#![feature(globs)]

//! A pure Rust audio digital signal processing library for Piston.

extern crate portaudio;
extern crate time;

pub use Envelope = envelope::Envelope;
pub use EnvPoint = envelope::Point;
pub use Frequency = frequency::Frequency;
pub use HasFrequency = frequency::HasFrequency;
pub use Node = node::Node;
pub use IsNode = node::IsNode;
pub use Oscillator = oscillator::Oscillator;
pub use Pitch = pitch::Pitch;
pub use Letter = pitch::letter::Letter;
pub use HasPitch = pitch::HasPitch;
pub use Signal = signal::Signal;
pub use SoundStream = sound_stream::SoundStream;
pub use SoundStreamSettings = sound_stream_settings::SoundStreamSettings;
pub use Waveform = waveform::Waveform;

mod envelope;
mod frequency;
mod math;
mod node;
mod oscillator;
mod pitch;
mod port_audio_back_end;
mod signal;
mod sound_stream;
mod sound_stream_settings;
mod waveform;

