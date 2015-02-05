
use buffer::DspBuffer;
use sound_stream::{
    AudioBuffer,
    Sample,
    Settings,
};
use std::num::Float;

/// The amplitude multiplier.
pub type Volume = f32;
/// The spacial positioning of the node. Currently only supports Stereo or Mono.
/// -1.0 = Left.
///  0.0 = Center.
///  1.0 = Right.
pub type Panning = f32;

/// DSP Node trait. Implement this for any audio instrument or effects types that are to be used
/// within your DSP chain. Override all methods that you wish. If the Node is a parent of other
/// DSP nodes, be sure to implement the `inputs` method.
pub trait Node<B> where B: DspBuffer {

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
    fn inputs(&mut self) -> Vec<&mut Node<B>> { Vec::new() }

    /// Determine the volume for each channel by considering
    /// both `vol` and `pan. In the future this will be
    /// replaced with an `n` channels method.
    #[inline]
    fn vol_per_channel(&self) -> [f32; 2] {
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
    fn audio_requested(&mut self, output: &mut B, settings: Settings) {
        let frames = settings.frames as usize;
        let channels = settings.channels as usize;
        let buffer_size = frames * channels;
        let vol_per_channel = self.vol_per_channel();
        for input in self.inputs() {
            let mut working: B = AudioBuffer::zeroed(buffer_size);
            // Call audio_requested for each input.
            input.audio_requested(&mut working, settings);
            // Sum all input nodes to output (considering pan, vol and interleaving).
            for i in range(0, frames) {
                for j in range(0, channels) {
                    use std::num::{ToPrimitive, from_f32};
                    let idx = i * channels + j;
                    let working_f32 = working.val(idx).to_f32().unwrap();
                    let current_f32 = output.val(idx).to_f32().unwrap();
                    let result_f32 = working_f32 * vol_per_channel[j] + current_f32;
                    *output.get_mut(idx) = from_f32(result_f32).unwrap();
                }
            }
        }
        // Custom buffer processing.
        self.process_buffer(output, settings);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    #[inline]
    fn process_buffer(&mut self, _output: &mut B, _settings: Settings) {}

}

