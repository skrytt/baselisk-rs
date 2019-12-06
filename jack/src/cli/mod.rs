mod tree;
mod completer;

use cli::tree::{
    Tree as Tree,
    Node as Node,
};
use cli::completer::Cli as Cli;
use baselisk_core::shared::{
    parameter::ParameterId,
    SharedState,
};
use std::str::{FromStr, SplitWhitespace};
use std::sync::Arc;

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

pub fn update_parameter_from_tokens(shared_state: &Arc<SharedState>,
                                    param: ParameterId,
                                    token_iter: &mut SplitWhitespace) -> Result<(), String>
{
    let token = match parse_from_next_token::<String>(token_iter) {
        Ok(token) => token,
        Err(reason) => return Err(reason),
    };

    // Try to get a text token: either "cc" or "learn"
    match token.as_str() {
        "cc" => {
            // Try to get a controller number
            let cc_number: u8 = match parse_from_next_token(token_iter) {
                Ok(val) => val,
                Err(reason) => return Err(reason),
            };
            shared_state.modmatrix.bind_parameter(cc_number, param);
            return Ok(())
        },
        "learn" => {
            // No further parameters
            shared_state.modmatrix.learn_parameter(param);
            return Ok(())
        },
        _ => (),
    }
    if let Err(reason) = shared_state.parameters.update_real_value_from_string(param, token) {
        return Err(String::from(reason))
    };
    Ok(())
}

fn build_tree() -> Tree
{
    let mut root = Node::new_with_children();
    {
        root.add_child("generator_routing", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorRouting,
                    &mut token_iter)
            },
            Some(String::from("<algo_name>")),
        ));

    }
    {
        root.add_child("pitchbend", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::PitchBendRange,
                    &mut token_iter)
            },
            Some(String::from("<semitones>")),
        ));
    }
    {
        let generator_a = root.add_child("generator_a", Node::new_with_children());

        generator_a.add_child("type", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorAType,
                    &mut token_iter)
            },
            Some(String::from("<type_name>")),
        ));

        generator_a.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorAPitch,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        generator_a.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorAPulseWidth,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        generator_a.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorAModIndex,
                    &mut token_iter)
            },
            Some(String::from("<index>")),
        ));
    }
    {
        let generator_b = root.add_child("generator_b", Node::new_with_children());

        generator_b.add_child("type", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorBType,
                    &mut token_iter)
            },
            Some(String::from("<type_name>")),
        ));

        generator_b.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorBPitch,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        generator_b.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorBPulseWidth,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        generator_b.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorBModIndex,
                    &mut token_iter)
            },
            Some(String::from("<index>")),
        ));
    }
    {
        let generator_c = root.add_child("generator_c", Node::new_with_children());

        generator_c.add_child("type", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorCType,
                    &mut token_iter)
            },
            Some(String::from("<type_name>")),
        ));

        generator_c.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorCPitch,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        generator_c.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorCPulseWidth,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        generator_c.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorCModIndex,
                    &mut token_iter)
            },
            Some(String::from("<index>")),
        ));
    }
    {
        let generator_d = root.add_child("generator_d", Node::new_with_children());

        generator_d.add_child("type", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorDType,
                    &mut token_iter)
            },
            Some(String::from("<type_name>")),
        ));

        generator_d.add_child("pitch", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorDPitch,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        generator_d.add_child("pulsewidth", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorDPulseWidth,
                    &mut token_iter)
            },
            Some(String::from("<width>")),
        ));

        generator_d.add_child("modindex", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::GeneratorDModIndex,
                    &mut token_iter)
            },
            Some(String::from("<index>")),
        ));
    }
    {
        let adsr = root.add_child("adsr", Node::new_with_children());

        adsr.add_child("attack", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::AdsrAttack,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("decay", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::AdsrDecay,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

        adsr.add_child("sustain", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::AdsrSustain,
                    &mut token_iter)
            },
            Some(String::from("<level>")),
        ));

        adsr.add_child("release", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::AdsrRelease,
                    &mut token_iter)
            },
            Some(String::from("<duration>")),
        ));

    }
    {
        let delay = root.add_child("delay", Node::new_with_children());

        delay.add_child("time_left", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::DelayTimeLeft,
                    &mut token_iter)
            },
            Some(String::from("<seconds>")),
        ));

        delay.add_child("time_right", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::DelayTimeRight,
                    &mut token_iter)
            },
            Some(String::from("<seconds>")),
        ));

        delay.add_child("feedback", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::DelayFeedback,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        {
            let lowpass = delay.add_child("lowpass", Node::new_with_children());

            lowpass.add_child("frequency", Node::new_dispatch_event(
                |mut token_iter, shared_state| {
                    update_parameter_from_tokens(
                        shared_state,
                        ParameterId::DelayLowPassFilterFrequency,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));
        }
        {
            let highpass = delay.add_child("highpass", Node::new_with_children());

            highpass.add_child("frequency", Node::new_dispatch_event(
                |mut token_iter, shared_state| {
                    update_parameter_from_tokens(
                        shared_state,
                        ParameterId::DelayHighPassFilterFrequency,
                        &mut token_iter)
                },
                Some(String::from("<Hz>")),
            ));
        }

        delay.add_child("wetgain", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::DelayWetGain,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));
    }
    {
        let filter = root.add_child("filter", Node::new_with_children());

        filter.add_child("frequency", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::FilterFrequency,
                    &mut token_iter)
            },
            Some(String::from("<Hz>")),
        ));

        filter.add_child("sweeprange", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::FilterSweepRange,
                    &mut token_iter)
            },
            Some(String::from("<octaves>")),
        ));

        filter.add_child("resonance", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::FilterQuality,
                    &mut token_iter)
            },
            Some(String::from("<q>")),
        ));
    }
    {
        let waveshaper = root.add_child("waveshaper", Node::new_with_children());

        waveshaper.add_child("inputgain", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::WaveshaperInputGain,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));

        waveshaper.add_child("outputgain", Node::new_dispatch_event(
            |mut token_iter, shared_state| {
                update_parameter_from_tokens(
                    shared_state,
                    ParameterId::WaveshaperOutputGain,
                    &mut token_iter)
            },
            Some(String::from("<gain>")),
        ));
    }
    Tree::new(root)
}

pub fn new(shared_state: Arc<SharedState>,
    ) -> Cli {
    Cli::new(build_tree(), shared_state)
}
