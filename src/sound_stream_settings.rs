
/// Settings required for SoundStream.
#[deriving(Show, Clone, PartialEq, Encodable, Decodable)]
pub struct SoundStreamSettings {
    /// The number of samples per second.
    pub samples_per_sec: u32,

    /// How many samples per channel requested at a time in the buffer.
    /// The more frames, the less likely to make glitches,
    /// but this gives slower response.
    pub frames: u16,
    
    /// Number of channels, for example 2 for stereo sound (left + right speaker).
    pub channels: u16
}

impl SoundStreamSettings {
    /// Custom constructor for the SoundStreamSettings.
    ///
    /// ToDo: It would be good to include a method that checked
    /// the feasibility of the requested settings (i.e. that
    /// channels isn't too large, and that samples_per_sec and frames
    /// are of a sound card standard).
    pub fn new(samples_per_sec: u32, frames: u16, channels: u16)
        -> SoundStreamSettings {
        SoundStreamSettings {
            samples_per_sec: samples_per_sec,
            frames: frames,
            channels: channels
        }
    }
    /// Default, standard constructor for SoundStreamSettings.
    pub fn cd_quality()
        -> SoundStreamSettings {
        SoundStreamSettings {
            samples_per_sec: 44100,
            frames: 512,
            channels: 2
        }
    }

    /// Return the length of a SoundBuffer that would use SoundStreamSettings.
    pub fn buffer_size(&self) -> uint {
        self.frames as uint * self.channels as uint
    }
}

impl ::std::default::Default for SoundStreamSettings {
    fn default() -> SoundStreamSettings { SoundStreamSettings::cd_quality() }
}

