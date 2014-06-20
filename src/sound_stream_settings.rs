
/// Settings required for SoundStream.
pub struct SoundStreamSettings {
    /// The number of samples per second.
    pub samples_per_second: f64,

    /// How many samples per channel requested at a time in the buffer.
    /// The more frames, the less likely to make glitches,
    /// but this gives slower response.
    pub frames: u32,
    
    /// Number of channels, for example 2 for stereo sound (left + right speaker).
    pub channels: i32
}

impl SoundStreamSettings {
    /// Custom constructor for the SoundStreamSettings.
    ///
    /// ToDo: It would be good to include a method that checked
    /// the feasibility of the requested settings (i.e. that
    /// channels isn't 500, and that samples_per_second and frames
    /// are of a sound card standard).
    pub fn new(samples_per_second: f64, frames: u32, channels: i32)
        -> SoundStreamSettings {
        SoundStreamSettings {
            samples_per_second: samples_per_second,
            frames: frames,
            channels: channels
        }
    }
    /// Default, standard constructor for SoundStreamSettings.
    pub fn cd_quality()
        -> SoundStreamSettings {
        SoundStreamSettings {
            samples_per_second: 44100f64,
            frames: 512u32,
            channels: 2i32
        }
    }
}

