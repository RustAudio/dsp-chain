//! Test app for rust-dsp.
//!
//! The app will spawn a SoundStream which will
//! feed audio input straight to the output. It
//! will then terminate after a few seconds of
//! playback.

extern crate dsp;

use dsp::{
    SoundStream,
    SoundStreamSettings
};

static SAMPLE_RATE: f64 = 44100f64;
static FRAMES: u32 = 128u32;
static CHANNELS: i32 = 2i32;

fn calc_sample_rate(settings: &SoundStreamSettings, dt: u64) -> f64 {
    let dtsec: f64 = dt as f64 / 1000000000f64;
    (1f64 / dtsec) * settings.frames as f64
}

/// This is our main sound application struct.
/// We'll implement SoundStream for it and run
/// it on it's own thread when the time comes
/// for non-blocking audio IO!
pub struct SoundApp {
    buffer: Vec<f32>,
    kill_chan: Receiver<bool>,
    should_exit: bool
}

/// Here we will implement the constructor for
/// our sound application. Notice the kill
/// channel! We need this or SoundStream will
/// refuse to die.
impl SoundApp {
    pub fn new(kill_chan: Receiver<bool>) -> SoundApp {
        SoundApp {
            buffer: Vec::with_capacity(FRAMES as uint * CHANNELS as uint),
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
    fn update(&mut self, settings: &SoundStreamSettings, dt: u64) {
        println!("Real-time sample rate: {}", calc_sample_rate(settings, dt));
        match self.kill_chan.try_recv() {
            Ok(msg) => self.should_exit = msg,
            Err(_) => ()
        }
    }
    fn audio_in(&mut self, input: &Vec<f32>, settings: &SoundStreamSettings) {
        assert!(input.len() == settings.frames as uint * settings.channels as uint);
        self.buffer = input.clone();
    }
    fn audio_out(&mut self, output: &mut Vec<f32>, settings: &SoundStreamSettings) {
        assert!(output.len() == settings.frames as uint * settings.channels as uint);
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
        let mut soundstream = SoundApp::new(receiver);
        //soundstream.run(SoundStreamSettings::cd_quality());
        soundstream.run(SoundStreamSettings::new(SAMPLE_RATE, FRAMES, CHANNELS));
    });
    std::io::timer::sleep(3000);
    sender.send(true);
}

