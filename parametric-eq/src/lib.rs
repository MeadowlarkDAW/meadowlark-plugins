#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(min_specialization)]

use baseplug::{Plugin, PluginContext, ProcessContext, WindowOpenResult};
use raw_window_handle::HasRawWindowHandle;
use serde::{Deserialize, Serialize};
use rtrb::{Consumer, Producer, RingBuffer};
use triple_buffer::{Input, Output, TripleBuffer};
use atomic_refcell::AtomicRefCell;

use std::sync::{Arc, Mutex};
use std::rc::Rc;
use std::cell::UnsafeCell;
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

// Shared state between UI and audio threads
pub struct ParametricEQShared {
    pub producer: Arc<AtomicRefCell<Input<Vec<f32>>>>,
    pub consumer: Arc<AtomicRefCell<Output<Vec<f32>>>>,
}

unsafe impl Send for ParametricEQShared {}
unsafe impl Sync for ParametricEQShared {}

impl PluginContext<ParametricEQ> for ParametricEQShared {
    fn new() -> Self {

        let triple_buffer = TripleBuffer::new(Vec::<f32>::with_capacity(1024));

        let (mut producer, mut consumer) = triple_buffer.split();

        Self {
            
            producer: Arc::new(AtomicRefCell::new(producer)),
            consumer: Arc::new(AtomicRefCell::new(consumer)),
            
        }
    }
}

struct ParametricEQ {
    buffer: Vec<f32>,
}

impl Plugin for ParametricEQ {
    const NAME: &'static str = "Parametric EQ";
    const PRODUCT: &'static str = "Parametric EQ";
    const VENDOR: &'static str = "RustyDAW";

    const INPUT_CHANNELS: usize = 2;
    const OUTPUT_CHANNELS: usize = 2;

    type Model = ParametricEQModel;
    type PluginContext = ParametricEQShared;

    #[inline]
    fn new(_sample_rate: f32, _model: &ParametricEQModel, shared: &ParametricEQShared) -> Self {
        Self {
            buffer: Vec::with_capacity(1024),
        }
    }

    #[inline]
    fn process(&mut self, model: &ParametricEQModelProcess, ctx: &mut ProcessContext<Self>, shared: &ParametricEQShared) {
        let input = &ctx.inputs[0].buffers;
        let output = &mut ctx.outputs[0].buffers;

        let input_length = input[0].len();

        for i in 0..ctx.nframes {
            // If the buffer is full then copy the samples into the tripple buffer
            if self.buffer.len() == 1024 {

                let mut producer = shared.producer.borrow_mut();

                let input = producer.input_buffer();

                input.clear();

                input.extend(self.buffer.drain(0..self.buffer.len()));

                producer.publish();

                self.buffer.clear();
            }

            self.buffer.push(input[0][i]);
        }


        for i in 0..ctx.nframes {
            output[0][i] = input[0][i] * model.out_gain[i];
            output[1][i] = input[1][i] * model.out_gain[i];
        }
    }
}

impl baseplug::PluginUI for ParametricEQ {
    type Handle = Producer<ui::UIHandleMsg>;

    fn ui_size() -> (i16, i16) {
        (800, 600)
    }

    fn ui_open(parent: &impl HasRawWindowHandle, shared: &ParametricEQShared) -> WindowOpenResult<Self::Handle> {
        let (handle_msg_tx, handle_msg_rx) = RingBuffer::new(1024).split();

        ui::build_and_run(handle_msg_rx, parent, shared);

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