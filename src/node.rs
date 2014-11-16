
use buffer::DspBuffer;
use sound_stream_settings::SoundStreamSettings;
use std::num::Float;

/// The DSP Node contains a vector of children
/// nodes (within the `MixerInput`s), from which
/// audio can be requested as well as the current
/// SoundStream settings.
#[deriving(Show, Clone, Encodable, Decodable)]
pub struct Data {
    /// SoundStreamSettings for buffer calculations.
    pub settings: SoundStreamSettings,
    /// Master volume for DSP node.
    pub vol: f32,
    /// Panning for DSP node.
    /// -1.0 = Left.
    ///  0.0 = Center.
    ///  1.0 = Right.
    pub pan: f32,
}

impl Data {
    /// Default constructor for a Data struct.
    pub fn new(settings: SoundStreamSettings) -> Data {
        Data {
            settings: settings,
            vol: 1f32,
            pan: 0f32,
        }
    }
}

/// DSP Node trait. Implement this for any audio
/// instrument or effects types. Be sure to add
/// the `Node` struct to a field as well, and
/// override the `get_node` and `get_node_mut`
/// methods by returning a ref (/mut) to it.
pub trait Node<B: DspBuffer> {

    /// Return an immutable reference a Data struct owned by the Node.
    fn get_node_data(&self) -> &Data;
    /// Return a mutable reference to a Data struct owned by the Node.
    fn get_node_data_mut(&mut self) -> &mut Data;
    /// Return a reference to the inputs for the Node.
    fn get_inputs<'a>(&'a self) -> Vec<&'a Node<B>> { Vec::new() }
    /// Return a mutable reference to the inputs for the Node.
    fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut Node<B>> { Vec::new() }

    /// Apply settings to self and all inputs.
    fn apply_settings(&mut self, settings: SoundStreamSettings) {
        for input in self.get_inputs_mut().into_iter() {
            input.apply_settings(settings);
        }
        self.get_node_data_mut().settings = settings;
    }

    /// Determine the volume for each channel by considering
    /// both `vol` and `pan. In the future this will be
    /// replaced with an `n` channels method.
    fn vol_per_channel(&self) -> [f32, ..2] {
        if self.get_node_data().pan >= 0.0 {
            [self.get_node_data().vol * (self.get_node_data().pan - 1.0).abs(),
             self.get_node_data().vol]
        } else {
            [self.get_node_data().vol,
             self.get_node_data().vol * (self.get_node_data().pan + 1.0)]
        }
    }

    /// Receive incoming audio (override this
    /// to do something with the input).
    fn audio_received(&mut self, _input: &B) {}

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    fn audio_requested(&mut self, output: &mut B) {
        let frames = self.get_node_data().settings.frames as uint;
        let channels = self.get_node_data().settings.channels as uint;
        let vol_per_channel = self.vol_per_channel();
        for input in self.get_inputs_mut().into_iter() {
            let mut working: B = DspBuffer::zeroed();
            // Call audio_requested for each input.
            input.audio_requested(&mut working);
            // Sum all input nodes to output (considering pan, vol and interleaving).
            for i in range(0, frames) {
                for j in range(0, channels) {
                    *output.get_mut(i * channels + j) +=
                        working.val(i * channels + j) * vol_per_channel[j];
                }
            }
        }
        // Custom buffer processing.
        self.process_buffer(output);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    fn process_buffer(&mut self, _output: &mut B) {}

}

