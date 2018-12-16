use audio;
use defs;
use dsp;
use dsp_node;
use event;
use processor::{gain, oscillator};
use std::io;
use std::io::prelude::*;
use std::sync::{Arc, mpsc};

pub fn read_and_parse(
    audio_interface: &mut audio::Interface,
    sender: &mpsc::Sender<event::Event>,
) -> bool {
    print!("{}> ", defs::PROGNAME);
    io::stdout().flush().ok().expect("Could not flush stdout");

    let input_line: String = read!("{}\n");
    let input_line: String = input_line.to_lowercase();
    let input_args: Vec<&str> = input_line.split(' ').filter(|s| s.len() > 0).collect();

    let mut input_args_iter = input_args.iter();

    if let Some(arg) = input_args_iter.next() {
        // Users may quit by typing 'quit'.
        if *arg == "quit" {
            println!("Quitting...");
            audio_interface.finish();
            return true; // Exit the main thread loop and terminate the program
        }
        // Commands to control PortAudio features
        else if *arg == "audio" {
            if let Some(arg) = input_args_iter.next() {
                // "portaudio list": list portaudio devices
                if *arg == "devices" {
                    audio_interface.list_devices();
                }
                // "portaudio open {device_index}": open a portaudio device
                else if *arg == "open" {
                    if let Some(arg) = input_args_iter.next() {
                        let device_index: u32;
                        scan!(arg.bytes() => "{}", device_index);
                        audio_interface.open(device_index).unwrap();
                    }
                }
                // "audio pause" and "audio resume" can control playback of an opened stream.
                else if *arg == "pause" {
                    let _ = audio_interface.pause();
                } else if *arg == "resume" {
                    let _ = audio_interface.resume();
                }
            }
        }

        // Commands to control PortMidi input devices
        else if *arg == "midi" {
            if let Some(arg) = input_args_iter.next() {
                // "midi list": list the enumerated midi devices
                if *arg == "list" {
                    audio_interface.exec_with_context_mut(|_context, event_buffer| {
                        let event_buffer = event_buffer.try_read()
                            .expect("Event buffer unexpectedly locked");
                        event_buffer.midi.print_devices();
                    })
                }
                // "midi input {device_id}": set device_id as the midi input device
                else if *arg == "input" {
                    if let Some(arg) = input_args_iter.next() {
                        audio_interface.exec_with_context_mut(|_context, event_buffer| {
                            let device_id: i32;
                            scan!(arg.bytes() => "{}", device_id);
                            let mut event_buffer = event_buffer.try_write()
                                .expect("Event buffer unexpectedly locked");
                            event_buffer.midi.set_port(device_id).unwrap();
                        })
                    }
                }
            }
        }
        // Commands to provide information about nodes
        else if *arg == "nodes" {
            if let Some(arg) = input_args_iter.next() {
                // "nodes list" : list the enumerated nodes of the graph
                if *arg == "list" {
                    // graph borrow scope, so that we release borrow
                    // before the audio interface claims it
                    audio_interface.exec_with_context_mut(|context, _event_buffer| {
                        let nodes_iter = context.graph.nodes_mut().enumerate();
                        for (i, node) in nodes_iter {
                            print!("{}: {}", i, node);
                            if dsp::NodeIndex::new(i) == context.selected_node {
                                print!(" [selected]");
                            }
                            println!();
                        }
                    })
                }
            }
        }
        // Commands to add oscillators
        else if *arg == "add" {
            if let Some(arg) = input_args_iter.next() {
                audio_interface.exec_with_context_mut(|context, event_buffer| {
                    match oscillator::new(*arg, Arc::clone(event_buffer)) {
                        Err(reason) => println!("{}", reason),
                        Ok(osc) => {
                            let master_index = context.graph.master_index().unwrap();
                            let (_, node_index) = context.graph.add_input(
                                dsp_node::DspNode::Processor(osc),
                                master_index);
                            context.selected_node = node_index;
                        }
                    }
                })
            }
        }
        // Command to select a node, which will be used as the subject for some other commands.
        else if *arg == "select" {
            if let Some(arg) = input_args_iter.next() {
                // Get the node in question by accepting a node index
                let node_index: usize;
                scan!(arg.bytes() => "{}", node_index);

                audio_interface.exec_with_context_mut(|context, _event_buffer| {
                    let node_index = dsp::NodeIndex::new(node_index);
                    match context.graph.node(node_index) {
                        None    => println!("Invalid node index"),
                        Some(_) => {
                            context.selected_node = node_index;
                        }
                    }
                })
            }
        }
        // Command to insert filters after the selected node
        // "extend <node_type>"
        else if *arg == "extend" {
            // Ok, the node exists, now make a new node to put after it
            if let Some(arg) = input_args_iter.next() {
                audio_interface.exec_with_context_mut(|context, event_buffer| {
                    match gain::new(*arg, Arc::clone(event_buffer)) {
                        Err(reason) => println!("{}", reason),
                        Ok(p) => {
                            let node_before_index = context.selected_node;

                            let synth_index = context.graph.master_index().unwrap();

                            // node_before is the node we'll be adding to.
                            // 1. Remove the connection between node_before and graph
                            context.graph.remove_connection(node_before_index, synth_index);

                            // 2. Connect node_before to p
                            let p_node = dsp_node::DspNode::Processor(p);
                            let (_, p_index) =
                                context.graph.add_output(node_before_index, p_node);

                            // 3. Connect p to graph
                            context.graph.add_connection(p_index, synth_index).unwrap();

                            context.selected_node = p_index;
                        }
                    }
                })
            }
        }
        // For the selected node, set a parameter.
        else if *arg == "paramset" {
            if let Some(param_name) = input_args_iter.next() {
                if let Some(param_val) = input_args_iter.next() {
                    let param_name = String::from(*param_name);
                    let param_val = String::from(*param_val);
                    sender.send(event::Event::Patch(event::PatchEvent::SetParam{
                        param_name,
                        param_val,
                    })).unwrap();
                }
            }
        }
    }

    false // keep running
}
