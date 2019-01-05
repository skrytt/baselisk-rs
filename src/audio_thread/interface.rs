extern crate portaudio;
extern crate portmidi;

use audio_thread;
use comms;
use defs;
use dsp;
use dsp::sample::ToFrameSliceMut;
use dsp::Node;
use event;
use std::rc::Rc;
use std::cell::RefCell;

pub type Stream = portaudio::Stream<portaudio::NonBlocking, portaudio::Output<defs::Output>>;

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
            Err(reason) => return Err(String::from(format!(
                        "PortAudio failed to open device with this ID: {}", reason))),
        };

        let params: portaudio::stream::Parameters<defs::Output> =
            portaudio::stream::Parameters::new(
                device_index,
                defs::CHANNELS as i32,
                true, // Interleaved audio: required for dsp-graph
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
                    let result = match event {
                        event::PatchEvent::MidiDeviceSet { device_id } => {
                            let mut events = context
                                .events
                                .borrow_mut();
                            events.midi.set_port(device_id)
                        }

                        event::PatchEvent::NodeSelect { node_index } => {
                            let node_index = dsp::NodeIndex::new(node_index);
                            match context.graph.node(node_index) {
                                None => Err(String::from("No node with specified index")),
                                Some(_) => {
                                    context.selected_node = node_index;
                                    Ok(())
                                }
                            }
                        }
                        event::PatchEvent::SelectedNodeSetParam {
                            param_name,
                            param_val,
                        } => {
                            let selected_node = context.selected_node;
                            match context.graph.node_mut(selected_node) {
                                None => Err(String::from("A non-existent node is selected")),
                                Some(node) => node.set_param(param_name, param_val),
                            }
                        }
                    };
                    context.comms.tx.send(result);
                }
            }

            context.events.borrow_mut().update_midi();

            let buffer: &mut [defs::Frame] = buffer.to_frame_slice_mut().unwrap();
            dsp::slice::equilibrium(buffer);

            context
                .graph
                .audio_requested(buffer, settings.sample_rate);

            portaudio::Continue
        };

        let stream = self.pa.open_non_blocking_stream(settings, callback).unwrap();

        self.stream = Some(stream);

        Ok(())
    }

    /// Start audio processing for a stream that has already been opened.
    pub fn start_stream(&mut self) -> Result<(), String> {
        match &mut self.stream {
            None => return Err(String::from("There is no stream open")),
            Some(stream) => {
                stream.start().unwrap();
                println!("Stream started");
                Ok(())
            }
        }
    }

    /// Stop audio processing for a stream that is open, then drop the stream handle.
    pub fn finish_stream(&mut self) -> Result<(), String> {
        if let Some(stream) = &mut self.stream {
            if stream.is_active().unwrap() {
                stream.stop().unwrap();
            }
        }
        self.stream = None;
        println!("Stream finished");
        Ok(())
    }

    /// Run a closure while the audio stream is paused, passing
    /// a mutable reference to this Context as an argument.
    /// Afterwards, restore the original state of the audio stream.
    /// This is the only permissible way that the main thread may gain
    /// any borrow of the audio thread context.
    pub fn exec_while_paused<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut audio_thread::Context),
    {
        let was_active = match &mut self.stream {
            None => false,
            Some(stream) => stream.is_active().unwrap(),
        };

        if was_active {
            if let Some(stream) = &mut self.stream {
                stream.stop().unwrap();
            }
        }

        // Give a temporary mutable borrow of this Context to the closure
        f(&mut self.context.borrow_mut());

        // If we're stopping, self.stream will be None.
        // Otherwise, resume the stream
        if was_active {
            if let Some(stream) = &mut self.stream {
                stream.start().unwrap();
            }
        }
    }
}
