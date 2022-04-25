//! Remember that the goal of this plugin project is **NOT** to create a reusable
//! shared DSP library (I believe that would be more hassle than it is worth). The
//! goal of this plugin project is to simply provide standalone "plugins", each with
//! their own optimized DSP implementation. We are however free to reference and
//! copy-paste portions of DSP across plugins as we see fit (as long as the other
//! plugins are also GPLv3).

#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use baseplug::{Plugin, ProcessContext};
use serde::{Deserialize, Serialize};

use example_gain_dsp::{ExampleGainEffect, ExampleGainHandle, ExampleGainPreset};

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ExampleGainModel {
        // Make sure this model matches the parameters in `ExampleGainEffect`.

        #[model(min = -90.0, max = 6.0)]
        #[parameter(name = "gain", unit = "Generic",  // Do *NOT* use baseplug's
        // "Decibels" unit because we are doing our own unit conversions.
            gradient = "Power(0.15)")]  // Make sure that this gradient matches
        // the gradients in `ExampleGainEffect`.
        #[unsmoothed]  // Make sure *ALL* parameters are "unsmoothed" because we are
        // doing our own parameter smoothing.
        gain_db: f32,
    }
}

// Insert the default preset here.
impl Default for ExampleGainModel {
    fn default() -> Self {
        Self { gain_db: 0.0 }
    }
}

struct ExampleGainPlug {
    effect: ExampleGainEffect,

    // If we had a GUI, we would send a clone of this handle to it.
    handle: ExampleGainHandle,
}

// Note: rust-analyzer thinks this is an error, but this should compile just fine.
baseplug::vst2!(ExampleGainPlug, b"5432");

impl Plugin for ExampleGainPlug {
    const NAME: &'static str = "Example Gain RustyDAW Plugin";
    const PRODUCT: &'static str = "Example Gain RustyDAW Plugin";
    const VENDOR: &'static str = "RustyDAW Org";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = ExampleGainModel;

    #[inline]
    fn new(sample_rate: f32, model: &ExampleGainModel) -> Self {
        // Convert from baseplug's preset to our preset.
        let preset = ExampleGainPreset {
            gain_db: model.gain_db,
        };

        let (effect, handle) = ExampleGainEffect::new(&preset, sample_rate.into());

        Self { effect, handle }
    }

    #[inline]
    fn process(&mut self, model: &ExampleGainModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        // Only process if the number of input/output buffers matches what we specified.
        if input.len() != 2 || output.len() != 2 {
            return;
        }

        // Update all the values from baseplug's model into our own smoothing model.
        self.effect.gain.set_value(*model.gain_db);

        // We just want the raw slices of the buffers.
        let in_l = input[0];
        let in_r = input[1];
        let (out_l, out_r) = output.split_first_mut().unwrap();
        let out_r = &mut out_r[0];

        #[cfg(all(
            any(target_arch = "x86", target_arch = "x86_64"),
            feature = "optimized_simd"
        ))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    self.effect
                        .process_avx2(ctx.nframes, in_l, in_r, out_l, out_r);
                };
                return;
            }
        }

        self.effect.process(ctx.nframes, in_l, in_r, out_l, out_r);
    }
}
