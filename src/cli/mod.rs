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

fn parse_from_next_token<T>(token_iter: &mut SplitWhitespace) -> Result<T, String>
where
    T: FromStr,
{
    let token: &str = match token_iter.next() {
        None => return Err(String::from("Expected more tokens in command!")),
        Some(token) => token,
    };
    match token.trim().parse::<T>() {
        Err(_) => Err(format!("Couldn't parse token '{}'!", token)),
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
                let type_name: String = parse_from_next_token(token_iter)?;
                Ok(PatchEvent::OscillatorTypeSet{ type_name })
            },
            Some(String::from("[sine|saw|pulse]")),
        ));

        oscillator.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter| {
                let semitones: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::OscillatorPitchSet{ semitones })
            },
            Some(String::from("<octaves>")),
        ));

        oscillator.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter| {
                let width: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::OscillatorPulseWidthSet{ width })
            },
            Some(String::from("<width>")),
        ));
    }
    {
        let adsr = root.add_child("adsr", Node::new_with_children());

        adsr.add_child("attack", Node::new_dispatch_event(
            |mut token_iter| {
                let duration: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::AdsrAttackSet{ duration })
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |mut token_iter| {
                let duration: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::AdsrDecaySet{ duration })
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |mut token_iter| {
                let level: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::AdsrSustainSet{ level })
            },
            Some(String::from("<level>")),
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |mut token_iter| {
                let duration: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::AdsrReleaseSet{ duration })
            },
            Some(String::from("<duration>")),
        ));

    }
    {
        let filter = root.add_child("filter", Node::new_with_children());

        filter.add_child("frequency", Node::new_dispatch_event(
            |mut token_iter| {
                let hz: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::FilterFrequencySet{ hz })
            },
            Some(String::from("<Hz>")),
        ));

        filter.add_child("sweeprange", Node::new_dispatch_event(
            |mut token_iter| {
                let octaves: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::FilterSweepRangeSet{ octaves })
            },
            Some(String::from("<octaves>")),
        ));

        filter.add_child("quality", Node::new_dispatch_event(
            |mut token_iter| {
                let q: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::FilterQualitySet{ q })
            },
            Some(String::from("<q>")),
        ));
    }
    {
        let waveshaper = root.add_child("waveshaper", Node::new_with_children());

        waveshaper.add_child("inputgain", Node::new_dispatch_event(
            |mut token_iter| {
                let gain: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::WaveshaperInputGainSet{ gain })
            },
            Some(String::from("<gain>")),
        ));

        waveshaper.add_child("outputgain", Node::new_dispatch_event(
            |mut token_iter| {
                let gain: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::WaveshaperOutputGainSet{ gain })
            },
            Some(String::from("<gain>")),
        ));
    }
    Tree::new(root)
}

pub fn new(tx: mpsc::SyncSender<PatchEvent>,
           rx: mpsc::Receiver<Result<(), &'static str>>,
    ) -> Cli {
    Cli::new(build_tree(), tx, rx)
}
