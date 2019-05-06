#![allow(clippy::cast_precision_loss)]

use defs;

use vst::plugin::{Info, Plugin};

#[derive(Default)]
struct BaseliskPlugin;

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

struct BaseliskPluginParameters {
    // To do...
}

plugin_main!(BaseliskPlugin);
