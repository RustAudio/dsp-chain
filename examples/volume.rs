//! An example of a simple volume node oscillating the amplitude of a synth node.

extern crate dsp;
extern crate portaudio;

use dsp::{Graph, Node, Sample, Settings, Wave};
use portaudio as pa;

const CHANNELS: i32 = 2;
const FRAMES: u32 = 64;
const SAMPLE_HZ: f64 = 44_100.0;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {

    // Construct our dsp graph.
    let mut graph = Graph::new();

    // Construct our fancy Synth and add it to the graph!
    let synth = graph.add_node(DspNode::Synth(0.0));

    // Output our synth to a marvellous volume node.
    let (_, volume) = graph.add_output(synth, DspNode::Volume(1.0));

    // Set the synth as the master node for the graph.
    graph.set_master(Some(volume));

    // We'll use this to count down from three seconds and then break from the loop.
    let mut timer: f64 = 0.0;

    // This will be used to determine the delta time between calls to the callback.
    let mut prev_time = None;

    // The callback we'll use to pass to the Stream. It will request audio from our graph.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, frames, time, .. }| {

        // Zero the sample buffer.
        Sample::zero_buffer(buffer);

        // Request audio from the graph.
        let settings = Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        graph.audio_requested(buffer, settings);

        // Oscillate the volume.
        if let &mut DspNode::Volume(ref mut vol) = &mut graph[volume] {
            *vol = (4.0 * timer as f32).sin();
        }

        let last_time = prev_time.unwrap_or(time.current);
        let dt = time.current - last_time;
        timer += dt;
        prev_time = Some(time.current);
        if timer <= 5.0 { pa::Continue } else { pa::Complete }
    };

    // Construct PortAudio and the stream.
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings(CHANNELS, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

    Ok(())
}


/// Our Node to be used within the Graph.
enum DspNode {
    Synth(f64),
    Volume(f32),
}

/// Implement the `Node` trait for our DspNode.
impl Node<f32> for DspNode {
    fn audio_requested(&mut self, buffer: &mut [f32], settings: Settings) {
        match *self {
            DspNode::Synth(ref mut phase) => {
                for frame in buffer.chunks_mut(settings.channels as usize) {
                    let val = sine_wave(*phase);
                    for channel in frame.iter_mut() {
                        *channel = val;
                    }
                    const SYNTH_HZ: f64 = 110.0;
                    *phase += SYNTH_HZ / settings.sample_hz as f64;
                }
            },
            DspNode::Volume(vol) => for sample in buffer.iter_mut() {
                *sample = *sample * vol;
            },
        }
    }
}

/// Return the amplitude for a given phase.
fn sine_wave<S: Sample>(phase: f64) -> S {
    use std::f64::consts::PI;
    Sample::from_wave((phase * PI * 2.0).sin() as Wave)
}
