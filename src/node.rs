

use sound_stream_settings::SoundStreamSettings;

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
    pub pan: f32
}

impl Data {
    /// Default constructor for a Data struct.
    pub fn new(settings: SoundStreamSettings) -> Data {
        Data {
            settings: settings,
            vol: 1f32,
            pan: 0.0f32,
        }
    }
}

/// DSP Node trait. Implement this for any audio
/// instrument or effects types. Be sure to add
/// the `Node` struct to a field as well, and
/// override the `get_node` and `get_node_mut`
/// methods by returning a ref (/mut) to it.
pub trait Node {

    /// Return a reference a Data struct owned by the Node.
    fn get_node_data<'a>(&'a mut self) -> &'a mut Data;
    /// Return a reference to the inputs for the Node.
    fn get_inputs<'a>(&'a self) -> Vec<&'a Node> { Vec::new() }
    /// Return a mutable reference to the inputs for the Node.
    fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut Node> { Vec::new() }

    /// Apply settings to self and all inputs.
    fn apply_settings(&mut self, settings: SoundStreamSettings) {
        for input in self.get_inputs_mut().into_iter() {
            input.apply_settings(settings);
        }
        self.get_node_data().settings = settings;
    }

    /// Receive incoming audio (override this
    /// to do something with the input).
    fn audio_received(&mut self, _input: &Vec<f32>) {}

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    fn audio_requested<'a>(&'a mut self, output: &mut Vec<f32>) {
        let frames = self.get_node_data().settings.frames as uint;
        let channels = self.get_node_data().settings.channels as uint;
        {
            let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
            for input in self.get_inputs_mut().into_iter() {
                let input_vol = input.get_node_data().vol;
                let input_pan = input.get_node_data().pan;
                let mut working: Vec<f32> = Vec::from_elem(frames * channels, 0f32);
                // Call audio_requested for each input.
                input.audio_requested(&mut working);
                // Construct precalculated volume and
                // pan array (for efficiency).
                let (mut vol_l, mut vol_r) = (input_vol, input_vol);
                if input_pan >= 0.0 {
                    vol_l *= (input_pan - 1.0) + 1.0;
                }
                else {
                    vol_r *= input_pan + 1.0;
                }
                for i in range(0, channels) {
                    *vol_per_channel.get_mut(i) = if i == 0 { vol_l } else { vol_r };
                }
                // Sum all input nodes to output (considering
                // pan, vol and interleaving).
                for i in range(0, frames) {
                    for j in range(0, channels) {
                        *output.get_mut(i * channels + j) +=
                            working[i * channels + j] * vol_per_channel[j];
                    }
                }
            }
        }
        // Custom buffer processing.
        self.process_buffer(output);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    fn process_buffer(&mut self, _output: &mut Vec<f32>) {}

}

