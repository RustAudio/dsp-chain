use {Frame, Sample};
use sample;
use std::collections::HashMap;

/// Types to be used as a **Node** within the DSP **Graph**.
pub trait Node<F>
where
    F: Frame,
{
    /// Request audio from the **Node** given some `sample_hz` (aka sample rate in hertz).
    /// If the **Node** has no inputs, the `buffer` will be zeroed.
    /// If the **Node** has some inputs, the `buffer` will consist of the inputs summed together.
    ///
    /// Any source/generator type nodes should simply render straight to the buffer.
    /// Any effects/processor type nodes should mutate the buffer directly.
    fn audio_requested(&mut self, buffer: &mut [F], sample_hz: f64);

    /// Requests audio from the **Node** like the `audio_requested` method but it has one
    /// additional argument `other_inputs`.
    /// This are additional inputs that can be used if the Node accepts those.
    /// There is a default implementation that can be overriden.
    fn audio_requested_by_id(
        &mut self,
        buffer: &mut [F],
        other_inputs: HashMap<usize, Box<[F]>>,
        sample_hz: f64,
    ) {
        if other_inputs.len() > 0 {
            for input in other_inputs {
                if input.0 == 0 {
                    sample::slice::zip_map_in_place(buffer, &input.1, |this_frame, other_frame| {
                        this_frame.zip_map(other_frame, |this_sample, other_sample| {
                            let this_signed = this_sample
                                .to_sample::<<F::Sample as Sample>::Signed>();
                            let other_signed = other_sample
                                .to_sample::<<F::Sample as Sample>::Signed>();
                            (this_signed + other_signed).to_sample::<F::Sample>()
                        })
                    });
                }
            }
        }
        self.audio_requested(buffer, sample_hz);
    }

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
    fn dry(&self) -> <F::Sample as Sample>::Float {
        Sample::equilibrium()
    }

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
    fn wet(&self) -> <F::Sample as Sample>::Float {
        <F::Sample as Sample>::identity()
    }
}

impl<F> Node<F> for Box<Node<F>>
where
    F: Frame,
{
    #[inline]
    fn audio_requested(&mut self, buffer: &mut [F], sample_hz: f64) {
        (**self).audio_requested(buffer, sample_hz);
    }
    #[inline]
    fn dry(&self) -> <F::Sample as Sample>::Float {
        (**self).dry()
    }
    #[inline]
    fn wet(&self) -> <F::Sample as Sample>::Float {
        (**self).wet()
    }
}
