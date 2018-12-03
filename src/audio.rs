extern crate portaudio;

use std::cell::RefCell;
use std::sync::Arc;
use dsp;
use dsp::sample::ToFrameSliceMut;
use dsp::Node;

use application;
use defs;

type Stream = portaudio::Stream<portaudio::NonBlocking, portaudio::Output<defs::Output>>;

pub struct Interface
{
    context: Arc<RefCell<application::Context>>,
    pa: portaudio::PortAudio,
    stream: Option<Stream>,
    running: bool,
}

impl Interface{
    pub fn new(context: Arc<RefCell<application::Context>>
    ) -> Result<Interface, &'static str> {
        let pa = portaudio::PortAudio::new().unwrap();

        Ok(Interface {
            context: context,
            pa: pa,
            stream: None,
            running: true,
        })
    }

    pub fn list_devices(&mut self) {
        let default_output_device_index = self.pa.default_output_device().unwrap();

        let devices = self.pa.devices().unwrap();
        for device in devices {
            if let Ok(device) = device {
                let (idx, info) = device;
                print!("{}) {} : {} out, {} Hz",
                         i32::from(idx),
                         info.name,
                         info.max_output_channels,
                         info.default_sample_rate,
                );
                if idx == default_output_device_index {
                    print!(" [default output]");
                };
                println!();
            }
        }
    }

    pub fn open(&mut self, device_index: u32) -> Result<(), String>
    {
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
            defs::SAMPLE_HZ,
            defs::FRAMES,
        );

        let context_clone = Arc::clone(&self.context);

        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
            // Refresh the MIDI intput buffer with new MIDI events
            let context_borrow = context_clone.borrow_mut();
            context_borrow.midi_input_buffer.borrow_mut().update();

            let buffer: &mut [defs::Frame] = buffer.to_frame_slice_mut().unwrap();
            dsp::slice::equilibrium(buffer);

            context_borrow.graph.borrow_mut().audio_requested(buffer, defs::SAMPLE_HZ);

            portaudio::Continue
        };

        self.stream = Some(self.pa.open_non_blocking_stream(settings, callback).unwrap());

        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), String> {
        let ref mut stream = match &mut self.stream {
            None         => return Err(format!("There is no stream to resume")),
            Some(stream) => stream
        };
        match stream.start() {
            Err(e) => Err(format!("Failed to resume stream: {}", e)),
            Ok(_) => Ok(()),
        }
    }
    pub fn pause(&mut self) -> Result<(), String> {
        let ref mut stream = match &mut self.stream {
            None         => return Err(format!("There is no stream to pause")),
            Some(stream) => stream
        };
        match stream.abort() {
            Err(e) => return Err(format!("Failed to pause stream: {}", e)),
            Ok(_) => Ok(()),
        }
    }

    pub fn finish(&mut self) {
        self.running = false;
        if self.is_active() {
            self.pause().unwrap();
        }
    }

    /// Whether the stream is active (i.e. callbacks being made)
    pub fn is_active(&self) -> bool {
        let ref stream = match &self.stream {
            None         => return false,
            Some(stream) => stream
        };
        let result = stream.is_active().unwrap();
        result
    }

    /// Run a closure while the audio stream is paused, passing
    /// a mutable reference to this Context as an argument.
    /// Afterwards, restore the original state of the audio stream.
    pub fn exec_while_paused<F>(&mut self, f: F)
    where
        F: Fn(&mut application::Context),
    {
        let was_active = self.is_active();
        if was_active {
            self.pause().unwrap();
        }

        // Give a temporary mutable borrow of this Context to the closure
        f(&mut self.context.borrow_mut());

        if was_active {
            self.resume().unwrap();
        }
    }
}
