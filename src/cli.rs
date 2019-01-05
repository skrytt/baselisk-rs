use audio_thread;
use comms;
use defs;
use dsp_node;
use event;
use processor;
use std::io;
use std::io::prelude::*;
use std::rc::Rc;
use view;

/// Read lines from standard input.
/// Try to parse those lines as commands, then execute those commands.
/// Return a bool indicating whether program execution should abort afterwards.
pub fn read_and_parse(
    audio: &mut audio_thread::Interface,
    view: &mut view::View,
    comms: &comms::MainThreadComms,
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
            return true; // Exit the main thread loop and terminate the program
        }
        // Commands to control PortAudio features
        else if *arg == "audio" {
            if let Some(arg) = input_args_iter.next() {
                // "portaudio list": list portaudio devices
                if *arg == "devices" {
                    println!("Audio devices:");
                    println!("{}", view.audio);
                }
            }
        }
        // Commands to control PortMidi input devices
        else if *arg == "midi" {
            if let Some(arg) = input_args_iter.next() {
                // "midi list": list the enumerated midi devices
                if *arg == "list" {
                    println!("Midi devices:");
                    println!("{}", view.midi);
                }
                // "midi input {device_id}": set device_id as the midi input device
                else if *arg == "input" {
                    if let Some(arg) = input_args_iter.next() {
                        let device_id: i32;
                        scan!(arg.bytes() => "{}", device_id);
                        comms
                            .tx
                            .send(event::Event::Patch(event::PatchEvent::MidiDeviceSet {
                                device_id,
                            }))
                            .unwrap();
                        let result = comms.rx.recv().unwrap();
                        if let Ok(_) = result {
                            view.midi.select_device(device_id as usize);
                            println!("OK");
                        }
                    }
                }
            }
        }
        // Commands to provide information about nodes
        else if *arg == "nodes" {
            if let Some(arg) = input_args_iter.next() {
                // "nodes list" : list the enumerated nodes of the graph
                // TODO: view needs updating...
                if *arg == "list" {
                    println!("{}", view.graph);
                }
            }
        }
        // Command to select a node, which will be used as the subject for some other commands.
        else if *arg == "select" {
            if let Some(arg) = input_args_iter.next() {
                // Get the node in question by accepting a node index
                let node_index: usize;
                scan!(arg.bytes() => "{}", node_index);
                comms
                    .tx
                    .send(event::Event::Patch(event::PatchEvent::NodeSelect {
                        node_index,
                    }))
                    .unwrap();
                let result = comms.rx.recv().unwrap();
                match result {
                    Err(reason) => println!("{}", reason),
                    Ok(_) => {
                        view.graph.set_selected(node_index);
                        println!("OK");
                    }
                }
            }
        }
        // Command to insert a processor after the selected node
        // "extend <node_type>"
        else if *arg == "extend" {
            if let Some(arg) = input_args_iter.next() {
                audio.exec_while_paused(|audio_thread_context| {
                    match processor::new(arg, Rc::clone(&audio_thread_context.events)) {
                        Err(reason) => println!("ERROR: {}", reason),
                        Ok(p) => {
                            let node_before_index = audio_thread_context.selected_node;

                            let synth_index = audio_thread_context.graph.master_index().unwrap();

                            // node_before is the node we'll be adding to.
                            // 1. Remove the connection between node_before and graph
                            audio_thread_context
                                .graph
                                .remove_connection(node_before_index, synth_index);

                            // 2. Connect node_before to p
                            let p_node = dsp_node::DspNode::Processor(p);
                            let (_, p_index) = audio_thread_context
                                .graph
                                .add_output(node_before_index, p_node);

                            // 3. Connect p to graph
                            audio_thread_context
                                .graph
                                .add_connection(p_index, synth_index)
                                .unwrap();

                            audio_thread_context.selected_node = p_index;

                            // Update the view now
                            view.graph.update_from_context(audio_thread_context);

                            println!("Extended node with new node and selected new node");
                        }
                    }
                })
            }
        }
        // Command to insert a processor before the selected node
        // "prepend <node_type>"
        else if *arg == "prepend" {
            if let Some(arg) = input_args_iter.next() {
                audio.exec_while_paused(|audio_thread_context| {
                    match processor::new(arg, Rc::clone(&audio_thread_context.events)) {
                        Err(reason) => println!("ERROR: {}", reason),
                        Ok(p) => {
                            let node_after_index = audio_thread_context.selected_node;

                            // node_after is the node we'll be adding to.
                            // Connect p to node_after
                            let p_node = dsp_node::DspNode::Processor(p);
                            let (_, p_index) = audio_thread_context
                                .graph
                                .add_input(p_node, node_after_index);

                            audio_thread_context.selected_node = p_index;

                            // Update the view now
                            view.graph.update_from_context(audio_thread_context);

                            println!("Prepended node with new node and selected new node");
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
                    comms
                        .tx
                        .send(event::Event::Patch(
                            event::PatchEvent::SelectedNodeSetParam {
                                param_name: param_name.clone(),
                                param_val: param_val.clone(),
                            },
                        ))
                        .unwrap();
                    let result = comms.rx.recv().unwrap();
                    match result {
                        Err(reason) => println!("{}", reason),
                        Ok(_) => {
                            view.graph.set_param(param_name, param_val).unwrap();
                            println!("OK");
                        }
                    }
                }
            }
        }
    }

    false // keep running
}
