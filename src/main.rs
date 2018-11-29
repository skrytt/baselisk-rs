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

    // Construct PortAudio and the stream.
    let mut audio_interface =
        audio::Interface::new(Arc::clone(&midi_input_buffer), Arc::clone(&graph)).unwrap();

    let mut context = application::Context {
        midi_input_buffer,
        graph,
        master_node,
        audio_interface: &mut audio_interface,
    };

    //  processing audio
    context.audio_interface.resume().unwrap();

    // Process lines of text input until told to quit or interrupted.
    while context.audio_interface.is_running() {
        cli::read_and_parse(&mut context);
    }

    Ok(())
}

fn main() {
    run().unwrap()
}
