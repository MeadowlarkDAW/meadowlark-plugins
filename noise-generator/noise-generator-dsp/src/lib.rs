/// Algorithms used:
///
/// https://github.com/bdejong/musicdsp/blob/master/source/Synthesis/216-fast-whitenoise-generator.rst
use meadowlark_core_types::{
    ParamBool, ParamBoolHandle, ParamF32, ParamF32Handle, ParamI32, ParamI32Handle, SampleRate,
    Unit, DEFAULT_DB_GRADIENT, DEFAULT_SMOOTH_SECS,
};

static SCALE: f32 = 2.0 / 0xffffffff_u32 as f32;
static X1_SEED: i32 = 0x70f4f854;
static X2_SEED: i32 = -504762201;

const MAX_BLOCKSIZE: usize = 256;

pub const MIN_DB: f32 = -90.0;
pub const MAX_DB: f32 = 6.0;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum NoiseColor {
    White = 0,
    Pink = 1,
    Brown = 2,
}

impl NoiseColor {
    pub const MIN_I32: i32 = 0;
    pub const MAX_I32: i32 = 2;
    pub const DEFAULT_I32: i32 = 0;

    pub fn from_i32(val: i32) -> Result<Self, ()> {
        match val {
            0 => Ok(NoiseColor::White),
            1 => Ok(NoiseColor::Pink),
            2 => Ok(NoiseColor::Brown),
            _ => Err(()),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            NoiseColor::White => 0,
            NoiseColor::Pink => 1,
            NoiseColor::Brown => 2,
        }
    }
}

pub struct NoiseGeneratorPreset {
    pub color: NoiseColor,
    pub gain_db: f32,
    pub bypassed: bool,
}

impl Default for NoiseGeneratorPreset {
    fn default() -> Self {
        Self {
            color: NoiseColor::White,
            gain_db: -12.0,
            bypassed: false,
        }
    }
}

pub struct NoiseGeneratorHandle {
    pub gain: ParamF32Handle,
    pub bypassed: ParamBoolHandle,

    pub color_i32: ParamI32Handle,
}

impl NoiseGeneratorHandle {
    pub fn color(&self) -> NoiseColor {
        NoiseColor::from_i32(self.color_i32.value()).unwrap()
    }

    pub fn set_color(&mut self, color: NoiseColor) {
        self.color_i32.set_value(color.as_i32())
    }

    pub fn set_preset(&mut self, preset: NoiseGeneratorPreset) {
        self.color_i32.set_value(preset.color.as_i32());
        self.gain.set_value(preset.gain_db);
    }

    pub fn get_preset(&self) -> NoiseGeneratorPreset {
        NoiseGeneratorPreset {
            color: self.color(),
            gain_db: self.gain.value(),
            bypassed: self.bypassed.value(),
        }
    }
}

pub struct NoiseGeneratorDSP {
    x1: i32,
    x2: i32,

    pub color: ParamI32,
    pub gain: ParamF32<MAX_BLOCKSIZE>,
    pub bypassed: ParamBool,
}

impl NoiseGeneratorDSP {
    pub fn new(
        preset: NoiseGeneratorPreset,
        sample_rate: SampleRate,
    ) -> (Self, NoiseGeneratorHandle) {
        let (color, color_handle) = ParamI32::from_value(
            preset.color.as_i32(),      // initial value
            NoiseColor::White.as_i32(), // default value
            NoiseColor::MIN_I32,        // min
            NoiseColor::MAX_I32,        // max
        );

        let (gain, gain_handle) = ParamF32::from_value(
            preset.gain_db,      // initial value
            -12.0,               // default value
            MIN_DB,              // min
            MAX_DB,              // max
            DEFAULT_DB_GRADIENT, // gradient
            Unit::Decibels,      // unit
            DEFAULT_SMOOTH_SECS, // smooth_secs
            sample_rate,         // sample_rate
        );

        let (bypassed, bypassed_handle) = ParamBool::from_value(
            preset.bypassed, // initial value
            false,           // default value
        );

        (
            Self {
                x1: X1_SEED,
                x2: X2_SEED,

                color,
                gain,
                bypassed,
            },
            NoiseGeneratorHandle {
                color_i32: color_handle,
                gain: gain_handle,
                bypassed: bypassed_handle,
            },
        )
    }

    pub fn set_sample_rate(&mut self, sample_rate: SampleRate) {
        self.gain.set_sample_rate(sample_rate);
    }

    pub fn set_preset(&mut self, preset: NoiseGeneratorPreset) {
        self.color.set_value(preset.color.as_i32());
        self.gain.set_value(preset.gain_db);
        self.bypassed.set_value(preset.bypassed);
    }

    pub fn get_preset(&self) -> NoiseGeneratorPreset {
        NoiseGeneratorPreset {
            color: NoiseColor::from_i32(self.color.value()).unwrap(),
            gain_db: self.gain.host_get_value(),
            bypassed: self.bypassed.value(),
        }
    }

    pub fn process_stereo(&mut self, frames: usize, out_l: &mut [f32], out_r: &mut [f32]) {
        debug_assert!(out_l.len() >= frames);
        debug_assert!(out_r.len() >= frames);

        if self.bypassed.value() {
            // Always make sure to clear the output buffers you don't use.
            out_l[0..frames].fill(0.0);
            out_r[0..frames].fill(0.0);

            return;
        }

        // Boilerplate to process in blocks. This is necessary because `ParamF32` has a
        // maximum block size it can operate on. Processing in blocks of around 64-256 frames
        // is usually more performant anyway because it minimizes CPU cache misses.
        let mut frames_processed = 0;
        while frames_processed < frames {
            let f = (frames - frames_processed).min(MAX_BLOCKSIZE);

            let out_l_part = &mut out_l[frames_processed..frames_processed + f];
            let out_r_part = &mut out_r[frames_processed..frames_processed + f];

            self.process_block(f, out_l_part, out_r_part);

            frames_processed += f;
        }
    }

    fn process_block(&mut self, frames: usize, out_l: &mut [f32], out_r: &mut [f32]) {
        debug_assert!(frames <= MAX_BLOCKSIZE);
        debug_assert!(out_l.len() >= frames);
        debug_assert!(out_r.len() >= frames);

        // TODO: SIMD optimizations?

        let color = NoiseColor::from_i32(self.color.value()).unwrap();
        let gain = self.gain.smoothed(frames);

        match color {
            NoiseColor::White => {
                for i in 0..frames {
                    self.x1 ^= self.x2;
                    out_l[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);

                    self.x1 ^= self.x2;
                    out_r[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);
                }
            }
            NoiseColor::Pink => {
                for i in 0..frames {
                    self.x1 ^= self.x2;
                    out_l[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);

                    self.x1 ^= self.x2;
                    out_r[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);

                    // TODO: filter
                }
            }
            NoiseColor::Brown => {
                for i in 0..frames {
                    self.x1 ^= self.x2;
                    out_l[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);

                    self.x1 ^= self.x2;
                    out_r[i] = self.x2 as f32 * SCALE * gain[i];
                    self.x2 = self.x2.wrapping_add(self.x1);

                    // TODO: filter
                }
            }
        }
    }

    pub fn can_sleep(&self) -> bool {
        self.bypassed.value()
    }
}
