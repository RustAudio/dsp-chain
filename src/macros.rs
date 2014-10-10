
#![macro_escape]

//!
//! Macros for simple implementation of Node trait methods.
//! 
//! Example:
//!
//! struct Synth {
//!     data: NodeData,
//!     oscillators: Vec<Oscillator>,
//! }
//!
//! impl Node for Synth {
//!     impl_dsp_node_get_data!(data)
//!     impl_dsp_node_get_inputs!(oscillators)
//!     ...
//! }
//!

/// Simplify implementation of the 'get_node_data' `Node` methods.
#[macro_export]
macro_rules! impl_dsp_node_get_data(
    ($($data:ident).*) => (
        /// Return a reference to the Data struct owned by the Node.
        fn get_node_data(&self) -> &::dsp::NodeData { &self$(.$data)+ }
        /// Return a reference to the Data struct owned by the Node.
        fn get_node_data_mut(&mut self) -> &mut ::dsp::NodeData { &mut self$(.$data)+ }
    );
)

/// Simplify implementation of the 'get_inputs' `Node` methods.
///
/// Here, we return a vector of references to each of our inputs. This is used primarily
/// for the audio_requested method, which will recurse through all inputs requesting for the
/// output audio buffer to be filled.
#[macro_export]
macro_rules! impl_dsp_node_get_inputs(
    ($($inputs:ident).*, $buffer:ty) => (
        /// Return all inputs as a vector of references to `Node` types.
        fn get_inputs<'a>(&'a self) -> Vec<&'a ::dsp::Node<$buffer>> {
            self$(.$inputs)+.iter().map(|input| input as &::dsp::Node<$buffer>).collect()
        }
        /// Return all inputs as a vector of mutable references to `Node` types.
        fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut ::dsp::Node<$buffer>> {
            self$(.$inputs)+.mut_iter().map(|input| input as &mut ::dsp::Node<$buffer>).collect()
        }
    );
)

/// Simplify implementation of DspBuffer trait.
macro_rules! impl_dsp_buffer(
    ($buffer:ty, $len:expr) => (

        impl DspBuffer for $buffer {
            #[inline]
            fn val(&self, idx: uint) -> f32 { self[idx] }
            #[inline]
            fn get(&self, idx: uint) -> &f32 { &self[idx] }
            #[inline]
            fn get_mut(&mut self, idx: uint) -> &mut f32 { &mut self[idx] }
            #[inline]
            fn as_slice(&self) -> &[f32] { self.as_slice() }
            #[inline]
            fn as_mut_slice(&mut self) -> &mut [f32] { self.as_mut_slice() }
            #[inline]
            fn iter<'a>(&'a self) -> Items<'a, f32> { self.as_slice().iter() }
            #[inline]
            fn iter_mut<'a>(&'a mut self) -> MutItems<'a, f32> { self.as_mut_slice().iter_mut() }
            #[inline]
            fn from_elem(val: f32) -> $buffer { [val, ..$len] }
            #[inline]
            fn zeroed() -> $buffer { [0f32, ..$len] }
            #[inline]
            fn len(&self) -> uint { $len }
            #[inline]
            fn mono_settings(samples_per_sec: u32) -> SoundStreamSettings {
                SoundStreamSettings::new(samples_per_sec, $len, 1)
            }
            #[inline]
            fn stereo_settings(samples_per_sec: u32) -> SoundStreamSettings {
                SoundStreamSettings::new(samples_per_sec, $len / 2, 2)
            }
        }

    )
)






