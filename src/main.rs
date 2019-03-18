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

fn run() -> Result<(), &'static str> {
    // Initialize the audio interface
    let mut audio_thread_interface = audio_thread::Interface::new();

    audio_thread_interface.connect_and_run(|tx, rx| {
        // Process lines of text input until told to quit or interrupted.
        cli::new(tx, rx).read_until_interrupted();
    });

    Ok(())
}

fn main() {
    run().unwrap()
}
