#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(min_specialization)]

use atomic_refcell::AtomicRefCell;
use baseplug::{Plugin, PluginContext, ProcessContext, WindowOpenResult};
use raw_window_handle::HasRawWindowHandle;
use rtrb::{Consumer, Producer, RingBuffer};
use serde::{Deserialize, Serialize};
use triple_buffer::{Input, Output, TripleBuffer};

use std::cell::UnsafeCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
mod ui;

mod eq_core;
use eq_core::{
    eq::{get_slope, FilterKind, FilterbandStereo},
    svf,
};

use eq_core::svf::SVFCoefficients;

const FILTER_COUNT: usize = 4;
const FILTER_POLE_COUNT: usize = 16;

baseplug::model! {
    #[derive(Debug, Serialize, Deserialize)]
    struct ParametricEQModel {
        #[model(min = 1.0, max = 10.0)]
        #[parameter(name = "band 1 kind", unit = "Generic",
            gradient = "Linear")]
        band_1_kind: f32,

        #[model(min = 20.0, max = 20000.0)]
        #[parameter(name = "band 1 freq", unit = "Generic",
            gradient = "Power(10.0)")]
        band_1_freq: f32,

        #[model(min = -24.0, max = 24.0)]
        #[parameter(name = "band 1 gain", unit = "Generic",
            gradient = "Linear")]
        band_1_gain: f32,

        #[model(min = 0.1, max = 24.0)]
        #[parameter(name = "band 1 bw", unit = "Generic",
            gradient = "Power(10.0)")]
        band_1_bw: f32,

        #[model(min = 1.0, max = 16.0)]
        #[parameter(name = "band 1 slope", unit = "Generic",
            gradient = "Linear")]
        band_1_slope: f32,

        #[model(min = 1.0, max = 10.0)]
        #[parameter(name = "band 2 kind", unit = "Generic",
            gradient = "Linear")]
        band_2_kind: f32,

        #[model(min = 20.0, max = 20000.0)]
        #[parameter(name = "band 2 freq", unit = "Generic",
            gradient = "Power(10.0)")]
        band_2_freq: f32,

        #[model(min = -24.0, max = 24.0)]
        #[parameter(name = "band 2 gain", unit = "Generic",
            gradient = "Linear")]
        band_2_gain: f32,

        #[model(min = 0.1, max = 24.0)]
        #[parameter(name = "band 2 bw", unit = "Generic",
            gradient = "Power(10.0)")]
        band_2_bw: f32,

        #[model(min = 1.0, max = 16.0)]
        #[parameter(name = "band 2 slope", unit = "Generic",
            gradient = "Linear")]
        band_2_slope: f32,

        #[model(min = 1.0, max = 10.0)]
        #[parameter(name = "band 3 kind", unit = "Generic",
            gradient = "Linear")]
        band_3_kind: f32,

        #[model(min = 20.0, max = 20000.0)]
        #[parameter(name = "band 3 freq", unit = "Generic",
            gradient = "Power(10.0)")]
        band_3_freq: f32,

        #[model(min = -24.0, max = 24.0)]
        #[parameter(name = "band 3 gain", unit = "Generic",
            gradient = "Linear")]
        band_3_gain: f32,

        #[model(min = 0.1, max = 24.0)]
        #[parameter(name = "band 3 bw", unit = "Generic",
            gradient = "Power(10.0)")]
        band_3_bw: f32,

        #[model(min = 1.0, max = 16.0)]
        #[parameter(name = "band 3 slope", unit = "Generic",
            gradient = "Linear")]
        band_3_slope: f32,

        #[model(min = 1.0, max = 10.0)]
        #[parameter(name = "band 4 kind", unit = "Generic",
            gradient = "Linear")]
        band_4_kind: f32,

        #[model(min = 20.0, max = 20000.0)]
        #[parameter(name = "band 4 freq", unit = "Generic",
            gradient = "Power(10.0)")]
        band_4_freq: f32,

        #[model(min = -24.0, max = 24.0)]
        #[parameter(name = "band 4 gain", unit = "Generic",
            gradient = "Linear")]
        band_4_gain: f32,

        #[model(min = 0.1, max = 24.0)]
        #[parameter(name = "band 4 bw", unit = "Generic",
            gradient = "Power(10.0)")]
        band_4_bw: f32,

        #[model(min = 1.0, max = 16.0)]
        #[parameter(name = "band 4 slope", unit = "Generic",
            gradient = "Linear")]
        band_4_slope: f32,

    }
}

impl Default for ParametricEQModel {
    fn default() -> Self {
        Self {
            band_1_kind: 1.0,
            band_1_freq: 1000.0,
            band_1_gain: 0.0,
            band_1_bw: 1.0,
            band_1_slope: 1.0,
            band_2_kind: 1.0,
            band_2_freq: 1000.0,
            band_2_gain: 0.0,
            band_2_bw: 1.0,
            band_2_slope: 1.0,
            band_3_kind: 1.0,
            band_3_freq: 1000.0,
            band_3_gain: 0.0,
            band_3_bw: 1.0,
            band_3_slope: 1.0,
            band_4_kind: 1.0,
            band_4_freq: 1000.0,
            band_4_gain: 0.0,
            band_4_bw: 1.0,
            band_4_slope: 1.0,
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
    filter_bands: Vec<FilterbandStereo>,
    sample_rate: f32,
}

fn update_eq_params(
    model: &ParametricEQModelProcess,
    filters: &mut Vec<FilterbandStereo>,
    i: usize,
    sample_rate: f32,
) {
    let sample_rate = sample_rate as f64;
    filters[0].update(
        FilterKind::from_f32(model.band_1_kind[i]),
        model.band_1_freq[i] as f64,
        model.band_1_gain[i] as f64,
        model.band_1_bw[i] as f64,
        get_slope(model.band_1_slope[i]),
        sample_rate,
    );
    filters[1].update(
        FilterKind::from_f32(model.band_2_kind[i]),
        model.band_2_freq[i] as f64,
        model.band_2_gain[i] as f64,
        model.band_2_bw[i] as f64,
        get_slope(model.band_2_slope[i]),
        sample_rate,
    );
    filters[2].update(
        FilterKind::from_f32(model.band_3_kind[i]),
        model.band_3_freq[i] as f64,
        model.band_3_gain[i] as f64,
        model.band_3_bw[i] as f64,
        get_slope(model.band_3_slope[i]),
        sample_rate,
    );
    filters[3].update(
        FilterKind::from_f32(model.band_4_kind[i]),
        model.band_4_freq[i] as f64,
        model.band_4_gain[i] as f64,
        model.band_4_bw[i] as f64,
        get_slope(model.band_4_slope[i]),
        sample_rate,
    );
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
    fn new(sample_rate: f32, _model: &ParametricEQModel, shared: &ParametricEQShared) -> Self {
        let filter_bands = (0..FILTER_COUNT)
            .map(|_| FilterbandStereo::new(48000.0))
            .collect::<Vec<FilterbandStereo>>();
        Self {
            buffer: Vec::with_capacity(1024),
            filter_bands,
            sample_rate,
        }
    }

    #[inline]
    fn process(
        &mut self,
        model: &ParametricEQModelProcess,
        ctx: &mut ProcessContext<Self>,
        shared: &ParametricEQShared,
    ) {
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
            //output[0][i] = input[0][i] * model.out_gain[i];
            //output[1][i] = input[1][i] * model.out_gain[i];

            update_eq_params(model, &mut self.filter_bands, i, self.sample_rate);

            let mut l = input[0][i] as f64;
            let mut r = input[1][i] as f64;

            for i in 0..self.filter_bands.len() {
                let [l_n, r_n] = self.filter_bands[i].process(l, r);
                l = l_n;
                r = r_n;
            }

            output[0][i] = l as f32;
            output[1][i] = r as f32;
        }
    }
}

impl baseplug::PluginUI for ParametricEQ {
    type Handle = Producer<ui::UIHandleMsg>;

    fn ui_size() -> (i16, i16) {
        (800, 600)
    }

    fn ui_open(
        parent: &impl HasRawWindowHandle,
        shared: &ParametricEQShared,
    ) -> WindowOpenResult<Self::Handle> {
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
