//!
//!  buffer.rs
//!
//!  Created by Mitchell Nordine at 07:27AM on November 29, 2014.
//!
//!

use sound_stream::{AudioBuffer, Sample};

/// A trait used to extend the AudioBuffer for generic use in Node::audio_requested.
pub trait DspBuffer: AudioBuffer {
    /// Return a mutable reference to the sample at the given index.
    fn get_mut(&mut self, idx: usize) -> &mut <Self as AudioBuffer>::Sample;
    /// Return the sample at the given index by value.
    fn val(&self, idx: usize) -> <Self as AudioBuffer>::Sample;
}

impl<S> DspBuffer for Vec<S> where S: Sample {
    fn get_mut(&mut self, idx: usize) -> &mut S { &mut self[idx] }
    fn val(&self, idx: usize) -> S { self[idx] }
}

macro_rules! impl_dsp_buffer(
    ($len:expr) => (
        impl<S> DspBuffer for [S; $len] where S: Sample {
            fn get_mut(&mut self, idx: usize) -> &mut S { &mut self[idx] }
            fn val(&self, idx: usize) -> S { self[idx] }
        }
    )
);

impl_dsp_buffer!(2);
impl_dsp_buffer!(4);
impl_dsp_buffer!(8);
impl_dsp_buffer!(16);
impl_dsp_buffer!(32);
impl_dsp_buffer!(64);
impl_dsp_buffer!(128);
impl_dsp_buffer!(256);
impl_dsp_buffer!(512);
impl_dsp_buffer!(1024);
impl_dsp_buffer!(2048);
impl_dsp_buffer!(4096);
impl_dsp_buffer!(8192);
impl_dsp_buffer!(16384);

