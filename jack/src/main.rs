//! Synthesizer.
extern crate baselisk_core;
extern crate clap;
extern crate jack;
extern crate rustyline;
extern crate sample;

mod cli;
mod midi;

use baselisk_core::defs;
use baselisk_core::engine;
use baselisk_core::shared::SharedState;

use sample::ToFrameSliceMut;
use std::sync::{Arc, RwLock};

#[warn(clippy::all)]
#[allow(clippy::cast_precision_loss)]


/// Try to open an audio stream with the device corresponding to the
/// Return a Result indicating whether this was successful.
pub fn connect_and_run<F: FnOnce()>(
    engine: &mut Arc<RwLock<engine::Engine>>,
    f: F
) -> Result<(), jack::Error>
{
    let (client, _status) = jack::Client::new(
        defs::PLUGIN_NAME, jack::ClientOptions::NO_START_SERVER)?;

    // Set up audio and MIDI ports with JACK
    let mut left_output_port = client.register_port(
        "output_left", jack::AudioOut::default())?;
    let mut right_output_port = client.register_port(
        "output_right", jack::AudioOut::default())?;
    let midi_input_port = client.register_port(
        "midi_input", jack::MidiIn::default())?;

    // Configure sample rate to match JACK
    engine.write().unwrap()
        .set_sample_rate(client.sample_rate() as defs::Sample);

    // We're going to start a background thread that will need a
    // reference to the audio engine object.
    let engine = Arc::clone(engine);

    // Define the background thread here
    let process = jack::ClosureProcessHandler::new(
        move |_client: &jack::Client, process_scope: &jack::ProcessScope| -> jack::Control {
            let left_output_buffer = left_output_port.as_mut_slice(process_scope)
                .to_frame_slice_mut().unwrap();
            let right_output_buffer = right_output_port.as_mut_slice(process_scope)
                .to_frame_slice_mut().unwrap();

            let raw_midi_iter = midi_input_port.iter(process_scope);

            let mut engine = engine.write().unwrap();

            // Clear old MIDI events and convert new JACK raw MIDI into a generic format
            engine.clear_midi_buffer();
            for jack_raw_midi_event in raw_midi_iter {
                engine.push_raw_midi(
                    midi::raw_midi_from_jack(&jack_raw_midi_event));
            }

            engine.audio_requested(left_output_buffer, right_output_buffer);

            jack::Control::Continue
        }
    );

    // active_client is not directly used, but must be kept in scope
    let _active_client = client.activate_async((), process).unwrap();

    f();

    // active_client will be dropped here
    Ok(())
}
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

    // Parameters will be shared between UI and audioengine threads
    let shared_state = Arc::new(SharedState::new());

    let mut engine = Arc::new(RwLock::new(engine::Engine::new(
        Arc::clone(&shared_state),
        matches.is_present("timing-dump") // Whether to enable timing info
    )));

    // Initialize the audio interface
    connect_and_run(&mut engine, || {
        let mut cli = cli::new(shared_state);

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
