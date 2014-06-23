
use sound_stream_settings::SoundStreamSettings;
use signal::Signal;
use node::{ Node, IsNode };
use frequency::Frequency;
use pitch::Pitch;
use waveform::{
    Waveform,
    Sine,
    Saw,
    Square,
    Noise,
    NoiseWalk
};

/// Oscillator - the fundamental component
/// of audio synthesis.
#[deriving(Show, Clone)]
pub struct Oscillator {
    node: Node,
    waveform: Waveform,
    phase: f32,
    freq: f64,
    signal: Signal<f32>
}

impl IsNode for Oscillator {

    /// Get reference to node for IsNode trait.
    fn get_node<'a>(&'a self) -> &'a Node { &self.node }
    /// Get mutable reference to node for IsNode trait.
    fn get_node_mut<'a>(&'a mut self) -> &'a mut Node { &mut self.node }

    /// Here we override the audio_requested method
    /// in order to perform our synthesis.
    fn audio_requested(&mut self, output: &mut Vec<f32>) {

    }

}


impl Oscillator {

    /// Oscillator constructor.
    pub fn new(settings: SoundStreamSettings, waveform: Waveform) -> Oscillator {
        Oscillator {
            node: Node::new(settings),
            waveform: waveform,
            phase: 0f32,
            freq: 0f64,
            signal: Signal::new(0f32)
        }
    }

    /// Set the waveform fro the oscillator to
    /// use for phase iteration.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Set frequency.
    pub fn set_freq(&mut self, freq: f64) {

    }

    /// Calculate and return the amplitude at
    /// the given ratio.
    pub fn get_amp_at_ratio(&mut self, ratio: f64) -> f64 {
        // Pass phase into signal generator (to return
        // signal for waveform later).
        let phase = self.phase;
        self.signal.set_val(phase);
        
        // Set frequency to determine phase advance.
        let freq_at_ratio = self.get_freq_at_ratio(ratio);
        //self.set_freq(freq_at_ratio * )

        // Advance phase according to frequency.
        self.update_phase();
        
        // Calculate amplitude for sample at ratio.
        0f64

    }

    /// Calculate and return the frequency at
    /// the given ratio.
    pub fn get_freq_at_ratio(&self, ratio: f64) {

    }

    /// Iterate the phase according to frequency
    /// and waveform.
    pub fn update_phase(&mut self) {

    }

}

