
//! Test app for rust-dsp.
//!
//! This file will normally be in a strange state;
//! it's really just a testing ground for the new
//! features as I add them. I'll get around to making
//! some proper examples soon!
//! 

extern crate dsp;

use dsp::{
    SoundStream,
    SoundStreamSettings,
    Node,
    NodeData,
};

/// We'll use these values for setting
/// up our SoundStream.
/// Note: there is also a default constructor
/// method that will be safe to use, if you're
/// unsure what you need. It is called like this:
/// SoundStreamSettings::cd_quality()
static SAMPLE_RATE: int = 44100;
static FRAMES: int = 128;
static CHANNELS: int = 2;

/// This struct is just used for demonstration as
/// an input for the Oscillator struct.
#[deriving(Show, Clone)]
struct AltOsc { node_data: NodeData }
impl Node for AltOsc {
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData { &mut self.node_data }
    /// This will get called for every input to the
    /// Oscillator struct.
    fn audio_requested(&mut self, _output: &mut Vec<f32>) {
        print!("woot, ");
    }
}
impl AltOsc {
    pub fn new(settings: SoundStreamSettings) -> AltOsc {
        AltOsc { node_data: NodeData::new(settings) }
    }
}

/// This struct is used to demonstrate implementation
/// of the Node trait.
#[deriving(Show, Clone)]
struct Oscillator {
    node_data: NodeData,
    inputs: Vec<AltOsc>,
}

impl Node for Oscillator {
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData { &mut self.node_data }
    /// Here, we return a vector of mutable references
    /// to each of our inputs. This is used primarily
    /// for the audio_requested method, which will
    /// recurse through all inputs requesting for the
    /// output audio buffer to be filled.
    fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut Node> {
        let mut vec: Vec<&'a mut Node> = Vec::new();
        for input in self.inputs.mut_iter() {
            vec.push(input);
        }
        vec
    }
}

impl Oscillator {
    pub fn new(settings: SoundStreamSettings) -> Oscillator {
        Oscillator {
            node_data: NodeData::new(settings),
            inputs: Vec::new()
        }
    }
}

/// This is our main sound application struct.
/// We'll implement SoundStream for it and run
/// it on it's own thread when the time comes
/// for non-blocking audio IO!
pub struct SoundApp {
    buffer: Vec<f32>,
    kill_chan: Receiver<bool>,
    should_exit: bool,
    oscillator: Oscillator,
}

/// Here we will implement the constructor for
/// our sound application. Notice the kill
/// channel! We need this or SoundStream will
/// refuse to die.
impl SoundApp {
    pub fn new(kill_chan: Receiver<bool>, settings: SoundStreamSettings) -> SoundApp {
        SoundApp {
            buffer: Vec::with_capacity(FRAMES as uint * CHANNELS as uint),
            oscillator: Oscillator::new(settings),
            kill_chan: kill_chan,
            should_exit: false
        }
    }
}

/// Here we implement SoundStream for our
/// sound application. SoundStream gives us
/// our tasty audio callback in the form of
/// `audio_in` and `audio_out` methods.
impl SoundStream for SoundApp {
    fn load(&mut self, settings: SoundStreamSettings) {
        // Add a bunch of inputs to our oscillator as a test.
        for _ in range(0, 1000) {
            self.oscillator.inputs.push(AltOsc::new(settings))
        }
    }
    fn update(&mut self, _settings: SoundStreamSettings, _dt: u64) {
        // Listen out for the kill message from the main thread.
        match self.kill_chan.try_recv() {
            Ok(msg) => self.should_exit = msg,
            Err(_) => ()
        }
    }
    fn audio_in(&mut self, input: &Vec<f32>, settings: SoundStreamSettings) {
        assert!(input.len() == settings.frames as uint * settings.channels as uint);
        // We'll copy the input here, and pass it to output later.
        // ... just for fun.
        self.buffer = input.clone();
    }
    fn audio_out(&mut self, output: &mut Vec<f32>, settings: SoundStreamSettings) {
        assert!(output.len() == settings.frames as uint * settings.channels as uint);
        // Here 'audio_requested' will call be called recursively
        // for all inputs.
        self.oscillator.audio_requested(output);
        // We'll pass the audio from the input straight to the
        // output here.
        *output = self.buffer.clone();
    }
    fn exit(&self) -> bool { self.should_exit }
}

/// Here we will launch our sound app. Notice
/// we launch our sound app in it's own
/// task! This is important for both performance
/// and so that it doesn't get blocked by
/// whatever we have on our main GUI thread.
fn main() {
    println!("Rust and the marvellous DSP!");
    let (sender, receiver) = channel();
    spawn(proc() {
        let settings = SoundStreamSettings::new(SAMPLE_RATE, FRAMES, CHANNELS);
        let mut soundstream = SoundApp::new(receiver, settings);
        soundstream.run(settings);
    });
    std::io::timer::sleep(3000);
    sender.send(true);
}

