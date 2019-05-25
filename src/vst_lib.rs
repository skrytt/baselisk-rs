//! Synthesizer.
//!

#[cfg(feature = "plugin_vst")]
#[macro_use]
extern crate vst;

extern crate sample;

mod buffer;
mod defs;
mod engine;
mod event;
mod parameter;
mod processor;

#[cfg(feature = "plugin_vst")]
use sample::ToFrameSliceMut;
#[cfg(feature = "plugin_vst")]
use vst::{
    api::Events,
    buffer::AudioBuffer,
    plugin::{Category, Info, Plugin, PluginParameters},
    util::AtomicFloat,
};
#[cfg(feature = "plugin_vst")]
use std::sync::Arc;


#[allow(clippy::cast_precision_loss)]

#[cfg(feature = "plugin_vst")]
struct BaseliskPlugin {
    engine: engine::Engine,
    params: Arc<BaseliskPluginParameters>,
}

#[cfg(feature = "plugin_vst")]
impl Default for BaseliskPlugin {
    fn default() -> BaseliskPlugin {
        BaseliskPlugin {
            engine: engine::Engine::new(false),
            params: Default::default(),
        }
    }
}

#[cfg(feature = "plugin_vst")]
impl Plugin for BaseliskPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: defs::PLUGIN_NAME.to_string(),
            unique_id: 5211,
            category: Category::Synth,
            parameters: NUM_PARAMS,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        self.engine.vst_process_events(events);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.engine.set_sample_rate(sample_rate as defs::Sample)
    }

    fn process(&mut self, vst_audio_buffer: &mut AudioBuffer<defs::Sample>) {
        let (_, outputs) = vst_audio_buffer.split();

        // Currently will only output audio to first output buffer
        let output_buffer = outputs.get_mut(0)
            .to_frame_slice_mut().unwrap();

        self.engine.audio_requested(output_buffer)
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
            Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

const PARAM_ADSR_ATTACK: i32 = 0;
const PARAM_ADSR_DECAY: i32 = 1;
const PARAM_ADSR_SUSTAIN: i32 = 2;
const PARAM_ADSR_RELEASE: i32 = 3;
const PARAM_DELAY_FEEDBACK: i32 = 4;
const PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY: i32 = 5;
const PARAM_DELAY_HIGH_PASS_FILTER_QUALITY: i32 = 6;
const PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY: i32 = 7;
const PARAM_DELAY_LOW_PASS_FILTER_QUALITY: i32 = 8;
const PARAM_DELAY_WET_GAIN: i32 = 9;
const PARAM_FILTER_FREQUENCY: i32 = 10;
const PARAM_FILTER_SWEEP_RANGE: i32 = 11;
const PARAM_FILTER_QUALITY: i32 = 12;
const PARAM_OSCILLATOR_PITCH: i32 = 13;
const PARAM_OSCILLATOR_PULSE_WIDTH: i32 = 14;
const PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO: i32 = 15;
const PARAM_OSCILLATOR_MOD_INDEX: i32 = 16;
const PARAM_WAVESHAPER_INPUT_GAIN: i32 = 17;
const PARAM_WAVESHAPER_OUTPUT_GAIN: i32 = 18;
const NUM_PARAMS: i32 = 19;

#[cfg(feature = "plugin_vst")]
struct BaseliskPluginParameters {
    adsr_attack: AtomicFloat,
    adsr_decay: AtomicFloat,
    adsr_sustain: AtomicFloat,
    adsr_release: AtomicFloat,
    delay_feedback: AtomicFloat,
    delay_high_pass_filter_frequency: AtomicFloat,
    delay_high_pass_filter_quality: AtomicFloat,
    delay_low_pass_filter_frequency: AtomicFloat,
    delay_low_pass_filter_quality: AtomicFloat,
    delay_wet_gain: AtomicFloat,
    filter_frequency: AtomicFloat,
    filter_sweep_range: AtomicFloat,
    filter_quality: AtomicFloat,
    oscillator_pitch: AtomicFloat,
    oscillator_pulse_width: AtomicFloat,
    oscillator_mod_frequency_ratio: AtomicFloat,
    oscillator_mod_index: AtomicFloat,
    waveshaper_input_gain: AtomicFloat,
    waveshaper_output_gain: AtomicFloat,
}

#[cfg(feature = "plugin_vst")]
impl Default for BaseliskPluginParameters {
    fn default() -> BaseliskPluginParameters {
        BaseliskPluginParameters {
            adsr_attack: AtomicFloat::new(0.02),
            adsr_decay: AtomicFloat::new(0.707),
            adsr_sustain: AtomicFloat::new(0.0),
            adsr_release: AtomicFloat::new(0.4),
            delay_feedback: AtomicFloat::new(0.6),
            delay_high_pass_filter_frequency: AtomicFloat::new(125.0),
            delay_high_pass_filter_quality: AtomicFloat::new(0.707),
            delay_low_pass_filter_frequency: AtomicFloat::new(5000.0),
            delay_low_pass_filter_quality: AtomicFloat::new(0.707),
            delay_wet_gain: AtomicFloat::new(0.4),
            filter_frequency: AtomicFloat::new(10.0),
            filter_sweep_range: AtomicFloat::new(6.5),
            filter_quality: AtomicFloat::new(0.707),
            oscillator_pitch: AtomicFloat::new(0.0),
            oscillator_pulse_width: AtomicFloat::new(0.5),
            oscillator_mod_frequency_ratio: AtomicFloat::new(1.0),
            oscillator_mod_index: AtomicFloat::new(1.0),
            waveshaper_input_gain: AtomicFloat::new(0.333),
            waveshaper_output_gain: AtomicFloat::new(0.2),

        }
    }
}

#[cfg(feature = "plugin_vst")]
impl PluginParameters for BaseliskPluginParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.get(),
            PARAM_ADSR_DECAY => self.adsr_decay.get(),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.get(),
            PARAM_ADSR_RELEASE => self.adsr_release.get(),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.get(),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.get(),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.get(),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.get(),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.get(),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.get(),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.get(),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.get(),
            PARAM_FILTER_QUALITY => self.filter_quality.get(),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.get(),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.get(),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.get(),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.get(),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.get(),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.get(),
            _ => 0.0,
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            PARAM_ADSR_ATTACK => format!("{}", self.adsr_attack.get()),
            PARAM_ADSR_DECAY => format!("{}", self.adsr_decay.get()),
            PARAM_ADSR_SUSTAIN => format!("{}", self.adsr_sustain.get()),
            PARAM_ADSR_RELEASE => format!("{}", self.adsr_release.get()),
            PARAM_DELAY_FEEDBACK => format!("{}", self.delay_feedback.get()),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => format!(
                "{}", self.delay_high_pass_filter_frequency.get()),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => format!(
                "{}", self.delay_high_pass_filter_quality.get()),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => format!(
                "{}", self.delay_low_pass_filter_frequency.get()),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => format!(
                "{}", self.delay_low_pass_filter_quality.get()),
            PARAM_DELAY_WET_GAIN => format!("{}", self.delay_wet_gain.get()),
            PARAM_FILTER_FREQUENCY => format!("{}", self.filter_frequency.get()),
            PARAM_FILTER_SWEEP_RANGE => format!("{}", self.filter_sweep_range.get()),
            PARAM_FILTER_QUALITY => format!("{}", self.filter_quality.get()),
            PARAM_OSCILLATOR_PITCH => format!("{}", self.oscillator_pitch.get()),
            PARAM_OSCILLATOR_PULSE_WIDTH => format!("{}", self.oscillator_pulse_width.get()),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => format!("{}", self.oscillator_mod_frequency_ratio.get()),
            PARAM_OSCILLATOR_MOD_INDEX => format!("{}", self.oscillator_mod_index.get()),
            PARAM_WAVESHAPER_INPUT_GAIN => format!("{}", self.waveshaper_input_gain.get()),
            PARAM_WAVESHAPER_OUTPUT_GAIN => format!("{}", self.waveshaper_output_gain.get()),
            _ => format!(""),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        match index {
            PARAM_ADSR_ATTACK => String::from("adsr attack"),
            PARAM_ADSR_DECAY => String::from("adsr decay"),
            PARAM_ADSR_SUSTAIN => String::from("adsr sustain"),
            PARAM_ADSR_RELEASE => String::from("adsr release"),
            PARAM_DELAY_FEEDBACK => String::from("delay feedback"),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => String::from(
                    "delay high pass filter frequency"),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => String::from(
                    "delay high pass filter quality"),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => String::from(
                    "delay low pass filter frequency"),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => String::from(
                    "delay low pass filter quality"),
            PARAM_DELAY_WET_GAIN => String::from("delay wet gain"),
            PARAM_FILTER_FREQUENCY => String::from("filter frequency"),
            PARAM_FILTER_SWEEP_RANGE => String::from("filter sweep range"),
            PARAM_FILTER_QUALITY => String::from("filter quality"),
            PARAM_OSCILLATOR_PITCH => String::from("oscillator pitch"),
            PARAM_OSCILLATOR_PULSE_WIDTH => String::from("oscillator pulse width"),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => String::from(
                    "oscillator mod frequency ratio"),
            PARAM_OSCILLATOR_MOD_INDEX => String::from("oscillator mod index"),
            PARAM_WAVESHAPER_INPUT_GAIN => String::from("waveshaper input gain"),
            PARAM_WAVESHAPER_OUTPUT_GAIN => String::from("waveshaper output gain"),
            _ => String::from(""),
        }
    }

    fn set_parameter(&self, index: i32, val: f32) {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.set(val),
            PARAM_ADSR_DECAY => self.adsr_decay.set(val),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.set(val),
            PARAM_ADSR_RELEASE => self.adsr_release.set(val),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.set(val),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.set(val),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.set(val),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.set(val),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.set(val),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.set(val),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.set(val),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.set(val),
            PARAM_FILTER_QUALITY => self.filter_quality.set(val),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.set(val),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.set(val),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.set(val),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.set(val),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.set(val),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.set(val),
            _ => (),
        }
    }
}

#[cfg(feature = "plugin_vst")]
plugin_main!(BaseliskPlugin);
