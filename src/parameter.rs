use defs;
use event::ModulatableParameterUpdateData;

pub trait Parameter {
    fn update_patch(&mut self, data: ModulatableParameterUpdateData) -> Result<(), &'static str>;

    fn update_cc(&mut self, cc_value: u8);

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample;
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
#[derive(Default)]
pub struct LinearParameter
{
    low_limit: defs::Sample,
    high_limit: defs::Sample,
    base_value: defs::Sample,
    max_value: defs::Sample,
    cc_influence_range: defs::Sample,
    cc_value: u8,
    current_value: defs::Sample,
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
            base_value,
            max_value: base_value,
            cc_influence_range: 0.0,
            cc_value: 0,
            current_value: base_value,
        }
    }

    fn update_current_value(&mut self) {
        self.current_value = self.base_value + (
            self.cc_influence_range * defs::Sample::from(self.cc_value) / 127.0);
    }
}
impl Parameter for LinearParameter {
    fn update_patch(&mut self, data: ModulatableParameterUpdateData) -> Result<(), &'static str> {
        match data {
            ModulatableParameterUpdateData::Base(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.base_value = value;
            },
            ModulatableParameterUpdateData::Max(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.max_value = value;
            },
        }
        self.cc_influence_range = self.max_value - self.base_value;
        self.update_current_value();
        Ok(())
    }

    fn update_cc(&mut self, cc_value: u8) {
        self.cc_value = cc_value;
        self.update_current_value();
    }

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample {
        self.current_value
    }
}

/// A parameter that can be modulated.
/// Its base value is the "unmodulated" value of the parameter.
#[derive(Default)]
pub struct FrequencyParameter
{
    low_limit: defs::Sample,
    high_limit: defs::Sample,
    base_value: defs::Sample,
    max_value: defs::Sample,
    cc_influence_range_octaves: defs::Sample,
    cc_value: u8,
    current_value: defs::Sample,
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
            base_value,
            max_value: base_value,
            cc_influence_range_octaves: 0.0,
            cc_value: 0,
            current_value: base_value,
        }
    }

    fn update_current_value(&mut self) {
        self.current_value = self.base_value * (
            1.0 + self.cc_influence_range_octaves * defs::Sample::from(self.cc_value) / 127.0);
    }
}

impl Parameter for FrequencyParameter {
    fn update_patch(&mut self, data: ModulatableParameterUpdateData) -> Result<(), &'static str> {
        match data {
            ModulatableParameterUpdateData::Base(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.base_value = value;
            },
            ModulatableParameterUpdateData::Max(value) => {
                let value = defs::Sample::max(self.low_limit, value);
                let value = defs::Sample::min(self.high_limit, value);
                self.max_value = value;
            },
        }
        self.cc_influence_range_octaves = defs::Sample::log2(self.max_value / self.base_value);
        self.update_current_value();
        Ok(())
    }

    fn update_cc(&mut self, cc_value: u8) {
        self.cc_value = cc_value;
        self.update_current_value();
    }

    /// Get the current value of the parameter.
    fn get(&self) -> defs::Sample {
        self.current_value
    }
}
