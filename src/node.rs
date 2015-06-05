
use sound_stream::{Sample, Settings};


/// Types to be used as a **Node** within the DSP **Graph**.
pub trait Node<S> where S: Sample {
    /// Request audio from the **Node** given some stream format **Settings**.
    /// If the **Node** has no inputs, the `buffer` will be zeroed.
    /// If the **Node** has some inputs, the `buffer` will consist of the inputs summed together.
    ///
    /// Any source/generator type nodes should simply render straight to the buffer.
    /// Any effects/processor type nodes should mutate the buffer directly.
    fn audio_requested(&mut self, buffer: &mut [S], settings: Settings);
}

