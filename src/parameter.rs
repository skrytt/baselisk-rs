use defs;

pub trait Parameter {
    /// Set the base value of the parameter.
    fn set_base(&mut self, base_value: defs::Sample);

    /// Update the range of influence of the CC controller on the parameter.
    fn set_range(&mut self, range: defs::Sample);

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample;

    /// Return the parameter's new value based on the value provided by a cc controller.
    fn update_from_cc(&mut self, cc_value: u8) -> defs::Sample;
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct LinearParameter
{
    base_value: defs::Sample,
    cc_influence_range: defs::Sample,
    current_value: defs::Sample,
}

/// Parameter with a frequency as the base_value and a
/// number of octaves as the cc_influence_range.
impl LinearParameter
{
    pub fn new(base_value: defs::Sample) -> LinearParameter {
        LinearParameter {
            base_value: base_value,
            cc_influence_range: 0.0,
            current_value: base_value,
        }
    }
}
impl Parameter for LinearParameter {
    /// Set the base value of the parameter.
    fn set_base(&mut self, base_value: defs::Sample) {
        self.base_value = base_value;
    }

    /// Update the range of influence of the CC controller on the parameter.
    fn set_range(&mut self, range: defs::Sample) {
        self.cc_influence_range = range;
    }

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample {
        self.current_value
    }

    /// Return the parameter's new value based on the value provided by a cc controller.
    fn update_from_cc(&mut self, cc_value: u8) -> defs::Sample {
        self.current_value = self.base_value + (
            self.cc_influence_range * (cc_value as defs::Sample) / 127.0);
        self.current_value
    }
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
pub struct FrequencyParameter
{
    base_value: defs::Sample,
    cc_influence_range_octaves: defs::Sample,
    current_value: defs::Sample,
}

/// Parameter with a frequency as the base_value and a
/// number of octaves as the cc_influence_range.
impl FrequencyParameter
{
    pub fn new(base_value: defs::Sample) -> FrequencyParameter {
        FrequencyParameter {
            base_value: base_value,
            cc_influence_range_octaves: 0.0,
            current_value: base_value,
        }
    }
}
impl Parameter for FrequencyParameter {
    /// Set the base value of the parameter.
    fn set_base(&mut self, base_value: defs::Sample) {
        self.base_value = base_value;
    }

    /// Update the range of influence of the CC controller on the parameter.
    fn set_range(&mut self, range: defs::Sample) {
        self.cc_influence_range_octaves = range;
    }

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample {
        self.current_value
    }

    /// Return the parameter's new value based on the value provided by a cc controller.
    fn update_from_cc(&mut self, cc_value: u8) -> defs::Sample {
        self.current_value = self.base_value * (
            1.0 + self.cc_influence_range_octaves * (cc_value as defs::Sample) / 127.0);
        self.current_value
    }
}
