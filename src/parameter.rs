use defs;

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

    /// Set the base value of the parameter.
    pub fn set_base(&mut self, base_value: defs::Sample) {
        self.base_value = base_value;
    }

    /// Update the range of influence of the CC controller on the parameter.
    pub fn set_range(&mut self, range_octaves: defs::Sample) {
        self.cc_influence_range_octaves = range_octaves;
    }

    /// Return the parameter's new value based on the value provided by a cc controller.
    pub fn update_from_cc(&mut self, cc_value: u8) -> defs::Sample {
        // TODO: generalize this
        self.current_value = self.base_value * (1.0 + self.cc_influence_range_octaves * (cc_value as defs::Sample) / 127.0);
        self.current_value
    }

    pub fn get(&self) -> defs::Sample {
        self.current_value
    }
}
