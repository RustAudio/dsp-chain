//! Test app for rust-dsp.
//!
//! This file will normally be in a strange state;
//! it's really just a testing ground for the new
//! features as I add them. I'll get around to making
//! some proper examples soon!
//!

#![feature(phase)]
#[phase(plugin, link)] extern crate dsp;

use std::time::duration::Duration;

use dsp::{
    DspBuffer,
    SoundStream,
    SoundStreamSettings,
    Node,
    NodeData,
};

/// We'll use these values for setting
/// up our SoundStream.
///
/// Note: FRAMES == (fixed-size buffer length / CHANNELS)
const SAMPLE_RATE: u32 = 44100;
const FRAMES: u16 = 256;
const CHANNELS: u16 = 2;

/// Choose a fixed-size buffer of f32 with a length
/// matching FRAMES * CHANNELS.
pub type AudioBuffer = [f32, .. FRAMES as uint * CHANNELS as uint];

/// This struct is just used for demonstration as
/// an input for the Oscillator struct.
#[deriving(Show, Clone)]
struct AltOsc { node_data: NodeData }
impl Node<AudioBuffer> for AltOsc {
    impl_dsp_node_get_data!(node_data)
    /// This will get called for every input to the
    /// Oscillator struct.
    fn audio_requested(&mut self, _output: &mut AudioBuffer) {
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

impl Node<AudioBuffer> for Oscillator {
    impl_dsp_node_get_data!(node_data)
    impl_dsp_node_get_inputs!(inputs, AudioBuffer)
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
    buffer: AudioBuffer,
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
            buffer: DspBuffer::zeroed(),
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
impl SoundStream<AudioBuffer> for SoundApp {
    fn load(&mut self, settings: SoundStreamSettings) {
        // Add a bunch of inputs to our oscillator as a test.
        for _ in range(0u, 2) {
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
        self.buffer = DspBuffer::from_vec(input);
    }
    fn audio_out(&mut self, output: &mut AudioBuffer, settings: SoundStreamSettings) {
        assert!(output.len() == settings.frames as uint * settings.channels as uint);
        // Here 'audio_requested' will call be called recursively
        // for all inputs.
        self.oscillator.audio_requested(output);
        // We'll pass the audio from the input straight to the
        // output here.
        *output = self.buffer;
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
    std::io::timer::sleep(Duration::seconds(3));
    sender.send(true);
}

