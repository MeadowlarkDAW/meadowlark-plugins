#[macro_use]
extern crate vst;

use vst::plugin::{Info, Plugin};

#[derive(Default)]
struct ExampleGain;

impl Plugin for ExampleGain {
    fn get_info(&self) -> Info {
        Info {
            name: "Example Gain".to_string(),
            unique_id: 1357,

            ..Default::default()
        }
    }
}

plugin_main!(ExampleGain);
