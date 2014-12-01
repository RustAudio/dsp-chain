//! 
//! An example of using dsp-chain to create a simple Synthesiser with 3 sine wave oscillators.
//!

extern crate dsp;

use dsp::{
    Event,
    Node,
    SoundStream,
    Settings,
};

/// The number of frames processed per second.
const SAMPLE_HZ: u32 = 44100;
/// The number of channels to use for output. We'll use two for stereo.
const CHANNELS: u16 = 2;
/// This value is equal to (fixed-size buffer length / CHANNELS).
const FRAMES: u16 = 256;

const SETTINGS: Settings = Settings { sample_hz: SAMPLE_HZ, frames: FRAMES, channels: CHANNELS };

const BUFFER_SIZE: uint = (FRAMES * CHANNELS) as uint;

type Input = f32;
type Output = f32;
type OutputBuffer = [f32, ..BUFFER_SIZE];

type Phase = f64;
type Frequency = f64;
type Volume = f64;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {

    // Construct the stream and handle any errors that may have occurred.
    let mut stream = match SoundStream::<OutputBuffer, Input, Output>::new(SETTINGS) {
        Ok(stream) => { println!("It begins!"); stream },
        Err(err) => panic!("An error occurred while constructing SoundStream: {}", err),
    };

    // Construct our fancy Synth!
    let mut synth = Synth([
            Oscillator(0.0, A5_HZ, 0.2), 
            Oscillator(0.0, D5_HZ, 0.1), 
            Oscillator(0.0, F5_HZ, 0.15)
        ]);

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The SoundStream iterator will automatically return these events in this order.
    for event in stream {
        match event {
            Event::Out(buffer) => synth.audio_requested(buffer, SETTINGS),
            Event::Update(dt) => if timer > 0.0 { timer -= dt } else { break },
            _ => (),
        }
    }

    // Close the stream and shut down PortAudio.
    match stream.close() {
        Ok(()) => println!("Great success!"),
        Err(err) => println!("An error occurred while closing SoundStream: {}", err),
    }

}


/// Synth will be our demonstration of a parent DspNode where the Oscillators
/// that it owns are it's children.
struct Synth([Oscillator, ..3]);

impl Node<OutputBuffer, Output> for Synth {
    /// Here we return a reference to each of our Oscillators as our `inputs`.
    /// This allows the default `audio_requested` method to draw input from
    /// each of our oscillators automatically.
    fn inputs(&mut self) -> Vec<&mut Node<OutputBuffer, Output>> {
        let Synth(ref mut oscillators) = *self;
        oscillators.iter_mut().map(|osc| osc as &mut Node<OutputBuffer, Output>).collect()
    }
}


/// Oscillator will be our generator type of node, meaning that we will override
/// the way it provides audio via its `audio_requested` method.
struct Oscillator(Phase, Frequency, Volume);

impl Node<OutputBuffer, Output> for Oscillator {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut OutputBuffer, settings: Settings) {
        let (frames, channels) = (settings.frames as uint, settings.channels as uint);
        let Oscillator(ref mut phase, frequency, volume) = *self;
        // For every frame in the buffer.
        for i in range(0u, frames) {
            *phase += frequency / settings.sample_hz as f64;
            let val = sine_wave(*phase, volume);
            // For each channel in the frame.
            for j in range(0u, channels) {
                let idx = i * channels + j;
                buffer[idx] = val;
            }
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave(phase: Phase, volume: Volume) -> Output {
    use std::f64::consts::PI_2;
    use std::num::FloatMath;
    ((phase * PI_2).sin() * volume) as Output
}

