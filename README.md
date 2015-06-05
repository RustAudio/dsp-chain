# dsp-chain [![Build Status](https://travis-ci.org/RustAudio/dsp-chain.svg?branch=master)](https://travis-ci.org/RustAudio/dsp-chain)

A simple library for chaining together multiple audio dsp processors/generators, written in Rust!


Usage
-----

Use cases for dsp-chain include:
- Designing effects.
- Creating an audio mixer.
- Making a sampler.
- Writing a dsp backend for a DAW.
- Any kind of modular audio synthesis/processing.

Here's what it looks like:

```Rust
// Construct our dsp graph.
let mut dsp_graph = Graph::new();

// Construct our fancy Synth and add it to the graph!
let synth = dsp_graph.add_node(DspNode::Synth);

// Construct a few oscillators, add them to the graph and connect them to the synth.
let oscillator_a = dsp_graph.add_node(DspNode::Oscillator(0.0, A5_HZ, 0.2));
let oscillator_b = dsp_graph.add_node(DspNode::Oscillator(0.0, D5_HZ, 0.1));
let oscillator_c = dsp_graph.add_node(DspNode::Oscillator(0.0, F5_HZ, 0.15));
dsp_graph.add_input(oscillator_a, synth).unwrap();
dsp_graph.add_input(oscillator_b, synth).unwrap();
dsp_graph.add_input(oscillator_c, synth).unwrap();

// Set the synth as the master node for the graph.
dsp_graph.set_master(Some(synth));

// Request audio from our Graph.
dsp_graph.audio_requested(&mut buffer, settings);
```

Here are [two working examples](https://github.com/PistonDevelopers/dsp-chain/blob/master/examples) of using dsp-chain to create a very basic synth and an oscillating volume.

Add dsp-chain to your Cargo.toml dependencies like so:

```toml
[dependencies]
dsp-chain = "*"
```


PortAudio
---------

dsp-chain uses [PortAudio](http://www.portaudio.com) as a cross-platform audio backend. The [rust-portaudio](https://github.com/jeremyletang/rust-portaudio) dependency will first try to find an already installed version on your system before trying to download it and build PortAudio itself.


License
-------

MIT - Same license as [PortAudio](http://www.portaudio.com/license.html).

