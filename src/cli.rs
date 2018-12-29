use audio;
use comms;
use defs;
use dsp_node;
use event;
use std::io;
use std::io::prelude::*;
use view;
use processor;
use std::sync::Arc;


pub fn read_and_parse(
    audio: &mut audio::Interface,
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
            audio.finish().unwrap();
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
                        //midi.set_port(device_id).unwrap();
                        comms.tx.send(event::Event::Patch(
                                event::PatchEvent::MidiDeviceSet{device_id})).unwrap();
                        let result = comms.rx.recv().unwrap();
                        if let Ok(_) = result {
                            //TODO: add some way of viewing which MIDI input device is in use
                            //println!("OK");
                            println!("OK, but view not updated");
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
                    println!("{}", view.nodes);
                }
            }
        }
        // Commands to add oscillators
        else if *arg == "add" {
            if let Some(arg) = input_args_iter.next() {
                audio.exec_while_paused(|audio_thread_context| {
                    match processor::new_source(arg, Arc::clone(&audio_thread_context.events)) {
                        Err(reason) => println!("{}", reason),
                        Ok((osc, _view)) => {
                            let master_index = audio_thread_context.graph.master_index().unwrap();
                            let (_, node_index) = audio_thread_context.graph.add_input(
                                dsp_node::DspNode::Processor(osc),
                                master_index);
                            audio_thread_context.selected_node = node_index;

                            // Update the view now
                            view.nodes.update_from_context(audio_thread_context);

                            println!("Added and selected node");
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
                comms.tx.send(event::Event::Patch(
                        event::PatchEvent::NodeSelect{node_index})).unwrap();
                let result = comms.rx.recv().unwrap();
                match result {
                    Err(reason) => println!("{}", reason),
                    Ok(_) => {
                        view.nodes.set_selected(node_index);
                        println!("OK");
                    }
                }
            }
        }
        // Command to insert filters after the selected node
        // "extend <node_type>"
        else if *arg == "extend" {
            // Ok, the node exists, now make a new node to put after it
            if let Some(arg) = input_args_iter.next() {
                audio.exec_while_paused(|audio_thread_context| {
                    match processor::new_processor(arg, Arc::clone(&audio_thread_context.events)) {
                        Err(reason) => println!("{}", reason),
                        Ok((p, _view)) => {
                            let node_before_index = audio_thread_context.selected_node;

                            let synth_index = audio_thread_context.graph.master_index().unwrap();

                            // node_before is the node we'll be adding to.
                            // 1. Remove the connection between node_before and graph
                            audio_thread_context.graph.remove_connection(node_before_index, synth_index);

                            // 2. Connect node_before to p
                            let p_node = dsp_node::DspNode::Processor(p);
                            let (_, p_index) =
                                audio_thread_context.graph.add_output(node_before_index, p_node);

                            // 3. Connect p to graph
                            audio_thread_context.graph.add_connection(p_index, synth_index).unwrap();

                            audio_thread_context.selected_node = p_index;

                            // Update the view now
                            view.nodes.update_from_context(audio_thread_context);

                            println!("Extended node with new node and selected new node");
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
                    comms.tx.send(event::Event::Patch(event::PatchEvent::SelectedNodeSetParam{
                        param_name,
                        param_val,
                    })).unwrap();
                    let result = comms.rx.recv().unwrap();
                    match result {
                        Err(reason) => println!("{}", reason),
                        Ok(_) => {
                            //TODO: implement update_param
                            //view.nodes.update_param(param_name, param_val);
                            //println!("OK");
                            println!("OK, but view not updated");
                        }
                    }
                }
            }
        }
    }

    false // keep running
}
