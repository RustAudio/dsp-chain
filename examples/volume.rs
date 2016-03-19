//! An example of a simple volume node oscillating the amplitude of a synth node.

extern crate dsp;
extern crate portaudio;

use dsp::{Graph, Frame, Node, FromSample, Sample, Settings};
use dsp::sample::ToFrameSliceMut;
use portaudio as pa;

const CHANNELS: usize = 2;
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

        let settings = Settings::new(SAMPLE_HZ as u32, frames as u16, CHANNELS as u16);
        let buffer: &mut [[f32; CHANNELS]] = buffer.to_frame_slice_mut().unwrap();

        // Zero the sample buffer.
        dsp::slice::equilibrium(buffer);

        // Request audio from the graph.
        graph.audio_requested(buffer, settings);

        // Oscillate the volume.
        if let &mut DspNode::Volume(ref mut vol) = &mut graph[volume] {
            *vol = (4.0 * timer as f32).sin() * 0.5;
        }

        let last_time = prev_time.unwrap_or(time.current);
        let dt = time.current - last_time;
        timer += dt;
        prev_time = Some(time.current);
        if timer <= 5.0 { pa::Continue } else { pa::Complete }
    };

    // Construct PortAudio and the stream.
    let pa = try!(pa::PortAudio::new());
    let settings = try!(pa.default_output_stream_settings::<f32>(CHANNELS as i32, SAMPLE_HZ, FRAMES));
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }

    Ok(())
}


/// Our Node to be used within the Graph.
enum DspNode {
    Synth(f64),
    Volume(f32),
}

/// Implement the `Node` trait for our DspNode.
impl Node<[f32; CHANNELS]> for DspNode {
    fn audio_requested(&mut self, buffer: &mut [[f32; CHANNELS]], settings: Settings) {
        match *self {
            DspNode::Synth(ref mut phase) => dsp::slice::map_in_place(buffer, |_| {
                let val = sine_wave(*phase);
                const SYNTH_HZ: f64 = 110.0;
                *phase += SYNTH_HZ / settings.sample_hz as f64;
                Frame::from_fn(|_| val)
            }),
            DspNode::Volume(vol) => dsp::slice::map_in_place(buffer, |f| f.map(|s| s.mul_amp(vol))),
        }
    }
}

/// Return a sine wave for the given phase.
fn sine_wave<S: Sample>(phase: f64) -> S
    where S: Sample + FromSample<f32>,
{
    use std::f64::consts::PI;
    ((phase * PI * 2.0).sin() as f32).to_sample::<S>()
}
