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

fn main() {
    // Initialize the audio interface
    let mut audio_thread_interface = audio_thread::Interface::new();

    audio_thread_interface.connect_and_run(|tx, rx| {
        // Process lines of text input until told to quit or interrupted.
        cli::new(tx, rx).read_until_interrupted();
    }).unwrap_or_else(|error_reason| {
        println!("{}", error_reason);
    });
}
