mod tree;
mod completer;

use cli::tree::{
    Tree as Tree,
    Node as Node,
};
use cli::completer::Cli as Cli;
use event::{ModulatableParameterUpdateData,
            ControllerBindData,
            PatchEvent,
};
use parameter::{
    PARAM_ADSR_ATTACK,
    PARAM_ADSR_DECAY,
    PARAM_ADSR_SUSTAIN,
    PARAM_ADSR_RELEASE,
    PARAM_DELAY_FEEDBACK,
    PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_HIGH_PASS_FILTER_QUALITY,
    PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY,
    PARAM_DELAY_LOW_PASS_FILTER_QUALITY,
    PARAM_DELAY_WET_GAIN,
    PARAM_FILTER_FREQUENCY,
    PARAM_FILTER_SWEEP_RANGE,
    PARAM_FILTER_QUALITY,
    PARAM_OSCILLATOR_PITCH,
    PARAM_OSCILLATOR_PULSE_WIDTH,
    PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO,
    PARAM_OSCILLATOR_MOD_INDEX,
    PARAM_WAVESHAPER_INPUT_GAIN,
    PARAM_WAVESHAPER_OUTPUT_GAIN,
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

pub fn update_parameter_from_tokens(param_id: i32,
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
                param_id,
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
                param_id,
                bind_type: ControllerBindData::CliInput(field_value),
            }
        },
        "learn" => {
            PatchEvent::ControllerBindUpdate {
                param_id,
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
                    PARAM_OSCILLATOR_PITCH,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        oscillator.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_OSCILLATOR_PULSE_WIDTH,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        oscillator.add_child("modfreqratio", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO,
                    &mut token_iter)
            },
            Some(String::from("<freq_ratio>")),
        ));

        oscillator.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_OSCILLATOR_MOD_INDEX,
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
                    PARAM_ADSR_ATTACK,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_ADSR_DECAY,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_ADSR_SUSTAIN,
                    &mut token_iter)
            },
            Some(String::from("<level>")),
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_ADSR_RELEASE,
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
                        PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));

            lowpass.add_child("quality", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        PARAM_DELAY_LOW_PASS_FILTER_QUALITY,
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
                        PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));

            highpass.add_child("quality", Node::new_dispatch_event(
                |mut token_iter| {
                    update_parameter_from_tokens(
                        PARAM_DELAY_HIGH_PASS_FILTER_QUALITY,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));
        }

        delay.add_child("feedback", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_DELAY_FEEDBACK,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        delay.add_child("wetgain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_DELAY_WET_GAIN,
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
                    PARAM_FILTER_FREQUENCY,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        filter.add_child("sweeprange", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_FILTER_SWEEP_RANGE,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        filter.add_child("quality", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_FILTER_QUALITY,
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
                    PARAM_WAVESHAPER_INPUT_GAIN,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));

        waveshaper.add_child("outputgain", Node::new_dispatch_event(
            |mut token_iter| {
                update_parameter_from_tokens(
                    PARAM_WAVESHAPER_OUTPUT_GAIN,
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
