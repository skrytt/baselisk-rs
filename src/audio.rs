extern crate portaudio;
extern crate portmidi;

use application;
use defs;
use dsp;
use dsp::sample::ToFrameSliceMut;
use dsp::Node;
use event;
use std::sync::{Arc, RwLock};

pub type Stream = portaudio::Stream<portaudio::NonBlocking, portaudio::Output<defs::Output>>;

pub struct Interface
{
    audio_thread_context: Arc<RwLock<application::AudioThreadContext>>,
    pa: portaudio::PortAudio,
    stream: Option<Stream>,
}

impl Interface{
    pub fn new(
        portaudio: portaudio::PortAudio,
        audio_thread_context: Arc<RwLock<application::AudioThreadContext>>,
    ) -> Result<Interface, &'static str> {
        Ok(Interface {
            audio_thread_context,
            pa: portaudio,
            stream: None,
        })
    }

    /// Open an audio stream.
    pub fn open(&mut self, device_index: u32) -> Result<(), String>
    {
        if let Some(_) = self.stream {
            return Err(String::from("Stream is already open"));
        }

        let device_index = portaudio::DeviceIndex(device_index);
        let device_info = self.pa.device_info(device_index).unwrap();

        let params: portaudio::stream::Parameters<defs::Output> = portaudio::stream::Parameters::new(
            device_index,
            defs::CHANNELS as i32,
            true, // Interleaved audio: required for dsp-graph
            device_info.default_low_output_latency
        );

        let settings = portaudio::stream::OutputSettings::new(
            params,
            device_info.default_sample_rate,
            defs::FRAMES,
        );

        // Clone some references for the audio callback
        let audio_thread_context_clone = Arc::clone(&self.audio_thread_context);

        // We don't use the Result for event handling, but the main thread does.
        #[allow(unused_must_use)]
        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
            let mut audio_thread_context = audio_thread_context_clone.try_write()
                .expect("Context was locked when audio callback was called");

            // Handle patch events but don't block if none are available.
            // Where view updates are required, send events back
            // to the main thread to indicate success or failure.
            while let Ok(event) = audio_thread_context.comms.rx.try_recv() {
                if let event::Event::Patch(event) = event {
                    let result = match event {
                        event::PatchEvent::MidiDeviceSet{device_id} => {
                            let mut events = audio_thread_context.events.try_write()
                                .expect("Event buffer unexpectedly locked");
                            events.midi.set_port(device_id)
                        },

                        event::PatchEvent::NodeSelect{node_index} => {
                            println!("Got event NodeSelect {}", node_index);
                            let node_index = dsp::NodeIndex::new(node_index);
                            match audio_thread_context.graph.node(node_index) {
                                None    => Err(String::from("No node with specified index")),
                                Some(_) => {
                                    audio_thread_context.selected_node = node_index;
                                    Ok(())
                                }
                            }
                        },
                        event::PatchEvent::SelectedNodeSetParam{param_name, param_val} => {
                            println!("Got event SelectedNodeSetParam {} {}", param_name, param_val);
                            let selected_node = audio_thread_context.selected_node;
                            match audio_thread_context.graph.node_mut(selected_node) {
                                None => Err(String::from("A non-existent node is selected")),
                                Some(node) => node.set_param(param_name, param_val),
                            }
                        },
                    };
                    audio_thread_context.comms.tx.send(result);
                }
            }

            audio_thread_context.events.try_write().unwrap().update_midi();

            let buffer: &mut [defs::Frame] = buffer.to_frame_slice_mut().unwrap();
            dsp::slice::equilibrium(buffer);

            audio_thread_context.graph.audio_requested(buffer, settings.sample_rate);

            portaudio::Continue
        };

        let stream = self.pa.open_non_blocking_stream(
                settings, callback).unwrap();

        self.stream = Some(stream);

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), String> {
        match &mut self.stream {
            None => return Err(String::from("There is no stream open")),
            Some(stream) => {
                stream.start().unwrap();
                println!("Stream started");
                Ok(())
            }
        }
    }

    pub fn finish(&mut self) -> Result<(), String> {
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
    pub fn exec_while_paused<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut application::AudioThreadContext),
    {
        let was_active = match &mut self.stream {
            None         => false,
            Some(stream) => stream.is_active().unwrap(),
        };

        if was_active {
            if let Some(stream) = &mut self.stream {
                stream.stop().unwrap();
            }
        }

        // Give a temporary mutable borrow of this Context to the closure
        f(
            &mut self.audio_thread_context.try_write().unwrap(),
        );

        // If we're stopping, self.stream will be None.
        // Otherwise, resume the stream
        if was_active {
            if let Some(stream) = &mut self.stream {
                stream.start().unwrap();
            }
        }
    }
}
