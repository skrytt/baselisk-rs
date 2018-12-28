//! Synthesizer based on dsp-graph, portaudio and portmidi.
//!
extern crate ansi_term;
extern crate dsp;
extern crate portaudio;
extern crate portmidi;

#[macro_use]
extern crate text_io;

mod application;
mod audio;
mod cli;
mod comms;
mod defs;
mod dsp_node;
mod event;
mod modulator;
mod processor;
mod view;

use std::io::prelude::*;
use std::io;
use std::sync::{Arc, RwLock};

fn run() -> Result<(), &'static str> {
    let mut graph = dsp::Graph::new();
    let master_node = graph.add_node(dsp_node::DspNode::Master);
    graph.set_master(Some(master_node));

    // thread comms objects will be used to transfer events back and forth between
    // the main and audio threads.
    let (main_thread_comms, audio_thread_comms) = comms::new_bidirectional();

    let portmidi = portmidi::PortMidi::new().unwrap();

    // Create portaudio interface now so that we can initialize the device lists in the view
    // before we pass ownership of these handles to the audio thread.
    let portaudio = portaudio::PortAudio::new().unwrap();

    // Initialize the view
    let mut view = view::View::new(&portaudio, &portmidi);

    let events = Arc::new(RwLock::new(event::Buffer::new(portmidi)));

    // The graph context will only be used in the audio thread,
    // never by the main thread.
    let audio_thread_context = Arc::new(RwLock::new(application::AudioThreadContext {
        graph,
        selected_node: master_node,
        comms: audio_thread_comms,
        events,
    }));

    // Initialize the audio interface
    let mut audio_interface = audio::Interface::new(
        portaudio,
        audio_thread_context
    ).unwrap();

    // Require the user to open an audio device
    println!("Audio devices:");
    println!("{}", view.audio);

    print!("Enter an output device ID: ");
    io::stdout().flush().ok().expect("Could not flush stdout");

    let device_index: u32 = read!();

    audio_interface.open(device_index).unwrap();
    audio_interface.start().unwrap();

    // Process lines of text input until told to quit or interrupted.
    let mut finished = false;
    while !finished {
        finished = cli::read_and_parse(&mut audio_interface, &mut view, &main_thread_comms);
    }

    audio_interface.finish().unwrap();

    Ok(())
}

fn main() {
    run().unwrap()
}
