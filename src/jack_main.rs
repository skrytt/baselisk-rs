//! Synthesizer.
//!
extern crate clap;
extern crate sample;
extern crate rustyline;
extern crate time;

// vst crate used here for AtomicFloat type
extern crate vst;

#[cfg(feature = "jack")]
extern crate jack;

mod cli;
mod engine;
mod defs;
mod event;
mod parameter;

use event::PatchEvent;
use sample::ToFrameSliceMut;
use std::sync::{Arc, RwLock, mpsc};

#[allow(clippy::cast_precision_loss)]

/// Try to open an audio stream with the device corresponding to the
/// Return a Result indicating whether this was successful.
pub fn connect_and_run<F>(engine: &mut Arc<RwLock<engine::Engine>>,
                          mut f: F) -> Result<(), &'static str>
where
    F: FnMut(
        mpsc::SyncSender<PatchEvent>,
        mpsc::Receiver<Result<(), &'static str>>,
    ),
{
    let (client, _status) = match jack::Client::new(defs::PLUGIN_NAME,
                                                    jack::ClientOptions::NO_START_SERVER)
    {
        Err(_) => return Err("Failed to connect to JACK server"),
        Ok((client, status)) => (client, status),
    };

    let mut left_output_port = match client.register_port("output_left",
                                                     jack::AudioOut::default())
    {
        Err(_) => return Err("Failed to open output audio port"),
        Ok(left_output_port) => left_output_port,
    };

    let mut right_output_port = match client.register_port("output_right",
                                                     jack::AudioOut::default())
    {
        Err(_) => return Err("Failed to open output audio port"),
        Ok(right_output_port) => right_output_port,
    };

    let midi_input_port = match client.register_port("midi_input",
                                                     jack::MidiIn::default())
    {
        Err(_) => return Err("Failed to open input midi port"),
        Ok(midi_input_port) => midi_input_port,
    };

    let engine_callback = Arc::clone(engine);

    let (tx_main_thread, rx_audio_thread) = mpsc::sync_channel(256);
    let (tx_audio_thread, rx_main_thread) = mpsc::sync_channel(256);

    let process = jack::ClosureProcessHandler::new(
        move |client: &jack::Client, process_scope: &jack::ProcessScope| -> jack::Control {
            let left_output_buffer = left_output_port.as_mut_slice(process_scope)
                .to_frame_slice_mut().unwrap();
            let right_output_buffer = right_output_port.as_mut_slice(process_scope)
                .to_frame_slice_mut().unwrap();

            let raw_midi_iter = midi_input_port.iter(process_scope);

            let mut engine_callback = engine_callback.write().unwrap();
            engine_callback.set_sample_rate(client.sample_rate() as defs::Sample);
            engine_callback.apply_patch_events(&rx_audio_thread, &tx_audio_thread);
            engine_callback.jack_audio_requested(left_output_buffer, right_output_buffer, raw_midi_iter);

            jack::Control::Continue
        }
    );

    // active_client is not directly used, but must be kept in scope
    let _active_client = client.activate_async((), process).unwrap();

    f(tx_main_thread, rx_main_thread);

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
    connect_and_run(&mut engine, |tx, rx| {
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
