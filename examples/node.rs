//! 
//! An example of using dsp-chain's `Node` trait to create a simple
//! Synthesiser with 3 sine wave oscillators.
//!

extern crate dsp;
extern crate num;

use dsp::{CallbackFlags, CallbackResult, Node, Sample, Settings, SoundStream, StreamParams, Wave};

/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type Output = f32;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {

    // Construct our fancy Synth!
    let mut synth = Synth([
            Oscillator(0.0, A5_HZ, 0.2), 
            Oscillator(0.0, D5_HZ, 0.1), 
            Oscillator(0.0, F5_HZ, 0.15)
        ]);

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The callback we'll use to pass to the Stream. It will request audio from our synth.
    let callback = Box::new(move |output: &mut[Output], settings: Settings, dt: f64, _: CallbackFlags| {
        Sample::zero_buffer(output);
        synth.audio_requested(output, settings);
        timer -= dt;
        if timer >= 0.0 { CallbackResult::Continue } else { CallbackResult::Complete }
    });

    // Construct the stream and handle any errors that may have occurred.
    let stream = SoundStream::new().output(StreamParams::new()).run_callback(callback).unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {}

}


/// Synth will be our demonstration of a parent DspNode where the Oscillators
/// that it owns are it's children.
#[derive(Debug)]
struct Synth([Oscillator; 3]);

impl Node<Output> for Synth {
    /// Here we return a reference to each of our Oscillators as our `inputs`.
    /// This allows the default `audio_requested` method to draw input from
    /// each of our oscillators automatically.
    fn inputs(&mut self) -> Vec<&mut Node<Output>> {
        let Synth(ref mut oscillators) = *self;
        oscillators.iter_mut().map(|osc| osc as &mut Node<Output>).collect()
    }
}


/// Oscillator will be our generator type of node, meaning that we will override
/// the way it provides audio via its `audio_requested` method.
#[derive(Debug)]
struct Oscillator(Phase, Frequency, Volume);

impl Node<Output> for Oscillator {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [Output], settings: Settings) {
        let Oscillator(ref mut phase, frequency, volume) = *self;
        for frame in buffer.chunks_mut(settings.channels as usize) {
            let val = sine_wave(*phase, volume);
            for channel in frame.iter_mut() {
                *channel = val;
            }
            *phase += frequency / settings.sample_hz as f64;
        }
    }
}

/// Return a sine wave for the given phase for any sample type.
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S {
    Sample::from_wave((phase * ::std::f64::consts::PI * 2.0).sin() as Wave * volume)
}

