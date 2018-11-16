//! Modified example of using dsp-chain's `Graph` type to create a simple synth.

extern crate dsp;
extern crate portaudio;

mod defs;
mod dsp_node;
mod generator;
mod midi;
mod oscillator;

use std::rc::Rc;
use std::cell::RefCell;
use dsp::{Node, Walker};
use dsp::sample::ToFrameSliceMut;

use portaudio as pa;

fn main() {
    run().unwrap()
}

fn setup_graph(midi_input_buffer: Rc<RefCell<midi::InputBuffer>>) -> (
    dsp::Graph<[defs::Output; 2], dsp_node::DspNode<defs::Output>>,
    dsp::NodeIndex,
)
{
    println!("Setting up graph...");

    let mut graph = dsp::Graph::new();
    let synth = graph.add_node(dsp_node::DspNode::Synth);

    graph.add_input(dsp_node::DspNode::Oscillator(Box::new(
        oscillator::SineOscillator{
            params: oscillator::Params::new(0.2),
            midi_input_buffer: Rc::clone(&midi_input_buffer),
        })),
        synth
    );

    graph.set_master(Some(synth));

    (graph, synth)
}

fn run() -> Result<(), pa::Error> {
    println!("Starting...");

    // TODO: don't hardcode value for MIDI device_id, use user input
    let midi_input_buffer = Rc::new(RefCell::new(midi::InputBuffer::new(5)));

    let (mut graph, synth) = setup_graph(Rc::clone(&midi_input_buffer));

    // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        // Refresh the MIDI input buffer with new MIDI events
        midi_input_buffer.borrow_mut().update();

        let buffer: &mut [[defs::Output; defs::CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
        dsp::slice::equilibrium(buffer);

        graph.audio_requested(buffer, defs::SAMPLE_HZ);
        let mut inputs = graph.inputs(synth);
        while let Some(_input_idx) = inputs.next_node(&graph) {
        }

        pa::Continue
    };

    // Construct PortAudio and the stream.
    println!("Setting up interface to PortAudio...");
    let pa = try!(pa::PortAudio::new());

    match pa.default_host_api() {
        Ok(v) => println!("Default Host API: {:#?}", v),
        Err(e) => panic!("Can't get default host API: {}", e),
    }

    let settings = try!(pa.default_output_stream_settings::<defs::Output>(
        defs::CHANNELS as i32,
        defs::SAMPLE_HZ,
        defs::FRAMES,
    ));

    println!("Opening and starting PortAudio stream...");
    let mut stream = try!(pa.open_non_blocking_stream(settings, callback));
    try!(stream.start());

    // Wait for our stream to finish.
    while let true = try!(stream.is_active()) {
        ::std::thread::sleep(::std::time::Duration::from_millis(16));
    }

    Ok(())
}

