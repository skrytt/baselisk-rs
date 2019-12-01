use defs;

use vst;
use vst::util::AtomicFloat;
use std::sync::atomic::{
    AtomicBool,
    AtomicUsize,
    Ordering
};

#[derive(Clone, Copy)]
pub enum ParameterId {
    AdsrAttack,
    AdsrDecay,
    AdsrSustain,
    AdsrRelease,
    DelayTimeLeft,
    DelayTimeRight,
    DelayFeedback,
    DelayHighPassFilterFrequency,
    DelayLowPassFilterFrequency,
    DelayWetGain,
    FilterFrequency,
    FilterSweepRange,
    FilterQuality,
    GeneratorAType,
    GeneratorAPitch,
    GeneratorAPulseWidth,
    GeneratorAModIndex,
    GeneratorBType,
    GeneratorBPitch,
    GeneratorBPulseWidth,
    GeneratorBModIndex,
    GeneratorCType,
    GeneratorCPitch,
    GeneratorCPulseWidth,
    GeneratorCModIndex,
    GeneratorDType,
    GeneratorDPitch,
    GeneratorDPulseWidth,
    GeneratorDModIndex,
    PitchBendRange,
    WaveshaperInputGain,
    WaveshaperOutputGain,
}

impl From<i32> for ParameterId {
    /// Use an integer ID to retrieve a parameter enum type.
    /// Used for VST support.
    fn from(id: i32) -> ParameterId {
        match id {
            0 => ParameterId::AdsrAttack,
            1 => ParameterId::AdsrDecay,
            2 => ParameterId::AdsrSustain,
            3 => ParameterId::AdsrRelease,
            4 => ParameterId::DelayTimeLeft,
            5 => ParameterId::DelayTimeRight,
            6 => ParameterId::DelayFeedback,
            7 => ParameterId::DelayHighPassFilterFrequency,
            8 => ParameterId::DelayLowPassFilterFrequency,
            9 => ParameterId::DelayWetGain,
            10 => ParameterId::FilterFrequency,
            11 => ParameterId::FilterSweepRange,
            12 => ParameterId::FilterQuality,
            13 => ParameterId::GeneratorAType,
            14 => ParameterId::GeneratorAPitch,
            15 => ParameterId::GeneratorAPulseWidth,
            16 => ParameterId::GeneratorAModIndex,
            17 => ParameterId::GeneratorBType,
            18 => ParameterId::GeneratorBPitch,
            19 => ParameterId::GeneratorBPulseWidth,
            20 => ParameterId::GeneratorBModIndex,
            21 => ParameterId::GeneratorCType,
            22 => ParameterId::GeneratorCPitch,
            23 => ParameterId::GeneratorCPulseWidth,
            24 => ParameterId::GeneratorCModIndex,
            25 => ParameterId::GeneratorDType,
            26 => ParameterId::GeneratorDPitch,
            27 => ParameterId::GeneratorDPulseWidth,
            28 => ParameterId::GeneratorDModIndex,
            29 => ParameterId::PitchBendRange,
            30 => ParameterId::WaveshaperInputGain,
            31 => ParameterId::WaveshaperOutputGain,
            _ => panic!("Parameter ID out of bounds"),
        }
    }
}
pub const NUM_PARAMS: i32 = 32;

pub enum ParameterUnit {
    NoUnit,
    Seconds,
    Hz,
    Semitones,
    Octaves,
    Percent,
}

fn unit_formatter(unit: &ParameterUnit, value: defs::Sample) -> String {
    match unit {
        ParameterUnit::NoUnit => format!("{:.1}", value),
        ParameterUnit::Seconds => format!("{:.1} Seconds", value),
        ParameterUnit::Hz => format!("{:.1} Hz", value),
        ParameterUnit::Semitones => format!("{:.1} Semitones", value),
        ParameterUnit::Octaves => format!("{:.1} Octaves", value),
        ParameterUnit::Percent => format!("{:.1} %", value * 100.0),
    }
}

pub struct BaseliskPluginParameters {
    adsr_attack: Parameter,
    adsr_decay: Parameter,
    adsr_sustain: Parameter,
    adsr_release: Parameter,
    delay_time_left: Parameter,
    delay_time_right: Parameter,
    delay_feedback: Parameter,
    delay_high_pass_filter_frequency: Parameter,
    delay_low_pass_filter_frequency: Parameter,
    delay_wet_gain: Parameter,
    filter_frequency: Parameter,
    filter_sweep_range: Parameter,
    filter_quality: Parameter,
    generator_a_type: Parameter,
    generator_a_pitch: Parameter,
    generator_a_pulse_width: Parameter,
    generator_a_mod_index: Parameter,
    generator_b_type: Parameter,
    generator_b_pitch: Parameter,
    generator_b_pulse_width: Parameter,
    generator_b_mod_index: Parameter,
    generator_c_type: Parameter,
    generator_c_pitch: Parameter,
    generator_c_pulse_width: Parameter,
    generator_c_mod_index: Parameter,
    generator_d_type: Parameter,
    generator_d_pitch: Parameter,
    generator_d_pulse_width: Parameter,
    generator_d_mod_index: Parameter,
    pitch_bend_range: Parameter,
    waveshaper_input_gain: Parameter,
    waveshaper_output_gain: Parameter,
}

impl Default for BaseliskPluginParameters {
    fn default() -> BaseliskPluginParameters {
        BaseliskPluginParameters {
            adsr_attack: Parameter::new_exponential(
                "adsr attack",
                ParameterUnit::Seconds, 0.001, 10.0, 0.02),
            adsr_decay: Parameter::new_exponential(
                "adsr decay",
                ParameterUnit::Seconds, 0.02, 10.0, 0.707),
            adsr_sustain: Parameter::new_linear(
                "adsr sustain",
                ParameterUnit::Percent, 0.0, 1.0, 0.0),
            adsr_release: Parameter::new_exponential(
                "adsr release",
                ParameterUnit::Seconds, 0.02, 10.0, 0.4),
            delay_time_left: Parameter::new_exponential(
                "delay time left",
                ParameterUnit::Seconds, 0.08, 1.0, 0.375),
            delay_time_right: Parameter::new_exponential(
                "delay time right",
                ParameterUnit::Seconds, 0.08, 1.0, 0.5),
            delay_feedback: Parameter::new_linear(
                "delay feedback",
                ParameterUnit::Percent, 0.0, 1.0, 0.6),
            delay_high_pass_filter_frequency: Parameter::new_exponential(
                "delay high pass filter frequency",
                ParameterUnit::Hz, 20.0, 22000.0, 100.0),
            delay_low_pass_filter_frequency: Parameter::new_exponential(
                "delay low pass filter frequency",
                ParameterUnit::Hz, 20.0, 22000.0, 5000.0),
            delay_wet_gain: Parameter::new_linear(
                "delay wet gain",
                ParameterUnit::NoUnit, 0.0, 1.0, 0.4),
            filter_frequency: Parameter::new_exponential(
                "filter frequency",
                ParameterUnit::Hz, 20.0, 22000.0, 100.0),
            filter_sweep_range: Parameter::new_linear(
                "filter sweep range",
                ParameterUnit::Octaves, 0.0, 10.0, 8.0),
            filter_quality: Parameter::new_exponential(
                "filter quality",
                ParameterUnit::NoUnit, 0.5, 10.0, 0.707),
            generator_a_type: Parameter::new_enum(
                "generator a type",
                vec!["sine", "saw", "pulse", "noise"],
                0,
            ),
            generator_a_pitch: Parameter::new_linear(
                "generator a pitch",
                ParameterUnit::Semitones, -36.0, 36.0, 0.0
            ).enable_int_snapping(),
            generator_a_pulse_width: Parameter::new_linear(
                "generator a pulse width",
                ParameterUnit::NoUnit, 0.01, 0.99, 0.5),
            generator_a_mod_index: Parameter::new_linear(
                "generator a mod index",
                ParameterUnit::NoUnit, 0.0, 8.0, 1.0),
            generator_b_type: Parameter::new_enum(
                "generator b type",
                vec!["sine", "saw", "pulse", "noise"],
                0,
            ),
            generator_b_pitch: Parameter::new_linear(
                "generator b pitch",
                ParameterUnit::Semitones, -36.0, 36.0, 0.0
            ).enable_int_snapping(),
            generator_b_pulse_width: Parameter::new_linear(
                "generator b pulse width",
                ParameterUnit::NoUnit, 0.01, 0.99, 0.5),
            generator_b_mod_index: Parameter::new_linear(
                "generator b mod index",
                ParameterUnit::NoUnit, 0.0, 8.0, 1.0),
            generator_c_type: Parameter::new_enum(
                "generator c type",
                vec!["sine", "saw", "pulse", "noise"],
                0,
            ),
            generator_c_pitch: Parameter::new_linear(
                "generator c pitch",
                ParameterUnit::Semitones, -36.0, 36.0, 0.0
            ).enable_int_snapping(),
            generator_c_pulse_width: Parameter::new_linear(
                "generator c pulse width",
                ParameterUnit::NoUnit, 0.01, 0.99, 0.5),
            generator_c_mod_index: Parameter::new_linear(
                "generator c mod index",
                ParameterUnit::NoUnit, 0.0, 8.0, 1.0),
            generator_d_type: Parameter::new_enum(
                "generator d type",
                vec!["sine", "saw", "pulse", "noise"],
                0,
            ),
            generator_d_pitch: Parameter::new_linear(
                "generator d pitch",
                ParameterUnit::Semitones, -36.0, 36.0, 0.0
            ).enable_int_snapping(),
            generator_d_pulse_width: Parameter::new_linear(
                "generator d pulse width",
                ParameterUnit::NoUnit, 0.01, 0.99, 0.5),
            generator_d_mod_index: Parameter::new_linear(
                "generator d mod index",
                ParameterUnit::NoUnit, 0.0, 8.0, 1.0),
            pitch_bend_range: Parameter::new_linear(
                "generator pitch bend range",
                ParameterUnit::Semitones, 0.0, 36.0, 2.0),
            waveshaper_input_gain: Parameter::new_linear(
                "waveshaper input gain",
                ParameterUnit::Percent, 0.0, 1.0, 0.333),
            waveshaper_output_gain: Parameter::new_linear(
                "waveshaper output gain",
                ParameterUnit::Percent, 0.0, 1.0, 1.0),
        }
    }
}


/// Implement the VST PluginParameters API using an adapter
/// that converts i32 (as used in the VST API) to ParameterId
/// (as preferred in this code).
impl vst::plugin::PluginParameters for BaseliskPluginParameters {
    fn get_parameter(&self, index: i32) -> defs::Sample {
         self.get_parameter(ParameterId::from(index))
    }
    fn get_parameter_text(&self, index: i32) -> String {
        self.get_parameter_text(ParameterId::from(index))
    }
    fn get_parameter_name(&self, index: i32) -> String {
        self.get_parameter_name(ParameterId::from(index))
    }
    fn set_parameter(&self, index: i32, val: defs::Sample) {
        self.set_parameter(ParameterId::from(index), val)
    }
}


impl BaseliskPluginParameters
{
    fn get_parameter_handle(&self, param: ParameterId) -> &Parameter {
        match param {
            ParameterId::AdsrAttack => &self.adsr_attack,
            ParameterId::AdsrDecay => &self.adsr_decay,
            ParameterId::AdsrSustain => &self.adsr_sustain,
            ParameterId::AdsrRelease => &self.adsr_release,
            ParameterId::DelayTimeLeft => &self.delay_time_left,
            ParameterId::DelayTimeRight => &self.delay_time_right,
            ParameterId::DelayFeedback => &self.delay_feedback,
            ParameterId::DelayHighPassFilterFrequency => &self.delay_high_pass_filter_frequency,
            ParameterId::DelayLowPassFilterFrequency => &self.delay_low_pass_filter_frequency,
            ParameterId::DelayWetGain => &self.delay_wet_gain,
            ParameterId::FilterFrequency => &self.filter_frequency,
            ParameterId::FilterSweepRange => &self.filter_sweep_range,
            ParameterId::FilterQuality => &self.filter_quality,
            ParameterId::GeneratorAType => &self.generator_a_type,
            ParameterId::GeneratorAPitch => &self.generator_a_pitch,
            ParameterId::GeneratorAPulseWidth => &self.generator_a_pulse_width,
            ParameterId::GeneratorAModIndex => &self.generator_a_mod_index,
            ParameterId::GeneratorBType => &self.generator_b_type,
            ParameterId::GeneratorBPitch => &self.generator_b_pitch,
            ParameterId::GeneratorBPulseWidth => &self.generator_b_pulse_width,
            ParameterId::GeneratorBModIndex => &self.generator_b_mod_index,
            ParameterId::GeneratorCType => &self.generator_c_type,
            ParameterId::GeneratorCPitch => &self.generator_c_pitch,
            ParameterId::GeneratorCPulseWidth => &self.generator_c_pulse_width,
            ParameterId::GeneratorCModIndex => &self.generator_c_mod_index,
            ParameterId::GeneratorDType => &self.generator_d_type,
            ParameterId::GeneratorDPitch => &self.generator_d_pitch,
            ParameterId::GeneratorDPulseWidth => &self.generator_d_pulse_width,
            ParameterId::GeneratorDModIndex => &self.generator_d_mod_index,
            ParameterId::PitchBendRange => &self.pitch_bend_range,
            ParameterId::WaveshaperInputGain => &self.waveshaper_input_gain,
            ParameterId::WaveshaperOutputGain => &self.waveshaper_output_gain,
        }
    }

    pub fn get_parameter(&self, param: ParameterId) -> defs::Sample {
        self.get_parameter_handle(param).get_vst_param()
    }

    pub fn get_parameter_text(&self, param: ParameterId) -> String {
        self.get_parameter_handle(param).get_value_text()
    }

    pub fn get_parameter_name(&self, param: ParameterId) -> String {
        String::from(self.get_parameter_handle(param).get_name())
    }

    pub fn set_parameter(&self, param: ParameterId, value: defs::Sample) {
        self.get_parameter_handle(param).update_vst_param(value)
    }

    pub fn update_real_value_from_string(&self,
                                         param: ParameterId,
                                         value: String) -> Result<(), &'static str>
    {
        self.get_parameter_handle(param).update_real_value_from_string(value)
    }

    pub fn get_real_value(&self, param: ParameterId) -> defs::Sample {
        self.get_parameter_handle(param).get_real_value()
    }
}

pub enum Parameter {
    Linear(LinearParameter),
    Exponential(ExponentialParameter),
    Enum(EnumParameter),
}

impl Parameter {
    fn new_linear(
        name: &'static str,
        unit: ParameterUnit,
        low_limit: defs::Sample,
        high_limit: defs::Sample,
        default_real_value: defs::Sample
    ) -> Self
    {
        Parameter::Linear(LinearParameter {
            name,
            unit,
            base_value: AtomicFloat::new(low_limit),
            param_influence_range: AtomicFloat::new(high_limit - low_limit),
            max_value: AtomicFloat::new(high_limit),
            current_value: AtomicFloat::new(default_real_value),
            int_value_snapping: AtomicBool::new(false),
        })
    }

    fn new_exponential(
        name: &'static str,
        unit: ParameterUnit,
        low_limit: defs::Sample,
        high_limit: defs::Sample,
        default_real_value: defs::Sample,
    ) -> Self
    {
        Parameter::Exponential(ExponentialParameter {
            name,
            unit,
            base_value: AtomicFloat::new(low_limit),
            param_influence_power: AtomicFloat::new(
                defs::Sample::log2(high_limit / low_limit)),
            max_value: AtomicFloat::new(high_limit),
            current_value: AtomicFloat::new(default_real_value),
        })
    }

    fn new_enum(
        name: &'static str,
        value_set: Vec<&'static str>,
        default_value_index: usize,
    ) -> Self
    {
        Parameter::Enum(EnumParameter {
            name,
            value_set,
            current_value_index: AtomicUsize::new(default_value_index),
        })
    }

    /// Update the parameter using the value the audio engine will use.
    fn update_real_value_from_string(&self, value: String) -> Result<(), &'static str> {
        match self {
            Parameter::Linear(p) => p.update_real_value_from_string(value),
            Parameter::Exponential(p) => p.update_real_value_from_string(value),
            Parameter::Enum(p) => p.update_real_value_from_string(value),
        }
    }

    /// Update the parameter using a control with range 0.0 > value > 1.0.
    fn update_vst_param(&self, value: defs::Sample)
    {
        match self {
            Parameter::Linear(p) => p.update_vst_param(value),
            Parameter::Exponential(p) => p.update_vst_param(value),
            Parameter::Enum(p) => p.update_vst_param(value),
        }
    }

    /// Builder method to enable snapping to integer values
    fn enable_int_snapping(self) -> Self {
        match self {
            Parameter::Linear(p) => Parameter::Linear(p.enable_int_snapping()),
            _ => panic!("enable_int_snapping not implemented"),
        }
    }

    /// Get the name of the parameter
    fn get_name(&self) -> &'static str {
        match self {
            Parameter::Linear(p) => p.get_name(),
            Parameter::Exponential(p) => p.get_name(),
            Parameter::Enum(p) => p.get_name(),
        }
    }

    /// Get the value the audio engine will use.
    fn get_real_value(&self) -> defs::Sample {
        match self {
            Parameter::Linear(p) => p.get_real_value(),
            Parameter::Exponential(p) => p.get_real_value(),
            Parameter::Enum(p) => p.get_real_value(),
        }
    }

    /// Get the value for a control with range 0.0 > value > 1.0.
    fn get_vst_param(&self) -> defs::Sample {
        match self {
            Parameter::Linear(p) => p.get_vst_param(),
            Parameter::Exponential(p) => p.get_vst_param(),
            Parameter::Enum(p) => p.get_vst_param(),
        }
    }

    /// Get a stringified version of the parameter value with a unit
    fn get_value_text(&self) -> String {
        match self {
            Parameter::Linear(p) => p.get_value_text(),
            Parameter::Exponential(p) => p.get_value_text(),
            Parameter::Enum(p) => p.get_value_text(),
        }
    }
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct LinearParameter
{
    name: &'static str,
    unit: ParameterUnit,
    base_value: AtomicFloat,
    param_influence_range: AtomicFloat,
    max_value: AtomicFloat,
    current_value: AtomicFloat,
    int_value_snapping: AtomicBool,
}

/// Parameter with a linear function between low and high limits.
impl LinearParameter
{
    fn get_value_from_param(&self, param: defs::Sample) -> defs::Sample {
        let mut param = defs::Sample::min(param, 1.0);
        param = defs::Sample::max(param, 0.0);

        let mut result = self.base_value.get() + (
            self.param_influence_range.get() * param);

        // If configured, snap to float representation of nearest int value
        if self.int_value_snapping.load(Ordering::Relaxed) {
            result = result.round()
        }

        result
    }

    fn get_param_from_value(&self, value: defs::Sample) -> defs::Sample {
        let mut value = defs::Sample::min(value, self.max_value.get());
        value = defs::Sample::max(value, self.base_value.get());
        (value - self.base_value.get()) / self.param_influence_range.get()
    }

    /// Consumes a parameter and returns the same parameter with int snapping enabled.
    pub fn enable_int_snapping(self) -> Self {
        self.int_value_snapping.store(true, Ordering::Relaxed);
        self
    }

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

    fn get_name(&self) -> &'static str {
        self.name
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

    fn get_value_text(&self) -> String {
        unit_formatter(&self.unit, self.get_real_value())
    }
}

/// Parameter with an exponential function between low and high limits.
pub struct ExponentialParameter
{
    name: &'static str,
    unit: ParameterUnit,
    base_value: AtomicFloat,
    param_influence_power: AtomicFloat,
    max_value: AtomicFloat,
    current_value: AtomicFloat,
}

/// Parameter with a frequency as the low_limit and a
/// number of octaves as the cc_influence_range.
impl ExponentialParameter
{
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

    fn get_name(&self) -> &'static str {
        self.name
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

    fn get_value_text(&self) -> String {
        unit_formatter(&self.unit, self.get_real_value())
    }
}

/// Parameter that represents one of a set of fixed values.
pub struct EnumParameter {
    name: &'static str,
    value_set: Vec<&'static str>,
    current_value_index: AtomicUsize,
}

impl EnumParameter {
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

    fn get_name(&self) -> &'static str {
        self.name
    }

    fn get_real_value(&self) -> defs::Sample {
        self.current_value_index.load(Ordering::Relaxed) as defs::Sample
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

    fn get_value_text(&self) -> String {
        format!("{}", self.value_set[self.current_value_index.load(Ordering::Relaxed)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Utility method to allow error tolerance in float calcs
    fn assert_float_eq(actual: f32, expected: f32) {
        let error_abs = f32::abs(actual - expected);
        assert!(error_abs < std::f32::EPSILON,
                "actual = {}, expected = {}, absolute error = {}",
                actual, expected, error_abs);
    }

    #[test]
    fn test_linear_parameter_map_by_real_value() {
        let parameter = Parameter::new_linear(
            "test param",
            ParameterUnit::NoUnit, 0.0, 10.0, 0.0);
        assert_float_eq(parameter.get_vst_param(), 0.0);

        parameter.update_real_value_from_string(String::from("5.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 0.5);

        parameter.update_real_value_from_string(String::from("10.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 1.0);

        parameter.update_real_value_from_string(String::from("0.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 0.0);
    }

    #[test]
    fn test_linear_parameter_map_by_vst_param() {
        let parameter = Parameter::new_linear(
            "test param",
            ParameterUnit::NoUnit, 0.0, 10.0, 0.0);
        assert_float_eq(parameter.get_real_value(), 0.0);

        parameter.update_vst_param(0.5);
        assert_float_eq(parameter.get_real_value(), 5.0);

        parameter.update_vst_param(1.0);
        assert_float_eq(parameter.get_real_value(), 10.0);

        parameter.update_vst_param(0.0);
        assert_float_eq(parameter.get_real_value(), 0.0);
    }

    #[test]
    fn test_linear_parameter_limits() {
        let parameter = Parameter::new_linear(
            "test param",
            ParameterUnit::NoUnit, 0.0, 10.0, 0.0);

        parameter.update_real_value_from_string(String::from("20.0")).unwrap();
        assert_float_eq(parameter.get_real_value(), 10.0);
        assert_float_eq(parameter.get_vst_param(), 1.0);

        parameter.update_real_value_from_string(String::from("-10.0")).unwrap();
        assert_float_eq(parameter.get_real_value(), 0.0);
        assert_float_eq(parameter.get_vst_param(), 0.0);
    }

    #[test]
    fn test_exponential_parameter_map_by_real_value() {
        let parameter = Parameter::new_exponential(
            "test param",
            ParameterUnit::NoUnit, 1.0, 16.0, 1.0);
        assert_float_eq(parameter.get_vst_param(), 0.0);

        parameter.update_real_value_from_string(String::from("4.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 0.5);

        parameter.update_real_value_from_string(String::from("16.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 1.0);

        parameter.update_real_value_from_string(String::from("1.0")).unwrap();
        assert_float_eq(parameter.get_vst_param(), 0.0);
    }

    #[test]
    fn test_exponential_parameter_map_by_vst_param() {
        let parameter = Parameter::new_exponential(
            "test param",
            ParameterUnit::NoUnit, 1.0, 16.0, 1.0);
        assert_float_eq(parameter.get_real_value(), 1.0);

        parameter.update_vst_param(0.5);
        assert_float_eq(parameter.get_real_value(), 4.0);

        parameter.update_vst_param(1.0);
        assert_float_eq(parameter.get_real_value(), 16.0);

        parameter.update_vst_param(0.0);
        assert_float_eq(parameter.get_real_value(), 1.0);
    }

    #[test]
    fn test_exponential_parameter_limits() {
        let parameter = Parameter::new_exponential(
            "test param",
            ParameterUnit::NoUnit, 1.0, 16.0, 1.0);

        parameter.update_real_value_from_string(String::from("123.0")).unwrap();
        assert_float_eq(parameter.get_real_value(), 16.0);
        assert_float_eq(parameter.get_vst_param(), 1.0);

        parameter.update_real_value_from_string(String::from("0.123")).unwrap();
        assert_float_eq(parameter.get_real_value(), 1.0);
        assert_float_eq(parameter.get_vst_param(), 0.0);
    }

    #[test]
    fn test_enum_parameter_map_by_real_value() {
        // Let's enumerate musical instruments!
        let parameter = Parameter::new_enum(
            "test param",
            vec!["trombone", "theremin", "triangle"],
            1, // default to "theremin"
        );
        // EnumParameter's real value is a usize
        assert_eq!(parameter.get_real_value() as usize, 1);
        assert_eq!(parameter.get_value_text(), "theremin");

        parameter.update_real_value_from_string(String::from("trombone"))
            .unwrap();
        assert_eq!(parameter.get_real_value() as usize, 0);
        assert_eq!(parameter.get_value_text(), "trombone");

        parameter.update_real_value_from_string(String::from("triangle"))
            .unwrap();
        assert_eq!(parameter.get_real_value() as usize, 2);
        assert_eq!(parameter.get_value_text(), "triangle");

        // Fake instrument!! Check for error
        parameter.update_real_value_from_string(String::from("synthesizer"))
            .unwrap_err();
        // ...and check the value hasn't changed
        assert_eq!(parameter.get_real_value() as usize, 2);
    }

    #[test]
    fn test_enum_parameter_map_by_vst_param() {
        let parameter = Parameter::new_enum(
            "test param",
            vec!["trombone", "theremin", "triangle"],
            1, // default to "theremin"
        );
        // EnumParameter's real value is a usize
        assert_eq!(parameter.get_real_value() as usize, 1);
        assert_eq!(parameter.get_value_text(), "theremin");

        // The VST param is divided into a list of ranges of equal size
        // where each range maps to an item in the parameter name list;
        // the ordering is the same between both lists.
        parameter.update_vst_param(0.17);
        assert_eq!(parameter.get_real_value() as usize, 0);
        assert_eq!(parameter.get_value_text(), "trombone");

        parameter.update_vst_param(0.83);
        assert_eq!(parameter.get_real_value() as usize, 2);
        assert_eq!(parameter.get_value_text(), "triangle");

        parameter.update_vst_param(0.5);
        assert_eq!(parameter.get_real_value() as usize, 1);
        assert_eq!(parameter.get_value_text(), "theremin");

        // Test VST parameter boundary values also
        parameter.update_vst_param(0.0);
        assert_eq!(parameter.get_real_value() as usize, 0);
        assert_eq!(parameter.get_value_text(), "trombone");

        parameter.update_vst_param(1.0);
        assert_eq!(parameter.get_real_value() as usize, 2);
        assert_eq!(parameter.get_value_text(), "triangle");
    }
}
