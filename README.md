# dsp-chain [![Build Status](https://travis-ci.org/PistonDevelopers/dsp-chain.svg?branch=master)](https://travis-ci.org/PistonDevelopers/dsp-chain)

A simple library for chaining together multiple audio dsp processors/generators, written in Rust!

Usage
-----

Here's [an example](https://github.com/PistonDevelopers/dsp-chain/blob/master/examples/test.rs) of using dsp-chain to create a very basic synth.

Other use cases for dsp-chain include:
- Designing effects.
- Creating an audio mixer.
- Making a sampler.
- Any kind of modular audio synthesis/processing.

PortAudio
---------

- You'll need to have [PortAudio](http://www.portaudio.com/download.html) installed on your system. Note: We're planning on integrating PA as a static lib to make life easier for those who don't already have it installed!

Maintainers: @mitchmindtree

