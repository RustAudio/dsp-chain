
use sound_stream_settings::SoundStreamSettings;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Show;

/* /// The mixer input for handling volume and
/// panning of incoming audio from inputs.
#[deriving(Clone)]
pub struct MixerInput {
    node: Rc<RefCell<Node>>,
    vol: f32, 
    pan: f32,
} */

/// Need to explicitly implement Show due to
/// Rc pointer.
impl<T: Node> Show for MixerInput<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "node: {}, vol: {}, pan{}",
               self.node.borrow_mut().get_node_data(), self.vol, self.pan)
    }
}

impl<T: Node> MixerInput<T> {
    /// Constructor for a Mixer Input.
    pub fn new(node: Rc<RefCell<T>>) -> MixerInput<T> {
        MixerInput {
            node: node,
            vol: 1f32,
            pan: 0f32
        }
    }
}

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

/// DSP Node trait. Implement this for any audio
/// instrument or effects types. Be sure to add
/// the `Node` struct to a field as well, and
/// override the `get_node` and `get_node_mut`
/// methods by returning a ref (/mut) to it.
pub trait Node: Clone {

    /// Return a reference a NodeData struct owned by the Node.
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData;
    /// Return a reference to the inputs for the Node.
    fn get_inputs<'a>(&'a self) -> Vec<&MixerInput> { Vec::new() }
    /// Return a mutable reference to the inputs for the Node.
    fn get_inputs_mut<'a>(&'a mut self) -> Vec<&mut MixerInput> { Vec::new() }

    /// Apply settings to self and all inputs.
    fn apply_settings<T: Node>(&mut self, settings: SoundStreamSettings) {
        match self.get_inputs::<T>() {
            Some(inputs) => {
                for input in inputs.iter() {
                    input.node.borrow_mut().apply_settings::<T>(settings);
                }
            },
            None => ()
        }
        self.get_node_data().settings = settings;
    }
    
    /// Add a new input.
    fn add_input<T: Node>(&mut self, input: Rc<RefCell<T>>) {
        match self.get_inputs_mut::<T>() {
            Some(inputs) => inputs.push(MixerInput::new(input)),
            None => ()
        }
    }

    /// Remove an input. Find the input by comparing
    /// the address of the given input to each of the
    /// inputs' addresses.
    fn remove_input<T: Node>(&mut self, to_remove: &Rc<RefCell<T>>) {
        match self.get_inputs_mut::<T>() {
            Some(inputs) => {
                for i in range(0, inputs.len()) {
                    if &(*inputs.get(i).node.borrow()) as *_ ==
                        &(*to_remove.borrow()) as *_ {
                        inputs.remove(i);
                        break;
                    }
                }
            },
            None => ()
        }
    }

    /// Remove all inputs from the `inputs` vector.
    fn remove_all_inputs<T: Node>(&mut self) {
        match self.get_inputs_mut::<T>() {
            Some(inputs) => inputs.clear(),
            None => ()
        }
    }

    /// Add all inputs held by another node.
    fn add_inputs_from<T: Node>(&mut self, other: &Rc<RefCell<T>>) {
        match self.get_inputs_mut::<T>() {
            Some(inputs) => {
                match other.borrow().get_inputs::<T>() {
                    Some(other_inputs) => inputs.push_all(other_inputs.as_slice()),
                    None => ()
                }
            },
            None => ()
        }
    }

    /// Receive incoming audio (override this
    /// to do something with the input).
    fn audio_received(&mut self, _input: &Vec<f32>) {}

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    fn audio_requested<T: Node>(&mut self, output: &mut Vec<f32>) {
        let master_vol = self.get_node_data().master_vol;
        let frames = self.get_node_data().settings.frames as uint;
        let channels = self.get_node_data().settings.channels as uint;
        let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
        for i in range(0, self.get_inputs::<T>().len()) {
            let input = self.get_inputs::<T>().get(i);
            let mut working: Vec<f32> = Vec::from_elem((frames * channels) as uint, 0f32);
            // Call audio_requested for each input.
            input.node.borrow_mut().audio_requested::<T>(&mut working);
            // Construct precalculated volume and
            // pan array (for efficiency).
            let vol_l: f32 = input.vol * (1f32 - input.pan);
            let vol_r: f32 = input.vol * input.pan;
            for j in range(0, vol_per_channel.len()) {
                *vol_per_channel.get_mut(j) = if j == 0 { vol_l } else { vol_r } * master_vol;
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
        // Custom buffer processing.
        self.process_buffer(output);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    fn process_buffer(&mut self, _output: &mut Vec<f32>) {}

}

