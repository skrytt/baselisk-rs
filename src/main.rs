//! Modified example of using dsp-chain's `Graph` type to create a simple synth.

extern crate dsp;
extern crate portaudio;

#[macro_use]
extern crate text_io;

mod defs;
mod dsp_node;
mod generator;
mod midi;
mod oscillator;

use std::cell::RefCell;
use std::io::prelude::*;
use std::io;
use std::rc::Rc;
use dsp::Node;
//use dsp::Walker;
use dsp::sample::ToFrameSliceMut;

use portaudio as pa;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), pa::Error> {
    println!("Starting...");

    // Use Rc+RefCell to retain usage of variables outside of the closure
    // TODO: don't hardcode value for MIDI device_id, use user input
    let midi_input_buffer = Rc::new(RefCell::new(midi::InputBuffer::new(5)));
    let graph = Rc::new(RefCell::new(dsp::Graph::new()));

    let synth = graph.borrow_mut().add_node(dsp_node::DspNode::Synth);

    graph.borrow_mut().set_master(Some(synth));

    // Clone this so we don't lose use of original due to use of move closure
    let midi_input_buffer_callback = Rc::clone(&midi_input_buffer);
    let graph_callback = Rc::clone(&graph);

    // The callback we'll use to pass to the Stream. It will request audio from our dsp_graph.
    let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
        // Refresh the MIDI input buffer with new MIDI events
        midi_input_buffer_callback.borrow_mut().update();

        let buffer: &mut [[defs::Output; defs::CHANNELS]] = buffer.to_frame_slice_mut().unwrap();
        dsp::slice::equilibrium(buffer);

        graph_callback.borrow_mut().audio_requested(buffer, defs::SAMPLE_HZ);

        //let mut inputs = graph.borrow_mut().inputs(synth);
        //while let Some(_input_idx) = inputs.next_node(graph_callback) {}

        pa::Continue
    };

    // Construct PortAudio and the stream.
    println!("Setting up interface to PortAudio...");
    let pa = pa::PortAudio::new()?;

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
    let mut running: bool = true;
    try!(stream.start());

    // Process lines of text input until told to quit or interrupted.
    while let true = try!(stream.is_active()) || running {
        print!("{}> ", defs::PROGNAME);
        io::stdout().flush().ok().expect("Could not flush stdout");

        let input_line: String = read!("{}\n");
        let input_line: String = input_line.to_lowercase();
        let input_args: Vec<&str> = input_line
            .split(' ')
            .filter( |s| { s.len() > 0 })
            .collect();
        let mut input_args_iter = input_args.iter();

        if let Some(arg) = input_args_iter.next() {

            // Users may quit by typing 'quit'.
            if *arg == "quit" {
                println!("Quitting...");
                let _ = stream.stop();
                running = false;
            }

            // The commands 'pause' and 'resume' can also control stream processing.
            else if *arg == "pause" {
                let _ = stream.stop();
            }
            else if *arg == "resume" {
                let _ = stream.start();
            }

            // Commands to add oscillators
            else if *arg == "add" {
                let stream_was_active = stream.is_active()?;

                if let Some(arg) = input_args_iter.next() {

                    match oscillator::new(*arg, Rc::clone(&midi_input_buffer)) {
                        Err(reason) => println!("{}", reason),
                        Ok(osc) => {
                            if stream_was_active {
                                let _ = stream.stop();
                            }

                            graph.borrow_mut().add_input(
                                dsp_node::DspNode::Oscillator(osc),
                                synth
                            );

                            if stream_was_active {
                                let _ = stream.start();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

