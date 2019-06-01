//! Synthesizer.
//!

#[macro_use]
extern crate vst;

extern crate sample;

mod defs;
mod engine;
mod event;
mod modmatrix;
mod parameter;
mod shared;

#[cfg(feature = "plugin_vst")]
use sample::ToFrameSliceMut;
#[cfg(feature = "plugin_vst")]
use vst::{
    api::Events,
    buffer::AudioBuffer,
    plugin::{Category, Info, Plugin, PluginParameters},
};
#[cfg(feature = "plugin_vst")]
use shared::SharedState;
#[cfg(feature = "plugin_vst")]
use std::sync::Arc;


#[allow(clippy::cast_precision_loss)]

#[cfg(feature = "plugin_vst")]
struct BaseliskPlugin {
    engine: engine::Engine,
}

#[cfg(feature = "plugin_vst")]
impl Default for BaseliskPlugin {
    fn default() -> BaseliskPlugin {
        // Parameters will be shared between threads
        let shared_state = Arc::new(SharedState::new());

        BaseliskPlugin {
            engine: engine::Engine::new(shared_state, false),
        }
    }
}

#[cfg(feature = "plugin_vst")]
impl Plugin for BaseliskPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: defs::PLUGIN_NAME.to_string(),
            unique_id: 5211,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: parameter::NUM_PARAMS,
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        self.engine.vst_process_events(events);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.engine.set_sample_rate(sample_rate as defs::Sample)
    }

    fn process(&mut self, vst_audio_buffer: &mut AudioBuffer<defs::Sample>) {
        let (_, outputs) = vst_audio_buffer.split();

        // Currently will only output audio to first output buffer
        let left_output_buffer = outputs.get_mut(0)
            .to_frame_slice_mut().unwrap();
        let right_output_buffer = outputs.get_mut(1)
            .to_frame_slice_mut().unwrap();

        self.engine.audio_requested(left_output_buffer, right_output_buffer)
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.engine.get_parameter_object()
    }
}

#[cfg(feature = "plugin_vst")]
plugin_main!(BaseliskPlugin);
