mod tree;
mod completer;

use cli::tree::{
    Tree as Tree,
    Node as Node,
};
use cli::completer::Cli as Cli;
use event::{ModulatableParameter,
            ModulatableParameterUpdateData,
            ControllerBindData,
            PatchEvent,
};
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

pub fn update_parameter_from_tokens(parameter: &ModulatableParameter,
                                    token_iter: &mut SplitWhitespace) -> Result<PatchEvent, String>
{
    let field_name: String = match parse_from_next_token(token_iter) {
        Ok(val) => val,
        Err(_) => return Err(String::from("Could not parse a field name")),
    };
    let patch_event: PatchEvent = match field_name.as_str() {
        "base" | "max" => {
            // Try to get a value
            let field_value: defs::Sample = match parse_from_next_token(token_iter) {
                Ok(val) => val,
                Err(_) => return Err(String::from("Could not parse a field value")),
            };
            let parameter_update_data = match field_name.as_str() {
                "base" => ModulatableParameterUpdateData::Base(field_value),
                "max" => ModulatableParameterUpdateData::Max(field_value),
                _ => panic!(), // Not actually possible - TODO refactor this
            };
            PatchEvent::ModulatableParameterUpdate {
                parameter: parameter.clone(),
                data: parameter_update_data,
            }
        },
        "cc" => {
            // Try to get a value
            let field_value: u8 = match parse_from_next_token(token_iter) {
                Ok(val) => val,
                Err(_) => return Err(String::from("Could not parse a field value")),
            };
            PatchEvent::ControllerBindUpdate {
                parameter: parameter.clone(),
                bind_type: ControllerBindData::CliInput(field_value),
            }
        },
        "learn" => {
            PatchEvent::ControllerBindUpdate {
                parameter: parameter.clone(),
                bind_type: ControllerBindData::MidiLearn,
            }
        },
        _ => return Err(String::from("Unknown field name")),
    };

    Ok(patch_event)
}

fn build_tree() -> Tree
{
    let mut root = Node::new_with_children();

    {
        root.add_child("pitchbend", Node::new_dispatch_event(
            |mut token_iter| {
                let semitones: defs::Sample = parse_from_next_token(&mut token_iter)?;
                Ok(PatchEvent::PitchBendRangeSet{ semitones })
            },
            Some(String::from("<semitones>")),
        ));
    }
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
                update_parameter_from_tokens(
                    &ModulatableParameter::OscillatorPitch,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        oscillator.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::OscillatorPulseWidth,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        oscillator.add_child("modfreqratio", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::OscillatorModFrequencyRatio,
                    &mut token_iter)
            },
            Some(String::from("<freq_ratio>")),
        ));

        oscillator.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::OscillatorModIndex,
                    &mut token_iter)
            },
            Some(String::from("<index>")),
        ));
    }
    {
        let adsr = root.add_child("adsr", Node::new_with_children());

        adsr.add_child("attack", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::AdsrAttack,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::AdsrDecay,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::AdsrSustain,
                    &mut token_iter)
            },
            Some(String::from("<level>")),
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::AdsrRelease,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

    }
    {
        let delay = root.add_child("delay", Node::new_with_children());

        {
            let lowpass = delay.add_child("lowpass", Node::new_with_children());

            lowpass.add_child("frequency", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        &ModulatableParameter::DelayLowPassFilterFrequency,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));

            lowpass.add_child("quality", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        &ModulatableParameter::DelayLowPassFilterQuality,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));
        }
        {
            let highpass = delay.add_child("highpass", Node::new_with_children());

            highpass.add_child("frequency", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        &ModulatableParameter::DelayHighPassFilterFrequency,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));

            highpass.add_child("quality", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        &ModulatableParameter::DelayHighPassFilterQuality,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));
        }

        delay.add_child("feedback", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::DelayFeedback,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        delay.add_child("wetgain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::DelayWetGain,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));
    }
    {
        let filter = root.add_child("filter", Node::new_with_children());

        filter.add_child("frequency", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::FilterFrequency,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        filter.add_child("sweeprange", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::FilterSweepRange,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        filter.add_child("quality", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::FilterQuality,
                    &mut token_iter)
            },
            Some(String::from("<q>")),
        ));
    }
    {
        let waveshaper = root.add_child("waveshaper", Node::new_with_children());

        waveshaper.add_child("inputgain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::WaveshaperInputGain,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));

        waveshaper.add_child("outputgain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    &ModulatableParameter::WaveshaperOutputGain,
                    &mut token_iter)
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
