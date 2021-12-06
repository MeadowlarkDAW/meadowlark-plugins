#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use baseplug::{Plugin, ProcessContext};
use serde::{Deserialize, Serialize};

use example_gain_dsp::ExampleGainDSP;

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ExampleGainModel {
        // Make sure this model matches the parameters in `ExampleGainDSP`.

        #[model(min = -90.0, max = 6.0)]  // Make sure this matches the values
        // in `ExampleGainDSP`.
        #[parameter(name = "left gain", unit = "Generic",  // Do *NOT* use baseplug's
        // "Decibels" unit because we are doing our own unit conversions.
            gradient = "Power(0.15)")]  // Make sure that this gradient matches
        // the gradients in `ExampleGainDSP`.
        #[unsmoothed]  // Make sure *ALL* parameters are "unsmoothed" because we are
        // doing our own parameter smoothing.
        gain_left: f32,

        #[model(min = -90.0, max = 6.0)]
        #[parameter(name = "right gain", unit = "Generic",
            gradient = "Power(0.15)")]
        #[unsmoothed]
        gain_right: f32,

        #[model(min = -90.0, max = 6.0)]
        #[parameter(name = "main gain", unit = "Generic",
            gradient = "Power(0.15)")]
        #[unsmoothed]
        gain_main: f32,
    }
}

struct ExampleGainPlug {
    example_gain_dsp: ExampleGainDSP<MAX_BLOCKSIZE>,
}

impl ExampleGainPlug {
    fn process_replacing(
        &mut self,
        model: &ExampleGainModelProcess,
        buf_left: &mut [f32],
        buf_right: &mut [f32],
    ) {
        // Update our parameters.
        self.example_gain_dsp.gain_left.set_value(*model.gain_left);
        self.example_gain_dsp
            .gain_right
            .set_value(*model.gain_right);
        self.example_gain_dsp.gain_main.set_value(*model.gain_main);

        self.example_gain_dsp.process_replacing(buf_left, buf_right);
    }
}

// --- Boilerplate stuff: ------------------------------------------------------------

// Insert the default preset here.
impl Default for ExampleGainModel {
    fn default() -> Self {
        Self {
            gain_left: 0.0,
            gain_right: 0.0,
            gain_main: 0.0,
        }
    }
}

/// This must stay the same as baseplug's internal `MAX_BLOCKSIZE` (128)
const MAX_BLOCKSIZE: usize = 128;

impl Plugin for ExampleGainPlug {
    const NAME: &'static str = "example gain plug";
    const PRODUCT: &'static str = "example gain plug";
    const VENDOR: &'static str = "RustyDAW";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = ExampleGainModel;

    #[inline]
    fn new(sample_rate: f32, model: &ExampleGainModel) -> Self {
        // If we had a UI we would also hold onto the Ui handle.
        let (example_gain_dsp, _) = ExampleGainDSP::new(
            -90.0, // min dB. Make sure this matches the parameters in the baseplug model.
            6.0,   // max dB. Make sure this matches the parameters in the baseplug model.
            model.gain_left,
            model.gain_right,
            model.gain_main,
            sample_rate.into(),
        );

        Self { example_gain_dsp }
    }

    #[inline]
    fn process(&mut self, model: &ExampleGainModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        // Only process if the number of input/output buffers matches what we specified.
        if input.len() != 2 || output.len() != 2 {
            return;
        }

        // Copy input buffers to output buffers.
        output[0].copy_from_slice(input[0]);
        output[1].copy_from_slice(input[1]);

        let (out_left, out_right) = output.split_first_mut().unwrap();
        let out_right = &mut out_right[0];

        // Process in blocks <= `MAX_BLOCKSIZE`
        let mut f = 0;
        let mut frames_left = out_left.len();
        while frames_left > 0 {
            let frames = frames_left.min(MAX_BLOCKSIZE);

            let buf_left_part = &mut out_left[f..f + frames];
            let buf_right_part = &mut out_right[f..f + frames];

            self.process_replacing(model, buf_left_part, buf_right_part);

            frames_left -= frames;
            f += frames;
        }
    }
}

baseplug::vst2!(ExampleGainPlug, b"5431");
