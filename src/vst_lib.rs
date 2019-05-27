//! Synthesizer.
//!

#[macro_use]
extern crate vst;

extern crate sample;

mod buffer;
mod defs;
mod engine;
mod event;
mod parameter;
mod processor;

#[cfg(feature = "plugin_vst")]
use sample::ToFrameSliceMut;
#[cfg(feature = "plugin_vst")]
use vst::{
    api::Events,
    buffer::AudioBuffer,
    plugin::{Category, Info, Plugin, PluginParameters},
};
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
        BaseliskPlugin {
            engine: engine::Engine::new(false),
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
        let output_buffer = outputs.get_mut(0)
            .to_frame_slice_mut().unwrap();

        self.engine.audio_requested(output_buffer)
    }

    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        self.engine.get_parameter_object()
    }
}

#[cfg(feature = "plugin_vst")]
plugin_main!(BaseliskPlugin);
