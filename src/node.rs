
use {Panning, Volume};
use dsp::Dsp;
use sound_stream::{Amplitude, Sample, Settings};

/// DSP Node trait. Implement this for any audio instrument or effects types that are to be used
/// within your DSP chain. Override all methods that you wish. If the Node is a parent of other
/// DSP nodes, be sure to implement the `inputs` method.
pub trait Node<S> where S: Sample {

    /// Return the volume for this Node.
    #[inline]
    fn vol(&self) -> Volume { 1.0 }

    /// Return the panning for this Node.
    /// -1.0 = Left.
    ///  0.0 = Center.
    ///  1.0 = Right.
    #[inline]
    fn pan(&self) -> Panning { 0.0 }

    /// Return mutable references to the inputs for the Node.
    /// TODO: Once "Abstract Return Types" land in Rust, we'll
    /// change this to return `impl Iterator<&mut Node<B, O>>`
    /// so that we don't have to allocate *anything* in the
    /// whole graph.
    #[inline]
    fn inputs(&mut self) -> Vec<&mut Node<S>> { Vec::new() }

    /// Determine the volume for each channel by considering
    /// both `vol` and `pan. In the future this will be
    /// replaced with an `n` channels method.
    #[inline]
    fn vol_per_channel(&self) -> [Amplitude; 2] {
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
    fn audio_requested(&mut self, output: &mut [S], settings: Settings) {
        let buffer_size = settings.buffer_size();
        let vol_per_channel = self.vol_per_channel();
        for input in self.inputs() {
            let mut working = vec![Sample::zero(); buffer_size];
            // Call audio_requested for each input.
            input.audio_requested(&mut working[..], settings);
            Sample::add_buffers(output, &working[..], &vol_per_channel[..]);
        }
        // Custom buffer processing.
        self.process_buffer(output, settings);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    #[inline]
    fn process_buffer(&mut self, _output: &mut [S], _settings: Settings) {}

}

