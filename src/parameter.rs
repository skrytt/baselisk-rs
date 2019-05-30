use defs;

use vst;
use vst::util::AtomicFloat;
use std::sync::atomic::{AtomicUsize, Ordering};

pub const PARAM_ADSR_ATTACK: i32 = 0;
pub const PARAM_ADSR_DECAY: i32 = 1;
pub const PARAM_ADSR_SUSTAIN: i32 = 2;
pub const PARAM_ADSR_RELEASE: i32 = 3;
pub const PARAM_DELAY_FEEDBACK: i32 = 4;
pub const PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY: i32 = 5;
pub const PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY: i32 = 6;
pub const PARAM_DELAY_WET_GAIN: i32 = 7;
pub const PARAM_FILTER_FREQUENCY: i32 = 8;
pub const PARAM_FILTER_SWEEP_RANGE: i32 = 9;
pub const PARAM_FILTER_RESONANCE: i32 = 10;
pub const PARAM_OSCILLATOR_TYPE: i32 = 11;
pub const PARAM_OSCILLATOR_PITCH: i32 = 12;
pub const PARAM_OSCILLATOR_PULSE_WIDTH: i32 = 13;
pub const PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO: i32 = 14;
pub const PARAM_OSCILLATOR_MOD_INDEX: i32 = 15;
pub const PARAM_WAVESHAPER_INPUT_GAIN: i32 = 16;
pub const PARAM_WAVESHAPER_OUTPUT_GAIN: i32 = 17;

#[cfg(feature = "plugin_vst")]
pub const NUM_PARAMS: i32 = 18;

pub struct BaseliskPluginParameters {
    adsr_attack: ExponentialParameter,
    adsr_decay: ExponentialParameter,
    adsr_sustain: LinearParameter,
    adsr_release: ExponentialParameter,
    delay_feedback: LinearParameter,
    delay_high_pass_filter_frequency: ExponentialParameter,
    delay_low_pass_filter_frequency: ExponentialParameter,
    delay_wet_gain: LinearParameter,
    filter_frequency: ExponentialParameter,
    filter_sweep_range: LinearParameter,
    filter_quality: ExponentialParameter,
    oscillator_type: EnumParameter,
    oscillator_pitch: LinearParameter,
    oscillator_pulse_width: LinearParameter,
    oscillator_mod_frequency_ratio: LinearParameter,
    oscillator_mod_index: LinearParameter,
    waveshaper_input_gain: LinearParameter,
    waveshaper_output_gain: LinearParameter,
}

impl Default for BaseliskPluginParameters {
    fn default() -> BaseliskPluginParameters {
        BaseliskPluginParameters {
            adsr_attack: ExponentialParameter::new(0.001, 10.0, 0.02),
            adsr_decay: ExponentialParameter::new(0.02, 10.0, 0.707),
            adsr_sustain: LinearParameter::new(0.0, 1.0, 0.0),
            adsr_release: ExponentialParameter::new(0.02, 10.0, 0.4),
            delay_feedback: LinearParameter::new(0.0, 1.0, 0.6),
            delay_high_pass_filter_frequency: ExponentialParameter::new(20.0, 22000.0, 100.0),
            delay_low_pass_filter_frequency: ExponentialParameter::new(20.0, 22000.0, 5000.0),
            delay_wet_gain: LinearParameter::new(0.0, 1.0, 0.4),
            filter_frequency: ExponentialParameter::new(20.0, 22000.0, 100.0),
            filter_sweep_range: LinearParameter::new(0.0, 10.0, 8.0),
            filter_quality: ExponentialParameter::new(0.5, 10.0, 0.707),
            oscillator_type: EnumParameter::new(
                vec!["sine", "saw", "pulse", "fm"],
                1,
            ),
            oscillator_pitch: LinearParameter::new(-36.0, 36.0, 0.0),
            oscillator_pulse_width: LinearParameter::new(0.01, 0.99, 0.5),
            oscillator_mod_frequency_ratio: LinearParameter::new(1.0, 8.0, 1.0),
            oscillator_mod_index: LinearParameter::new(0.0, 8.0, 1.0),
            waveshaper_input_gain: LinearParameter::new(0.0, 1.0, 0.333),
            waveshaper_output_gain: LinearParameter::new(0.0, 1.0, 1.0),

        }
    }
}

/// Here we implement the VST PluginParameters trait.
/// VST plugins use a range of 0.0 >= value >= 1.0 for all parameters.
/// This means that when using the value, we need to transform i
impl vst::plugin::PluginParameters for BaseliskPluginParameters {
    fn get_parameter(&self, index: i32) -> defs::Sample {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.get_vst_param(),
            PARAM_ADSR_DECAY => self.adsr_decay.get_vst_param(),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.get_vst_param(),
            PARAM_ADSR_RELEASE => self.adsr_release.get_vst_param(),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.get_vst_param(),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.get_vst_param(),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.get_vst_param(),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.get_vst_param(),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.get_vst_param(),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.get_vst_param(),
            PARAM_FILTER_RESONANCE => self.filter_quality.get_vst_param(),
            PARAM_OSCILLATOR_TYPE => self.oscillator_type.get_vst_param(),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.get_vst_param(),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.get_vst_param(),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.get_vst_param(),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.get_vst_param(),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.get_vst_param(),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.get_vst_param(),
            _ => 0.0,
        }
    }

    fn get_parameter_text(&self, index: i32) -> String {
        match index {
            PARAM_ADSR_ATTACK => format!(
                "{} s", self.adsr_attack.get_real_value()),
            PARAM_ADSR_DECAY => format!(
                "{} s", self.adsr_decay.get_real_value()),
            PARAM_ADSR_SUSTAIN => format!(
                "{} %", 100.0 * self.adsr_sustain.get_real_value()),
            PARAM_ADSR_RELEASE => format!(
                "{} s", self.adsr_release.get_real_value()),
            PARAM_DELAY_FEEDBACK => format!(
                "{} %", 100.0 * self.delay_feedback.get_real_value()),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => format!(
                "{} Hz", self.delay_high_pass_filter_frequency.get_real_value()),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => format!(
                "{} Hz", self.delay_low_pass_filter_frequency.get_real_value()),
            PARAM_DELAY_WET_GAIN => format!(
                "{} %", 100.0 * self.delay_wet_gain.get_real_value()),
            PARAM_FILTER_FREQUENCY => format!(
                "{} Hz", self.filter_frequency.get_real_value()),
            PARAM_FILTER_SWEEP_RANGE => format!(
                "{} octaves", self.filter_sweep_range.get_real_value()),
            PARAM_FILTER_RESONANCE => format!(
                "{}", self.filter_quality.get_real_value()),
            PARAM_OSCILLATOR_TYPE => format!(
                "{}", self.oscillator_type.get_real_value()),
            PARAM_OSCILLATOR_PITCH => format!(
                "{} semitones", self.oscillator_pitch.get_real_value()),
            PARAM_OSCILLATOR_PULSE_WIDTH => format!(
                "{} %", 100.0 * self.oscillator_pulse_width.get_real_value()),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => format!(
                "{}", self.oscillator_mod_frequency_ratio.get_real_value()),
            PARAM_OSCILLATOR_MOD_INDEX => format!(
                "{}", self.oscillator_mod_index.get_real_value()),
            PARAM_WAVESHAPER_INPUT_GAIN => format!(
                "{} %", 100.0 * self.waveshaper_input_gain.get_real_value()),
            PARAM_WAVESHAPER_OUTPUT_GAIN => format!(
                "{} %", 100.0 * self.waveshaper_output_gain.get_real_value()),
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
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => String::from(
                    "delay low pass filter frequency"),
            PARAM_DELAY_WET_GAIN => String::from("delay wet gain"),
            PARAM_FILTER_FREQUENCY => String::from("filter frequency"),
            PARAM_FILTER_SWEEP_RANGE => String::from("filter sweep range"),
            PARAM_FILTER_RESONANCE => String::from("filter quality"),
            PARAM_OSCILLATOR_TYPE => String::from("oscillator type"),
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

    /// set_parameter is called when a VST parameter is changed.
    fn set_parameter(&self, index: i32, val: defs::Sample) {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.update_vst_param(val),
            PARAM_ADSR_DECAY => self.adsr_decay.update_vst_param(val),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_vst_param(val),
            PARAM_ADSR_RELEASE => self.adsr_release.update_vst_param(val),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_vst_param(val),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_vst_param(val),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_vst_param(val),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_vst_param(val),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_vst_param(val),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_vst_param(val),
            PARAM_FILTER_RESONANCE => self.filter_quality.update_vst_param(val),
            PARAM_OSCILLATOR_TYPE => self.oscillator_type.update_vst_param(val),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.update_vst_param(val),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.update_vst_param(val),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.update_vst_param(val),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.update_vst_param(val),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.update_vst_param(val),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.update_vst_param(val),
            _ => (),
        }
    }
}

impl BaseliskPluginParameters
{
    pub fn update_real_value_from_string(&self,
                                         index: i32,
                                         value: String) -> Result<(), &'static str>
    {
        return match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.update_real_value_from_string(value),
            PARAM_ADSR_DECAY => self.adsr_decay.update_real_value_from_string(value),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_real_value_from_string(value),
            PARAM_ADSR_RELEASE => self.adsr_release.update_real_value_from_string(value),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_real_value_from_string(value),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_real_value_from_string(value),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_real_value_from_string(value),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_real_value_from_string(value),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_real_value_from_string(value),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_real_value_from_string(value),
            PARAM_FILTER_RESONANCE => self.filter_quality.update_real_value_from_string(value),
            PARAM_OSCILLATOR_TYPE => self.oscillator_type.update_real_value_from_string(value),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.update_real_value_from_string(value),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.update_real_value_from_string(value),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.update_real_value_from_string(value),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.update_real_value_from_string(value),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.update_real_value_from_string(value),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.update_real_value_from_string(value),
            _ => Err("Unknown parameter"),
        }
    }

    pub fn get_real_value(&self, index: i32) -> defs::Sample {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.get_real_value(),
            PARAM_ADSR_DECAY => self.adsr_decay.get_real_value(),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.get_real_value(),
            PARAM_ADSR_RELEASE => self.adsr_release.get_real_value(),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.get_real_value(),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY =>
                self.delay_high_pass_filter_frequency.get_real_value(),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY =>
                self.delay_low_pass_filter_frequency.get_real_value(),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.get_real_value(),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.get_real_value(),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.get_real_value(),
            PARAM_FILTER_RESONANCE => self.filter_quality.get_real_value(),
            // Ew - need to make this better... can't return usize here.
            PARAM_OSCILLATOR_TYPE => self.oscillator_type.get_real_value() as defs::Sample,
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.get_real_value(),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.get_real_value(),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO =>
                self.oscillator_mod_frequency_ratio.get_real_value(),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.get_real_value(),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.get_real_value(),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.get_real_value(),
            _ => 0.0,
        }
    }
}

pub trait Parameter<T> {
    /// Update the parameter using the value the audio engine will use.
    fn update_real_value_from_string(&self, value: String) -> Result<(), &'static str>;

    /// Update the parameter using a control with range 0.0 > value > 1.0.
    fn update_vst_param(&self, value: defs::Sample);

    /// Get the value the audio engine will use.
    fn get_real_value(&self) -> T;

    /// Get the value for a control with range 0.0 > value > 1.0.
    fn get_vst_param(&self) -> defs::Sample;
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct LinearParameter
{
    base_value: AtomicFloat,
    param_influence_range: AtomicFloat,
    max_value: AtomicFloat,
    current_value: AtomicFloat,
}

/// Parameter with a linear function between low and high limits.
impl LinearParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               default_real_value: defs::Sample) -> Self
    {
        Self {
            base_value: AtomicFloat::new(low_limit),
            param_influence_range: AtomicFloat::new(high_limit - low_limit),
            max_value: AtomicFloat::new(high_limit),
            current_value: AtomicFloat::new(default_real_value),
        }
    }

    fn get_value_from_param(&self, param: defs::Sample) -> defs::Sample {
        let mut param = defs::Sample::min(param, 1.0);
        param = defs::Sample::max(param, 0.0);
        self.base_value.get() + (
            self.param_influence_range.get() * param)
    }

    fn get_param_from_value(&self, value: defs::Sample) -> defs::Sample {
        let mut value = defs::Sample::min(value, self.max_value.get());
        value = defs::Sample::max(value, self.base_value.get());
        (value - self.base_value.get()) / self.param_influence_range.get()
    }
}
impl Parameter<defs::Sample> for LinearParameter
{
    fn update_real_value_from_string(&self, value: String) -> Result<(), &'static str> {
        let mut value = match value.parse::<defs::Sample>() {
            Err(_) => return Err("Can't parse parameter value"),
            Ok(value) => value,
        };
        value = defs::Sample::min(value, self.max_value.get());
        value = defs::Sample::max(value, self.base_value.get());
        self.current_value.set(value);
        Ok(())
    }

    fn get_real_value(&self) -> defs::Sample {
        self.current_value.get()
    }

    fn update_vst_param(&self, param: defs::Sample) {
        self.current_value.set(
            self.get_value_from_param(param));
    }

    fn get_vst_param(&self) -> defs::Sample {
        self.get_param_from_value(
            self.current_value.get())
    }
}

/// Parameter with an exponential function between low and high limits.
pub struct ExponentialParameter
{
    base_value: AtomicFloat,
    param_influence_power: AtomicFloat,
    max_value: AtomicFloat,
    current_value: AtomicFloat,
}

/// Parameter with a frequency as the low_limit and a
/// number of octaves as the cc_influence_range.
impl ExponentialParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               default_real_value: defs::Sample) -> Self
    {
        Self {
            base_value: AtomicFloat::new(low_limit),
            param_influence_power: AtomicFloat::new(
                defs::Sample::log2(high_limit / low_limit)),
            max_value: AtomicFloat::new(high_limit),
            current_value: AtomicFloat::new(default_real_value),
        }
    }

    fn get_value_from_param(&self, param: defs::Sample) -> defs::Sample {
        let mut param = defs::Sample::min(param, 1.0);
        param = defs::Sample::max(param, 0.0);
        self.base_value.get() * defs::Sample::exp2(
            self.param_influence_power.get() * param)
    }

    fn get_param_from_value(&self, value: defs::Sample) -> defs::Sample {
        let mut value = defs::Sample::min(value, self.max_value.get());
        value = defs::Sample::max(value, self.base_value.get());
        defs::Sample::log2(value / self.base_value.get()) / self.param_influence_power.get()
    }
}

impl Parameter<defs::Sample> for ExponentialParameter {
    fn update_real_value_from_string(&self, value: String) -> Result<(), &'static str> {
        let mut value = match value.parse::<defs::Sample>() {
            Err(_) => return Err("Can't parse parameter value"),
            Ok(value) => value,
        };
        value = defs::Sample::min(value, self.max_value.get());
        value = defs::Sample::max(value, self.base_value.get());
        self.current_value.set(value);
        Ok(())
    }

    fn get_real_value(&self) -> defs::Sample {
        self.current_value.get()
    }

    fn get_vst_param(&self) -> defs::Sample {
        self.get_param_from_value(
            self.current_value.get())
    }

    fn update_vst_param(&self, param: defs::Sample) {
        self.current_value.set(
            self.get_value_from_param(param));
    }
}

/// Parameter that represents one of a set of fixed values.
pub struct EnumParameter {
    value_set: Vec<&'static str>,
    current_value_index: AtomicUsize,
}

impl EnumParameter {
    pub fn new(value_set: Vec<&'static str>,
               default_value_index: usize) -> Self
    {
        Self {
            value_set,
            current_value_index: AtomicUsize::new(default_value_index),
        }
    }
}

impl Parameter<usize> for EnumParameter {
    fn update_real_value_from_string(&self, value: String) -> Result<(), &'static str> {
        // Try to find the index of the item in the string
        for (i, candidate) in self.value_set.iter().enumerate() {
             if *candidate == value {
                self.current_value_index.store(i, Ordering::Relaxed);
                return Ok(())
             }
        }
        Err("Unrecognised value passed as argument")
    }

    fn get_real_value(&self) -> usize {
        self.current_value_index.load(Ordering::Relaxed)
    }

    fn get_vst_param(&self) -> defs::Sample {
        (self.current_value_index.load(Ordering::Relaxed) as defs::Sample + 0.5)
        / self.value_set.len() as defs::Sample
    }

    fn update_vst_param(&self, param: defs::Sample) {
        let new_value = usize::min(
            (param * (self.value_set.len() as defs::Sample)) as usize,
            self.value_set.len() - 1, // so that when param == 1.0, we don't overshoot
        );
        self.current_value_index.store(
            new_value,
            Ordering::Relaxed,
        );
    }
}
