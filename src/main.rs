//! Synthesizer.
//!
extern crate clap;
extern crate sample;
extern crate rustyline;
extern crate time;

#[cfg(feature = "vst")]
#[macro_use]
extern crate vst;

#[cfg(feature = "jack")]
extern crate jack;

mod audio_interface;
mod buffer;
mod cli;
mod engine;
mod defs;
mod event;
mod parameter;
mod processor;

use std::sync::{Arc, RwLock};

fn main() {
    // Command-line argument parsing
    let matches = clap::App::new("Baselisk")
        .about("A monophonic synthesizer")

        .arg(clap::Arg::with_name("patchfile")
             .short("f")
             .long("patchfile")
             .help("Load a patch from a text file")
             .takes_value(true))

        .arg(clap::Arg::with_name("timing-dump")
             .long("timing-dump")
             .help("Output timing information to stderr"))

        .get_matches();

    let mut engine = Arc::new(RwLock::new(engine::Engine::new(
        matches.is_present("timing-dump") // Whether to enable timing info
    )));

    // Initialize the audio interface
    // Behaviour depends on what plugin format we're compiling for
    #[cfg(all(not(feature = "vst"), not(feature = "jack")))]
    panic!("Need to specify one of 'jack' or 'vst' as a compilation feature");

    #[cfg(all(feature = "vst", feature = "jack"))]
    panic!("Cannot specify both 'jack' and 'vst' as compilation features");

    #[cfg(feature = "vst")]
    panic!("VST unimplemented yet");

    #[cfg(feature = "jack")]
    audio_interface::connect_and_run(&mut engine, |tx, rx| {
        let mut cli = cli::new(tx, rx);

        // If a patchfile is specified, load and process it now.
        if let Some(file_path) = matches.value_of("patchfile") {
            cli.read_from_file(file_path).unwrap();
        }

        // Process lines of text input until told to quit or interrupted.
        cli.read_until_interrupted();

    }).unwrap_or_else(|error_reason| {
        println!("{}", error_reason);
    });
}
