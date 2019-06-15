# baselisk-rs

[![CircleCI](https://circleci.com/gh/skrytt/baselisk-rs/tree/master.svg?style=svg)](https://circleci.com/gh/skrytt/baselisk-rs/tree/master)

Baselisk is a monophonic subtractive synthesizer with a high-note-priority note selection algorithm. Moreover, Baselisk is a hobby project to give me the opportunity to:

* Learn more about the Rust programming language
* Learn more about digital signal processing
* Practice maintaining and refactoring a codebase

Code is an everlasting work in progress.

## Dependencies

- Linux system (Windows, Mac unsupported at this time)
- JACK Audio Connection Kit
  - Server
  - Development libraries
  - (Optional) qjackctl, a helpful GUI utility for configuring a JACK server

## Usage

To build: `cargo build --release`

Make sure your MIDI and audio devices are connected and powered on, and that the JACK audio server is running. Then:

`cargo run --release`

This application uses [Rustyline](https://github.com/kkawakam/rustyline) to provide readline-like tab completion of commands. 

Parameters that would take numeric values can currently be manipulated in these ways:

- `filter frequency <value>`: Sets the value of the parameter. Useful only if the parameter isn't assigned to a controller.
- `filter frequency learn`: Enables MIDI learn for this parameter; the next MIDI CC event will bind that CC to this parameter.
- `filter frequency cc <controller>`: Binds the CC number `controller` to this parameter.

The command tree itself is structured roughly like this.

- `adsr`
  - `attack  <seconds>`: Set the attack time for the ADSR used by the gain and filter processors.
  - `decay <seconds>`: Set the ADSR decay time.
  - `sustain <level>`: Set the ADSR sustain level. `1.0` is considered maximum and `0.0` is minimum.
  - `release <duration>`: Set the ADSR release time.
- `oscillator`
  - `type <str>`: Sets the oscillator type. Supported are `saw`, `pulse` and `sine`. Does not support CC learning/modulation.
  - `pitch <octaves>`: Sets the pitch offset of the oscillator.
  - `pulsewidth <width>`: Sets the pulsewidth of the `pulse` waveform. `0.5` is a square wave. Minumum is `0.0` and maximum is `1.0`.
- `filter`
  - `frequency <hz>`: Set the cutoff frequency of the filter.
  - `resonance <q>`: Set the quality (resonance) of the filter. `0.7` is flatter; values as high as `5.0` produce more resonance.
- `waveshaper`
  - `inputgain <gain>`: Set the input gain to the waveshaper stage, controlling waveshaper tone. `1.0` is unity gain.
  - `outputgain <gain>`: Set the output gain of the waveshaper. Does not affect waveshaper tone. `1.0` is unity gain.
- `delay`
  - `feedback <proportion>`: Set the proportion of feedback in the delay loop. `1.0` is unity gain; `0.0` is silence. 
  - `highpass`
    - `frequency <hz>`: Set the cutoff frequency of the delay feedback loop highpass filter.
    - `quality <q>`: Set the quality of the delay feedback loop highpass filter.
  - `lowpass`
    - `frequency <hz>`: Set the cutoff frequency of the delay feedback loop highpass filter.
    - `quality <q>`: Set the quality of the delay feedback loop highpass filter.
