use defs;
use event::ModulatableParameterUpdateData;

use vst;
use vst::util::AtomicFloat;

pub const PARAM_ADSR_ATTACK: i32 = 0;
pub const PARAM_ADSR_DECAY: i32 = 1;
pub const PARAM_ADSR_SUSTAIN: i32 = 2;
pub const PARAM_ADSR_RELEASE: i32 = 3;
pub const PARAM_DELAY_FEEDBACK: i32 = 4;
pub const PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY: i32 = 5;
pub const PARAM_DELAY_HIGH_PASS_FILTER_QUALITY: i32 = 6;
pub const PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY: i32 = 7;
pub const PARAM_DELAY_LOW_PASS_FILTER_QUALITY: i32 = 8;
pub const PARAM_DELAY_WET_GAIN: i32 = 9;
pub const PARAM_FILTER_FREQUENCY: i32 = 10;
pub const PARAM_FILTER_SWEEP_RANGE: i32 = 11;
pub const PARAM_FILTER_QUALITY: i32 = 12;
pub const PARAM_OSCILLATOR_PITCH: i32 = 13;
pub const PARAM_OSCILLATOR_PULSE_WIDTH: i32 = 14;
pub const PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO: i32 = 15;
pub const PARAM_OSCILLATOR_MOD_INDEX: i32 = 16;
pub const PARAM_WAVESHAPER_INPUT_GAIN: i32 = 17;
pub const PARAM_WAVESHAPER_OUTPUT_GAIN: i32 = 18;
pub const NUM_PARAMS: i32 = 19;

pub struct BaseliskPluginParameters {
    adsr_attack: LinearParameter,
    adsr_decay: LinearParameter,
    adsr_sustain: LinearParameter,
    adsr_release: LinearParameter,
    delay_feedback: LinearParameter,
    delay_high_pass_filter_frequency: FrequencyParameter,
    delay_high_pass_filter_quality: LinearParameter,
    delay_low_pass_filter_frequency: FrequencyParameter,
    delay_low_pass_filter_quality: LinearParameter,
    delay_wet_gain: LinearParameter,
    filter_frequency: FrequencyParameter,
    filter_sweep_range: LinearParameter,
    filter_quality: LinearParameter,
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
            adsr_attack: LinearParameter::new(0.0, 10.0, 0.02),
            adsr_decay: LinearParameter::new(0.0, 10.0, 0.707),
            adsr_sustain: LinearParameter::new(0.0, 1.0, 0.0),
            adsr_release: LinearParameter::new(0.0, 10.0, 0.4),
            delay_feedback: LinearParameter::new(0.0, 1.0, 0.6),
            delay_high_pass_filter_frequency: FrequencyParameter::new(1.0, 22000.0, 125.0),
            delay_high_pass_filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
            delay_low_pass_filter_frequency: FrequencyParameter::new(1.0, 22000.0, 5000.0),
            delay_low_pass_filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
            delay_wet_gain: LinearParameter::new(0.0, 1.0, 0.4),
            filter_frequency: FrequencyParameter::new(1.0, 22000.0, 10.0),
            filter_sweep_range: LinearParameter::new(0.0, 20.0, 6.5),
            filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
            oscillator_pitch: LinearParameter::new(-36.0, 36.0, 0.0),
            oscillator_pulse_width: LinearParameter::new(0.01, 0.99, 0.5),
            oscillator_mod_frequency_ratio: LinearParameter::new(1.0, 8.0, 1.0),
            oscillator_mod_index: LinearParameter::new(0.0, 8.0, 1.0),
            waveshaper_input_gain: LinearParameter::new(0.0, 1.0, 0.333),
            waveshaper_output_gain: LinearParameter::new(0.0, 1.0, 0.2),

        }
    }
}

/// Here we implement the VST PluginParameters trait.
/// VST plugins use a range of 0.0 >= value >= 1.0 for all parameters.
/// This means that when using the value, we need to transform i
impl vst::plugin::PluginParameters for BaseliskPluginParameters {
    fn get_parameter(&self, index: i32) -> f32 {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.get_vst_param(),
            PARAM_ADSR_DECAY => self.adsr_decay.get_vst_param(),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.get_vst_param(),
            PARAM_ADSR_RELEASE => self.adsr_release.get_vst_param(),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.get_vst_param(),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.get_vst_param(),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.get_vst_param(),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.get_vst_param(),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.get_vst_param(),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.get_vst_param(),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.get_vst_param(),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.get_vst_param(),
            PARAM_FILTER_QUALITY => self.filter_quality.get_vst_param(),
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
            PARAM_ADSR_ATTACK => format!("{}", self.adsr_attack.get_vst_param()),
            PARAM_ADSR_DECAY => format!("{}", self.adsr_decay.get_vst_param()),
            PARAM_ADSR_SUSTAIN => format!("{}", self.adsr_sustain.get_vst_param()),
            PARAM_ADSR_RELEASE => format!("{}", self.adsr_release.get_vst_param()),
            PARAM_DELAY_FEEDBACK => format!("{}", self.delay_feedback.get_vst_param()),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => format!(
                "{}", self.delay_high_pass_filter_frequency.get_vst_param()),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => format!(
                "{}", self.delay_high_pass_filter_quality.get_vst_param()),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => format!(
                "{}", self.delay_low_pass_filter_frequency.get_vst_param()),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => format!(
                "{}", self.delay_low_pass_filter_quality.get_vst_param()),
            PARAM_DELAY_WET_GAIN => format!("{}", self.delay_wet_gain.get_vst_param()),
            PARAM_FILTER_FREQUENCY => format!("{}", self.filter_frequency.get_vst_param()),
            PARAM_FILTER_SWEEP_RANGE => format!("{}", self.filter_sweep_range.get_vst_param()),
            PARAM_FILTER_QUALITY => format!("{}", self.filter_quality.get_vst_param()),
            PARAM_OSCILLATOR_PITCH => format!("{}", self.oscillator_pitch.get_vst_param()),
            PARAM_OSCILLATOR_PULSE_WIDTH => format!("{}", self.oscillator_pulse_width.get_vst_param()),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => format!("{}", self.oscillator_mod_frequency_ratio.get_vst_param()),
            PARAM_OSCILLATOR_MOD_INDEX => format!("{}", self.oscillator_mod_index.get_vst_param()),
            PARAM_WAVESHAPER_INPUT_GAIN => format!("{}", self.waveshaper_input_gain.get_vst_param()),
            PARAM_WAVESHAPER_OUTPUT_GAIN => format!("{}", self.waveshaper_output_gain.get_vst_param()),
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
            PARAM_ADSR_ATTACK => self.adsr_attack.update_vst_param(val),
            PARAM_ADSR_DECAY => self.adsr_decay.update_vst_param(val),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_vst_param(val),
            PARAM_ADSR_RELEASE => self.adsr_release.update_vst_param(val),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_vst_param(val),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_vst_param(val),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.update_vst_param(val),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_vst_param(val),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.update_vst_param(val),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_vst_param(val),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_vst_param(val),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_vst_param(val),
            PARAM_FILTER_QUALITY => self.filter_quality.update_vst_param(val),
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
    pub fn get_real_value(&self, index: i32) -> f32 {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.get_real_value(),
            PARAM_ADSR_DECAY => self.adsr_decay.get_real_value(),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.get_real_value(),
            PARAM_ADSR_RELEASE => self.adsr_release.get_real_value(),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.get_real_value(),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.get_real_value(),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.get_real_value(),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.get_real_value(),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.get_real_value(),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.get_real_value(),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.get_real_value(),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.get_real_value(),
            PARAM_FILTER_QUALITY => self.filter_quality.get_real_value(),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.get_real_value(),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.get_real_value(),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.get_real_value(),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.get_real_value(),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.get_real_value(),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.get_real_value(),
            _ => 0.0,
        }
    }
    pub fn update_patch(&self,
                        index: i32,
                        data: ModulatableParameterUpdateData) -> Result<(), &'static str> {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.update_patch(data),
            PARAM_ADSR_DECAY => self.adsr_decay.update_patch(data),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_patch(data),
            PARAM_ADSR_RELEASE => self.adsr_release.update_patch(data),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_patch(data),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_patch(data),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.update_patch(data),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_patch(data),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.update_patch(data),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_patch(data),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_patch(data),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_patch(data),
            PARAM_FILTER_QUALITY => self.filter_quality.update_patch(data),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.update_patch(data),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.update_patch(data),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.update_patch(data),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.update_patch(data),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.update_patch(data),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.update_patch(data),
            _ => Err("Unknown param ID"),
        }
    }
}

pub trait Parameter {
    fn update_patch(&self, data: ModulatableParameterUpdateData) -> Result<(), &'static str>;

    fn update_cc(&mut self, cc_value: u8);

    fn update_vst_param(&self, value: f32);

    /// Get the current value of the parameter.
    fn get_real_value(&self) -> defs::Sample;

    fn get_vst_param(&self) -> f32;
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct LinearParameter
{
    low_limit: defs::Sample,
    high_limit: defs::Sample,
    base_value: AtomicFloat,
    max_value: AtomicFloat,
    param_influence_range: AtomicFloat,
    param_value: AtomicFloat,
}

/// Parameter with a frequency as the base_value and a
/// number of octaves as the cc_influence_range.
impl LinearParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               base_value: defs::Sample) -> Self
    {
        Self {
            low_limit,
            high_limit,
            base_value: AtomicFloat::new(base_value),
            max_value: AtomicFloat::new(base_value),
            param_influence_range: AtomicFloat::new(0.0),
            param_value: AtomicFloat::new(0.0),
        }
    }
}
impl Parameter for LinearParameter {
    fn update_patch(&self, data: ModulatableParameterUpdateData) -> Result<(), &'static str> {
        match data {
            ModulatableParameterUpdateData::Base(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.base_value.set(value);
            },
            ModulatableParameterUpdateData::Max(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.max_value.set(value);
            },
        }
        self.param_influence_range.set(
            self.max_value.get() - self.base_value.get());
        Ok(())
    }

    fn update_cc(&mut self, cc_value: u8) {
        self.param_value.set(defs::Sample::from(cc_value) / 127.0);
    }

    fn update_vst_param(&self, value: f32) {
        assert!(value >= 0.0);
        assert!(value <= 1.0);
        self.param_value.set(value);
    }

    /// Get the current value of the parameter.
    fn get_real_value(&self) -> defs::Sample {
        self.base_value.get() + (
            self.param_influence_range.get() * self.param_value.get())
    }

    fn get_vst_param(&self) -> f32 {
        self.param_value.get()
    }
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct FrequencyParameter
{
    low_limit: defs::Sample,
    high_limit: defs::Sample,
    base_value: AtomicFloat,
    max_value: AtomicFloat,
    param_influence_range_octaves: AtomicFloat,
    param_value: AtomicFloat,
}

/// Parameter with a frequency as the base_value and a
/// number of octaves as the cc_influence_range.
impl FrequencyParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               base_value: defs::Sample) -> Self
    {
        Self {
            low_limit,
            high_limit,
            base_value: AtomicFloat::new(base_value),
            max_value: AtomicFloat::new(base_value),
            param_influence_range_octaves: AtomicFloat::new(0.0),
            param_value: AtomicFloat::new(0.0),
        }
    }
}

impl Parameter for FrequencyParameter {
    fn update_patch(&self, data: ModulatableParameterUpdateData) -> Result<(), &'static str> {
        match data {
            ModulatableParameterUpdateData::Base(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.base_value.set(value);
            },
            ModulatableParameterUpdateData::Max(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.max_value.set(value);
            },
        }
        self.param_influence_range_octaves.set(
            defs::Sample::log2(self.max_value.get() / self.base_value.get()));
        Ok(())
    }

    fn update_cc(&mut self, cc_value: u8) {
        self.param_value.set(defs::Sample::from(cc_value) / 127.0);
    }

    fn update_vst_param(&self, value: f32) {
        assert!(value >= 0.0);
        assert!(value <= 1.0);
        self.param_value.set(value);
    }

    /// Get the current value of the parameter.
    fn get_real_value(&self) -> defs::Sample {
        self.base_value.get() * (
            1.0 + self.param_influence_range_octaves.get() * self.param_value.get())
    }

    fn get_vst_param(&self) -> f32 {
        self.param_value.get()
    }
}
