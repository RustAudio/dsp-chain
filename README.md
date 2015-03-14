# dsp-chain [![Build Status](https://travis-ci.org/RustAudio/dsp-chain.svg?branch=master)](https://travis-ci.org/RustAudio/dsp-chain)

A simple library for chaining together multiple audio dsp processors/generators, written in Rust!


Usage
-----

Here are [two examples](https://github.com/PistonDevelopers/dsp-chain/blob/master/examples) of using dsp-chain to create a very basic synth.

Other use cases for dsp-chain include:
- Designing effects.
- Creating an audio mixer.
- Making a sampler.
- Writing a dsp backend for a DAW.
- Any kind of modular audio synthesis/processing.

Add dsp-chain to your Cargo.toml dependencies like so:
```
[dependencies]
dsp-chain = "*"
```


More Details
------------

There are two primary modules of interest within this library, both of which
are unrelated and are designed to be used separately.

1. [node.rs](https://github.com/RustAudio/dsp-chain/blob/master/src/node.rs) and the `Node` trait.
2. [graph.rs](https://github.com/RustAudio/dsp-chain/blob/master/src/graph.rs) and the `Graph` type.

The `Node` trait offers a DSP chaining design via its `inputs` method. It is
slightly simpler to use than the `Graph` type however also slightly more limited.
Using the `Node` trait, it is impossible for two nodes to reference the same
input `Node` making it difficult to perform tasks like complex "bussing" and "side-chaining".

The `Graph` type constructs a directed, acyclic graph of DSP nodes. It is
the recommended approach for more advanced DSP chains that involve things like
"bussing", "side-chaining" or more DAW-esque behaviour. The `Graph` type requires
its nodes to have implemented the `Dsp` trait (a slightly simplified version of the
`Node` trait, though entirely unrelated). Internally, `Graph` uses bluss's petgraph
crate. See more [here](https://crates.io/crates/petgraph).


PortAudio
---------

dsp-chain uses [PortAudio](http://www.portaudio.com) as a cross-platform audio backend. The [rust-portaudio](https://github.com/jeremyletang/rust-portaudio) dependency will first try to find an already installed version on your system before trying to download it and build PortAudio itself.


License
-------

MIT - Same license as [PortAudio](http://www.portaudio.com/license.html).

