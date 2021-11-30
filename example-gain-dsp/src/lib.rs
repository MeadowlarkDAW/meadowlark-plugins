use rusty_daw_core::{
    block_buffer::StereoBlockBuffer, Gradient, ParamF32, ParamF32Handle, SampleRate, Seconds,
    SmoothOutputF32, Unit,
};

// The time in seconds of the parameter smoothing window (for removing pops
// when moving/automating a paramter).
pub const SMOOTH_SECS: Seconds = Seconds(5.0 / 1_000.0);

/// This handle can be sent to the UI/
pub struct ExmpleGainEffectHandle {
    pub gain_left: ParamF32Handle,
    pub gain_right: ParamF32Handle,
    pub gain_main: ParamF32Handle,
}

pub struct ExampleGainEffect<const MAX_BLOCKSIZE: usize> {
    gain_left: ParamF32<MAX_BLOCKSIZE>,
    gain_right: ParamF32<MAX_BLOCKSIZE>,
    gain_main: ParamF32<MAX_BLOCKSIZE>,
}

pub struct ExampleGainEffectSmoothed<'a, const MAX_BLOCKIZE: usize> {
    pub gain_left: SmoothOutputF32<'a, MAX_BLOCKIZE>,
    pub gain_right: SmoothOutputF32<'a, MAX_BLOCKIZE>,
    pub gain_main: SmoothOutputF32<'a, MAX_BLOCKIZE>,
}

impl<const MAX_BLOCKSIZE: usize> ExampleGainEffect<MAX_BLOCKSIZE> {
    pub const DEFAULT_DB_GRADIENT: Gradient = Gradient::Power(0.15);

    pub fn new(
        min_db: f32,
        max_db: f32,
        left_gain_db: f32,
        right_gain_db: f32,
        main_gain_db: f32,
        sample_rate: SampleRate,
        // The gradient of the parameters. For example here we want
        // a small change to the db value per movement of the knob/slider on
        // the large end (i.e. one tick on the slider goes from 0.0dB to -0.1dB),
        // and a large change to the db value per movement on the other side of the
        // knob/slider (i.e. one tick goes from -85.0dB to -90.0dB).
        db_gradient: Gradient,
    ) -> (Self, ExmpleGainEffectHandle) {
        // Make sure that `MAX_BLOCKIZE` is indeed a power of 2.
        // TODO: static assert?
        assert!(MAX_BLOCKSIZE.is_power_of_two());

        // Construct the parameters

        let (gain_left, gain_left_handle) = ParamF32::from_value(
            left_gain_db,
            min_db,
            max_db,
            db_gradient,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        let (gain_right, gain_right_handle) = ParamF32::from_value(
            right_gain_db,
            min_db,
            max_db,
            db_gradient,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        let (gain_main, gain_main_handle) = ParamF32::from_value(
            main_gain_db,
            min_db,
            max_db,
            db_gradient,
            Unit::Decibels,
            SMOOTH_SECS,
            sample_rate,
        );

        (
            Self {
                gain_left,
                gain_right,
                gain_main,
            },
            ExmpleGainEffectHandle {
                gain_left: gain_left_handle,
                gain_right: gain_right_handle,
                gain_main: gain_main_handle,
            },
        )
    }

    // Self-contained process loop.
    pub fn process(&mut self, frames: usize, buf: &mut StereoBlockBuffer<f32, MAX_BLOCKSIZE>) {
        // This hints to the compiler that it is safe to elid all bounds checking
        // on array indexing.
        let frames = frames.min(MAX_BLOCKSIZE);

        let smoothed = self.smoothed(frames);

        if !smoothed.gain_left.is_smoothing()
            && !smoothed.gain_right.is_smoothing()
            && !smoothed.gain_main.is_smoothing()
        {
            // If nothing is being smoothed then we can optimize by using a constant gain factor
            // (this should auto-vectorize nicely).
            let left_gain = smoothed.gain_left[0] * smoothed.gain_main[0];
            let right_gain = smoothed.gain_right[0] * smoothed.gain_main[0];
            for i in 0..frames {
                buf.left[i] *= left_gain;
                buf.right[i] *= right_gain;
            }
        } else {
            // While this of course is not the most efficient way to vectorize a gain operation,
            // most real-world DSP will operate on a per-frame basis so I'm including this as
            // an example of how to acheive a nice api with packed_simd.
            for i in 0..frames {
                #[cfg(not(feature = "packed-simd"))]
                {
                    let (out_l, out_r) = smoothed.proc_inline(i, buf.left[i], buf.right[i]);
                    buf.left[i] = out_l;
                    buf.right[i] = out_r;
                }

                #[cfg(feature = "packed-simd")]
                {
                    let in_v = packed_simd_2::f32x2::new(buf.left[i], buf.right[i]);
                    let out_v = smoothed.proc_inline_f32x2(i, in_v);
                    buf.left[i] = out_v.extract(0);
                    buf.right[i] = out_v.extract(1);
                }
            }
        }
    }

    pub fn smoothed<'a>(
        &'a mut self,
        frames: usize,
    ) -> ExampleGainEffectSmoothed<'a, MAX_BLOCKSIZE> {
        ExampleGainEffectSmoothed {
            gain_left: self.gain_left.smoothed(frames),
            gain_right: self.gain_right.smoothed(frames),
            gain_main: self.gain_main.smoothed(frames),
        }
    }
}

#[cfg(feature = "packed-simd")]
use packed_simd_2::{f32x2, f32x4, f32x8};

impl<'a, const MAX_BLOCKSIZE: usize> ExampleGainEffectSmoothed<'a, MAX_BLOCKSIZE> {
    /// Single-frame inline version of the function for use within loops.
    #[inline(always)]
    pub fn proc_inline(&self, i: usize, in_left: f32, in_right: f32) -> (f32, f32) {
        // While "gain" is quite trivial, I'm providing this as an example on
        // creating inline methods for more advanced "real-world" DSP.

        (
            in_left * self.gain_left[i] * self.gain_main[i],
            in_right * self.gain_right[i] * self.gain_main[i],
        )
    }

    /// Single-frame inline version of the function for use within loops.
    /// (f32x2 packed vector version).
    #[cfg(feature = "packed-simd")]
    #[inline(always)]
    pub fn proc_inline_f32x2(&self, i: usize, lr_v: f32x2) -> f32x2 {
        // While "gain" is quite trivial, I'm providing this as an example on
        // creating inline methods for more advanced "real-world" DSP.

        let left_gain = self.gain_left[i] * self.gain_main[i];
        let right_gain = self.gain_right[i] * self.gain_main[i];

        let gain_v = f32x2::new(left_gain, right_gain);
        gain_v * lr_v
    }

    /// Single-frame inline version of the function for use within loops.
    /// (f32x4 packed vector version for processing two parallel stereo
    /// channels at once).
    #[cfg(feature = "packed-simd")]
    #[inline(always)]
    pub fn proc_inline_f32x4(&self, i: usize, lr_lr_v: f32x4) -> f32x4 {
        // While "gain" is quite trivial, I'm providing this as an example on
        // creating inline methods for more advanced "real-world" DSP.

        let left_gain = self.gain_left[i] * self.gain_main[i];
        let right_gain = self.gain_right[i] * self.gain_main[i];

        let gain_v = f32x4::new(left_gain, right_gain, left_gain, right_gain);
        gain_v * lr_lr_v
    }

    /// Single-frame inline version of the function for use within loops.
    /// (f32x8 packed vector version for processing four parallel stereo
    /// channels at once).
    #[cfg(feature = "packed-simd")]
    #[inline(always)]
    pub fn proc_inline_f32x8(&self, i: usize, lr_lr_lr_lr_v: f32x8) -> f32x8 {
        // While "gain" is quite trivial, I'm providing this as an example on
        // creating inline methods for more advanced "real-world" DSP.

        let left_gain = self.gain_left[i] * self.gain_main[i];
        let right_gain = self.gain_right[i] * self.gain_main[i];

        let gain_v = f32x8::new(
            left_gain, right_gain, left_gain, right_gain, left_gain, right_gain, left_gain,
            right_gain,
        );
        gain_v * lr_lr_lr_lr_v
    }
}
