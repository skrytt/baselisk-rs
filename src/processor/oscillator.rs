extern crate sample;

use buffer::ResizableFrameBuffer;
use defs;
use event::{Event, MidiEvent};
use std::slice;


/// Convert a u8 note number to a corresponding frequency,
/// using 440 Hz as the pitch of the A above middle C.
fn get_frequency(note: defs::Sample) -> defs::Sample {
    440.0 * ((note - 69.0) / 12.0).exp2()
}

/// Internal state used by oscillator types.
pub struct State {
    frequency_current: defs::Sample,
    pitch_offset: defs::Sample, // Semitones
    pitch_bend: defs::Sample,   // Semitones
    pulse_width: defs::Sample,  // 0.001 <= pulse_width <= 0.999
    phase: defs::Sample,        // 0 <= phase <= 1
    sample_rate: defs::Sample,
    frequency_buffer: ResizableFrameBuffer<defs::MonoFrame>,
}

impl State {
    pub fn new() -> State {
        State {
            frequency_current: 0.0,
            pitch_offset: 0.0,
            pitch_bend: 0.0,
            pulse_width: 0.5,
            phase: 0.0,
            sample_rate: 0.0,
            frequency_buffer: ResizableFrameBuffer::new(),
        }
    }

    pub fn reset(&mut self) {
        self.frequency_current = 0.0;
        self.pitch_bend = 0.0;
        self.phase = 0.0;
        self.sample_rate = 0.0;
    }

    pub fn set_pitch_bend(&mut self, value: u16, range: defs::Sample) {
        // Value is 14-bit (range 0 <= value <= 16383)
        // 0 => -2
        // 8192 => 0
        // 16383 => ~= +2
        self.pitch_bend = range * (value as defs::Sample - 8192.0) / 8192.0;
    }

    /// Process any events and update the internal state accordingly.
    fn update(&mut self,
              midi_iter: slice::Iter<(usize, Event)>,
              mut selected_note_iter: slice::Iter<(usize, Option<u8>)>,
              sample_rate: defs::Sample,
              buffer_size: usize)
    {
        // Process MIDI events first.
        // We only handle note/sound off events, and silence everything for this buffer
        // if one is handled.
        for (_frame_num, event) in midi_iter {
            if let Event::Midi(midi_event) = event {
                match midi_event {
                    MidiEvent::AllNotesOff | MidiEvent::AllSoundOff => {
                        // Reset state, silence buffers and don't process anything
                        // further this buffer.
                        self.reset();
                        let frequency_buffer = self.frequency_buffer.get_sized_mut(buffer_size);
                        for frame in frequency_buffer.iter_mut() {
                            for sample in frame {
                                *sample = 0.0;
                            }
                        }
                        return
                    },
                    _ => (),
                }
            }
        }

        // Sample rate used by the generator functions
        self.sample_rate = sample_rate;

        // Calculate the frequencies per-frame
        let frequency_buffer = self.frequency_buffer.get_sized_mut(buffer_size);
        let mut frame_num_current: usize = 0;
        let mut frequency_current: defs::Sample = self.frequency_current;
        let mut frame_num_next: usize;
        let mut frequency_next: defs::Sample = frequency_current;
        loop {
            // Get next selected note, if there is one.
            match selected_note_iter.next() {
                Some((frame_num, note_change)) => {
                    // Ignore note-offs so that Release ADSR stages work as intended.
                    match note_change {
                        None => continue,
                        Some(note) => {
                            frame_num_next = *frame_num;
                            frequency_next = get_frequency(*note as defs::Sample);
                        },
                    }
                },
                None => {
                    // No more note change events.
                    frame_num_next = buffer_size;
                },
            };

            // Apply the current frequency up until the next change
            if let Some(frequency_buffer_slice) = frequency_buffer.get_mut(frame_num_current..frame_num_next) {
                for frequency_frame in frequency_buffer_slice {
                    for frequency_sample in frequency_frame {
                        *frequency_sample = frequency_current;
                    }
                }
            }

            // Exit condition: reached the end of the buffer.
            if frame_num_next == buffer_size {
                break
            }

            // Update the current frequency and frame num for next iteration.
            frequency_current = frequency_next;
            frame_num_current = frame_num_next;
        }
        self.frequency_current = frequency_current;
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
            _ => return Err("Unknown oscillator name"),
        };
        self.generator_func = generator_func;
        Ok(())
    }

    pub fn set_pitch(&mut self, semitones: defs::Sample) -> Result<(), &'static str>
    {
        if semitones < -36.0 {
            Err("Pitch offset must be >= -36.0 semitones")
        } else if semitones > 36.0 {
            Err("Pitch offset must be <= 36.0 semitones")
        } else {
            self.state.pitch_offset = semitones;
            Ok(())
        }
    }

    pub fn set_pulse_width(&mut self, width: defs::Sample) -> Result<(), &'static str>
    {
        if width < 0.001 || width > 0.999 {
            Err("Pulse width must be in range 0.001 <= width <= 0.999")
        } else {
            self.state.pulse_width = width;
            Ok(())
        }
    }

    pub fn process_buffer(&mut self,
               buffer: &mut defs::MonoFrameBufferSlice,
               selected_note_iter: slice::Iter<(usize, Option<u8>)>,
               midi_iter: slice::Iter<(usize, Event)>,
               sample_rate: defs::Sample,
    ) {
        self.state.update(midi_iter, selected_note_iter, sample_rate, buffer.len());

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
                return 2.0 * x - x * x - 1.0;
            }
            // Apply PolyBLEP Smoothing for (1.0 - (freq / sample_rate)) < phase < 1.0:
            //   phase == (1.0 - step): x = 1.0
            //   phase == 1.0:          x = 0.0
            else if phase > (1.0 - step) {
                let x = (phase - 1.0) / step;
                return 2.0 * x + x * x + 1.0;
            } else {
                0.0
            }
        };
        // Apply PolyBLEP to the first (upward) discontinuity
        res += polyblep(phase, step);
        // Apply PolyBLEP to the second (downward) discontinuity
        res -= polyblep((phase + 1.0 - state.pulse_width) % 1.0, step);

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
