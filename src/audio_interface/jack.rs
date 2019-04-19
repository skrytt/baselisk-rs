extern crate jack;

use defs;
use engine;
use event::PatchEvent;
use sample::ToFrameSliceMut;
use std::sync::{Arc, RwLock, mpsc};

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
    let (client, _status) = match jack::Client::new(defs::JACK_CLIENT_NAME,
                                                    jack::ClientOptions::NO_START_SERVER)
    {
        Err(_) => return Err("Failed to connect to JACK server"),
        Ok((client, status)) => (client, status),
    };

    let mut output_port = match client.register_port("output",
                                                     jack::AudioOut::default())
    {
        Err(_) => return Err("Failed to open output audio port"),
        Ok(output_port) => output_port,
    };

    let midi_input_port = match client.register_port("midi_input",
                                                     jack::MidiIn::default())
    {
        Err(_) => return Err("Failed to open input midi port"),
        Ok(midi_input_port) => midi_input_port,
    };

    // Set the engine buffer sizes here, before the audio callbacks start
    engine.write().unwrap()
        .set_buffer_sizes(client.buffer_size() as usize);

    let engine_callback = Arc::clone(engine);

    let (tx_main_thread, rx_audio_thread) = mpsc::sync_channel(256);
    let (tx_audio_thread, rx_main_thread) = mpsc::sync_channel(256);

    let process = jack::ClosureProcessHandler::new(
        move |client: &jack::Client, process_scope: &jack::ProcessScope| -> jack::Control {
            let buffer = output_port.as_mut_slice(process_scope)
                .to_frame_slice_mut().unwrap();

            let raw_midi_iter = midi_input_port.iter(process_scope);

            let mut engine_callback = engine_callback.write().unwrap();
            engine_callback.apply_patch_events(&rx_audio_thread, &tx_audio_thread);
            engine_callback.audio_requested(buffer,
                                            raw_midi_iter,
                                            client.sample_rate() as defs::Sample);

            jack::Control::Continue
        }
    );

    // active_client is not directly used, but must be kept in scope
    let _active_client = client.activate_async((), process).unwrap();

    f(tx_main_thread, rx_main_thread);

    // active_client will be dropped here
    Ok(())
}
