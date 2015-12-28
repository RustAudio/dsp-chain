pub type SampleHz = u32;
pub type Frames = u16;
pub type Channels = u16;

/// Settings required for SoundStream.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Settings {
    /// The number of samples per second.
    pub sample_hz: SampleHz,
    /// How many samples per channel requested at a time in the buffer.
    ///
    /// The more frames, the less likely to make glitches, but this gives slower response.
    pub frames: Frames,
    /// Number of channels, for example 2 for stereo sound (left + right speaker).
    pub channels: Channels,
}

impl Settings {

    /// Custom constructor for the Settings.
    pub fn new(sample_hz: SampleHz, frames: Frames, channels: Channels) -> Settings {
        Settings {
            sample_hz: sample_hz,
            frames: frames,
            channels: channels
        }
    }

    /// Default, standard constructor for Settings.
    pub fn cd_quality() -> Settings {
        Settings {
            sample_hz: 44100,
            frames: 256,
            channels: 2
        }
    }

    /// Return the length of a SoundBuffer that would use Settings.
    pub fn buffer_size(&self) -> usize {
        self.frames as usize * self.channels as usize
    }

}

impl ::std::default::Default for Settings {
    fn default() -> Settings { Settings::cd_quality() }
}
