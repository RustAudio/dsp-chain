
use sound_stream_settings::SoundStreamSettings;
use std::fmt::Show;

/// The DSP Node contains a vector of children
/// nodes (within the `MixerInput`s), from which
/// audio can be requested as well as the current
/// SoundStream settings.
#[deriving(Show, Clone)]
pub struct NodeData {
    /// SoundStreamSettings for buffer calculations.
    pub settings: SoundStreamSettings,
    /// Master volume for DSP node.
    pub vol: f32,
    /// Panning for DSP node.
    pub pan: f32
}

impl NodeData {
    /// Default constructor for a NodeData struct.
    pub fn new(settings: SoundStreamSettings) -> NodeData {
        NodeData {
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
pub trait Node: Clone + Show {

    /// Return a reference a NodeData struct owned by the Node.
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData;
    /// Return a reference to the inputs for the Node.
    fn get_inputs<'a>(&'a self) -> Vec<&'a Node> { Vec::new() }
    /// Return a mutable reference to the inputs for the Node.
    fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut Node> { Vec::new() }

    /// Apply settings to self and all inputs.
    fn apply_settings(&mut self, settings: SoundStreamSettings) {
        for input in self.get_inputs_mut().mut_iter() {
            input.apply_settings(settings);
        }
        self.get_node_data().settings = settings;
    }

    /// Remove all inputs from the `inputs` vector.
    fn remove_all_inputs(&mut self) {
        self.get_inputs_mut().clear();
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
            let mut inputs = self.get_inputs_mut();
            let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
            for i in range(0, inputs.len()) {
                let input = inputs.get_mut(i);
                let input_vol = input.get_node_data().vol;
                let input_pan = input.get_node_data().pan;
                let mut working: Vec<f32> = Vec::from_elem((frames * channels) as uint, 0f32);
                // Call audio_requested for each input.
                input.audio_requested(&mut working);
                // Construct precalculated volume and
                // pan array (for efficiency).
                let vol_l: f32 = input_vol * (1f32 - input_pan);
                let vol_r: f32 = input_vol * input_pan;
                for j in range(0, vol_per_channel.len()) {
                    *vol_per_channel.get_mut(j) = if j == 0 { vol_l } else { vol_r };
                }
                // Sum all input nodes to output (considering
                // pan, vol and interleaving).
                for j in range(0, frames) {
                    for k in range(0, channels) {
                        *output.get_mut(j * channels + k) +=
                            *working.get(j * channels + k) * *vol_per_channel.get(k);
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

