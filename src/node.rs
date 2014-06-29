
use sound_stream_settings::SoundStreamSettings;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Show;


/// The mixer input for handling volume and
/// panning of incoming audio from inputs.
#[deriving(Clone)]
pub struct MixerInput<T> {
    node: Rc<RefCell<T>>,
    vol: f32, 
    pan: f32
}

/// Need to explicitly implement Show due to
/// Rc pointer.
impl<T> Show for MixerInput<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "vol: {}, pan{}", self.vol, self.pan)
    }
}

impl<T: IsNode> MixerInput<T> {
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
pub struct Node<T> {
    inputs: Vec<MixerInput<T>>,
    /// SoundStreamSettings for buffer calculations.
    pub settings: SoundStreamSettings,
    /// Master volume for DSP node.
    pub master_vol: f32
}

pub struct EndNode {
    /// SoundStreamSettings for buffer calculations.
    pub settings: SoundStreamSettings,
    /// Master volume for DSP node.
    pub master_vol: f32
}

/// DSP Node trait. Implement this for any audio
/// instrument or effects types. Be sure to add
/// the `Node` struct to a field as well, and
/// override the `get_node` and `get_node_mut`
/// methods by returning a ref (/mut) to it.
pub trait IsNode: Clone {

    /// Return a reference to the Node.
    fn get_node<'a, T: IsNode>(&'a self) -> &'a Node<T>;
    /// Return a mutable reference to the Node.
    fn get_node_mut<'a, T: IsNode>(&'a mut self) -> &'a mut Node<T>;

    /// Apply settings to self and all inputs.
    fn apply_settings<T: IsNode>(&mut self, settings: SoundStreamSettings) {
        {
            let inputs: &Vec<MixerInput<T>> = &self.get_node_mut::<T>().inputs;
            for input in inputs.iter() {
                input.node.borrow_mut().apply_settings::<T>(settings);
            }
        }
        self.get_node_mut::<T>().settings = settings;
    }
    
    /// Add a new input.
    fn add_input<T: IsNode>(&mut self, input: Rc<RefCell<T>>) {
        self.get_node_mut().inputs.push(MixerInput::new(input))
    }

    /// Remove an input. Find the input by comparing
    /// the address of the given input to each of the
    /// inputs' addresses.
    fn remove_input<T: IsNode>(&mut self, to_remove: &Rc<RefCell<T>>) {
        for i in range(0, self.get_node_mut::<Self>().inputs.len()) {
            if &(*self.get_node_mut::<Self>().inputs.get(i).node.borrow()) as *_ ==
                &(*to_remove.borrow()) as *_ {
                self.get_node_mut::<Self>().inputs.remove(i);
                break;
            }
        }
    }

    /// Remove all inputs from the `inputs` vector.
    fn remove_all_inputs<T: IsNode>(&mut self) {
        self.get_node_mut::<Self>().inputs.clear();
    }

    /// Add all inputs held by another node.
    fn add_inputs_from<T: IsNode>(&mut self, other: &Rc<RefCell<T>>) {
        self.get_node_mut::<Self>().inputs.push_all(other.borrow().get_node::<T>().inputs.as_slice());
    }

    /// Receive incoming audio (override this
    /// to do something with the input).
    fn audio_received<T: IsNode>(&mut self, input: &Vec<f32>) {}

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    fn audio_requested<T: IsNode>(&mut self, output: &mut Vec<f32>) {
        let master_vol = self.get_node_mut::<Self>().master_vol;
        let frames = self.get_node_mut::<Self>().settings.frames as uint;
        let channels = self.get_node_mut::<Self>().settings.channels as uint;
        let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
        for i in range(0, self.get_node_mut::<Self>().inputs.len()) {
            let input = self.get_node_mut::<Self>().inputs.get(i);
            let mut working: Vec<f32> =
                Vec::from_elem((frames * channels) as uint, 0f32);

            // Call audio_requested for each input.
            input.node.borrow_mut().get_node_mut::<T>().audio_requested::<T>(&mut working);

            // Construct precalculated volume and
            // pan array (for efficiency).
            let vol_l: f32 = input.vol * (1f32 - input.pan);
            let vol_r: f32 = input.vol * input.pan;
            for j in range(0, vol_per_channel.len()) {
                *vol_per_channel.get_mut(j) =
                    if j == 0 { vol_l } else { vol_r } * master_vol;
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
        self.get_node_mut::<Self>().process_buffer::<Self>(output);
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    fn process_buffer<T>(&mut self, output: &mut Vec<f32>) {}

}

/// We only have to implement these two methods
/// for any of our types that derive from our
/// `IsNode` DSP node trait.
impl<T: IsNode> IsNode for Node<T> {
    fn get_node<'a, T: IsNode>(&'a self)         -> &'a Node<T>     { self }
    fn get_node_mut<'a, T: IsNode>(&'a mut self) -> &'a mut Node<T> { self }
}

/// We only have to implement these two methods
/// for any of our types that derive from our
/// `IsNode` DSP node trait.
impl<T: IsNode> IsNode for EndNode {
    fn get_node<'a, T: IsNode>(&'a self)         -> &'a Node<T>     { self }
    fn get_node_mut<'a, T: IsNode>(&'a mut self) -> &'a mut Node<T> { self }
}

impl<T: IsNode> Node<T> {
    /// Constructor for the DSP Node.
    pub fn new<T>(settings: SoundStreamSettings) -> Node {
        Node {
            inputs: Vec::new(),
            settings: settings,
            master_vol: 1f32
        }
    }
}

