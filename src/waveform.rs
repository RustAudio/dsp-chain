
/// An Oscillator must use one of a variety
/// of waveform types.
#[deriving(Clone, Show)]
pub enum Waveform {
    /// Sine Wave
    Sine,
    /// Saw Wave
    Saw,
    /// Square Wave
    Square,
    /// Noise
    Noise,
    /// Noise Walk
    NoiseWalk
}


