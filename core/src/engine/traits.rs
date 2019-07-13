use defs;

pub trait Processor {

    /// Implement custom behaviour when the sample rate is adjusted
    fn set_sample_rate(&mut self, _sample_rate: defs::Sample) {}

    /// Implement custom behaviour when receiving a MIDI panic message
    fn panic(&mut self) {}

}
