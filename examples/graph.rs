//! 
//! An example of using dsp-chain's `Graph` type to create a simple
//! Synthesiser with 3 sine wave oscillators.
//!

#![feature(core)]

extern crate dsp;

use dsp::{Dsp, Event, Graph, Sample, Settings, SoundStream, Wave};

/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type AudioSample = f32;

type Input = AudioSample;
type Output = AudioSample;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {

    // Construct the stream and handle any errors that may have occurred.
    let mut stream = match SoundStream::<Input, Output>::new().run() {
        Ok(stream) => { println!("It begins!"); stream },
        Err(err) => panic!("An error occurred while constructing SoundStream: {}", err),
    };

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
        println!("{:?}", ::std::error::Error::description(&err));
    }

    // Set the synth as the master node for the graph.
    dsp_graph.set_master(Some(synth));

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // The SoundStream iterator will automatically return these events in this order.
    for event in stream.by_ref() {
        match event {
            Event::Out(buffer, settings) => dsp_graph.audio_requested(buffer, settings),
            Event::Update(dt) => {
                if timer > 0.0 { timer -= dt } else { break }
                // Pitch down each of the oscillators (just for fun).
                for input in dsp_graph.inputs_mut(synth) {
                    if let DspNode::Oscillator(_, ref mut pitch, _) = *input {
                        *pitch -= 1.0;
                    }
                }
            },
            _ => (),
        }
    }

    // Close the stream and shut down PortAudio.
    match stream.close() {
        Ok(()) => println!("Great success!"),
        Err(err) => println!("An error occurred while closing SoundStream: {}", err),
    }

}

/// Our type for which we will implement the `Dsp` trait.
enum DspNode {
    /// Synth will be our demonstration of a master GraphNode.
    Synth,
    /// Oscillator will be our generator type of node, meaning that we will override
    /// the way it provides audio via its `audio_requested` method.
    Oscillator(Phase, Frequency, Volume),
}

impl Dsp<Output> for DspNode {
    /// Here we'll override the audio_requested method and generate a sine wave.
    fn audio_requested(&mut self, buffer: &mut [Output], settings: Settings) {
        match *self {
            DspNode::Synth => (),
            DspNode::Oscillator(ref mut phase, frequency, volume) => {
                for frame in buffer.chunks_mut(settings.channels as usize) {
                    *phase += frequency / settings.sample_hz as f64;
                    let val = sine_wave(*phase, volume);
                    for channel in frame.iter_mut() {
                        *channel = val;
                    }
                }
            },
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S {
    use std::f64::consts::PI_2;
    use std::num::Float;
    Sample::from_wave((phase * PI_2).sin() as Wave * volume)
}

