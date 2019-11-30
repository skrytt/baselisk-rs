use defs;
use engine::{
    pitch_bend,
    traits,
};
use rand;
use shared::{
    event::EngineEvent,
    parameter::{
        BaseliskPluginParameters,
        ParameterId,
    },
};
use std::slice;

/// Convert a note number to a corresponding frequency,
/// using 440 Hz as the pitch of the A above middle C.
fn get_frequency(note: defs::Sample) -> defs::Sample {
    440.0 * ((note - 69.0) / 12.0).exp2()
}

/// Internal state used by generator types.
pub struct State {
    sample_rate: defs::Sample,
    note: u8,
    pitch_bend_wheel_value: u16,
    pulse_width: defs::Sample,
    base_frequency: defs::Sample,
    target_base_frequency: defs::Sample,
    pitchbend_portamento_multiplier: defs::Sample,
    mod_index: defs::Sample,
    phase: defs::Sample, // 0 <= phase <= 1
}

impl State {
    pub fn new() -> Self {
        Self {
            note: 69,
            pitch_bend_wheel_value: 8192,
            pulse_width: 0.5,
            base_frequency: 1.0,
            target_base_frequency: 0.0,
            pitchbend_portamento_multiplier: 1.0,
            mod_index: 4.0,
            phase: 0.0,
            sample_rate: 0.0,
        }
    }

    pub fn panic(&mut self) {
        self.phase = 0.0;
    }
}

/// Signal generator type that will be used for audio processing.
pub struct Generator {
    id: usize,
    state: State,
}

enum GeneratorParams {
    Type,
    Pitch,
    PulseWidth,
    ModIndex,
}

impl Generator {
/// Function to construct new generators.
    pub fn new(id: usize) -> Self
    {
        Self {
            id,
            state: State::new(),
        }
    }

    fn get_parameter(&self, param: GeneratorParams) -> ParameterId {
        match self.id {
            0 => match param {
                GeneratorParams::Type => ParameterId::GeneratorAType,
                GeneratorParams::Pitch => ParameterId::GeneratorAPitch,
                GeneratorParams::PulseWidth => ParameterId::GeneratorAPulseWidth,
                GeneratorParams::ModIndex => ParameterId::GeneratorAModIndex,
            },
            1 => match param {
                GeneratorParams::Type => ParameterId::GeneratorBType,
                GeneratorParams::Pitch => ParameterId::GeneratorBPitch,
                GeneratorParams::PulseWidth => ParameterId::GeneratorBPulseWidth,
                GeneratorParams::ModIndex => ParameterId::GeneratorBModIndex,
            },
            _ => panic!("Unknown generator ID")
        }
    }

    /// There are multiple generators, and some parameter changes correspond to
    /// only one of them. This method returns true if a parameter change applies
    /// to this generator.
    fn should_trigger_keyframe_for_param(&self, param_id: ParameterId) -> bool {
        match self.id {
            0 => match param_id {
                ParameterId::GeneratorAType |
                ParameterId::GeneratorAPitch |
                ParameterId::GeneratorAPulseWidth |
                ParameterId::GeneratorAModIndex |
                ParameterId::PitchBendRange => true,
                _ => false,
            },
            1 => match param_id {
                ParameterId::GeneratorBType |
                ParameterId::GeneratorBPitch |
                ParameterId::GeneratorBPulseWidth |
                ParameterId::GeneratorBModIndex |
                ParameterId::PitchBendRange => true,
                _ => false,
            },
            _ => panic!("Unknown generator ID")
        }

    }

    pub fn process_buffer(&mut self,
               buffer: &mut defs::MonoFrameBufferSlice,
               mod_buffer: &defs::MonoFrameBufferSlice,
               mut engine_event_iter: slice::Iter<(usize, EngineEvent)>,
               sample_rate: defs::Sample,
               params: &BaseliskPluginParameters,
    ) {
        self.state.sample_rate = sample_rate;
        // Store buffer len to avoid multiple mutable buffer accesses later on
        let buffer_len = buffer.len();

        let generator_func: Option<fn(
            &mut State,
            &defs::MonoFrameBufferSlice,
            &mut defs::MonoFrameBufferSlice
        )> = match params.get_real_value(self.get_parameter(GeneratorParams::Type)) as usize {
                0 => Some(sine_generator),
                1 => Some(sawtooth_generator),
                2 => Some(pulse_generator),
                3 => Some(noise_generator),
                _ => None,
        };

        // Generate the outputs per-frame
        let mut this_keyframe: usize = 0;
        let mut next_keyframe: usize;
        loop {
            // Get next selected note, if there is one.
            let next_event = engine_event_iter.next();

            if let Some((frame_num, engine_event)) = next_event {
                match engine_event {
                    // Note changes will trigger keyframes only if there is a new note
                    // (i.e. not None)
                    EngineEvent::NoteChange{ note } => {
                        if note.is_none() {
                            continue
                        }
                    },
                    // Pitch bends and generator parameter changes will also trigger keyframes
                    EngineEvent::PitchBend{ .. } => (),
                    EngineEvent::ModulateParameter { param_id, .. } =>
                        if !self.should_trigger_keyframe_for_param(*param_id) { continue },
                }
                next_keyframe = *frame_num;
            } else {
                // No more events, so we'll process to the end of the buffer.
                next_keyframe = buffer_len;
            };

            // Apply the old parameters up until next_keyframe.
            let pitch_bend_semitones = pitch_bend::get_pitch_bend_semitones(
                self.state.pitch_bend_wheel_value, params);

            self.state.target_base_frequency = get_frequency(defs::Sample::from(self.state.note)
                                                + params.get_real_value(
                                                    self.get_parameter(GeneratorParams::Pitch))
                                                + pitch_bend_semitones);

            // Smoothing for pitch bends, to reduce audible stepping for wide pitch bends
            // (e.g. 12+ semitones).
            // The algorithm activates when the pitch wheel is used, and gently accelerates
            // the actual base_frequency towards the target_base_frequency.
            let diff = self.state.target_base_frequency - self.state.base_frequency;
            if diff > 0.0 {
                self.state.pitchbend_portamento_multiplier = defs::Sample::min(
                    self.state.pitchbend_portamento_multiplier * 1.005,
                    1.05,
                );
                self.state.base_frequency = defs::Sample::min(
                    self.state.pitchbend_portamento_multiplier * self.state.base_frequency,
                    self.state.target_base_frequency,
                );
            } else if diff < 0.0 {
                self.state.pitchbend_portamento_multiplier = defs::Sample::max(
                    self.state.pitchbend_portamento_multiplier * (1.0 / 1.005),
                    1.0 / 1.05,
                );
                self.state.base_frequency = defs::Sample::max(
                    self.state.pitchbend_portamento_multiplier * self.state.base_frequency,
                    self.state.target_base_frequency,
                );
            } else {
                self.state.pitchbend_portamento_multiplier = 1.0;
            }

            self.state.pulse_width = params.get_real_value(
                    self.get_parameter(GeneratorParams::PulseWidth));
            self.state.mod_index = params.get_real_value(
                    self.get_parameter(GeneratorParams::ModIndex));

            // Generate all the samples for this buffer
            let buffer_slice = buffer.get_mut(this_keyframe..next_keyframe).unwrap();
            let mod_buffer_slice = mod_buffer.get(this_keyframe..next_keyframe).unwrap();
            if let Some(generator_func) = generator_func {
                (generator_func)(&mut self.state, &mod_buffer_slice, buffer_slice);
            }

            // We've reached the next_keyframe.
            this_keyframe = next_keyframe;

            // What we do now depends on whether we reached the end of the buffer.
            if this_keyframe == buffer_len {
                // Loop exit condition: reached the end of the buffer.
                break
            } else {
                // Before the next iteration, use the event at this keyframe
                // to update the current state.
                let (_, event) = next_event.unwrap();
                match event {
                    EngineEvent::NoteChange{ note } => {
                        if let Some(note) = note {
                            self.state.note = *note;
                            // No portamento (set base frequency to what target
                            // frequency will be next iteration)
                            self.state.base_frequency = get_frequency(
                                    defs::Sample::from(self.state.note)
                                    + params.get_real_value(
                                        self.get_parameter(GeneratorParams::Pitch))
                                    + pitch_bend_semitones);
                        }
                    },
                    EngineEvent::PitchBend{ wheel_value } => {
                        self.state.pitch_bend_wheel_value = *wheel_value;
                    },
                    EngineEvent::ModulateParameter { param_id, value } =>
                        if self.should_trigger_keyframe_for_param(*param_id) {
                            params.set_parameter(*param_id, *value);
                        }
                };
            }
        }
    }
}

impl traits::Processor for Generator {
    fn panic(&mut self) {
        self.state.panic();
    }
}

/// Generator function that produces a frequency-modulated wave.
fn sine_generator(
    state: &mut State,
    mod_buffer: &defs::MonoFrameBufferSlice,
    buffer: &mut defs::MonoFrameBufferSlice,
)
{
    let mut phase = state.phase;

    for (frame, mod_frame) in buffer.iter_mut().zip(mod_buffer.iter()) {
        // Modulator influence is a function of modulator output value
        // and the mod_index of this oscillator (i.e. how much we want the value of
        // modulator oscillator to influence the freuqnecy of this oscillator)
        // TBC: should this be multiplied by mod freq, not this osc freq?
        let freq_offset = state.mod_index * mod_frame[0] * state.base_frequency;

        // Advance carrier phase
        // Enforce range 0.0 <= phase < 1.0
        let step = (state.base_frequency + freq_offset) / state.sample_rate;
        phase = (phase + step) % 1.0;

        frame[0] = defs::Sample::sin(2.0 as defs::Sample * defs::PI * phase);
    }

    // Store the phases for next iteration
    state.phase = phase;
}

/// Generator function that produces a pulse wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn pulse_generator(
    state: &mut State,
    mod_buffer: &defs::MonoFrameBufferSlice,
    buffer: &mut defs::MonoFrameBufferSlice,
)
{
    let mut phase = state.phase;

    for (frame, mod_frame) in buffer.iter_mut().zip(mod_buffer.iter()) {
        // Modulator influence is a function of modulator output value
        // and the mod_index of this oscillator (i.e. how much we want the value of
        // modulator oscillator to influence the freuqnecy of this oscillator)
        // TBC: should this be multiplied by mod freq, not this osc freq?
        let freq_offset = state.mod_index * mod_frame[0] * state.base_frequency;

        // Enforce range 0.0 <= phase < 1.0
        let step = (state.base_frequency + freq_offset) / state.sample_rate;
        phase = (phase + step) % 1.0;

        // Get the aliasing pulse value
        let mut res = if phase < state.pulse_width {
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
        res -= polyblep((phase + 1.0 - state.pulse_width) % 1.0, step);

        // Done
        frame[0] = res;
    }

    // Store the phase for next iteration
    state.phase = phase;
}

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn sawtooth_generator(
    state: &mut State,
    mod_buffer: &defs::MonoFrameBufferSlice,
    buffer: &mut defs::MonoFrameBufferSlice,
)
{
    let mut phase = state.phase;

    for (frame, mod_frame) in buffer.iter_mut().zip(mod_buffer.iter()) {
        // Modulator influence is a function of modulator output value
        // and the mod_index of this oscillator (i.e. how much we want the value of
        // modulator oscillator to influence the freuqnecy of this oscillator)
        // TBC: should this be multiplied by mod freq, not this osc freq?
        let freq_offset = state.mod_index * mod_frame[0] * state.base_frequency;

        let step = (state.base_frequency + freq_offset) / state.sample_rate;
        // Enforce range 0.0 <= phase < 1.0
        phase = (phase + step) % 1.0;

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

        // Done
        frame[0] = res;
    }

    // Store the phase for next iteration
    state.phase = phase;
}

/// Generator function that produces noise.
fn noise_generator(
    _state: &mut State,
    _mod_buffer: &defs::MonoFrameBufferSlice,
    buffer: &mut defs::MonoFrameBufferSlice
)
{
    for frame_num in 0..buffer.len() {
        buffer[frame_num][0] = rand::random();
    }
}
