mod tree;
mod completer;

use cli::tree::{
    Tree as Tree,
    Node as Node,
};
use cli::completer::Cli as Cli;
use comms::MainThreadComms;
use event::{Event, PatchEvent};

use defs;

fn build_tree() -> Tree
{
    let mut root = Node::new_with_children();

    {
        let midi = root.add_child("midi", Node::new_with_children());

        // midi.add_child("list", Node::??);
        midi.add_child("input", Node::new_dispatch_event(
            |token_vec| {
                let device_id = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: midi input <device_id>")),
                    Some(device_id_str) => {
                        let device_id: i32;
                        scan!(device_id_str.bytes() => "{}", device_id);
                        device_id
                    },
                };
                Ok(Event::Patch(PatchEvent::MidiDeviceSet{ device_id }))
            }

        ));
    }
    {
        let oscillator = root.add_child("oscillator", Node::new_with_children());

        oscillator.add_child("type", Node::new_dispatch_event(
            |token_vec| {
                let type_name = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: oscillator type <type_name>")),
                    Some(type_name) => type_name.to_string(),
                };
                Ok(Event::Patch(PatchEvent::OscillatorTypeSet{ type_name }))
            }
        ));

        oscillator.add_child("pitch", Node::new_dispatch_event(
            |token_vec| {
                let semitones = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: oscillator pitch <pitch_octaves>")),
                    Some(semitones_str) => {
                        let semitones: defs::Sample;
                        scan!(semitones_str.bytes() => "{}", semitones);
                        semitones
                    },
                };
                Ok(Event::Patch(PatchEvent::OscillatorPitchSet{ semitones }))
            }
        ));

        oscillator.add_child("pulsewidth", Node::new_dispatch_event(
            |token_vec| {
                let width = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: oscillator pulsewidth <width>")),
                    Some(width_str) => {
                        let width: defs::Sample;
                        scan!(width_str.bytes() => "{}", width);
                        width
                    },
                };
                Ok(Event::Patch(PatchEvent::OscillatorPulseWidthSet{ width }))
            }
        ));
    }
    {
        let adsr = root.add_child("adsr", Node::new_with_children());

        adsr.add_child("attack", Node::new_dispatch_event(
            |token_vec| {
                let duration = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: envelope attack <duration>")),
                    Some(duration_str) => {
                        let duration: defs::Sample;
                        scan!(duration_str.bytes() => "{}", duration);
                        duration
                    },
                };
                Ok(Event::Patch(PatchEvent::AdsrAttackSet{ duration }))
            }
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |token_vec| {
                let duration = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: envelope decay <duration>")),
                    Some(duration_str) => {
                        let duration: defs::Sample;
                        scan!(duration_str.bytes() => "{}", duration);
                        duration
                    },
                };
                Ok(Event::Patch(PatchEvent::AdsrDecaySet{ duration }))
            }
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |token_vec| {
                let level = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: envelope sustain <level>")),
                    Some(level_str) => {
                        let level: defs::Sample;
                        scan!(level_str.bytes() => "{}", level);
                        level
                    },
                };
                Ok(Event::Patch(PatchEvent::AdsrSustainSet{ level }))
            }
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |token_vec| {
                let duration = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: envelope release <duration>")),
                    Some(duration_str) => {
                        let duration: defs::Sample;
                        scan!(duration_str.bytes() => "{}", duration);
                        duration
                    },
                };
                Ok(Event::Patch(PatchEvent::AdsrReleaseSet{ duration }))
            }
        ));

    }
    {
        let filter = root.add_child("filter", Node::new_with_children());

        filter.add_child("frequency", Node::new_dispatch_event(
            |token_vec| {
                let hz = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: filter frequency <hz>")),
                    Some(hz_str) => {
                        let hz: defs::Sample;
                        scan!(hz_str.bytes() => "{}", hz);
                        hz
                    },
                };
                Ok(Event::Patch(PatchEvent::FilterFrequencySet{ hz }))
            }
        ));

        filter.add_child("quality", Node::new_dispatch_event(
            |token_vec| {
                let q = match token_vec.iter().next() {
                    None => return Err(String::from(
                            "Syntax: filter quality <q>")),
                    Some(q_str) => {
                        let q: defs::Sample;
                        scan!(q_str.bytes() => "{}", q);
                        q
                    },
                };
                Ok(Event::Patch(PatchEvent::FilterQualitySet{ q }))
            }
        ));
    }
    Tree::new(root)
}

pub fn new(comms: MainThreadComms) -> Cli {
    Cli::new(build_tree(), comms)
}
