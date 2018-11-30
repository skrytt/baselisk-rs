use application;
use defs;
use dsp;
use dsp_node;
use gain;
use oscillator;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;

pub fn read_and_parse(ctx: &mut application::Context) {
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
            let _ = ctx.audio_interface.finish();
        }
        // The commands 'pause' and 'resume' can also control stream processing.
        else if *arg == "pause" {
            let _ = ctx.audio_interface.pause();
        } else if *arg == "resume" {
            let _ = ctx.audio_interface.resume();
        }
        // Commands to control MIDI input devices
        else if *arg == "midi" {
            if let Some(arg) = input_args_iter.next() {
                // "midi list": list the enumerated midi devices
                if *arg == "list" {
                    ctx.midi_input_buffer.borrow().print_devices();
                }
                // "midi input {device_id}": set device_id as the midi input device
                else if *arg == "input" {
                    if let Some(arg) = input_args_iter.next() {
                        ctx.exec_while_paused(|ctx| {
                            let device_id: i32;
                            scan!(arg.bytes() => "{}", device_id);
                            ctx.midi_input_buffer
                                .borrow_mut()
                                .set_port(device_id)
                                .unwrap();
                        })
                    }
                }
            }
        }
        else if *arg == "portaudio" {
            if let Some(arg) = input_args_iter.next() {
                if *arg == "list" {
                    ctx.exec_while_paused(|ctx| {
                        ctx.audio_interface.list_devices();
                    })
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
                    ctx.exec_while_paused(|ctx| {
                        let mut graph_borrow = ctx.graph.borrow_mut();
                        let nodes_iter = graph_borrow.nodes_mut().enumerate();
                        for (i, node) in nodes_iter {
                            println!("{}: {}", i, node);
                        }
                    })
                }
            }
        }
        // Commands to add oscillators
        else if *arg == "add" {
            if let Some(arg) = input_args_iter.next() {
                ctx.exec_while_paused(|ctx| {
                    match oscillator::new(*arg, Arc::clone(&ctx.midi_input_buffer)) {
                        Err(reason) => println!("{}", reason),
                        Ok(osc) => {
                            ctx.graph
                                .borrow_mut()
                                .add_input(dsp_node::DspNode::Source(osc), ctx.master_node);
                        }
                    }
                })
            }
        }
        // Commands to insert filters after other nodes
        else if *arg == "extend" {
            if let Some(arg) = input_args_iter.next() {
                // Get the node in question by accepting a node index
                let node_index: usize;
                scan!(arg.bytes() => "{}", node_index);

                let node_before_index = dsp::NodeIndex::new(node_index);

                // Ok, the node exists, now make a new node to put after it
                if let Some(arg) = input_args_iter.next() {
                    ctx.exec_while_paused(|ctx| {
                        match gain::new(*arg, Arc::clone(&ctx.midi_input_buffer)) {
                            Err(reason) => println!("{}", reason),
                            Ok(p) => {
                                // graph borrow scope, so that we release borrow
                                // before the audio interface claims it
                                let mut graph_borrow = ctx.graph.borrow_mut();

                                let synth_index = graph_borrow.master_index().unwrap();

                                // node_before is the node we'll be adding to.
                                // 1. Remove the connection between node_before and graph
                                graph_borrow.remove_connection(node_before_index, synth_index);

                                // 2. Connect node_before to p
                                let p_node = dsp_node::DspNode::Processor(p);
                                let (_, p_index) =
                                    graph_borrow.add_output(node_before_index, p_node);

                                // 3. Connect p to graph
                                graph_borrow.add_connection(p_index, synth_index).unwrap();
                            }
                        }
                    })
                }
            }
        }
    }
}
