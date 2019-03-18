extern crate jack;

use audio_thread;
use defs;
use event::Event;
use sample::ToFrameSliceMut;
use std::sync::{Arc, RwLock, mpsc};

pub struct Interface {
    engine: Arc<RwLock<audio_thread::Engine>>,
}

impl Interface {
    pub fn new() -> Interface {
        Interface {
            engine: Arc::new(RwLock::new(audio_thread::Engine::new())),
        }
    }

    /// Try to open an audio stream with the device corresponding to the
    /// Return a Result indicating whether this was successful.
    pub fn connect_and_run<F>(&mut self, mut f: F)
    where
        F: FnMut(
            mpsc::SyncSender<Event>,
            mpsc::Receiver<Result<(), &'static str>>,
        ),
    {
        let (client, _status) =
            jack::Client::new(defs::JACK_CLIENT_NAME, jack::ClientOptions::NO_START_SERVER).unwrap();

        let mut output_port = client
            .register_port("output", jack::AudioOut::default())
            .unwrap();

        let midi_input_port = client
            .register_port("midi_input", jack::MidiIn::default())
            .unwrap();

        let engine_callback = Arc::clone(&self.engine);

        let (tx_main_thread, rx_audio_thread) = mpsc::sync_channel(256);
        let (tx_audio_thread, rx_main_thread) = mpsc::sync_channel(256);

        // We don't use the Result for event handling, but the main thread does.
        let process = jack::ClosureProcessHandler::new(
            move |client: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
                let buffer = output_port.as_mut_slice(ps)
                    .to_frame_slice_mut().unwrap();

                let raw_midi_iter = midi_input_port.iter(ps);

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
    }
}
