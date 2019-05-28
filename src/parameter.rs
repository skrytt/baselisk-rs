use defs;

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
            adsr_attack: LinearParameter::new(0.001, 10.0, 0.02),
            adsr_decay: LinearParameter::new(0.001, 10.0, 0.707),
            adsr_sustain: LinearParameter::new(0.0, 1.0, 0.0),
            adsr_release: LinearParameter::new(0.001, 10.0, 0.4),
            delay_feedback: LinearParameter::new(0.0, 1.0, 0.6),
            delay_high_pass_filter_frequency: FrequencyParameter::new(20.0, 22000.0, 100.0),
            delay_high_pass_filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
            delay_low_pass_filter_frequency: FrequencyParameter::new(20.0, 22000.0, 5000.0),
            delay_low_pass_filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
            delay_wet_gain: LinearParameter::new(0.0, 1.0, 0.4),
            filter_frequency: FrequencyParameter::new(20.0, 22000.0, 100.0),
            filter_sweep_range: LinearParameter::new(0.0, 10.0, 8.0),
            filter_quality: LinearParameter::new(0.5, 10.0, 0.707),
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
                "{}Hz", self.delay_high_pass_filter_frequency.get_real_value()),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => format!(
                "{}Hz", self.delay_high_pass_filter_quality.get_real_value()),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => format!(
                "{}Hz", self.delay_low_pass_filter_frequency.get_real_value()),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => format!(
                "{}Hz", self.delay_low_pass_filter_quality.get_real_value()),
            PARAM_DELAY_WET_GAIN => format!(
                "{} %", 100.0 * self.delay_wet_gain.get_real_value()),
            PARAM_FILTER_FREQUENCY => format!(
                "{} Hz", self.filter_frequency.get_real_value()),
            PARAM_FILTER_SWEEP_RANGE => format!(
                "{} octaves", self.filter_sweep_range.get_real_value()),
            PARAM_FILTER_QUALITY => format!(
                "{}", self.filter_quality.get_real_value()),
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

    /// set_parameter is called when a VST parameter is changed.
    fn set_parameter(&self, index: i32, val: defs::Sample) {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.update_param(val),
            PARAM_ADSR_DECAY => self.adsr_decay.update_param(val),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_param(val),
            PARAM_ADSR_RELEASE => self.adsr_release.update_param(val),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_param(val),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_param(val),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.update_param(val),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_param(val),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.update_param(val),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_param(val),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_param(val),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_param(val),
            PARAM_FILTER_QUALITY => self.filter_quality.update_param(val),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.update_param(val),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.update_param(val),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.update_param(val),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.update_param(val),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.update_param(val),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.update_param(val),
            _ => (),
        }
    }
}

impl BaseliskPluginParameters
{
    pub fn update_real_value(&self, index: i32, value: f32) {
        match index {
            PARAM_ADSR_ATTACK => self.adsr_attack.update_real_value(value),
            PARAM_ADSR_DECAY => self.adsr_decay.update_real_value(value),
            PARAM_ADSR_SUSTAIN => self.adsr_sustain.update_real_value(value),
            PARAM_ADSR_RELEASE => self.adsr_release.update_real_value(value),
            PARAM_DELAY_FEEDBACK => self.delay_feedback.update_real_value(value),
            PARAM_DELAY_HIGH_PASS_FILTER_FREQUENCY => self.delay_high_pass_filter_frequency.update_real_value(value),
            PARAM_DELAY_HIGH_PASS_FILTER_QUALITY => self.delay_high_pass_filter_quality.update_real_value(value),
            PARAM_DELAY_LOW_PASS_FILTER_FREQUENCY => self.delay_low_pass_filter_frequency.update_real_value(value),
            PARAM_DELAY_LOW_PASS_FILTER_QUALITY => self.delay_low_pass_filter_quality.update_real_value(value),
            PARAM_DELAY_WET_GAIN => self.delay_wet_gain.update_real_value(value),
            PARAM_FILTER_FREQUENCY => self.filter_frequency.update_real_value(value),
            PARAM_FILTER_SWEEP_RANGE => self.filter_sweep_range.update_real_value(value),
            PARAM_FILTER_QUALITY => self.filter_quality.update_real_value(value),
            PARAM_OSCILLATOR_PITCH => self.oscillator_pitch.update_real_value(value),
            PARAM_OSCILLATOR_PULSE_WIDTH => self.oscillator_pulse_width.update_real_value(value),
            PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO => self.oscillator_mod_frequency_ratio.update_real_value(value),
            PARAM_OSCILLATOR_MOD_INDEX => self.oscillator_mod_index.update_real_value(value),
            PARAM_WAVESHAPER_INPUT_GAIN => self.waveshaper_input_gain.update_real_value(value),
            PARAM_WAVESHAPER_OUTPUT_GAIN => self.waveshaper_output_gain.update_real_value(value),
            _ => (),
        }
    }

    pub fn get_real_value(&self, index: i32) -> defs::Sample {
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
}

pub trait Parameter {
    /// Update the parameter using the value the audio engine will use.
    fn update_real_value(&self,value: defs::Sample);

    /// Update the parameter using a control with range 0.0 > value > 1.0.
    fn update_param(&self, value: defs::Sample);

    /// Get the value the audio engine will use.
    fn get_real_value(&self) -> defs::Sample;

    /// Get the value for a control with range 0.0 > value > 1.0.
    fn get_vst_param(&self) -> defs::Sample;
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct LinearParameter
{
    base_value: AtomicFloat,
    param_influence_range: AtomicFloat,
    param: AtomicFloat,
}

/// Parameter with a frequency as the low_limit and a
/// number of octaves as the cc_influence_range.
impl LinearParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               default_real_value: defs::Sample) -> Self
    {
        let result = Self {
            base_value: AtomicFloat::new(low_limit),
            param_influence_range: AtomicFloat::new(high_limit - low_limit),
            param: AtomicFloat::new(0.0),
        };
        result.update_real_value(default_real_value);
        result
    }
}
impl Parameter for LinearParameter {
    fn update_real_value(&self, value: defs::Sample) {
        let new_param = (value - self.base_value.get()) / self.param_influence_range.get();
        self.update_param(new_param);
    }

    fn update_param(&self, value: defs::Sample) {
        let mut value = defs::Sample::min(value, 1.0);
        value = defs::Sample::max(value, 0.0);

        assert!(value >= 0.0);
        assert!(value <= 1.0);

        self.param.set(value);
    }

    /// Get the real value of the parameter,
    /// the one that will be used for computation in the audio callback
    fn get_real_value(&self) -> defs::Sample {
        self.base_value.get() + (
            self.param_influence_range.get() * self.param.get())
    }

    /// VST param is a value in the range 0.0 <= value <= 1.0 ,
    /// representing the param shown on the fader/slider/etc
    fn get_vst_param(&self) -> defs::Sample {
        self.param.get()
    }
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct FrequencyParameter
{
    base_value: AtomicFloat,
    param_influence_range_octaves: AtomicFloat,
    param: AtomicFloat,
}

/// Parameter with a frequency as the low_limit and a
/// number of octaves as the cc_influence_range.
impl FrequencyParameter
{
    pub fn new(low_limit: defs::Sample,
               high_limit: defs::Sample,
               default_real_value: defs::Sample) -> Self
    {
        let result = Self {
            base_value: AtomicFloat::new(low_limit),
            param_influence_range_octaves: AtomicFloat::new(
                defs::Sample::log2(high_limit / low_limit)),
            param: AtomicFloat::new(0.0),
        };
        result.update_real_value(default_real_value);
        result
    }
}

impl Parameter for FrequencyParameter {
    fn update_real_value(&self, value: defs::Sample) {
        let new_param = defs::Sample::log2(value / self.base_value.get())
                        / self.param_influence_range_octaves.get();
        self.update_param(new_param);
    }

    fn update_param(&self, value: defs::Sample) {
        let mut value = defs::Sample::min(value, 1.0);
        value = defs::Sample::max(value, 0.0);

        assert!(value >= 0.0);
        assert!(value <= 1.0);

        self.param.set(value);
    }

    /// Get the current value of the parameter.
    fn get_real_value(&self) -> defs::Sample {
        self.base_value.get() * defs::Sample::exp2(
            self.param_influence_range_octaves.get() * self.param.get())
    }

    fn get_vst_param(&self) -> defs::Sample {
        self.param.get()
    }
}
