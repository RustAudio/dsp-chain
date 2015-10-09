
extern crate dsp;

use dsp::{CallbackFlags, CallbackResult, Graph, Node, Sample,
          Settings, SoundStream, StreamParams, Wave};


fn main() {

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

    // The callback we'll use to pass to the Stream. It will request audio from our graph.
    let callback = Box::new(move |output: &mut[f32], settings: Settings, dt: f64, _: CallbackFlags| {

        // Zero the sample buffer.
        Sample::zero_buffer(output);

        // Request audio from the graph.
        graph.audio_requested(output, settings);

        // Oscillate the volume.
        if let &mut DspNode::Volume(ref mut vol) = &mut graph[volume] {
            *vol = (4.0 * timer as f32).sin();
        }

        timer += dt;
        if timer <= 5.0 { CallbackResult::Continue } else { CallbackResult::Complete }
    });

    // Construct the stream and handle any errors that may have occurred.
    let stream = SoundStream::new().output(StreamParams::new()).run_callback(callback).unwrap();

    // Wait for our stream to finish.
    while let Ok(true) = stream.is_active() {
        ::std::thread::sleep_ms(16);
    }

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

