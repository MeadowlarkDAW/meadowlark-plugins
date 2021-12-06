#![cfg_attr(feature = "portable-simd", feature(portable_simd))]

#[cfg(feature = "portable-simd")]
use std::simd::f32x2;

use rusty_daw_core::{
    ParamF32, ParamF32UiHandle, SampleRate, Unit, DEFAULT_DB_GRADIENT, DEFAULT_SMOOTH_SECS,
};

pub struct ExampleGainDSP<const MAX_BLOCKSIZE: usize> {
    pub gain_left: ParamF32<MAX_BLOCKSIZE>,
    pub gain_right: ParamF32<MAX_BLOCKSIZE>,
    pub gain_main: ParamF32<MAX_BLOCKSIZE>,
}

impl<const MAX_BLOCKSIZE: usize> ExampleGainDSP<MAX_BLOCKSIZE> {
    pub fn new(
        min_db: f32,
        max_db: f32,
        initial_gain_left_db: f32,
        initial_gain_right_db: f32,
        initial_gain_main_db: f32,
        sample_rate: SampleRate,
    ) -> (ExampleGainDSP<MAX_BLOCKSIZE>, ExampleGainUiHandle) {
        let (gain_left, gain_left_handle) = ParamF32::from_value(
            initial_gain_left_db,
            min_db,
            max_db,
            DEFAULT_DB_GRADIENT,
            Unit::Decibels,
            DEFAULT_SMOOTH_SECS,
            sample_rate,
        );
        let (gain_right, gain_right_handle) = ParamF32::from_value(
            initial_gain_right_db,
            min_db,
            max_db,
            DEFAULT_DB_GRADIENT,
            Unit::Decibels,
            DEFAULT_SMOOTH_SECS,
            sample_rate,
        );
        let (gain_main, gain_main_handle) = ParamF32::from_value(
            initial_gain_main_db,
            min_db,
            max_db,
            DEFAULT_DB_GRADIENT,
            Unit::Decibels,
            DEFAULT_SMOOTH_SECS,
            sample_rate,
        );

        (
            ExampleGainDSP {
                gain_left,
                gain_right,
                gain_main,
            },
            ExampleGainUiHandle {
                gain_left: gain_left_handle,
                gain_right: gain_right_handle,
                gain_main: gain_main_handle,
            },
        )
    }

    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
        self.gain_left.set_sample_rate(sample_rate);
        self.gain_right.set_sample_rate(sample_rate);
        self.gain_main.set_sample_rate(sample_rate);
    }

    pub fn reset_buffers(&mut self) {
        self.gain_left.reset();
        self.gain_right.reset();
        self.gain_main.reset();
    }

    /// Process the given buffers.
    ///
    /// Note that only a maximum of `MAX_BLOCKSIZE` frames will be processed, even if
    /// `buf_left` or `buf_right` has a length greater than `MAX_BLOCKSIZE`.
    pub fn process_replacing(&mut self, buf_left: &mut [f32], buf_right: &mut [f32]) {
        // This tells the compiler that it is safe to elid all bounds checking on
        // array indexing.
        let frames = buf_left.len().min(buf_right.len()).min(MAX_BLOCKSIZE);

        // Get the smoothed parameter buffers.
        let gain_left = self.gain_left.smoothed(frames);
        let gain_right = self.gain_right.smoothed(frames);
        let gain_main = self.gain_main.smoothed(frames);

        if (!gain_left.is_smoothing() && !gain_right.is_smoothing() && !gain_main.is_smoothing())
            || false
        {
            // If nothing is being smoothed then we can optimize by using a constant gain factor
            // (this should auto-vectorize nicely).
            let left_gain = gain_left[0] * gain_main[0];
            let right_gain = gain_right[0] * gain_main[0];
            for i in 0..frames {
                buf_left[i] *= left_gain;
                buf_right[i] *= right_gain;
            }
        } else {
            #[cfg(not(feature = "portable-simd"))]
            {
                for i in 0..frames {
                    buf_left[i] *= gain_left[i] * gain_main[i];
                    buf_right[i] *= gain_right[i] * gain_main[i];
                }
            }
            #[cfg(feature = "portable-simd")]
            {
                // While this of course is not the most efficient way to vectorize a gain operation,
                // most real-world DSP will operate on a per-frame basis so I'm including this as
                // an example of how portable-simd will typically be used.
                for i in 0..frames {
                    let in_lr = f32x2::from_array([buf_left[i], buf_right[i]]);
                    let gain_lr = f32x2::from_array([gain_left[i], gain_right[i]]);
                    let gain_main = f32x2::splat(gain_main[i]);

                    let out_lr = in_lr * gain_lr * gain_main;

                    let out_lr = out_lr.to_array();
                    buf_left[i] = out_lr[0];
                    buf_right[i] = out_lr[1];
                }
            }
        }
    }
}

pub struct ExampleGainUiHandle {
    pub gain_left: ParamF32UiHandle,
    pub gain_right: ParamF32UiHandle,
    pub gain_main: ParamF32UiHandle,
}
