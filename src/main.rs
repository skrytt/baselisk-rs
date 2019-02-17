//! Synthesizer based on dsp-graph, portaudio and portmidi.
//!
extern crate ansi_term;
extern crate arraydeque;
extern crate dsp;
extern crate portaudio;
extern crate portmidi;

#[macro_use]
extern crate text_io;

mod audio_thread;
mod cli;
mod comms;
mod defs;
mod dsp_unit;
mod event;
mod processor;
mod view;

use std::io;
use std::io::prelude::*;

fn run() -> Result<(), &'static str> {
    // thread comms objects will be used to transfer events back and forth between
    // the main and audio threads.
    let (main_thread_comms, audio_thread_comms) = comms::new_bidirectional();

    // The PortMidi and PortAudio interfaces are required for program operation.
    // Panic if either of them fail to initialize.
    let portmidi = portmidi::PortMidi::new().unwrap();
    let portaudio = portaudio::PortAudio::new().unwrap();

    // Initialize the view with lists of audio and MIDI devices.
    // The instances of PortMidi and PortAudio cache the lists of devices for their lifetime.
    // It's necessary to restart this program to update the lists if the devices change.
    let mut view = view::View::new(&portaudio, &portmidi);

    // Initialize the audio interface
    let mut audio_thread_interface = audio_thread::Interface::new(
        portaudio, portmidi, audio_thread_comms);

    // The user must input which audio device to open here.
    println!("Audio devices:");
    println!("{}", view.audio);

    let mut device_index: u32;
    loop {
        print!("Enter an output device ID: ");
        io::stdout().flush().ok().expect("Could not flush stdout");

        // 1. verify input is representable as an unsigned integer
        match try_read!() {
            Err(_) => {
                println!("Device ID must be a non-negative integer");
                continue
            }
            Ok(input) => {
                device_index = input;
            }
        };

        // 2. verify audio device with this index can be opened
        match audio_thread_interface.open_stream(device_index) {
            Err(reason) => {
                println!("{}", reason);
                continue
            },
            Ok(_) => break
        };
    };

    audio_thread_interface.start_stream().unwrap();

    // Process lines of text input until told to quit or interrupted.
    let mut finished = false;
    while !finished {
        finished = cli::read_and_parse(&mut audio_thread_interface, &mut view, &main_thread_comms);
    }

    audio_thread_interface.finish_stream().unwrap();

    Ok(())
}

fn main() {
    run().unwrap()
}
