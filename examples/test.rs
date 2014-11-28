//! Test app for rust-dsp.
//!
//! This file will normally be in a strange state;
//! it's really just a testing ground for the new
//! features as I add them. I'll get around to making
//! some proper examples soon!
//!

extern crate dsp;

use dsp::{
    DspBuffer,
    Node,
    SoundStream,
    SoundStreamError,
    Settings,
};

/// We'll use these values for setting
/// up our SoundStream.
///
/// Note: FRAMES == (fixed-size buffer length / CHANNELS)
const SAMPLE_RATE: u32 = 44100;
const FRAMES: u16 = 256;
const CHANNELS: u16 = 2;

/// Choose a fixed-size buffer of f32 with a length
/// matching FRAMES * CHANNELS.
pub type AudioBuffer = [f32, .. FRAMES as uint * CHANNELS as uint];

fn main() {}

