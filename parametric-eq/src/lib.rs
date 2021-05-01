#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(min_specialization)]

use baseplug::{Plugin, ProcessContext, WindowOpenResult};
use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};
use rtrb::{RingBuffer, Producer};

mod ui;

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ParametricEQModel {
        #[model(min = -90.0, max = 3.0)]
        #[parameter(name = "out gain", unit = "Decibels",
            gradient = "Power(0.15)")]
        out_gain: f32,
    }
}

impl Default for ParametricEQModel {
    fn default() -> Self {
        Self {
            // "gain" is converted from dB to coefficient in the parameter handling code,
            // so in the model here it's a coeff.
            // -0dB == 1.0
            out_gain: 1.0,
        }
    }
}

struct ParametricEQ {}

impl Plugin for ParametricEQ {
    const NAME: &'static str = "Parametric EQ";
    const PRODUCT: &'static str = "Parametric EQ";
    const VENDOR: &'static str = "RustyDAW";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = ParametricEQModel;

    #[inline]
    fn new(_sample_rate: f32, _model: &ParametricEQModel) -> Self {
        Self {}
    }

    #[inline]
    fn process(&mut self, model: &ParametricEQModelProcess, ctx: &mut ProcessContext<Self>) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        for i in 0..ctx.nframes {
            output[0][i] = input[0][i] * model.out_gain[i];
            output[1][i] = input[1][i] * model.out_gain[i];
        }
    }
}

impl baseplug::PluginUI for ParametricEQ {
    type Handle = Producer<ui::UIHandleMsg>;

    fn ui_size() -> (i16, i16) {
        (230, 130)
    }

    fn ui_open(parent: &impl HasRawWindowHandle) -> WindowOpenResult<Self::Handle> {
        let (handle_msg_tx, handle_msg_rx) = RingBuffer::new(1024).split();

        ui::build_and_run(handle_msg_rx, parent);

        Ok(handle_msg_tx)
    }

    fn ui_param_notify(
        _handle: &Self::Handle,
        _param: &'static baseplug::Param<Self, <Self::Model as baseplug::Model<Self>>::Smooth>,
        _val: f32,
    ) {
        // TODO: implement this
    }

    fn ui_close(mut handle: Self::Handle) {
        // TODO: Do something more elegant than panicking if this fails.
        let _ = handle.push(ui::UIHandleMsg::CloseWindow).unwrap();
    }
}

baseplug::vst2!(ParametricEQ, b"tAnE");