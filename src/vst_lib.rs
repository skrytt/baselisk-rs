//! Synthesizer.
//!

#[cfg(feature = "vst")]
#[macro_use]
extern crate vst;

extern crate sample;

mod buffer;
mod defs;
mod engine;
mod event;
mod parameter;
mod processor;

#[cfg(feature = "vst")]
use vst::{
    api::Events,
    plugin::{Info, Plugin},
};


#[allow(clippy::cast_precision_loss)]

#[cfg(feature = "vst")]
struct BaseliskPlugin {
    engine: engine::Engine,
}

#[cfg(feature = "vst")]
impl Default for BaseliskPlugin {
    fn default() -> BaseliskPlugin {
        BaseliskPlugin {
            engine: engine::Engine::new(false),
        }
    }
}

#[cfg(feature = "vst")]
impl Plugin for BaseliskPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: defs::PLUGIN_NAME.to_string(),
            unique_id: 5211,
            // Parameters to be added
            ..Default::default()
        }
    }

    fn process_events(&mut self, events: &Events) {
        self.engine.vst_process_events(events);
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.engine.set_sample_rate(defs::Sample::from(sample_rate))
    }
}

#[cfg(feature = "vst")]
plugin_main!(BaseliskPlugin);
