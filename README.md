# baselisk-rs
Learning Rust by writing a musical synthesizer

Thanks for looking! This is a hobby project written in spare time. My aims are to

* Learn more about the Rust programming language
* Learn more about digital signal processing
* Practice maintaining and refactoring a codebase

Code is an everlasting work in progress.

## Dependencies

- Linux system (Windows, Mac unsupported at this time)
- PortAudio
- PortMidi

## Usage

To build: `cargo build --release`

Make sure your MIDI and audio devices are connected and powered on. Then:

`cargo run --release`

You will be shown a list of audio devices. Enter the ID of your chosen output device, then press Return.

PortAudio will be initialized and audio callbacks will begin. You will be shown a list of MIDI devices and IDs for convenience, but none will be selected until you specify one (see the `midi input` command below).

This application uses Rustyline to provide readline-like tab completion of commands. The following commands are supported at the time of writing:

- `midi input <device_id>`: Sets the MIDI device that PortMidi will receive messages from. Only one device may be selected at a time.
- `oscillator type <str>`: Sets the oscillator type. Supported are `saw`, `pulse` and `sine`.
- `oscillator pitch <pitch>`: Sets the pitch offset of the oscillator.
- `oscillator pulsewidth <width>`: Sets the pulse width of the `pulse` waveform.
- `adsr attack <duration>`: Set the attack time for the ADSR used by the gain and filter processors.
- `adsr decay <duration>`: Set the ADSR decay time.
- `adsr sustain <level>`: Set the ADSR sustain level.
- `adsr release <duration>`: Set the ADSR release time.
- `filter frequency <hz>`: Set the base cutoff frequency of the filter.
- `filter quality <q>`: Set the quality of the filter. `0.7` is flatter; higher values produce higher resonance.
- `waveshaper inputgain <gain>`: Set the input gain of the waveshaper.
- `waveshaper outputgain <gain>`: Set the output gain of the waveshaper.
