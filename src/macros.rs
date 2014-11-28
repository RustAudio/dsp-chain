
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
            self$(.$inputs)+.iter_mut().map(|input| input as &mut ::dsp::Node<$buffer>).collect()
        }
    );
)

