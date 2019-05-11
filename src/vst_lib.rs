//! Synthesizer.
//!

#[cfg(feature = "vst")]
#[macro_use]
extern crate vst;

extern crate sample;

mod defs;

#[cfg(feature = "vst")]
use vst::plugin::{Info, Plugin};

#[allow(clippy::cast_precision_loss)]

#[cfg(feature = "vst")]
#[derive(Default)]
struct BaseliskPlugin;

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
}

#[cfg(feature = "vst")]
plugin_main!(BaseliskPlugin);
