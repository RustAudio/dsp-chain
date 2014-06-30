//
//  node_enum.rs
//
//  Created by Mitchell Nordine at 02:53PM on June 29, 2014.
//
//

use sound_stream_settings::SoundStreamSettings;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Formatter;
use std::fmt::Result;
use std::fmt::Show;

/// The mixer input for handling volume and
/// panning of incoming audio from inputs.
#[deriving(Clone)]
pub struct MixerInput<'a> {
    node: Rc<RefCell<Node<'a>>>,
    vol: f32, 
    pan: f32
}

/// Need to explicitly implement Show due to
/// Rc pointer.
impl<'a> Show for MixerInput<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "vol: {}, pan{}", self.vol, self.pan)
    }
}

impl<'a> MixerInput<'a> {
    /// Constructor for a Mixer Input.
    pub fn new<'a>(node: Rc<RefCell<Node<'a>>>) -> MixerInput<'a> {
        MixerInput {
            node: node,
            vol: 1f32,
            pan: 0f32
        }
    }
}

/// The Node System is constructed of NodeTypes.
pub enum Node<'a> {
    /// The DSP Node contains a vector of children
    /// nodes (within the `MixerInput`s), from which
    /// audio can be requested as well as the current
    /// SoundStream settings.
    Active {
        inputs: Vec<MixerInput<'a>>,
        /// SoundStreamSettings for buffer calculations.
        pub settings: SoundStreamSettings,
        /// Master volume for DSP node.
        pub master_vol: f32,
        /// Function for receiving audio.
        pub received: |input: &Vec<f32>|:'a,
        /// Function for processing audio.
        pub process: |buffer: &mut Vec<f32>|:'a,
        /// Function for requesting audio.
        pub requested: |output: &mut Vec<f32>,
                        children: &mut Vec<MixerInput>,
                        settings: SoundStreamSettings,
                        master_vol: f32|:'a,
    },
    /// For signalling the end of the node system.
    Deactive
}

impl<'a> Node<'a> {

    /// Constructor for the DSP Node.
    pub fn new<'a>(settings: SoundStreamSettings) -> Node<'a> {
        Active {
            inputs: Vec::new(),
            settings: settings,
            master_vol: 1f32,
            received: |input: &Vec<f32>| {},
            process: |buffer: &mut Vec<f32>| {},
            requested: |output: &mut Vec<f32>,
                        children: &mut Vec<MixerInput>,
                        settings: SoundStreamSettings,
                        master_vol: f32| {
                let frames = settings.frames as uint;
                let channels = settings.channels as uint;
                let mut vol_per_channel: Vec<f32> = Vec::from_elem(channels, 1f32);
                for i in range(0, children.len()) {
                    let input = children.get(i);
                    let mut working: Vec<f32> =
                        Vec::from_elem((frames * channels) as uint, 0f32);

                    // Call audio_requested for each input.
                    input.node.borrow_mut().audio_requested(&mut working);

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
                                *working.get(j * channels + k)
                                * *vol_per_channel.get(k);
                        }
                    }
                }
            }
        }
    }

    /// Apply settings to self and all inputs.
    pub fn apply_settings(&mut self, settings: SoundStreamSettings) {
        match self {
            Active{ inputs: inputs, .. } => {
                for input in inputs.iter() {
                    input.node.borrow_mut().apply_settings(settings);
                }
                Active.settings = settings;
            },
            Deactive => return
        }
    }

    /// Add a new input.
    pub fn add_input(&mut self, input: Rc<RefCell<Node>>) {
        self.inputs.push(MixerInput::new(input))
    }

    /// Remove an input. Find the input by comparing
    /// the address of the given input to each of the
    /// inputs' addresses.
    pub fn remove_input(&mut self, to_remove: &Rc<RefCell<Node>>) {
        for i in range(0, self.inputs.len()) {
            if &(*self.inputs.get(i).node.borrow()) as *_ ==
                &(*to_remove.borrow()) as *_ {
                self.inputs.remove(i);
                break;
            }
        }
    }

    /// Remove all inputs from the `inputs` vector.
    pub fn remove_all_inputs(&mut self) {
        self.inputs.clear()
    }

    /// Add all inputs held by another node.
    pub fn add_inputs_from(&mut self, other: &Rc<RefCell<Node>>) {
        self.inputs.push_all(other.inputs.as_slice())
    }

    /// Receive incoming audio (override this to
    /// do something with the input).
    pub fn audio_received(&mut self, input: &Vec<f32>) {
        (self.received)(input)
    }

    /// Request audio from inputs, process and
    /// pass back to the output! Override this
    /// method for any synthesis or generative
    /// types.
    pub fn audio_requested(&mut self, output: &mut Vec<f32>) {
        (self.requested)(output)
    }

    /// Override for custom processing of audio per
    /// buffer. This is mainly for audio effects. Get's
    /// called at the end of audio_requested.
    pub fn process_buffer<T>(&mut self, buffer: &mut Vec<f32>) {
        (self.process)(buffer) 
    }

}
