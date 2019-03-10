extern crate portaudio;
extern crate portmidi;

use audio_thread;
use comms;
use defs;
use event;
use sample::{slice, ToFrameSliceMut};
use std::rc::Rc;
use std::cell::RefCell;

pub type Stream = portaudio::Stream<portaudio::NonBlocking, portaudio::Output<defs::Sample>>;

pub struct Interface {
    context: Rc<RefCell<audio_thread::Context>>,
    pa: portaudio::PortAudio,
    stream: Option<Stream>,
}

impl Interface {
    pub fn new(
        portaudio: portaudio::PortAudio,
        portmidi: portmidi::PortMidi,
        comms: comms::AudioThreadComms,
    ) -> Interface {

        // Create an object to store state of the audio thread's processing.
        let context = Rc::new(RefCell::new(audio_thread::Context::new(
            comms,      // How the audio thread will communicate with the main thread
            portmidi,   // Needed for MIDI event handling
        )));

        Interface {
            context,
            pa: portaudio,
            stream: None,
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

        // Clone some references for the audio callback
        let context_clone = Rc::clone(&self.context);

        // We don't use the Result for event handling, but the main thread does.
        #[allow(unused_must_use)]
        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
            let mut context = context_clone.borrow_mut();

            // Handle patch events but don't block if none are available.
            // Where view updates are required, send events back
            // to the main thread to indicate success or failure.
            while let Ok(event) = context.comms.rx.try_recv() {
                if let event::Event::Patch(event) = event {
                    let result: Result<(), &'static str> = match event {

                        event::PatchEvent::MidiDeviceSet { device_id } => {
                            let mut events = context
                                .events
                                .borrow_mut();
                            events.midi.set_port(device_id);
                            Ok(())
                        },
                        event::PatchEvent::OscillatorTypeSet { type_name } => {
                            context.engine.oscillator.set_type(&type_name)
                        },
                        event::PatchEvent::OscillatorPitchSet { semitones } => {
                            context.engine.oscillator.set_pitch(semitones)
                        },
                        event::PatchEvent::OscillatorPulseWidthSet { width } => {
                            context.engine.oscillator.set_pulse_width(width)
                        },
                        event::PatchEvent::FilterFrequencySet { hz } => {
                            context.engine.low_pass_filter.set_frequency(hz)
                        },
                        event::PatchEvent::FilterQualitySet { q } => {
                            context.engine.low_pass_filter.set_quality(q)
                        },
                        event::PatchEvent::AdsrAttackSet { duration } => {
                            context.engine.adsr.set_attack(duration)
                        },
                        event::PatchEvent::AdsrDecaySet { duration } => {
                            context.engine.adsr.set_decay(duration)
                        },
                        event::PatchEvent::AdsrSustainSet { level } => {
                            context.engine.adsr.set_sustain(level)
                        },
                        event::PatchEvent::AdsrReleaseSet { duration } => {
                            context.engine.adsr.set_release(duration)
                        },
                    };
                    context.comms.tx.send(result);
                }
            }

            context.events.borrow_mut().update_midi();

            let buffer: &mut defs::FrameBuffer = buffer.to_frame_slice_mut().unwrap();
            slice::equilibrium(buffer);

            context
                .engine
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
