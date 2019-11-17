#![feature(test)]

pub type Sample = f32;                       // A single sample
pub type MonoFrame = sample::frame::Mono<Sample>;    // A slice of samples
pub type MonoFrameBufferSlice = [MonoFrame]; // A slice of frames

pub const PI: Sample = std::f32::consts::PI;
pub const TWOPI: Sample = 2.0 * PI;

extern crate test;

pub struct State {
    sample_rate: Sample,
    base_frequency: Sample,
    target_base_frequency: Sample,
    main_phase: Sample, // 0 <= main_phase <= 1
}

impl State {
    pub fn new() -> Self {
        Self {
            base_frequency: 1.0,
            target_base_frequency: 0.0,
            main_phase: 0.0,
            sample_rate: 0.0,
        }
    }
}

/// Generator function that produces a sine wave.
fn sine_generator(state: &mut State, buffer: &mut MonoFrameBufferSlice)
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
        buffer[frame_num][0] = Sample::sin(2.0 as Sample * PI * main_phase);
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// This version doesn't use frames.
/// My impression is the performance is the same as with frames. Have not
/// inspected the compiler output but I would guess it's similar.
fn sine_generator_no_frame(state: &mut State, buffer: &mut [Sample])
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
        buffer[frame_num] = Sample::sin(2.0 as Sample * PI * main_phase);
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// Using an iterator. Again seems to produce same performance.
fn sine_generator_iter(state: &mut State, buffer: &mut MonoFrameBufferSlice)
{
    let step = state.base_frequency / state.sample_rate;
    let mut main_phase = state.main_phase;

    let buffer_iter = buffer.iter_mut();
    for frame in buffer_iter {
        // Advance main_phase
        // Enforce range 0.0 <= main_phase < 1.0
        main_phase += step;
        while main_phase >= 1.0 {
            main_phase -= 1.0;
        }

        // Compute the output
        (*frame)[0] = Sample::sin(2.0 as Sample * PI * main_phase);
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

/// Using modulo to enforce phase range. Again seems to produce same performance.
fn sine_generator_iter_modulo(state: &mut State, buffer: &mut MonoFrameBufferSlice)
{
    let step = state.base_frequency / state.sample_rate;
    let mut main_phase = state.main_phase;
    let buffer_iter = buffer.iter_mut();

    for frame in buffer_iter {
        main_phase = (main_phase + step) % 1.0;
        (*frame)[0] = Sample::sin(2.0 * PI * main_phase);
    }

    // Store the main_phase for next iteration
    state.main_phase = main_phase;
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    const BUFSIZE: usize = 4096;

    #[bench]
    fn bench_sine_generator(b: &mut Bencher) {
        let mut state = State::new();
        state.sample_rate = 48000.0;
        state.base_frequency = 100.0;
        state.target_base_frequency = 100.0;
        let mut buffer = [[0.0]; BUFSIZE];
        b.iter(|| sine_generator(&mut state, &mut buffer));
    }

    #[bench]
    fn bench_sine_generator_no_frame(b: &mut Bencher) {
        let mut state = State::new();
        state.sample_rate = 48000.0;
        state.base_frequency = 100.0;
        state.target_base_frequency = 100.0;
        let mut buffer = [0.0; BUFSIZE];
        b.iter(|| sine_generator_no_frame(&mut state, &mut buffer));
    }

    #[bench]
    fn bench_sine_generator_iter(b: &mut Bencher) {
        let mut state = State::new();
        state.sample_rate = 48000.0;
        state.base_frequency = 100.0;
        state.target_base_frequency = 100.0;
        let mut buffer = [[0.0]; BUFSIZE];
        b.iter(|| sine_generator_iter(&mut state, &mut buffer));
    }

    #[bench]
    fn bench_sine_generator_iter_modulo(b: &mut Bencher) {
        let mut state = State::new();
        state.sample_rate = 48000.0;
        state.base_frequency = 100.0;
        state.target_base_frequency = 100.0;
        let mut buffer = [[0.0]; BUFSIZE];
        b.iter(|| sine_generator_iter(&mut state, &mut buffer));
    }
}
