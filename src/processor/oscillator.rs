extern crate sample;

use buffer::ResizableFrameBuffer;
use defs;
use event::{EngineEvent, ModulatableParameter, ModulatableParameterUpdateData};
use parameter::{Parameter, LinearParameter};
use std::slice;


/// Convert a u8 note number to a corresponding frequency,
/// using 440 Hz as the pitch of the A above middle C.
fn get_frequency(note: defs::Sample) -> defs::Sample {
    440.0 * ((note - 69.0) / 12.0).exp2()
}

/// Internal state used by oscillator types.
pub struct State {
    pitch_offset: LinearParameter, // Semitones
    pulse_width: LinearParameter,
    note: u8,

    pitch_bend: defs::Sample,   // Semitones
    frequency: defs::Sample,
    phase: defs::Sample,        // 0 <= phase <= 1
    sample_rate: defs::Sample,
    frequency_buffer: ResizableFrameBuffer<defs::MonoFrame>,
}

impl State {
    pub fn new() -> State {
        State {
            pitch_offset: LinearParameter::new(-36.0, 36.0, 0.0),
            pulse_width: LinearParameter::new(0.01, 0.99, 0.5),
            note: 69,
            pitch_bend: 0.0,
            frequency: 0.0,
            phase: 0.0,
            sample_rate: 0.0,
            frequency_buffer: ResizableFrameBuffer::new(),
        }
    }

    pub fn reset(&mut self) {
        self.frequency = 0.0;
        self.pitch_bend = 0.0;
        self.phase = 0.0;
        self.sample_rate = 0.0;
    }

    /// Process any events and update the internal state accordingly.
    fn update(&mut self,
              mut engine_event_iter: slice::Iter<(usize, EngineEvent)>,
              sample_rate: defs::Sample,
              buffer_size: usize)
    {
        // Sample rate used by the generator functions
        self.sample_rate = sample_rate;

        // Calculate the frequencies per-frame
        let frequency_buffer = self.frequency_buffer.get_sized_mut(buffer_size);
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            // This match block continues on events that are unimportant to this processor.
            match next_event {
                Some((frame_num, engine_event)) => {
                    match engine_event {
                        EngineEvent::NoteChange{ note } => match note {
                            Some(_) => (),
                            None => continue,
                        },
                        EngineEvent::PitchBend{ .. } => (),
                        EngineEvent::ModulateParameter { parameter, .. } => match parameter {
                            ModulatableParameter::OscillatorPitch => (),
                            ModulatableParameter::OscillatorPulseWidth => (),
                            _ => continue,
                        },
                    }
                    next_keyframe = *frame_num;
                },
                None => {
                    // No more note change events, so we'll process to the end of the buffer.
                    next_keyframe = buffer_size;
                },
            };

            // Apply the old parameters up until next_keyframe.
            if let Some(frequency_buffer_slice) = frequency_buffer.get_mut(
                    this_keyframe..next_keyframe)
            {
                self.frequency = get_frequency(defs::Sample::from(self.note)
                                               + self.pitch_offset.get()
                                               + self.pitch_bend);
                for frequency_frame in frequency_buffer_slice {
                    for frequency_sample in frequency_frame {
                        *frequency_sample = self.frequency;
                    }
                }
            }

            // We've reached the next_keyframe.
            this_keyframe = next_keyframe;

            // What we do now depends on whether we reached the end of the buffer.
            if this_keyframe == buffer_size {
                // Loop exit condition: reached the end of the buffer.
                break
            } else {
                // Before the next iteration, use the event at this keyframe
                // to update the current state.
                let (_, event) = next_event.unwrap();
                match event {
                    EngineEvent::NoteChange{ note } => {
                        if let Some(note) = note {
                            self.note = *note;
                        }
                    },
                    EngineEvent::PitchBend{ semitones } => {
                        self.pitch_bend = *semitones;
                    },
                    EngineEvent::ModulateParameter { parameter, value } => match parameter {
                        ModulatableParameter::OscillatorPitch => {
                            self.pitch_offset.update_cc(*value);
                        },
                        ModulatableParameter::OscillatorPulseWidth => {
                            self.pulse_width.update_cc(*value);
                        },
                        _ => (),
                    },
                };
            }
        }
    }
}

/// Oscillator type that will be used for audio processing.
pub struct Oscillator {
    state: State,
    generator_func: fn(&mut State, &mut defs::MonoFrameBufferSlice),
}

impl Oscillator {
/// Function to construct new oscillators.
    pub fn new() -> Oscillator
    {
        Oscillator {
            state: State::new(),
            generator_func: sine_generator,
        }
    }

    pub fn midi_panic(&mut self) {
        self.state.reset();
    }

    pub fn set_type(&mut self, type_name: &str) -> Result<(), &'static str>
    {
        let generator_func = match type_name {
            "sine" => sine_generator,
            "saw" => sawtooth_generator,
            "pulse" => pulse_generator,
            _ => return Err("Unknown oscillator type specified"),
        };
        self.generator_func = generator_func;
        Ok(())
    }

    pub fn update_pitch(&mut self, data: ModulatableParameterUpdateData)
                        -> Result<(), &'static str>
    {
        self.state.pitch_offset.update_patch(data)
    }

    pub fn update_pulse_width(&mut self, data: ModulatableParameterUpdateData)
                              -> Result<(), &'static str>
    {
        self.state.pulse_width.update_patch(data)
    }

    pub fn process_buffer(&mut self,
               buffer: &mut defs::MonoFrameBufferSlice,
               engine_event_iter: slice::Iter<(usize, EngineEvent)>,
               sample_rate: defs::Sample,
    ) {
        self.state.update(engine_event_iter, sample_rate, buffer.len());

        // Generate all the samples for this buffer
        (self.generator_func)(&mut self.state, buffer);
    }
}

/// Generator function that produces a sine wave.
fn sine_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let frequency_buffer = state.frequency_buffer.get_sized_mut(buffer.len());

    for (frame_num, frame) in buffer.iter_mut().enumerate() {
        // Advance phase
        // Enforce range 0.0 <= phase < 1.0
        let step = frequency_buffer.get(frame_num).unwrap()[0] / state.sample_rate;
        let mut phase = state.phase + step;
        while phase >= 1.0 {
            phase -= 1.0;
        }
        // Store the phase for next iteration
        state.phase = phase;

        let res = defs::Sample::sin(2.0 as defs::Sample * defs::PI * phase);
        *frame = [res];
    }
}

/// Generator function that produces a pulse wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn pulse_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let frequency_buffer = state.frequency_buffer.get_sized_mut(buffer.len());

    for (frame_num, frame) in buffer.iter_mut().enumerate() {
        // Advance phase
        // Enforce range 0.0 <= phase < 1.0
        let step = frequency_buffer.get(frame_num).unwrap()[0] / state.sample_rate;
        let mut phase = state.phase + step;
        while phase >= 1.0 {
            phase -= 1.0;
        }
        // Store the phase for next iteration
        state.phase = phase;

        // Get the aliasing pulse value
        let mut res = if phase < state.pulse_width.get() {
            1.0
        } else {
            -1.0
        };

        // PolyBLEP smoothing algorithm to reduce aliasing by smoothing discontinuities.
        let polyblep = |phase: defs::Sample, step: defs::Sample| -> defs::Sample {
            // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
            //   phase == 0:    x = 0.0
            //   phase == step: x = 1.0
            if phase < step {
                let x = phase / step;
                2.0 * x - x * x - 1.0
            }
            // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
            //   phase == (1.0 - step): x = 1.0
            //   phase == 1.0:          x = 0.0
            else if phase > (1.0 - step) {
                let x = (phase - 1.0) / step;
                2.0 * x + x * x + 1.0
            } else {
                0.0
            }
        };
        // Apply PolyBLEP to the first (upward) discontinuity
        res += polyblep(phase, step);
        // Apply PolyBLEP to the second (downward) discontinuity
        res -= polyblep((phase + 1.0 - state.pulse_width.get()) % 1.0, step);

        // Done
        *frame = [res as defs::Sample];
    }
}

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn sawtooth_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let frequency_buffer = state.frequency_buffer.get_sized_mut(buffer.len());

    for (frame_num, frame) in buffer.iter_mut().enumerate() {
        // Advance phase
        // Enforce range 0.0 <= phase < 1.0
        let step = frequency_buffer.get(frame_num).unwrap()[0] / state.sample_rate;
        let mut phase = state.phase + step;
        while phase >= 1.0 {
            phase -= 1.0;
        }
        // Store the phase for next iteration
        state.phase = phase;

        // Get the aliasing saw value
        let mut res = 1.0 - 2.0 * phase;

        // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
        // which always occur at phase == 0.0.
        // Apply PolyBLEP Smoothing for 0 < phase < (freq / sample_rate)
        //   phase == 0:    x = 0.0
        //   phase == step: x = 1.0
        if phase < step {
            let x = phase / step;
            res += 2.0 * x - x * x - 1.0;
        }
        // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
        //   phase == (1.0 - step): x = 1.0
        //   phase == 1.0:          x = 0.0
        else if phase > (1.0 - step) {
            let x = (phase - 1.0) / step;
            res += 2.0 * x + x * x + 1.0;
        }

        *frame = [res as defs::Sample]
    }
}
