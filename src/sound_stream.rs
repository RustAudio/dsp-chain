//! SoundStream (real-time audio IO).


use port_audio_back_end::StreamPA;
use time::precise_time_ns;
use sound_stream_settings::SoundStreamSettings;


/// Implement this for your real-time audio IO engine.
pub trait SoundStream {

    /// Perform tasks for loading before showing anything.
    fn load(&mut self) {}

    /// Update the physical state of the SoundStream.
    fn update(&mut self, settings: &SoundStreamSettings, dt: u64) {}

    /// Offers input via buffer of interleaved f32 samples (amplitude between -1 to 1).
    /// The input buffer's size is num_frames * num_channels.
    /// Get's called at a rate of (sample_rate / num_frames)hz.
    fn audio_in(&mut self, _input: &Vec<f32>, settings: &SoundStreamSettings) {}

    /// Requests output via buffer as interleaved f32 samples (amplitude between -1 to 1).
    /// The output buffer's size is num_frames * num_channels.
    /// Get's called at a rate of (sample_rate / num_frames)hz.
    fn audio_out(&mut self, _output: &mut Vec<f32>, settings: &SoundStreamSettings) {}

    /// Override this with your exit condition for the soundstream task.
    fn exit(&self) -> bool { false }

    /// Executes the SoundStream loop.
    fn run(&mut self, settings: SoundStreamSettings) {
        let mut stream_pa = StreamPA::new();
        stream_pa.setup(&settings);
        //stream_pa.run(settings, self);
        self.load();
        stream_pa.start();
        let mut last_time: u64 = precise_time_ns();
        let mut this_time: u64;
        let mut diff_time: u64;
        loop {
            this_time = precise_time_ns();
            diff_time = this_time - last_time;
            last_time = this_time;
            self.update(&settings, diff_time);
            if self.exit() {
                stream_pa.is_streaming = false;
                break;
            }
            else if stream_pa.is_streaming {
                stream_pa.callback(&settings, self);
            }
        }
        stream_pa.stop();
    }

}

