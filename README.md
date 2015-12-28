# dsp-chain [![Build Status](https://travis-ci.org/RustAudio/dsp-chain.svg?branch=master)](https://travis-ci.org/RustAudio/dsp-chain) [![Crates.io](https://img.shields.io/crates/v/dsp-chain.svg)](https://crates.io/crates/dsp-chain) [![Crates.io](https://img.shields.io/crates/l/dsp-chain.svg)](https://github.com/RustAudio/dsp-chain/blob/master/LICENSE)


A library for chaining together multiple audio dsp processors/generators, written in Rust!

Use cases for dsp-chain include:
- Designing effects.
- Creating an audio mixer.
- Making a sampler.
- Writing a dsp backend for a DAW.
- Any kind of modular audio synthesis/processing.


Documenation
------------

[API documentation here!](http://RustAudio.github.io/dsp-chain/dsp)


Usage
-----

Here's what it looks like:

```Rust
// Construct our dsp graph.
let mut graph = Graph::new();

// Construct our fancy Synth and add it to the graph!
let synth = graph.add_node(DspNode::Synth);

// Add a few oscillators as inputs to the synth.
graph.add_input(DspNode::Oscillator(0.0, A5_HZ, 0.2), synth);
graph.add_input(DspNode::Oscillator(0.0, D5_HZ, 0.1), synth);
graph.add_input(DspNode::Oscillator(0.0, F5_HZ, 0.15), synth);

// Set the synth as the master node for the graph.
// This can be inferred by the graph so calling this is optional, but it's nice to be explicit.
graph.set_master(Some(synth));

// Request audio from our Graph.
graph.audio_requested(&mut buffer, settings);
```

Here are [two working examples](https://github.com/PistonDevelopers/dsp-chain/blob/master/examples) of using dsp-chain to create a very basic synth and an oscillating volume.

Add dsp-chain to your Cargo.toml dependencies like so:

```toml
[dependencies]
dsp-chain = "*"
```
