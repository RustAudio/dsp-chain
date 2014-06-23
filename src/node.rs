
use sound_stream_settings::SoundStreamSettings;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Show;

/// The mixer input for handling volume and
/// panning of incoming audio from inputs.
#[deriving(Clone)]
pub struct MixerInput {
    node: Rc<RefCell<Node>>,
    vol: f32,
    pan: f32
}

/// Need to explicitly implement Show due to
/// Rc pointer.
impl Show for MixerInput {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "node: {}, vol: {}, pan{}", self.node.borrow(), self.vol, self.pan)
    }
}

impl MixerInput {
    /// Constructor for a mixer input.
    pub fn new(node: Rc<RefCell<Node>>) -> MixerInput {
        MixerInput {
            node: node,
            vol: 1f32,
            pan: 0f32
        }
    }
}

/// The DSP Node contains a vector of children
/// nodes (within the `MixerInput`s from which
/// audio can be requested as well as the current
/// SoundStream settings.
#[deriving(Show, Clone)]
pub struct Node {
    inputs: Vec<MixerInput>,
    settings: SoundStreamSettings,
    master_vol: f32
}

/// DSP Node trait. Implement this for any audio
/// instrument or effects types. Be sure to add
/// the `Node` struct to a field as well, and
/// override the `get_node` and `get_node_mut`
/// methods by returning a ref (/mut) to it.
pub trait IsNode {

    /// Return a reference to the Node.
    fn get_node<'a>(&'a self) -> &'a Node;
    /// Return a mutable reference to the Node.
    fn get_node_mut<'a>(&'a mut self) -> &'a mut Node;

    /// Apply settings to self and all inputs.
    fn apply_settings(&mut self, settings: SoundStreamSettings) {
        for input in self.get_node_mut().inputs.mut_iter() {
            input.node.borrow_mut().apply_settings(settings);
        }
        self.get_node_mut().settings = settings;
    }
    
    /// Add a new input.
    fn add_input(&mut self, input: Rc<RefCell<Node>>) {
        self.get_node_mut().inputs.push(MixerInput::new(input))
    }

    /// Remove an input. Find the input by comparing
    /// the address of the given input to each of the
    /// inputs' addresses.
    fn remove_input(&mut self, to_remove: &Node) {
        for i in range(0, self.get_node_mut().inputs.len()) {
            if &(*self.get_node_mut().inputs.get(i).node.borrow()) as *_ == to_remove as *_ {
                self.get_node_mut().inputs.remove(i);
                break;
            }
        }
    }

    /// Remove all inputs from the `inputs` vector.
    fn remove_all_inputs(&mut self) {
        self.get_node_mut().inputs.clear();
    }

    /// Add all inputs held by another node.
    fn add_inputs_from(&mut self, other: &Node) {
        self.get_node_mut().inputs.push_all(other.inputs.as_slice());
    }

    /// Receive incoming audio (override this
    /// to do something with the input).
    fn audio_received(&mut self, input: &Vec<f32>) {}

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    fn audio_requested(&mut self, output: &mut Vec<f32>) {
        let master_vol = self.get_node_mut().master_vol;
        let frames = self.get_node_mut().settings.frames as uint;
        let channels = self.get_node_mut().settings.channels as uint;
        let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
        for i in range(0, self.get_node_mut().inputs.len()) {
            let input = self.get_node_mut().inputs.get(i);
            let mut working: Vec<f32> =
                Vec::from_elem((frames * channels) as uint, 0f32);

            // Call audio_requested for each input.
            input.node.borrow_mut().audio_requested(&mut working);
            
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
                        *working.get_mut(j * channels + k) * *vol_per_channel.get_mut(k);
                }
            }
        }
        // Custom buffer processing.
        self.get_node_mut().process_buffer(output);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    fn process_buffer(&mut self, output: &mut Vec<f32>) {}

}

/// We only have to implement these two methods
/// for any of our types that derive from our
/// `IsNode` DSP node trait.
impl IsNode for Node {
    fn get_node<'a>(&'a self) -> &'a Node { self }
    fn get_node_mut<'a>(&'a mut self) -> &'a mut Node { self }
}

impl Node {
    /// Constructor for the DSP Node.
    pub fn new(settings: SoundStreamSettings) -> Node {
        Node {
            inputs: Vec::new(),
            settings: settings,
            master_vol: 1f32
        }
    }
}

