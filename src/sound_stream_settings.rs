
/// Settings required for SoundStream.
#[deriving(Show, Clone, PartialEq)]
pub struct SoundStreamSettings {
    /// The number of samples per second.
    pub samples_per_sec: int,

    /// How many samples per channel requested at a time in the buffer.
    /// The more frames, the less likely to make glitches,
    /// but this gives slower response.
    pub frames: int,
    
    /// Number of channels, for example 2 for stereo sound (left + right speaker).
    pub channels: int
}

impl SoundStreamSettings {
    /// Custom constructor for the SoundStreamSettings.
    ///
    /// ToDo: It would be good to include a method that checked
    /// the feasibility of the requested settings (i.e. that
    /// channels isn't 500, and that samples_per_sec and frames
    /// are of a sound card standard).
    pub fn new(samples_per_sec: int, frames: int, channels: int)
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

