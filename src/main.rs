//! Synthesizer based on dsp-graph, portaudio and portmidi.
//!
extern crate ansi_term;
extern crate dsp;

#[macro_use]
extern crate text_io;

mod application;
mod audio;
mod cli;
mod defs;
mod dsp_node;
mod event;
mod modulator;
mod processor;

use std::sync::{Arc, RwLock, mpsc};

fn run() -> Result<(), &'static str> {
    // Create a channel for transferring messages from main thread to audio thread.
    let (sender, receiver) = mpsc::channel();

    let event_buffer = Arc::new(RwLock::new(
            event::Buffer::new(receiver)));

    let mut graph = dsp::Graph::new();
    let master_node = graph.add_node(dsp_node::DspNode::Master);
    graph.set_master(Some(master_node));

    let context = Arc::new(RwLock::new(application::Context {
        graph,
        selected_node: master_node,
    }));

    let mut audio_interface = audio::Interface::new(event_buffer, context).unwrap();

    // Process lines of text input until told to quit or interrupted.
    let mut finished = false;
    while !finished {
        finished = cli::read_and_parse(&mut audio_interface, &sender);
    }

    audio_interface.finish();

    Ok(())
}

fn main() {
    run().unwrap()
}
