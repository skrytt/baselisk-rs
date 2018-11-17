//! Modified example of using dsp-chain's `Graph` type to create a simple synth.

extern crate dsp;

#[macro_use]
extern crate text_io;


mod defs;
mod dsp_node;
mod generator;
mod midi;
mod oscillator;
mod audio;

use std::cell::RefCell;
use std::io::prelude::*;
use std::io;
use std::rc::Rc;

fn main() {
    run().unwrap()
}

fn run() -> Result<(), &'static str> {
    println!("Starting...");

    // Use Rc+RefCell to retain usage of variables outside of the closure
    let midi_input_buffer = Rc::new(RefCell::new(midi::InputBuffer::new()));
    let graph = Rc::new(RefCell::new(dsp::Graph::new()));
    let synth = graph.borrow_mut().add_node(dsp_node::DspNode::Synth);

    graph.borrow_mut().set_master(Some(synth));

    // Construct PortAudio and the stream.
    let mut audio_interface = audio::Interface::new(
        Rc::clone(&midi_input_buffer),
        Rc::clone(&graph),
    )?;

    audio_interface.resume().unwrap();

    // Process lines of text input until told to quit or interrupted.
    while let true = audio_interface.is_running() {
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
                let _ = audio_interface.finish();
            }

            // The commands 'pause' and 'resume' can also control stream processing.
            else if *arg == "pause" {
                let _ = audio_interface.pause();
            }
            else if *arg == "resume" {
                let _ = audio_interface.resume();
            }

            // Commands to control MIDI input devices
            else if *arg == "midi" {
                if let Some(arg) = input_args_iter.next() {
                    if *arg == "list" {
                        midi_input_buffer.borrow().print_devices();
                    }
                    else if *arg == "input" {
                        if let Some(arg) = input_args_iter.next() {
                            let device_id: i32;
                            scan!(arg.bytes() => "{}", device_id);
                            midi_input_buffer.borrow_mut().set_port(device_id).unwrap();
                        }
                    }
                }
            }

            // Commands to add oscillators
            else if *arg == "add" {
                let audio_was_active = audio_interface.is_active();

                if let Some(arg) = input_args_iter.next() {

                    match oscillator::new(*arg, Rc::clone(&midi_input_buffer)) {
                        Err(reason) => println!("{}", reason),
                        Ok(osc) => {
                            if audio_was_active {
                                let _ = audio_interface.pause();
                            }

                            graph.borrow_mut().add_input(
                                dsp_node::DspNode::Oscillator(osc),
                                synth
                            );

                            if audio_was_active {
                                let _ = audio_interface.resume();
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

