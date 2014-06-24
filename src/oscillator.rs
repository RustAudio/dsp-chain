
use sound_stream_settings::SoundStreamSettings;
use signal::Signal;
use node::{ Node, IsNode };
use envelope::Envelope;
use frequency::Frequency;
use frequency::constants::{
    HIGHEST_HZ,
    LOWEST_HZ
};
use gaussian::Gaussian;
use pitch::Pitch;
use pitch::letter::Letter;
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
    signal: Signal<f32>,
    amplitude: Envelope,
    frequency: Envelope,
    gaussian: Gaussian,
    is_playing: bool
}

impl IsNode for Oscillator {

    /// Get reference to node for IsNode trait.
    fn get_node<'a>(&'a self) -> &'a Node { &self.node }
    /// Get mutable reference to node for IsNode trait.
    fn get_node_mut<'a>(&'a mut self) -> &'a mut Node { &mut self.node }

    /// Here we override the audio_requested method
    /// in order to perform our synthesis.
    fn audio_requested(&mut self, output: &mut Vec<f32>) {
        let master_vol = self.node.master_vol;
        let frames = self.node.settings.frames;
        let ratio @ value = 0f32;
        for i in range(0, frames) {
            
        }
    }

}

impl Oscillator {

    /// Oscillator constructor.
    pub fn new(settings: SoundStreamSettings, waveform: Waveform) -> Oscillator {
        Oscillator {
            node: Node::new(settings),
            waveform: waveform,
            phase: 0f32,
            signal: Signal::new(0f32),
            amplitude: Envelope::new_amplitude_env(),
            frequency: Envelope::new_frequency_env(),
            gaussian: Gaussian::new(),
            is_playing: false
        }
    }

    /// Set the waveform fro the oscillator to
    /// use for phase iteration.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
    }

    /// Calculate and return the amplitude at
    /// the given ratio.
    pub fn get_amp_at_ratio(&mut self, ratio: f32) -> f32 {
        // Pass phase into signal generator (to return
        // signal for waveform later).
        let phase = self.phase;
        self.signal.set_val(phase);
        // Set frequency to determine phase advance.
        let mut freq_at_ratio = self.get_freq_at_ratio(ratio);
        freq_at_ratio = freq_at_ratio /* * note_freq_multiplier */ ;
        let advance_per_sample = freq_at_ratio / self.node.settings.samples_per_sec as f32;
        // Advance phase according to frequency.
        self.update_phase(advance_per_sample);
        // Calculate amplitude for sample at ratio.
        self.get_waveform_value() * self.amplitude.get_value(ratio)
    }

    /// Calculate and return the frequency at
    /// the given ratio.
    pub fn get_freq_at_ratio(&self, ratio: f32) -> f32 {
        self.frequency.get_value(ratio) * (HIGHEST_HZ - LOWEST_HZ) + LOWEST_HZ
    }

    /// Iterate the phase according to frequency
    /// and waveform.
    fn update_phase(&mut self, advance_per_sample: f32) {
        let old_phase = self.phase;
        let new_phase = match self.waveform {
            Sine | NoiseWalk | Square => old_phase + advance_per_sample,
            Saw | Noise => (old_phase + advance_per_sample) % 2f32
        };
        self.phase = new_phase;
    }

    /// Return the waveform value for current phase
    fn get_waveform_value(&self) -> f32 {
        match self.waveform {
            Sine => self.signal.get_sin(),
            Saw => self.signal.get_saw(),
            Square => self.signal.get_sqr(),
            Noise => self.signal.get_noise(),
            NoiseWalk => self.signal.get_noise_walk()
        }
    }

    /// Start playback by turning on `is_playing`.
    pub fn play(&mut self) {
        self.is_playing = true;
    }

}

