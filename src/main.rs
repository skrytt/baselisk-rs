//! Synthesizer based on dsp-graph, portaudio and portmidi.
//!
extern crate ansi_term;
extern crate arraydeque;
extern crate jack;
extern crate sample;
extern crate rustyline;

mod audio_thread;
mod cli;
mod defs;
mod event;
mod processor;

use std::sync::{Arc, RwLock};

fn main() {
    let mut engine = Arc::new(RwLock::new(audio_thread::Engine::new()));

    // Initialize the audio interface
    audio_thread::connect_and_run(&mut engine, |tx, rx| {
        // Process lines of text input until told to quit or interrupted.
        cli::new(tx, rx).read_until_interrupted();
    }).unwrap_or_else(|error_reason| {
        println!("{}", error_reason);
    });
}
