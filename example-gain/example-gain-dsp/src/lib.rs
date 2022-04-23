//! The goal here is to separate the pure DSP of a plugin into its own separate crate.
//! This makes it easier to reuse for any plugin format and with any frontend someone
//! may choose.
//!
//! Remember that the goal of this plugin project is **NOT** to create a reusable
//! shared DSP library (I believe that would be more hassle than it is worth). The
//! goal is to have each plugin have their own standalone and optimized DSP. We are
//! however free to reference and copy-paste portions of DSP across plugins as we see
//! fit (as long as the other plugins are also GPLv3).
//!
//! Refer to the DSP Design Doc for more details:
//! https://github.com/MeadowlarkDAW/Meadowlark/blob/main/DSP_DESIGN_DOC.md

// We will use some standard types defined in this crate in this plugin project.
use rusty_daw_core::{
    ParamF32, ParamF32UiHandle, SampleRate, Unit, DEFAULT_DB_GRADIENT, DEFAULT_SMOOTH_SECS,
};

/// Prefer to use block-based processing. For this plugin we will process in block sizes
/// of 128 samples. Based on the plugin, somewhere between 32 samples and 256 samples is
/// usually the most performant. (That may seem small but it's actually the sweet spot
/// to avoid CPU cache misses when doing block-based DSP processing. Always profile!)
pub const MAX_BLOCKSIZE: usize = 128;

pub const MIN_DB: f32 = -90.0;
pub const MAX_DB: f32 = 12.0;

/// This struct will live in the realtime thread.
///
/// Here you may use whatever lock-free method for state synchronization from the
/// "UiHandle" counterpart of the plugin as you see fit, such as lock-free ring buffers
/// or atomics. (In this case we are using the `ParamF32` struct which internally uses an
/// atomic for state syncronization.)
///
/// If you need to send data allocated on the heap to the rt thread, prefer to
/// use the smart pointers in the `basedrop` crate which automatically
/// sends data dropped on the rt thread to a "garbage collection" thread.
pub struct ExampleGainDSP {
    /// The `ParamF32` type in `rusty_daw_core` provides convenient auto-smoothing of
    /// parameters as well as auto-syncing with its `ParamF32UiHandle` counterpart
    /// in the UI thread.
    pub gain: ParamF32<MAX_BLOCKSIZE>,
}

/// This struct will live in the host/UI thread.
///
/// It is meant as a way for the host/UI to manipulate and synchronize state to/from the
/// audio-thread counterpart.
pub struct ExampleGainUiHandle {
    pub gain: ParamF32UiHandle,
}

/// This struct defines a preset for the plugin.
#[derive(Debug, Clone, Copy)]
pub struct ExampleGainPreset {
    // It doesn't matter if you use normalized values for the preset or the actual values
    // for the preset. Just remember to keep it consistent across the plugin.
    pub gain_db: f32,
}

impl ExampleGainDSP {
    /// The constructor of the DSP struct should always return itself along with the
    /// "UIHandle" counterpart that is sent to the UI thread for syncronization.
    pub fn new(
        preset: &ExampleGainPreset,
        sample_rate: SampleRate,
    ) -> (ExampleGainDSP, ExampleGainUiHandle) {
        // Construct a parameter.
        //
        // If your preset uses normalized values, use `ParamF32::from_normalized()` instead.
        let (gain, gain_handle) = ParamF32::from_value(
            // The initial value of this parameter. This will automatically be clamped in the range
            // `[min_db, max_db]` defined below.
            preset.gain_db,
            MIN_DB, // The minimum value of this parameter.
            MAX_DB, // The maximum value of this parameter.
            // The "gradient" defines the mapping curve from the knob/slider presented in
            // the UI to the value of the parameter. This makes some parameters feel more "natural"
            // to use.
            //
            // For example in parameters that deal with decibels we generally want the top
            // end of the knob/slider to make small increments to the db value (i.e a tick going
            // from -1.0db to -1.1db) and the bottom end of the knob/slider to make large increments
            // to the db value (i.e a tick going from -80.0db to -85.0db).
            DEFAULT_DB_GRADIENT,
            // The "unit" defines how the value of the parameter should be presented in the
            // audio-thread code.
            //
            // For parameters that deal with decibels, we generally want to process
            // the value in raw amplitude and not the value in decibels. Setting this to `Unit::Decibels`
            // sets this struct to automatically do this for use.
            //
            // Use `Unit::Generic` if you want the audio thread to recieve the same value as the
            // parameter itself.
            Unit::Decibels,
            // This defines the amount of smoothing this parameter should use. This default value
            // should be fine in most cases.
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

    /// An example of a safe unoptimized version of the DSP.
    ///
    /// Note you should avoid using unsafe and SIMD until the DSP is fully working and it's time
    /// to optimize.
    ///
    /// The caller **MUST** pass in buffers that are all the same length.
    pub fn process(&mut self, in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        let frames = in_l.len();

        // Process in blocks. This is necessary because `ParamF32` has a maximum block size it
        // can operate on.
        let mut frames_processed = 0;
        while frames_processed < frames {
            let frames = (frames - frames_processed).min(MAX_BLOCKSIZE);

            // Retrieve the auto-smoothed output of the parameter.
            let gain = self.gain.smoothed(frames);

            let in_l_part = &in_l[frames_processed..frames_processed + frames];
            let in_r_part = &in_r[frames_processed..frames_processed + frames];
            let out_l_part = &mut out_l[frames_processed..frames_processed + frames];
            let out_r_part = &mut out_r[frames_processed..frames_processed + frames];

            for i in 0..frames {
                out_l_part[i] = in_l_part[i] * gain.values[i];
                out_r_part[i] = in_r_part[i] * gain.values[i];
            }

            frames_processed += frames;
        }
    }

    /// An example of an unsafe and SIMD optimized version of the DSP. (This is overkill for a
    /// simple gain plugin, but I'm provided this as an example for more complex plugins.)
    ///
    /// Note you should avoid using unsafe and SIMD until the DSP is fully working and it's time
    /// to optimize.
    ///
    /// The caller **MUST** pass in buffers that are all the same length.
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx"
    ))]
    pub unsafe fn process_avx(
        &mut self,
        in_l: &[f32],
        in_r: &[f32],
        out_l: &mut [f32],
        out_r: &mut [f32],
    ) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        let total_frames = in_l.len();
        let n_simd_frames = total_frames - (total_frames % 8);

        // Process in blocks. This is necessary because `ParamF32` has a maximum block size it
        // can operate on.
        let mut frames_processed = 0;
        while frames_processed < n_simd_frames {
            let frames = (n_simd_frames - frames_processed).min(MAX_BLOCKSIZE);

            // Retrieve the auto-smoothed output of the parameter.
            let gain = self.gain.smoothed(frames);

            if gain.is_smoothing() {
                for i in (0..frames).step_by(8) {
                    let in_l_v = _mm256_loadu_ps(in_l.as_ptr().add(frames_processed + i));
                    let in_r_v = _mm256_loadu_ps(in_r.as_ptr().add(frames_processed + i));
                    let gain_v = _mm256_loadu_ps(gain.values.as_ptr().add(i));

                    let out_l_v = _mm256_mul_ps(in_l_v, gain_v);
                    let out_r_v = _mm256_mul_ps(in_r_v, gain_v);

                    _mm256_storeu_ps(out_l.as_mut_ptr().add(frames_processed + i), out_l_v);
                    _mm256_storeu_ps(out_r.as_mut_ptr().add(frames_processed + i), out_r_v);
                }
            } else {
                // If gain is not smoothing then we can just load the value once to save SIMD load instructions.
                let gain_v = _mm256_set1_ps(gain.values[0]);

                for i in (0..frames).step_by(8) {
                    let in_l_v = _mm256_loadu_ps(in_l.as_ptr().add(frames_processed + i));
                    let in_r_v = _mm256_loadu_ps(in_r.as_ptr().add(frames_processed + i));

                    let out_l_v = _mm256_mul_ps(in_l_v, gain_v);
                    let out_r_v = _mm256_mul_ps(in_r_v, gain_v);

                    _mm256_storeu_ps(out_l.as_mut_ptr().add(frames_processed + i), out_l_v);
                    _mm256_storeu_ps(out_r.as_mut_ptr().add(frames_processed + i), out_r_v);
                }
            }

            frames_processed += frames;
        }

        // Process remaining elements that don't fit into an SIMD lane.
        if frames_processed < total_frames {
            let frames = total_frames - frames_processed;
            let gain = self.gain.smoothed(frames);

            for i in 0..frames {
                *out_l.get_unchecked_mut(frames_processed + i) =
                    *in_l.get_unchecked(frames_processed + i) * *gain.values.get_unchecked(i);
                *out_r.get_unchecked_mut(frames_processed + i) =
                    *in_r.get_unchecked(frames_processed + i) * *gain.values.get_unchecked(i);
            }
        }
    }
}
