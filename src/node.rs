
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

    /// Following the call to the `Node`'s `audio_requested` method, the `Graph` will sum together
    /// some of the original (dry) signal with some of the processed (wet) signal.
    ///
    /// This method specifies the amount of the dry signal to be used (0.0 ... 1.0).
    ///
    /// By default, we don't want any of the original signal. This default is useful for generator
    /// types, where the original signal is often 0.0 anyway.
    ///
    /// For processors and effects, this should be overridden to return the amount of the
    /// non-processed signal that should be summed with the processed.
    ///
    /// Note: overriding this method will be more efficient than implementing your own dry/wet
    /// summing in audio_requested, as `Graph` reserves a single buffer especially for this.
    fn dry(&self) -> f32 { 0.0 }

    /// Following the call to the `Node`'s `audio_requested` method, the `Graph` will sum together
    /// some of the original (dry) signal with some of the processed (wet) signal.
    ///
    /// This method specifies the amount of the wet signal to be used (0.0 ... 1.0).
    ///
    /// By default, we want only the fully wet signal (1.0). This default is useful for generator
    /// types where we are generating a brand new signal.
    ///
    /// For processors and effects, this should be overridden to return the amount of the processed
    /// signal that should be summed with the non-processed.
    ///
    /// Note: overriding this method will be more efficient than implementing your own dry/wet
    /// summing in audio_requested, as `Graph` reserves a single buffer especially for this.
    fn wet(&self) -> f32 { 1.0 }

}

