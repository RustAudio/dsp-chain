//! An example of using dsp-chain's `Graph` type to create a simple Synthesiser with 3 sine wave
//! oscillators.

extern crate dsp;
extern crate portaudio;

use dsp::{sample, Graph, Node, FromSample, Sample, Settings, Walker};
use portaudio as pa;


/// SoundStream is currently generic over i8, i32 and f32. Feel free to change it!
type Output = f32;

type Phase = f64;
type Frequency = f64;
type Volume = f32;

const CHANNELS: i32 = 2;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;

const A5_HZ: Frequency = 440.0;
const D5_HZ: Frequency = 587.33;
const F5_HZ: Frequency = 698.46;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

    // Construct our dsp graph.
    let mut graph = Graph::new();

    // Construct our fancy Synth and add it to the graph!
    let synth = graph.add_node(DspNode::Synth);

    // Connect a few oscillators to the synth.
    let (_, oscillator_a) = graph.add_input(DspNode::Oscillator(0.0, A5_HZ, 0.2), synth);
    graph.add_input(DspNode::Oscillator(0.0, D5_HZ, 0.1), synth);
    graph.add_input(DspNode::Oscillator(0.0, F5_HZ, 0.15), synth);

    // If adding a connection between two nodes would create a cycle, Graph will return an Err.
    if let Err(err) = graph.add_connection(synth, oscillator_a) {
        println!("Testing for cycle error: {:?}", ::std::error::Error::description(&err));
    }

    // Set the synth as the master node for the graph.
    graph.set_master(Some(synth));

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 3.0;

    // This will be used to determine the delta time between calls to the callback.
    let mut prev_time = None;

    // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, time, .. }| {

        sample::buffer::equilibrium(buffer);
        let settings = Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        graph.audio_requested(buffer, settings);

        let last_time = prev_time.unwrap_or(time.current);
        let dt = time.current - last_time;
        timer -= dt;
        prev_time = Some(time.current);

        // Traverse inputs or outputs of a node with the following pattern.
        let mut inputs = graph.inputs(synth);
        while let Some(input_idx) = inputs.next_node(&graph) {
            if let DspNode::Oscillator(_, ref mut pitch, _) = graph[input_idx] {
                // Pitch down our oscillators for fun.
                *pitch -= 0.1;
            }
        }

        if timer >= 0.0 { pa::Continue } else { pa::Complete }
    };

    // Construct PortAudio and the stream.
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings(CHANNELS, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let true = try!(stream.is_active()) {
        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }

    Ok(())
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
fn sine_wave<S: Sample>(phase: Phase, volume: Volume) -> S
    where S: Sample + FromSample<f32>,
{
    use std::f64::consts::PI;
    ((phase * PI * 2.0).sin() as f32 * volume).to_sample::<S>()
}
