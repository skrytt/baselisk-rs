use defs;
use rand;
use shared::{
    event::EngineEvent,
    parameter::{
        BaseliskPluginParameters,
        PARAM_OSCILLATOR_TYPE,
        PARAM_OSCILLATOR_PITCH,
        PARAM_OSCILLATOR_PULSE_WIDTH,
        PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO,
        PARAM_OSCILLATOR_MOD_INDEX,
        PARAM_OSCILLATOR_PITCH_BEND_RANGE,
    },
};
use engine::pitch_bend;
use std::slice;
use vst::plugin::PluginParameters;

/// Convert a note number to a corresponding frequency,
/// using 440 Hz as the pitch of the A above middle C.
fn get_frequency(note: defs::Sample) -> defs::Sample {
    440.0 * ((note - 69.0) / 12.0).exp2()
}

/// Internal state used by oscillator types.
pub struct State {
    sample_rate: defs::Sample,
    note: u8,
    pitch_bend_wheel_value: u16,
    pulse_width: defs::Sample,
    base_frequency: defs::Sample,
    mod_frequency: defs::Sample,
    target_base_frequency: defs::Sample,
    pitchbend_portamento_multiplier: defs::Sample,
    mod_index: defs::Sample,
    main_phase: defs::Sample, // 0 <= main_phase <= 1
    mod_phase: defs::Sample,  // 0 <= mod_phase <= 1
}

impl State {
    pub fn new() -> Self {
        Self {
            note: 69,
            pitch_bend_wheel_value: 8192,
            pulse_width: 0.5,
            base_frequency: 1.0,
            mod_frequency: 1.0,
            target_base_frequency: 0.0,
            pitchbend_portamento_multiplier: 1.0,
            mod_index: 4.0,
            main_phase: 0.0,
            mod_phase: 0.0,
            sample_rate: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.base_frequency = 1.0;
        self.target_base_frequency = 0.0;
        self.pitch_bend_wheel_value = 8192;
        self.main_phase = 0.0;
        self.mod_phase = 0.0;
        self.sample_rate = 0.0;
    }
}

/// Oscillator type that will be used for audio processing.
pub struct Oscillator {
    state: State,
}

impl Oscillator {
/// Function to construct new oscillators.
    pub fn new() -> Self
    {
        Self {
            state: State::new(),
        }
    }

    pub fn midi_panic(&mut self) {
        self.state.reset();
    }

    pub fn process_buffer(&mut self,
               buffer: &mut defs::MonoFrameBufferSlice,
               mut engine_event_iter: slice::Iter<(usize, EngineEvent)>,
               sample_rate: defs::Sample,
               params: &BaseliskPluginParameters,
    ) {
        self.state.sample_rate = sample_rate;
        // Store buffer len to avoid multiple mutable buffer accesses later on
        let buffer_len = buffer.len();


        let generator_func: Option<fn(&mut State, &mut defs::MonoFrameBufferSlice)> =
            match params.get_real_value(PARAM_OSCILLATOR_TYPE) as usize {
                0 => Some(sine_generator),
                1 => Some(sawtooth_generator),
                2 => Some(pulse_generator),
                3 => Some(frequency_modulated_generator),
                4 => Some(noise_generator),
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
                    // Pitch bends and oscillator parameter changes will also trigger keyframes
                    EngineEvent::PitchBend{ .. } => (),
                    EngineEvent::ModulateParameter { param_id, .. } => match *param_id {
                        PARAM_OSCILLATOR_TYPE |
                        PARAM_OSCILLATOR_PITCH |
                        PARAM_OSCILLATOR_PULSE_WIDTH |
                        PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO |
                        PARAM_OSCILLATOR_MOD_INDEX |
                        PARAM_OSCILLATOR_PITCH_BEND_RANGE => (),
                        _ => continue,
                    },
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
                                                + params.get_real_value(PARAM_OSCILLATOR_PITCH)
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

            self.state.mod_frequency = self.state.base_frequency
                                       * params.get_real_value(PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO);

            // Generate all the samples for this buffer
            let buffer_slice = buffer.get_mut(this_keyframe..next_keyframe).unwrap();
            if let Some(generator_func) = generator_func {
                (generator_func)(&mut self.state, buffer_slice);
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
                                                + params.get_real_value(PARAM_OSCILLATOR_PITCH)
                                                + pitch_bend_semitones);
                        }
                    },
                    EngineEvent::PitchBend{ wheel_value } => {
                        self.state.pitch_bend_wheel_value = *wheel_value;
                    },
                    EngineEvent::ModulateParameter { param_id, value } => match *param_id {
                        PARAM_OSCILLATOR_TYPE |
                        PARAM_OSCILLATOR_PITCH |
                        PARAM_OSCILLATOR_MOD_FREQUENCY_RATIO |
                        PARAM_OSCILLATOR_PITCH_BEND_RANGE => {
                            params.set_parameter(*param_id, *value);
                        },
                        PARAM_OSCILLATOR_PULSE_WIDTH => {
                            params.set_parameter(*param_id, *value);
                            self.state.pulse_width = params.get_real_value(
                                PARAM_OSCILLATOR_PULSE_WIDTH);
                        },
                        PARAM_OSCILLATOR_MOD_INDEX => {
                            params.set_parameter(*param_id, *value);
                            self.state.mod_index = params.get_real_value(PARAM_OSCILLATOR_MOD_INDEX);
                        }
                        _ => (),
                    },
                };
            }
        }
    }
}

/// Generator function that produces a sine wave.
fn sine_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let step = state.base_frequency / state.sample_rate;
    let mut main_phase = state.main_phase;

    for frame_num in 0..buffer.len() {
        // Advance main_phase
        // Enforce range 0.0 <= main_phase < 1.0
        main_phase += step;
        while main_phase >= 1.0 {
            main_phase -= 1.0;
        }

        // Compute the output
        buffer[frame_num][0] = defs::Sample::sin(2.0 as defs::Sample * defs::PI * main_phase);
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// Generator function that produces a pulse wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn pulse_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let step = state.base_frequency / state.sample_rate;
    let mut main_phase = state.main_phase;

    for frame_num in 0..buffer.len() {
        // Advance main_phase
        // Enforce range 0.0 <= main_phase < 1.0
        main_phase += step;
        while main_phase >= 1.0 {
            main_phase -= 1.0;
        }

        // Get the aliasing pulse value
        let mut res = if main_phase < state.pulse_width {
            1.0
        } else {
            -1.0
        };

        // PolyBLEP smoothing algorithm to reduce aliasing by smoothing discontinuities.
        let polyblep = |main_phase: defs::Sample, step: defs::Sample| -> defs::Sample {
            // Apply PolyBLEP Smoothing for 0 < main_phase < (freq / sample_rate)
            //   main_phase == 0:    x = 0.0
            //   main_phase == step: x = 1.0
            if main_phase < step {
                let x = main_phase / step;
                2.0 * x - x * x - 1.0
            }
            // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < main_phase < 1.0:
            //   main_phase == (1.0 - step): x = 1.0
            //   main_phase == 1.0:          x = 0.0
            else if main_phase > (1.0 - step) {
                let x = (main_phase - 1.0) / step;
                2.0 * x + x * x + 1.0
            } else {
                0.0
            }
        };
        // Apply PolyBLEP to the first (upward) discontinuity
        res += polyblep(main_phase, step);
        // Apply PolyBLEP to the second (downward) discontinuity
        res -= polyblep((main_phase + 1.0 - state.pulse_width) % 1.0, step);

        // Done
        buffer[frame_num][0] = res;
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// Generator function that produces a sawtooth wave.
/// Uses PolyBLEP smoothing to reduce aliasing.
fn sawtooth_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let step = state.base_frequency / state.sample_rate;
    let mut main_phase = state.main_phase;

    for frame_num in 0..buffer.len() {
        // Advance main_phase
        // Enforce range 0.0 <= main_phase < 1.0
        main_phase += step;
        while main_phase >= 1.0 {
            main_phase -= 1.0;
        }

        // Get the aliasing saw value
        let mut res = 1.0 - 2.0 * main_phase;

        // PolyBLEP smoothing to reduce aliasing by smoothing discontinuities,
        // which always occur at main_phase == 0.0.
        // Apply PolyBLEP Smoothing for 0 < main_phase < (freq / sample_rate)
        //   main_phase == 0:    x = 0.0
        //   main_phase == step: x = 1.0
        if main_phase < step {
            let x = main_phase / step;
            res += 2.0 * x - x * x - 1.0;
        }
        // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < main_phase < 1.0:
        //   main_phase == (1.0 - step): x = 1.0
        //   main_phase == 1.0:          x = 0.0
        else if main_phase > (1.0 - step) {
            let x = (main_phase - 1.0) / step;
            res += 2.0 * x + x * x + 1.0;
        }

        // Done
        buffer[frame_num][0] = res;
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// Generator function that produces a sine wave.
fn frequency_modulated_generator(state: &mut State, buffer: &mut defs::MonoFrameBufferSlice)
{
    let mod_freq = state.mod_frequency;
    let mod_step = mod_freq / state.sample_rate;
    let mut mod_phase = state.mod_phase;

    let mut main_phase = state.main_phase;

    for frame_num in 0..buffer.len() {
        // Advance main_phase of mod oscillator
        // Enforce range 0.0 <= main_phase < 1.0
        mod_phase += mod_step;
        while mod_phase >= 1.0 {
            mod_phase -= 1.0;
        }

        let mod_res = defs::Sample::sin(2.0 as defs::Sample * defs::PI * mod_phase);
        let mod_freq_offset = state.mod_index * mod_res * mod_freq;

        // Advance main_phase of main oscillator
        // Enforce range 0.0 <= main_phase < 1.0
        let main_step = (state.base_frequency + mod_freq_offset) / state.sample_rate;
        main_phase += main_step;
        // Due to rate modulation main_phase can go backwards as well as forwards
        while main_phase < 0.0 {
            main_phase += 1.0;
        }
        while main_phase >= 1.0 {
            main_phase -= 1.0;
        }
        buffer[frame_num][0] = defs::Sample::sin(2.0 as defs::Sample * defs::PI * main_phase);
    }

    // Store the phases for next iteration
    state.mod_phase = mod_phase;
    state.main_phase = main_phase;
}

/// Generator function that produces noise.
fn noise_generator(_state: &mut State, buffer: &mut defs::MonoFrameBufferSlice) {
    for frame_num in 0..buffer.len() {
        buffer[frame_num][0] = rand::random();
    }
}
