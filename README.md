# dsp-chain [![Build Status](https://travis-ci.org/RustAudio/dsp-chain.svg?branch=master)](https://travis-ci.org/RustAudio/dsp-chain)

A simple library for chaining together multiple audio dsp processors/generators, written in Rust!


Usage
-----

Here are [two examples](https://github.com/PistonDevelopers/dsp-chain/blob/master/examples) of using dsp-chain to create a very basic synth and an oscillating volume.

Use cases for dsp-chain include:
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


PortAudio
---------

dsp-chain uses [PortAudio](http://www.portaudio.com) as a cross-platform audio backend. The [rust-portaudio](https://github.com/jeremyletang/rust-portaudio) dependency will first try to find an already installed version on your system before trying to download it and build PortAudio itself.


License
-------

MIT - Same license as [PortAudio](http://www.portaudio.com/license.html).

