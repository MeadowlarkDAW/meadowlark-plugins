//! Remember that the goal of this plugin project is **NOT** to create a reusable
//! shared DSP library (I believe that would be more hassle than it is worth). The
//! goal of this plugin project is to simply provide standalone "plugins", each with
//! their own optimized DSP implementation. We are however free to reference and
//! copy-paste portions of DSP across plugins as we see fit (as long as the other
//! plugins are also GPLv3).

#![cfg_attr(feature = "portable-simd", feature(portable_simd))]

#[cfg(feature = "portable-simd")]
use std::simd::{f32x2, LaneCount, Simd, SupportedLaneCount};

use rusty_daw_core::{
    ParamF32, ParamF32UiHandle, SampleRate, Unit, DEFAULT_DB_GRADIENT, DEFAULT_SMOOTH_SECS,
};

/// This struct will live in the realtime thread.
///
/// Here you may use whatever lock-free method for state synchronization from
/// "UiHandle" you need, such as lock-free ring buffers or atomics. (Note that
/// the `ParamF32` struct internally uses an atomic.)
///
/// If you need to send data allocated on the heap to the rt thread, prefer to
/// use the smart pointers in the `basedrop` crate which automatically
/// sends data dropped on the rt thread to a "garbage collection" thread.
pub struct ExampleGainDSP<const MAX_BLOCKSIZE: usize> {
    pub gain: ParamF32<MAX_BLOCKSIZE>,
}

impl<const MAX_BLOCKSIZE: usize> ExampleGainDSP<MAX_BLOCKSIZE> {
    pub fn new(
        // We don't really need to have `min_db` and `max_db` be parameters since this
        // is meant to be a standalone plugin and not a shared DSP library, but the
        // option is here if we want.
        min_db: f32,
        max_db: f32,
        initial_db: f32,
        sample_rate: SampleRate,
    ) -> (ExampleGainDSP<MAX_BLOCKSIZE>, ExampleGainUiHandle) {
        let (gain, gain_handle) = ParamF32::from_value(
            initial_db,
            min_db,
            max_db,
            DEFAULT_DB_GRADIENT,
            Unit::Decibels,
            DEFAULT_SMOOTH_SECS,
            sample_rate,
        );

        (
            ExampleGainDSP { gain },
            ExampleGainUiHandle { gain: gain_handle },
        )
    }

    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
        self.gain.set_sample_rate(sample_rate);
    }

    pub fn reset_buffers(&mut self) {
        self.gain.reset();
    }

    // -- Process methods -----------------------------------------
    //
    // Remember that the goal of this plugin project is **NOT** to create a reusable
    // shared DSP library (I believe that would be more hassle than it is worth). The
    // goal of this plugin project is to simply provide standalone "plugins", each with
    // their own optimized DSP implementation. We are however free to reference and
    // copy-paste portions of DSP across plugins as we see fit (as long as the other
    // plugins are also GPLv3).
    //
    // Here you can add whatever process/process_replacing methods you need. Note that
    // you should prefer to use `process_replacing` if the effect can benefit from it.
    // If you do use a `process_replacing` method, there is no need to create a seperate
    // `non-process-replacing` method (the caller will be responsible for copying buffers
    // if they need that functionality).
    //
    // Even though gain is such a simple effect, this example provides several different
    // methods to more effectly take advantage of SIMD optimizations. This will be more
    // necessary for algorithms that can only process one frame at a time such as filters.
    ///
    /// If a given buffer slice has a length greather than `MAX_BLOCKSIZE`, then the
    /// behavior should be to only process the number of frames up to MAX_BLOCKSIZE.
    //
    // You must also take care to zero-out any unused (non-process-replacing) output
    // buffers.

    /// Process a single channel.
    ///
    /// This is the "fallback"/"auto-vectorized" version.
    ///
    /// Also, if the length of `buf_1` or `buf_2` is greather than `MAX_BLOCKSIZE`, then the
    /// behavior should be to only process the number of frames up to MAX_BLOCKSIZE.
    ///
    /// Please note this will **NOT** work correctly if you try and call this mutliple
    /// times for mutliple mono signals. The internal parameter-smoothing algorithm only works
    /// correctly when this is called once per process cycle.
    pub fn process_replacing_mono_fb(&mut self, buf: &mut [f32]) {
        // This tells the compiler that it is safe to elid all bounds checking on
        // array indexing..
        let frames = buf.len().min(MAX_BLOCKSIZE);

        // Get the auto-smoothed parameter buffers.
        let gain = self.gain.smoothed(frames.into());

        if !gain.is_smoothing() {
            // If the gain parameter is not currently being smoothed (because it has not moved in
            // short while), we can optimize even more here by using a constant gain factor (which
            // will have better SIMD efficiency). Note this kind of optimization may not be
            // necessary, but I'm adding it here as an example.
            let gain = gain[0];

            for i in 0..frames {
                buf[i] *= gain;
            }
        } else {
            for i in 0..frames {
                buf[i] *= gain[i];
            }
        }
    }

    /// Process a stereo channel.
    ///
    /// This is the "fallback"/"auto-vectorized" version.
    ///
    /// Also, if the length of `buf_1` or `buf_2` is greather than `MAX_BLOCKSIZE`, then the
    /// behavior should be to only process the number of frames up to MAX_BLOCKSIZE.
    ///
    /// Please note this will **NOT** work correctly if you try and call this mutliple
    /// times for multiple stereo signals. The internal parameter-smoothing algorithm only works
    /// correctly when this is called once per process cycle.
    pub fn process_replacing_stereo_fb(&mut self, buf_1: &mut [f32], buf_2: &mut [f32]) {
        // This tells the compiler that it is safe to elid all bounds checking on
        // array indexing..
        let frames = buf_1.len().min(buf_2.len()).min(MAX_BLOCKSIZE);

        // Get the auto-smoothed parameter buffers.
        let gain = self.gain.smoothed(frames.into());

        if !gain.is_smoothing() {
            // If the gain parameter is not currently being smoothed (because it has not moved in
            // short while), we can optimize even more here by using a constant gain factor (which
            // will have better SIMD efficiency). Note this kind of optimization may not be
            // necessary, but I'm adding it here as an example.
            let gain = gain[0];

            for i in 0..frames {
                buf_1[i] *= gain;
                buf_2[i] *= gain;
            }
        } else {
            for i in 0..frames {
                buf_1[i] *= gain[i];
                buf_2[i] *= gain[i];
            }
        }
    }

    /// Process a single stereo signal.
    ///
    /// This is the explicit horizontally-optimized SIMD version. (Note that only some algorithms
    /// like gain will work with horizontally-optimized SIMD. Any algorithm that inclues a filter
    /// will not work with this type of optimization).
    ///
    /// Note you should make sure your "fallback"/"auto-vectorized" version works before
    /// attempting to make this optimization.
    ///
    /// Also, if the length of `buf_1` or `buf_2` is greather than `MAX_BLOCKSIZE`, then the
    /// behavior should be to only process the number of frames up to MAX_BLOCKSIZE.
    ///
    /// Please note this will **NOT** work correctly if you try and call this mutliple
    /// times for multiple stereo signals. The internal parameter-smoothing algorithm only works
    /// correctly when this is called once per process cycle.
    #[cfg(feature = "portable-simd")]
    pub fn process_replacing_stereo_h<const LANES: usize>(
        &mut self,
        buf_1: &mut [f32],
        buf_2: &mut [f32],
    ) where
        LaneCount<LANES>: SupportedLaneCount,
    {
        // This tells the compiler that it is safe to elid all bounds checking on
        // array indexing.
        let frames = buf_1.len().min(buf_2.len()).min(MAX_BLOCKSIZE);

        // Get the auto-smoothed parameter buffers.
        let gain = self.gain.smoothed(frames.into());

        if !gain.is_smoothing() {
            // If the gain parameter is not currently being smoothed (because it has not moved in
            // short while), we can optimize even more here by using a constant gain factor (which
            // will have better SIMD efficiency). Note this kind of optimization may not be
            // necessary, but I'm adding it here as an example.
            let gain_v = Simd::<f32, LANES>::splat(gain[0]);

            let mut i = 0;
            while i + LANES <= frames {
                let buf_1_v = Simd::<f32, LANES>::from_slice(&buf_1[i..i + LANES]);
                let buf_2_v = Simd::<f32, LANES>::from_slice(&buf_2[i..i + LANES]);

                let out_1_v = buf_1_v * gain_v;
                let out_2_v = buf_2_v * gain_v;

                buf_1[i..i + LANES].copy_from_slice(out_1_v.as_array());
                buf_2[i..i + LANES].copy_from_slice(out_2_v.as_array());

                i += LANES;
            }

            // Process remaining elements.
            let gain = gain[0];
            for i2 in i..frames {
                buf_1[i2] *= gain;
                buf_2[i2] *= gain;
            }
        } else {
            let mut i = 0;
            while i + LANES <= frames {
                let buf_1_v = Simd::<f32, LANES>::from_slice(&buf_1[i..i + LANES]);
                let buf_2_v = Simd::<f32, LANES>::from_slice(&buf_2[i..i + LANES]);
                let gain_v = Simd::<f32, LANES>::from_slice(&gain[i..i + LANES]);

                let out_1_v = buf_1_v * gain_v;
                let out_2_v = buf_2_v * gain_v;

                buf_1[i..i + LANES].copy_from_slice(out_1_v.as_array());
                buf_2[i..i + LANES].copy_from_slice(out_2_v.as_array());

                i += LANES;
            }

            // Process remaining elements.
            for i2 in i..frames {
                buf_1[i2] *= gain[i2];
                buf_2[i2] *= gain[i2];
            }
        }
    }

    /// Process two channels at a time.
    ///
    /// This is the explicit vertically-optimized SIMD version. (This is not really necessary or
    /// even optimial for gain, but most algorithms that include filters will generally be
    /// optimized this way, so I provide this as an example of how to do that.
    ///
    /// Note you should make sure your "fallback"/"auto-vectorized" version works before
    /// attempting to make this optimization.
    ///
    /// Also, if the length of `buf_1` or `buf_2` is greather than `MAX_BLOCKSIZE`, then the
    /// behavior should be to only process the number of frames up to MAX_BLOCKSIZE.
    ///
    /// Please note this will **NOT** work correctly if you try and call this mutliple
    /// times for multiple stereo signals. The internal parameter-smoothing algorithm only works
    /// correctly when this is called once per process cycle.
    #[cfg(feature = "portable-simd")]
    pub fn process_replacing_stereo_v(&mut self, buf_1: &mut [f32], buf_2: &mut [f32]) {
        // This tells the compiler that it is safe to elid all bounds checking on
        // array indexing.
        let frames = buf_1.len().min(buf_2.len()).min(MAX_BLOCKSIZE);

        // Get the auto-smoothed parameter buffers.
        let gain = self.gain.smoothed(frames.into());

        if !gain.is_smoothing() {
            // If the gain parameter is not currently being smoothed (because it has not moved in
            // short while), we can optimize even more here by using a constant gain factor (which
            // will have better SIMD efficiency). Note this kind of optimization may not be
            // necessary, but I'm adding it here as an example.
            let gain_v = f32x2::splat(gain[0]);

            for i in 0..frames {
                let buf_v = f32x2::from_array([buf_1[i], buf_2[i]]);

                let out_v = buf_v * gain_v;

                let out = out_v.as_array();
                buf_1[i] = out[0];
                buf_2[i] = out[1];
            }
        } else {
            for i in 0..frames {
                let buf_v = f32x2::from_array([buf_1[i], buf_2[i]]);
                let gain_v = f32x2::splat(gain[i]);

                let out_v = buf_v * gain_v;

                let out = out_v.as_array();
                buf_1[i] = out[0];
                buf_2[i] = out[1];
            }
        }
    }
}

/// This struct will live in the host/UI thread. It is meant as a way for the
/// host/UI to manipulate the DSP portion.
///
/// Here you may use whatever lock-free method for state synchronization you need,
/// such as lock-free ring buffers or atomics. (Note that the `ParamF32UiHandle`
/// struct internally uses an atomic.)
///
/// If you need to send data allocated on the heap to the rt thread, prefer to
/// use the smart pointers in the `basedrop` crate which automatically
/// sends data dropped on the rt thread to a "garbage collection" thread.
pub struct ExampleGainUiHandle {
    pub gain: ParamF32UiHandle,
}
