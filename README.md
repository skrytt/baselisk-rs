# baselisk-rs
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

Make sure your MIDI and audio devices are connected and powered on. Then:

`cargo run --release`

You will be shown a list of audio devices. Enter the ID of your chosen output device, then press Return.

PortAudio will be initialized and audio callbacks will begin. You will be shown a list of MIDI devices and IDs for convenience, but none will be selected until you specify one (see the `midi input` command below).

This application uses [Rustyline](https://github.com/kkawakam/rustyline) to provide readline-like tab completion of commands. 

Parameters that would take numeric values can currently be manipulated in these ways:

- `filter frequency base <value>`: Sets the value of the parameter when the modulating CC controller's value is minimum (0).
- `filter frequency max <value>`: Sets the value of the parameter when the modulating CC controller's value is maximum (127).
- `filter frequency learn`: Enables MIDI learn for this parameter; the next MIDI CC event will bind that CC to this parameter.
- `filter frequency cc <controller>`: Binds the CC number `controller` to this parameter.

The command tree itself is structured roughly like this.

- `adsr`
  - `attack  <duration>`: Set the attack time for the ADSR used by the gain and filter processors.
  - `decay <duration>`: Set the ADSR decay time.
  - `sustain <level>`: Set the ADSR sustain level.
  - `release <duration>`: Set the ADSR release time.
- `oscillator`
  - `type <str>`: Sets the oscillator type. Supported are `saw`, `pulse` and `sine`. Does not support CC learning/modulation.
  - `pitch <pitch>`: Sets the pitch offset of the oscillator.
  - `pulsewidth <width>`: Sets the pulse width of the `pulse` waveform.
- `filter`
  - `frequency <hz>`: Set the cutoff frequency of the filter.
  - `quality <q>`: Set the quality of the filter. `0.7` is flatter; higher values produce higher resonance.
- `waveshaper`
  - `inputgain <gain>`: Set the input gain of the waveshaper.
  - `outputgain <gain>`: Set the output gain of the waveshaper.
- `delay`
  - `feedback <proportion`: Set the proportion of feedback. `1.0` is unity gain; `0.0` is silence. 
  - `highpass`
    - `frequency <hz>`: Set the cutoff frequency of the delay feedback loop highpass filter.
    - `quality <q>`: Set the quality of the delay feedback loop highpass filter.
  - `lowpass`
    - `frequency <hz>`: Set the cutoff frequency of the delay feedback loop highpass filter.
    - `quality <q>`: Set the quality of the delay feedback loop highpass filter.
