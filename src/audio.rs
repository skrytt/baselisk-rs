
extern crate portaudio;

use std::cell::RefCell;
use std::rc::Rc;
use dsp;
use dsp::sample::ToFrameSliceMut;
use dsp::Node;

use midi;
use defs;
use dsp_node;

pub struct Interface {
    stream: portaudio::Stream<portaudio::NonBlocking, portaudio::Output<f32>>,
    running: bool,
}

impl Interface
{
    pub fn new(
        midi_input_buffer: Rc<RefCell<midi::InputBuffer>>,
        graph: Rc<RefCell<dsp::Graph<[f32; 2], dsp_node::DspNode<f32>>>>,
    ) -> Result<Interface, &'static str>
    {
        println!("Setting up interface to PortAudio...");
        let pa = portaudio::PortAudio::new().unwrap();

        println!("{}", pa.default_host_api().unwrap());

        let settings = pa.default_output_stream_settings::<defs::Output>(
            defs::CHANNELS as i32,
            defs::SAMPLE_HZ,
            defs::FRAMES,
        ).unwrap();

        // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
        let callback = move |portaudio::OutputStreamCallbackArgs { buffer, .. }| {
            // Refresh the MIDI input buffer with new MIDI events
            midi_input_buffer.borrow_mut().update();

            let buffer: &mut [[defs::Output; defs::CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
            dsp::slice::equilibrium(buffer);

            graph.borrow_mut().audio_requested(buffer, defs::SAMPLE_HZ);

            //let mut inputs = graph.borrow_mut().inputs(synth);
            //while let Some(_input_idx) = inputs.next_node(graph_callback) {}

            portaudio::Continue
        };

        println!("Opening PortAudio stream...");
        let stream = pa.open_non_blocking_stream(settings, callback).unwrap();

        Ok(Interface{stream, running: true})
    }

    pub fn resume(&mut self) -> Result<(), String> {
        match self.stream.start() {
            Err(e) => Err(format!("Failed to start stream: {}", e)),
            Ok(_) => Ok(()),
        }
    }
    pub fn pause(&mut self) -> Result<(), String> {
        match self.stream.stop() {
            Err(e) => return Err(format!("Failed to start stream: {}", e)),
            Ok(_) => Ok(()),
        }
    }

    pub fn finish(&mut self) {
        self.running = false;
        self.pause().unwrap();
    }

    /// Whether the stream is active (i.e. callbacks being made)
    pub fn is_active(&self) -> bool {
        self.stream.is_active().unwrap()
    }

    /// Whether the stream is open but not active (i.e. no callbacks)
    pub fn is_running(&self) -> bool {
        self.stream.is_active().unwrap() || self.running
    }
}