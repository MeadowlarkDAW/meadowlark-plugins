//! The goal here is to separate the pure DSP of a plugin into its own separate crate.
//! This makes it easier to reuse for any plugin format with any frontend someone may
//! choose.
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
const MAX_BLOCKSIZE: usize = 128;

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
    /// 
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
        // We don't really need to have `min_db` and `max_db` be parameters since this
        // is meant to be a standalone plugin and not a shared DSP library, but the
        // option is here if we want.
        min_db: f32,
        max_db: f32,
        sample_rate: SampleRate,
    ) -> (ExampleGainDSP, ExampleGainUiHandle) {
        // Construct a parameter.
        //
        // If your preset uses normalized values, use `ParamF32::from_normalized()` instead.
        let (gain, gain_handle) = ParamF32::from_value(
            // The initial value of this parameter. This will automatically be clamped in the range
            // `[min_db, max_db]` defined below.
            preset.gain_db,
            min_db,  // The minimum value of this parameter.
            max_db,  // The maximum value of this parameter.
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

    /// The process method.
    /// 
    /// We mark this as "unsafe" because we are stipulating that the caller **MUST** pass in buffers that
    /// are all at-least the length of `frames`, and that it's considered undefined behavior if they don't.
    /// Note the caller is allowed to send slices that are longer than `frames`.
    pub unsafe fn process_stereo(&mut self, frames: usize, in_l: &[f32], in_r: &[f32], out_l: &mut [f32], out_r: &mut [f32]) {
        // Process in blocks. This is probably not necessary for something as simple as a gain plugin, but
        // I'm doing it this way as an example.

        self.gain.smoothed(frames)

    }
}


