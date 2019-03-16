extern crate portaudio;
extern crate portmidi;

use audio_thread;
use comms;
use defs;
use sample::{slice, ToFrameSliceMut};
use std::rc::Rc;
use std::cell::RefCell;

pub type Stream = portaudio::Stream<portaudio::NonBlocking, portaudio::Output<defs::Sample>>;

pub struct Interface {
    pa: portaudio::PortAudio,
    stream: Option<Stream>,
    engine: Rc<RefCell<audio_thread::Engine>>,
}

impl Interface {
    pub fn new(
        portaudio: portaudio::PortAudio,
        portmidi: portmidi::PortMidi,
        comms: comms::AudioThreadComms,
    ) -> Interface {
        Interface {
            pa: portaudio,
            stream: None,
            engine: Rc::new(RefCell::new(audio_thread::Engine::new(comms, portmidi))),
        }
    }

    /// Try to open an audio stream with the device corresponding to the
    /// provided device_index.
    /// Return a Result indicating whether this was successful.
    pub fn open_stream(&mut self, device_index: u32) -> Result<(), String> {
        if let Some(_) = self.stream {
            return Err(String::from("Stream is already open"));
        }

        let device_index = portaudio::DeviceIndex(device_index);
        let device_info = match self.pa.device_info(device_index) {
            Ok(result) => result,
            Err(reason) => return Err(format!(
                    "PortAudio failed to open specified device: {}", reason))
        };

        let params: portaudio::stream::Parameters<defs::Sample> =
            portaudio::stream::Parameters::new(
                device_index,
                defs::CHANNELS as i32,
                true, // Interleaved audio
                device_info.default_low_output_latency,
            );

        let settings = portaudio::stream::OutputSettings::new(
            params,
            device_info.default_sample_rate,
            defs::FRAMES,
        );

        let engine_callback = Rc::clone(&self.engine);

        // We don't use the Result for event handling, but the main thread does.
        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
            let buffer: &mut defs::FrameBuffer = buffer.to_frame_slice_mut().unwrap();
            slice::equilibrium(buffer);

            engine_callback.borrow_mut()
                .audio_requested(buffer, settings.sample_rate as defs::Sample);

            portaudio::Continue
        };

        let stream = self.pa.open_non_blocking_stream(settings, callback).unwrap();

        self.stream = Some(stream);

        Ok(())
    }

    /// Start audio processing for a stream that has already been opened.
    pub fn start_stream(&mut self) -> Result<(), &'static str> {
        match &mut self.stream {
            None => return Err("There is no stream open"),
            Some(stream) => {
                stream.start().unwrap();
                println!("Stream started");
                Ok(())
            }
        }
    }

    /// Stop audio processing for a stream that is open, then drop the stream handle.
    pub fn finish_stream(&mut self) -> Result<(), &'static str> {
        if let Some(stream) = &mut self.stream {
            if stream.is_active().unwrap() {
                stream.stop().unwrap();
            }
        }
        self.stream = None;
        println!("Stream finished");
        Ok(())
    }
}
