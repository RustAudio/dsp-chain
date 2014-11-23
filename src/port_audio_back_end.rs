//! PortAudio backend!

use buffer::DspBuffer;
use portaudio::pa::;
use portaudio::pa::error::Error;
use sound_stream::SoundStream;
use sound_stream_settings::SoundStreamSettings;

/// PortAudio Stream Parameters (required to setup stream).
pub struct StreamParamsPA {
    input: pa::StreamParameters,
    output: pa::StreamParameters,
}

/// PortAudio Stream (for reading from and writing to real-time audio stream).
pub struct StreamPA {
    stream: pa::Stream<f32, f32>,
    pub is_streaming: bool
}

fn get_result_text(res: Result<(), Error>) -> String {
    let err = match res {
        Ok(()) => Error::NoError,
        Err(e) => e,
    };
    pa::get_error_text(err)
}

/// The parameters required to set up the PortAudio stream.
impl StreamParamsPA {

    /// Creates the port audio stream parameters.
    pub fn new(channels: u16) -> StreamParamsPA {
        let res = pa::initialize();
        println!("Portaudio init error : {}", get_result_text(res));

        println!("Creating StreamParamsPA");
        let def_input = pa::device::get_default_input();
        let def_output = pa::device::get_default_output();

        println!("Creating input");
        let stream_params_in = pa::StreamParameters {
            device: def_input,
            channel_count: channels as i32,
            sample_format: pa::SampleFormat::Float32,
            suggested_latency: pa::device::get_info(def_input).unwrap().default_low_input_latency
        };
        println!("Creating output");
        let stream_params_out = pa::StreamParameters {
            device: def_output,
            channel_count: channels as i32,
            sample_format: pa::SampleFormat::Float32,
            suggested_latency: pa::device::get_info(def_output).unwrap().default_low_output_latency
        };

        StreamParamsPA {
            input: stream_params_in,
            output: stream_params_out
        }
    }

}


impl StreamPA {

    /// Constructor for the portaudio stream.
    pub fn new() -> StreamPA {
        StreamPA {
            stream: pa::Stream::new(),
            is_streaming: true
        }
    }

    /// Setup the portaudio stream.
    pub fn setup(&mut self, settings: SoundStreamSettings) {
        let params = StreamParamsPA::new(settings.channels);
        match self.stream.open(Some(&params.input),
                         Some(&params.output),
                         settings.samples_per_sec as f64,
                         settings.frames as u32,
                         pa::StreamFlags::ClipOff) {
            Ok(_) => {},
            Err(e) => panic!(format!("Portaudio etup error: {}", get_result_text(Err(e))))
        };
    }

    /// Performs the audio read/write.
    pub fn callback<B: DspBuffer, T: SoundStream<B>>
    (&mut self, settings: SoundStreamSettings, stream: &mut T) {
        let mut ready = 0;
        while ready == 0 {
            ready = self.stream.get_stream_write_available();
        }
        //let empty_buffer = Vec::from_elem(settings.buffer_size(), 0f32);
        //let mut read: Vec<f32> = empty_buffer.clone();
        self.read(settings, stream);
        let mut write: B = DspBuffer::zeroed();
        self.write(&mut write, settings, stream);
    }

    /// Read audio in from stream.
    pub fn read<B: DspBuffer, T: SoundStream<B>>
    (&self, settings: SoundStreamSettings, stream: &mut T) {
        match self.stream.read(settings.frames as u32) {
            Ok(in_buffer) => {
                stream.audio_in(&in_buffer, settings);
            },
            Err(err) => {
                panic!(format!("Portaudio error read : {}", pa::get_error_text(err)));
            }
        }
    }

    /// Write audio to stream
    pub fn write<B: DspBuffer, T: SoundStream<B>>
    (&mut self, buffer: &mut B, settings: SoundStreamSettings, stream: &mut T) {
        stream.audio_out(buffer, settings);
        /*
        println!("OUT OF AUDIO_OUT, {}", buffer.len());
        let mut write = Vec::with_capacity(buffer.len());
        println!("OUT OF AUDIO_OUTx, {}", write.capacity());
        for i in range(0u, buffer.len()) {
            println!("z");
            write.push(buffer.val(i));
        }
        */

        //let write: Vec<f32> = buffer.iter().map(|f| { println!("z"); *f }).collect();
        match self.stream.write(buffer.to_vec(), settings.frames as u32) {
            Ok(_) => {},
            Err(e) => {
                panic!(format!("Portaudio write error : {}", pa::get_error_text(e)));
            }
        }
    }

    /// Start the audio stream.
    pub fn start(&mut self) {
        let res = self.stream.start();
        println!("Portaudio Start Stream : {}", get_result_text(res));
    }

    /// Stop the audio stream.
    pub fn stop(&mut self) {
        let err = Error::NotInitialized;
        println!("Portaudio [NotInitialized msg] : {}", pa::get_error_text(err));
        let res = self.stream.close();
        println!("Portaudio Closing Stream : {}", get_result_text(res));
        let res = pa::terminate();
        println!("Portaudio Termination Message : {}", get_result_text(res));
    }

}

/// Ensure that the stream closes properly upon object destruction.
impl Drop for StreamPA {
    fn drop(&mut self) {
        if self.is_streaming {
            self.stop();
        }
    }
}

