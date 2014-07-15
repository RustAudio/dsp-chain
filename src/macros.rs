
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
        fn get_node_data<'a>(&'a mut self) -> &'a mut ::dsp::NodeData { &mut self$(.$data)+ }
    );
)

/// Simplify implementation of the 'get_inputs' `Node` methods.
///
/// Here, we return a vector of references to each of our inputs. This is used primarily
/// for the audio_requested method, which will recurse through all inputs requesting for the
/// output audio buffer to be filled.
#[macro_export]
macro_rules! impl_dsp_node_get_inputs(
    ($($inputs:ident).*) => (
        /// Return all inputs as a vector of references to `Node` types.
        fn get_inputs<'a>(&'a self) -> Vec<&'a ::dsp::Node> {
            self$(.$inputs)+.iter().map(|input| input as &::dsp::Node).collect()
        }
        /// Return all inputs as a vector of mutable references to `Node` types.
        fn get_inputs_mut<'a>(&'a mut self) -> Vec<&'a mut ::dsp::Node> {
            self$(.$inputs)+.mut_iter().map(|input| input as &mut ::dsp::Node).collect()
        }
    );
)

