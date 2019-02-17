use audio_thread;
use comms;
use defs;
use event;
use std::io;
use std::io::prelude::*;
use view;

/// Read lines from standard input.
/// Try to parse those lines as commands, then execute those commands.
/// Return a bool indicating whether program execution should abort afterwards.
pub fn read_and_parse(
    _audio: &mut audio_thread::Interface,
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
        else if *arg == "osc" {
            if let Some(arg) = input_args_iter.next() {
                // "osc <osc_type_name>" : set the osc type
                comms
                    .tx
                    .send(event::Event::Patch(event::PatchEvent::OscillatorTypeSet {
                        type_name: String::from(*arg),
                    }))
                    .unwrap();
                let result = comms.rx.recv().unwrap();
                if let Ok(_) = result {
                    println!("OK");
                }
            }
        }
    }

    false // keep running
}
