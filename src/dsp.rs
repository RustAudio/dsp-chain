
use {Panning, Volume};
use sound_stream::{Amplitude, Sample, Settings};

/// Dsp trait. Implement this for any audio instrument or effects types that are to be used
/// within your DSP Graph. Override all methods that you wish.
pub trait Dsp<S> where S: Sample {

    /// Return the volume for this Node.
    #[inline]
    fn vol(&self) -> Volume { 1.0 }

    /// Return the panning for this Node.
    /// -1.0 = Left.
    ///  0.0 = Center.
    ///  1.0 = Right.
    #[inline]
    fn pan(&self) -> Panning { 0.0 }

    /// Determine the volume for each channel by considering
    /// both `vol` and `pan. In the future this will be
    /// replaced with an `n` channels method.
    #[inline]
    fn vol_per_channel(&self) -> [Amplitude; 2] {
        use std::num::Float;
        if self.pan() >= 0.0 {
            [self.vol() * (self.pan() - 1.0).abs(), self.vol()]
        } else {
            [self.vol(), self.vol() * (self.pan() + 1.0)]
        }
    }

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    #[inline]
    fn audio_requested(&mut self, _output: &mut [S], _settings: Settings) {}

}

