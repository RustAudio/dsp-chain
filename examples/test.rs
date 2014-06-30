
//! Test app for rust-dsp.
//!
//! The app will spawn a SoundStream which will
//! feed audio input straight to the output. It
//! will then terminate after a few seconds of
//! playback.

extern crate dsp;

use dsp::{
    SoundStream,
    SoundStreamSettings,
    Node,
    NodeData,
};

static SAMPLE_RATE: int = 44100;
static FRAMES: int = 128;
static CHANNELS: int = 2;

/// Calculates the real-time sample-rate from dt.
fn calc_sample_rate(settings: SoundStreamSettings, dt: u64) -> f64 {
    let dtsec: f64 = dt as f64 / 1000000000f64;
    (1f64 / dtsec) * settings.frames as f64
}

#[deriving(Show, Clone)]
struct AltOsc { node_data: NodeData }
impl Node for AltOsc {
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData { &mut self.node_data }
    fn audio_requested(&mut self, _output: &mut Vec<f32>) {
        print!("woot, ");
    }
}
impl AltOsc {
    pub fn new(settings: SoundStreamSettings) -> AltOsc {
        AltOsc { node_data: NodeData::new(settings) }
    }
}

#[deriving(Show, Clone)]
struct Oscillator {
    node_data: NodeData,
    inputs: Vec<AltOsc>,
}

impl Node for Oscillator {
    fn get_node_data<'a>(&'a mut self) -> &'a mut NodeData { &mut self.node_data }
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
        for _ in range(0, 1000) {
            self.oscillator.inputs.push(AltOsc::new(settings))
        }
    }
    fn update(&mut self, settings: SoundStreamSettings, dt: u64) {
        //println!("Real-time sample rate: {}", calc_sample_rate(settings, dt));
        match self.kill_chan.try_recv() {
            Ok(msg) => self.should_exit = msg,
            Err(_) => ()
        }
    }
    fn audio_in(&mut self, input: &Vec<f32>, settings: SoundStreamSettings) {
        assert!(input.len() == settings.frames as uint * settings.channels as uint);
        self.buffer = input.clone();
    }
    fn audio_out(&mut self, output: &mut Vec<f32>, settings: SoundStreamSettings) {
        assert!(output.len() == settings.frames as uint * settings.channels as uint);
        //let mut oscillator = Oscillator::new(settings.clone());
        //println!("requested");
        self.oscillator.audio_requested(output);
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

