mod tree;
mod completer;

use cli::tree::{
    Tree as Tree,
    Node as Node,
};
use cli::completer::Cli as Cli;
use event::PatchEvent;
use std::str::{FromStr, SplitWhitespace};
use std::sync::mpsc;

use defs;

fn parse_from_next_token<T>(token_iter: &mut SplitWhitespace) -> Result<T, ()>
where
    T: FromStr,
{
    let token: &str = match token_iter.next() {
        None => return Err(()),
        Some(token) => token,
    };
    match token.trim().parse::<T>() {
        Err(_) => Err(()),
        Ok(value) => Ok(value),
    }
}

fn build_tree() -> Tree
{
    let mut root = Node::new_with_children();

    {
        let oscillator = root.add_child("oscillator", Node::new_with_children());

        oscillator.add_child("type", Node::new_dispatch_event(
            |token_iter| {
                let usage_str = "Syntax: oscillator type <type_name>";
                let type_name: String = match parse_from_next_token(token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(type_name) => type_name,
                };
                Ok(PatchEvent::OscillatorTypeSet{ type_name })
            }
        ));

        oscillator.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: oscillator pitch <pitch_octaves>";
                let semitones: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::OscillatorPitchSet{ semitones })
            }
        ));

        oscillator.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: oscillator pulsewidth <width>";
                let width: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::OscillatorPulseWidthSet{ width })
            }
        ));
    }
    {
        let adsr = root.add_child("adsr", Node::new_with_children());

        adsr.add_child("attack", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: adsr attack <duration>";
                let duration: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::AdsrAttackSet{ duration })
            }
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: adsr decay <duration>";
                let duration: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::AdsrDecaySet{ duration })
            }
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: adsr sustain <level>";
                let level: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::AdsrSustainSet{ level })
            }
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: adsr release <duration>";
                let duration: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::AdsrReleaseSet{ duration })
            }
        ));

    }
    {
        let filter = root.add_child("filter", Node::new_with_children());

        filter.add_child("frequency", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: filter frequency <hz>";
                let hz: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::FilterFrequencySet{ hz })
            }
        ));

        filter.add_child("sweeprange", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: filter sweeprange <octaves>";
                let octaves: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::FilterSweepRangeSet{ octaves })
            }
        ));

        filter.add_child("quality", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: filter quality <q>";
                let q: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::FilterQualitySet{ q })
            }
        ));
    }
    {
        let waveshaper = root.add_child("waveshaper", Node::new_with_children());

        waveshaper.add_child("inputgain", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: waveshaper inputgain <gain>";
                let gain: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::WaveshaperInputGainSet{ gain })
            }
        ));

        waveshaper.add_child("outputgain", Node::new_dispatch_event(
            |mut token_iter| {
                let usage_str = "Syntax: waveshaper outputgain <gain>";
                let gain: defs::Sample = match parse_from_next_token(&mut token_iter) {
                    Err(_) => return Err(String::from(usage_str)),
                    Ok(value) => value,
                };
                Ok(PatchEvent::WaveshaperOutputGainSet{ gain })
            }
        ));
    }
    Tree::new(root)
}

pub fn new(tx: mpsc::SyncSender<PatchEvent>,
           rx: mpsc::Receiver<Result<(), &'static str>>,
    ) -> Cli {
    Cli::new(build_tree(), tx, rx)
}
