#![allow(clippy::cast_precision_loss)]

mod adsr;
mod buffer;
mod delay;
mod gain;
mod generator;
mod filter;
mod note_selector;
mod pitch_bend;
mod traits;
mod waveshaper;

use defs;
use shared::{
    event::{
        EngineEvent,
        MidiEvent,
        RawMidi
    },
    SharedState,
};
use engine::{
    adsr::Adsr,
    buffer::ResizableFrameBuffer,
    delay::Delay,
    generator::Generator,
    filter::Filter,
    note_selector::MonoNoteSelector,
    traits::Processor,
};
use sample::slice;
use std::sync::Arc;

use vst::plugin::PluginParameters;

pub struct Engine
{
    // Misc
    sample_rate: defs::Sample,
    shared_state: Arc<SharedState>,
    raw_midi_buffer: Vec<RawMidi>,
    engine_event_buffer: Vec<(usize, EngineEvent)>,
    note_selector: MonoNoteSelector,
    timing_data: TimingData,
    dump_timing_info: bool,
    // Buffers
    mono_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    generator_a_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    generator_b_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    adsr_buffer: ResizableFrameBuffer<defs::MonoFrame>,
    // DSP Units
    generator_a: Generator,
    generator_b: Generator,
    adsr: Adsr,
    filter: Filter,
    delay: Delay,
}

impl Engine
{
    pub fn new(shared_state: Arc<SharedState>,
               dump_timing_info: bool) -> Self {
        Self {
            // Engine Event Processing
            sample_rate: 0.0,
            shared_state,
            raw_midi_buffer: Vec::with_capacity(defs::RAW_MIDI_BUF_LEN),
            engine_event_buffer: Vec::with_capacity(defs::ENGINE_EVENT_BUF_LEN),
            note_selector: MonoNoteSelector::new(),
            timing_data: TimingData::default(),
            dump_timing_info,
            // Buffers
            mono_buffer: ResizableFrameBuffer::new(),
            adsr_buffer: ResizableFrameBuffer::new(),
            generator_a_buffer: ResizableFrameBuffer::new(),
            generator_b_buffer: ResizableFrameBuffer::new(),
            // DSP Units
            generator_a: Generator::new(0),
            generator_b: Generator::new(1),
            adsr: Adsr::new(),
            filter: Filter::new(),
            delay: Delay::new(),
        }
    }

    pub fn get_parameter_object(&self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.shared_state.parameters) as Arc<dyn PluginParameters>
    }

    pub fn set_sample_rate(&mut self,
                           sample_rate: defs::Sample)
    {
        self.sample_rate = sample_rate;
        self.delay.set_sample_rate(sample_rate);
    }

    pub fn clear_midi_buffer(&mut self) {
        self.raw_midi_buffer.clear();
    }

    pub fn push_raw_midi(&mut self, event: RawMidi) {
        self.raw_midi_buffer.push(event);
    }

    /// Request audio.
    /// Buffer is a mutable slice of frames,
    /// where each frame is a slice containing a single sample.
    pub fn audio_requested(&mut self,
                           left_output_buffer: &mut defs::MonoFrameBufferSlice,
                           right_output_buffer: &mut defs::MonoFrameBufferSlice)
    {
        let engine_start_time = time::precise_time_ns();

        slice::equilibrium(left_output_buffer);
        slice::equilibrium(right_output_buffer);

        self.engine_event_buffer.clear();
        let mut midi_panic = false;
        for raw_midi_event in self.raw_midi_buffer.iter() {
            if let Some((frame_num, midi_event)) = MidiEvent::parse(&raw_midi_event, None) {
                // Check for MIDI panics.
                match midi_event {
                    MidiEvent::AllNotesOff | MidiEvent::AllSoundOff => {
                        midi_panic = true;
                        break
                    },
                    _ => (),
                }
                if let Some(engine_event) = self.note_selector.process_event(&midi_event) {
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
                if let MidiEvent::PitchBend{ value } = midi_event
                {
                    let engine_event = EngineEvent::PitchBend{ wheel_value: value };
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
                if let Some(engine_event) = self.shared_state.modmatrix.process_event(&midi_event) {
                    self.engine_event_buffer.push((frame_num, engine_event));
                }
            }
        }

        // If we are panicking, we run this alternate code to reset state
        // and do not process audio this buffer.
        if midi_panic {
            self.handle_midi_panic();
            return
        }

        self.timing_data.pre = (time::precise_time_ns() - engine_start_time) / 1000;

        let frames_this_buffer = left_output_buffer.len();

        let mut mono_buffer = self.mono_buffer.get_sized_mut(frames_this_buffer);
        slice::equilibrium(mono_buffer);

        // ADSR buffer for Gain and Filter (shared for now).
        let adsr_start_time = time::precise_time_ns();

        let adsr_buffer = self.adsr_buffer.get_sized_mut(frames_this_buffer);

        let adsr_any_nonzero_output = self.adsr.process_buffer(
            adsr_buffer,
            self.engine_event_buffer.iter(),
            self.sample_rate,
            &self.shared_state.parameters
        );

        self.timing_data.adsr = (time::precise_time_ns() - adsr_start_time) / 1000;

        // Optimization: when ADSR is in the off state for a whole buffer,
        // the result of the generator and gain stages is silence
        if adsr_any_nonzero_output {

            // Signal Generator
            let generator_start_time = time::precise_time_ns();

            let mut generator_a_buffer = self.generator_a_buffer.get_sized_mut(frames_this_buffer);
            let mut generator_b_buffer = self.generator_b_buffer.get_sized_mut(frames_this_buffer);

            self.generator_a.process_buffer(
                &mut generator_a_buffer,
                self.engine_event_buffer.iter(),
                self.sample_rate,
                &self.shared_state.parameters
            );

            self.generator_b.process_buffer(
                &mut generator_b_buffer,
                self.engine_event_buffer.iter(),
                self.sample_rate,
                &self.shared_state.parameters
            );

            sample::slice::add_in_place(&mut mono_buffer, &generator_a_buffer);
            sample::slice::add_in_place(&mut mono_buffer, &generator_b_buffer);

            self.timing_data.generator = (time::precise_time_ns() - generator_start_time) / 1000;

            // Use ADSR to apply gain to generator output
            let gain_start_time = time::precise_time_ns();
            gain::process_buffer(adsr_buffer, mono_buffer);
            self.timing_data.gain = (time::precise_time_ns() - gain_start_time) / 1000;

        }

        // Filter
        let filter_start_time = time::precise_time_ns();

        self.filter.process_buffer(
            adsr_buffer,
            mono_buffer,
            self.engine_event_buffer.iter(),
            self.sample_rate,
            &self.shared_state.parameters
        );

        self.timing_data.filter = (time::precise_time_ns() - filter_start_time) / 1000;

        // Waveshaper
        let waveshaper_start_time = time::precise_time_ns();

        waveshaper::process_buffer(
            mono_buffer,
            self.engine_event_buffer.iter(),
            &self.shared_state.parameters
        );

        self.timing_data.waveshaper = (time::precise_time_ns() - waveshaper_start_time) / 1000;

        // Copy the mono buffer to both outputs
        sample::slice::write(left_output_buffer, mono_buffer);
        sample::slice::write(right_output_buffer, mono_buffer);

        // Delay
        let delay_start_time = time::precise_time_ns();

        self.delay.process_buffer(
            left_output_buffer,
            right_output_buffer,
            self.engine_event_buffer.iter(),
            self.sample_rate,
            &self.shared_state.parameters
        );

        self.timing_data.delay = (time::precise_time_ns() - delay_start_time) / 1000;

        self.timing_data.total = (time::precise_time_ns() - engine_start_time) / 1000;

        self.timing_data.window = 1_000_000.0 * left_output_buffer.len() as f32 / self.sample_rate;

        if self.dump_timing_info {
            self.timing_data.dump_to_stderr();
        }
    }

    fn handle_midi_panic(&mut self) {
        self.note_selector.panic();
        self.generator_a.panic();
        self.generator_b.panic();
        self.adsr.panic();
        self.filter.panic();
        self.delay.panic();
    }
}

#[derive(Default)]
struct TimingData {
    pre: u64,
    generator: u64,
    adsr: u64,
    gain: u64,
    filter: u64,
    waveshaper: u64,
    delay: u64,
    total: u64,
    window: f32,
}
impl TimingData {
    fn dump_to_stderr(&self) {
        eprintln!("pre:{:3}us gen:{:3}us adsr:{:3}us gain:{:3}us fltr:{:3}us wshp:{:3}us dly:{:3}us total:{:3}us [{:3.3}%]",
                self.pre,
                self.generator,
                self.adsr,
                self.gain,
                self.filter,
                self.waveshaper,
                self.delay,
                self.total,
                100.0 * (self.total as f32 / self.window),
        );
    }
}
