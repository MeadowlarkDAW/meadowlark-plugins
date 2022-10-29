use nih_plug::prelude::*;
use std::sync::Arc;

/// This plugin is derived from 
/// https://github.com/robbert-vdh/nih-plug/tree/master/plugins/examples/gain

struct GainPanStereo {
    params: Arc<GainPanStereoParams>,
}

/// The [`Params`] derive macro gathers all of the information needed for the wrapper to know about
/// the plugin's parameters, persistent serializable fields, and nested parameter groups. You can
/// also easily implement [`Params`] by hand if you want to, for instance, have multiple instances
/// of a parameters struct for multiple identical oscillators/filters/envelopes.
#[derive(Params)]
struct GainPanStereoParams {

    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "pan"]
    pub pan: FloatParam,

    #[id = "stereo"]
    pub stereo: FloatParam,

}


impl Default for GainPanStereo {
    fn default() -> Self {
        Self {
            params: Arc::new(GainPanStereoParams::default()),
        }
    }
}

impl Default for GainPanStereoParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                // NOTE: Maybe have this be linear or SymmetricalSkewed
                FloatRange::Skewed { 
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            // Persisted fields can be initialized like any other fields, and they'll keep their
            // values when restoring the plugin's state.

            pan: FloatParam::new(
                "Pan",    
                std::f32::consts::PI / 4.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: std::f32::consts::PI / 2.0,
                },
            )
            // NOTE: Exponential seemed like the better smoother.
            // It's just here for now.
            .with_smoother(SmoothingStyle::Exponential(40.0))
            .with_unit(" radians"),
            // NOTE: Does not read well with current min max and default 
            //.with_value_to_string(formatters::v2s_f32_panning())
            //.with_string_to_value(formatters::s2v_f32_panning()),

            stereo: FloatParam::new(
                "Stereo",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage())

        }
    }
}

impl Plugin for GainPanStereo {
    const NAME: &'static str = "Birdy Gain";
    const VENDOR: &'static str = "Meadowlark";
    const URL: &'static str = "https://youtu.be/dQw4w9WgXcQ";
    const EMAIL: &'static str = "info@example.com";

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    // Setting this to `true` will tell the wrapper to split the buffer up into smaller blocks
    // whenever there are inter-buffer parameter changes. This way no changes to the plugin are
    // required to support sample accurate automation and the wrapper handles all of the boring
    // stuff like making sure transport and other timing information stays consistent between the
    // splits.
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        // This works with any symmetrical IO layout
        config.num_input_channels == config.num_output_channels && config.num_input_channels > 0
    }

    // This plugin doesn't need any special initialization, but if you need to do anything expensive
    // then this would be the place. State is kept around when the host reconfigures the
    // plugin. If we do need special initialization, we could implement the `initialize()` and/or
    // `reset()` methods

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        
        let mut mid: f32;
        let mut side: f32;

        for mut channel_samples in buffer.iter_samples() {
            
            let mut chan_iter_mut = channel_samples.iter_mut(); 
            let leftptr: &mut f32 = chan_iter_mut.next().unwrap();
            let rightptr: &mut f32 = chan_iter_mut.next().unwrap();

            // Stereo widener
            let stereo = self.params.stereo.smoothed.next();
            mid = (*leftptr + *rightptr) * (1.0 - stereo);
            side = (*leftptr - *rightptr) * stereo;
            *leftptr = mid + side;
            *rightptr = mid - side;

            // Panning
            let pan = self.params.pan.smoothed.next();
            *leftptr *= pan.cos();
            *rightptr *= pan.sin();

            // Gain
            let gain = self.params.gain.smoothed.next();
            *leftptr *= gain;
            *rightptr *= gain;
            
        }

        ProcessStatus::Normal
    }

    // This can be used for cleaning up special resources like socket connections whenever the
    // plugin is deactivated. Most plugins won't need to do anything here.
    fn deactivate(&mut self) {}
}

impl ClapPlugin for GainPanStereo {
    const CLAP_ID: &'static str = "com.moist-plugins-gmbh.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A smoothed gain parameter example plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for GainPanStereo {
    const VST3_CLASS_ID: [u8; 16] = *b"MBirdyGainPlugin";
    const VST3_CATEGORIES: &'static str = "Fx|Dynamics";
}

nih_export_clap!(GainPanStereo);
nih_export_vst3!(GainPanStereo);
