//! A module for experimenting with sound generators in the form of Iterators.

use sound_stream::{Sample, Settings};

/// Dsp trait. Implement this for any audio instrument or effects types that are to be used
/// within your DSP Graph. Override all methods that you wish.
pub trait Dsp<S> where S: Sample {
    /// Used to generate audio samples.
    type Generator: Iterator<Item=S>;

    /// Return an iterator that would produce
    /// samples for a buffer with the given settings.
    #[inline]
    fn sample_generator<I>(&mut self, input_samples: I, settings: Settings) -> Self::Generator
        where I: Iterator<Item=S>;

}


