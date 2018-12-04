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
mod gain;
mod modulator;
mod oscillator;
mod processor;

use std::sync::{Arc, RwLock};

fn run() -> Result<(), &'static str> {
    // Use Arc+RwLock to retain usage of variables outside of the closure
    let event_buffer = Arc::new(RwLock::new(event::Buffer::new()));

    let graph = Arc::new(RwLock::new(dsp::Graph::new()));
    let master_node = graph.write().unwrap().add_node(dsp_node::DspNode::Master);
    graph.write().unwrap().set_master(Some(master_node));

    let context = Arc::new(RwLock::new(application::Context {
        event_buffer,
        graph,
        master_node,
    }));

    let mut audio_interface = audio::Interface::new(context).unwrap();

    // Process lines of text input until told to quit or interrupted.
    let mut finished = false;
    while !finished {
        finished = cli::read_and_parse(&mut audio_interface);
    }

    audio_interface.finish();

    Ok(())
}

fn main() {
    run().unwrap()
}
