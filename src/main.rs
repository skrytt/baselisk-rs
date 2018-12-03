//! Synthesizer based on dsp-graph, portaudio and portmidi.

extern crate dsp;

#[macro_use]
extern crate text_io;

mod application;
mod audio;
mod cli;
mod defs;
mod dsp_node;
mod gain;
mod midi;
mod modulator;
mod oscillator;
mod processor;

use std::cell::RefCell;
use std::sync::Arc;

fn run() -> Result<(), &'static str> {
    // Use Arc+RefCell to retain usage of variables outside of the closure
    let midi_input_buffer = Arc::new(RefCell::new(midi::InputBuffer::new()));

    let graph = Arc::new(RefCell::new(dsp::Graph::new()));
    let master_node = graph.borrow_mut().add_node(dsp_node::DspNode::Master);
    graph.borrow_mut().set_master(Some(master_node));

    let context = Arc::new(RefCell::new(application::Context {
        midi_input_buffer,
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
