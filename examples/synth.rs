//! 
//! An example of using dsp-chain's `Graph` type to create a simple
//! Synthesiser with 3 sine wave oscillators.
//!

extern crate dsp;
extern crate num;

use dsp::{CallbackFlags, CallbackResult, Graph, Node, Sample,
          Settings, SoundStream, StreamParams, Wave};

/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type Output = f32;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {

    // Construct our dsp graph.
    let mut dsp_graph = Graph::new();

    // Construct our fancy Synth and add it to the graph!
    let synth = dsp_graph.add_node(DspNode::Synth);

    // Construct a few oscillators, add them to the graph and connect them to the synth.
    let oscillator_a = dsp_graph.add_node(DspNode::Oscillator(0.0, A5_HZ, 0.2));
    let oscillator_b = dsp_graph.add_node(DspNode::Oscillator(0.0, D5_HZ, 0.1));
    let oscillator_c = dsp_graph.add_node(DspNode::Oscillator(0.0, F5_HZ, 0.15));
    dsp_graph.add_input(oscillator_a, synth).unwrap();
    dsp_graph.add_input(oscillator_b, synth).unwrap();
    dsp_graph.add_input(oscillator_c, synth).unwrap();

    // If adding a connection between two nodes would create a cycle, Graph will return an Err.
    if let Err(err) = dsp_graph.add_input(synth, oscillator_a) {
        println!("Test for graph cycle error: {:?}", ::std::error::Error::description(&err));
    }

    // Set the synth as the master node for the graph.
    dsp_graph.set_master(Some(synth));

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
    let callback = Box::new(move |output: &mut[Output], settings: Settings, dt: f64, _: CallbackFlags| {
        Sample::zero_buffer(output);
        dsp_graph.audio_requested(output, settings);
        timer -= dt;
        for input in dsp_graph.inputs_mut(synth) {
            if let DspNode::Oscillator(_, ref mut pitch, _) = *input {
                *pitch -= 0.1;
            }
        }
        if timer >= 0.0 { CallbackResult::Continue } else { CallbackResult::Complete }
    });

    // Construct the stream and handle any errors that may have occurred.
    let stream = SoundStream::new().output(StreamParams::new()).run_callback(callback).unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

}

/// Our type for which we will implement the `Dsp` trait.
#[derive(Debug)]
enum DspNode {
    /// Synth will be our demonstration of a master GraphNode.
    Synth,
    /// Oscillator will be our generator type of node, meaning that we will override
    /// the way it provides audio via its `audio_requested` method.
    Oscillator(Phase, Frequency, Volume),
}

impl Node<Output> for DspNode {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [Output], settings: Settings) {
        match *self {
            DspNode::Synth => (),
            DspNode::Oscillator(ref mut phase, frequency, volume) => {
                for frame in buffer.chunks_mut(settings.channels as usize) {
                    let val = sine_wave(*phase, volume);
                    for channel in frame.iter_mut() {
                        *channel = val;
                    }
                    *phase += frequency / settings.sample_hz as f64;
                }
            },
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S {
    use std::f64::consts::PI;
    use num::Float;
    Sample::from_wave((phase * PI * 2.0).sin() as Wave * volume)
}

