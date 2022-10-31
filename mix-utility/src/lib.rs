use nih_plug::prelude::*;
use std::sync::Arc;

/// This plugin is derived from 
/// https://github.com/robbert-vdh/nih-plug/tree/master/plugins/examples/gain

struct MixUtility {
    params: Arc<MixUtilityParams>,
}

#[derive(Params)]
struct MixUtilityParams {

    #[id = "left_polarity"]
    pub left_polarity: BoolParam,

    #[id = "right_polarity"]
    pub right_polarity: BoolParam,

    #[id = "stereo"]
    pub stereo: FloatParam,

    #[id = "pan"]
    pub pan: FloatParam,

    #[id = "gain"]
    pub gain: FloatParam,

}


impl Default for MixUtility {
    fn default() -> Self {
        Self {
            params: Arc::new(MixUtilityParams::default()),
        }
    }
}

impl Default for MixUtilityParams {
    fn default() -> Self {
        Self {

            left_polarity: BoolParam::new(
                "Left Polarity",
                false,
            ),

            right_polarity: BoolParam::new(
                "Right Polarity",
                false,
            ),

            stereo: FloatParam::new(
                "Stereo",
                0.5,
                FloatRange::Linear {
                    min: 0.0, // Full Mid
                    max: 1.0, // Full Stereo
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_value_to_string(formatters::v2s_f32_percentage(2))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            pan: FloatParam::new(
                "Pan",    
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_smoother(SmoothingStyle::Exponential(50.0))
            .with_value_to_string(formatters::v2s_f32_panning())
            .with_string_to_value(formatters::s2v_f32_panning()),


            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed { 
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    // This makes the range appear as if it was linear when displaying the values as
                    // decibels
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),


        }
    }
}

impl Plugin for MixUtility {
    const NAME: &'static str = "Mix Utility";
    const VENDOR: &'static str = "Meadowlark";
    const URL: &'static str = "https://github.com/MeadowlarkDAW/meadowlark-plugins.git";
    const EMAIL: &'static str = "vibeniumproductions@gmail.com"; // For now

    const VERSION: &'static str = "0.0.1";

    const DEFAULT_INPUT_CHANNELS: u32 = 2;
    const DEFAULT_OUTPUT_CHANNELS: u32 = 2;

    const DEFAULT_AUX_INPUTS: Option<AuxiliaryIOConfig> = None;
    const DEFAULT_AUX_OUTPUTS: Option<AuxiliaryIOConfig> = None;

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn accepts_bus_config(&self, config: &BusConfig) -> bool {
        config.num_input_channels == config.num_output_channels 
            && config.num_input_channels > 0
    }


    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        
        let mut left_polarity_param_value: f32;
        let mut right_polarity_param_value: f32;
        let mut mid: f32;
        let mut side: f32;
        let mut stereo_param_value: f32;
        let mut pan_param_value: f32;
        let mut theta: f32;
        let mut gain_param_value: f32;

        for mut channel_samples in buffer.iter_samples() {
            
            // Setting up
            let mut chan_iter_mut = channel_samples.iter_mut(); 
            let left_ptr: &mut f32 = chan_iter_mut.next().unwrap();
            let right_ptr: &mut f32 = chan_iter_mut.next().unwrap();

            // Polarity
            left_polarity_param_value 
                = self.params.left_polarity.normalized_value();
            right_polarity_param_value
                = self.params.right_polarity.normalized_value();
            *left_ptr *= -2.0 * left_polarity_param_value + 1.0;
            *right_ptr *= -2.0 * right_polarity_param_value + 1.0;

            // Stereo widener
            stereo_param_value = self.params.stereo.smoothed.next();
            // Mid = ((Left + Right) / 2) and Side = ((Left - Right) / 2)
            // Since default value is 0.5 and is between 0 and 1,
            mid = (*left_ptr + *right_ptr) * (1.0 - stereo_param_value);
            side = (*left_ptr - *right_ptr) * stereo_param_value;
            *left_ptr = mid + side;
            *right_ptr = mid - side;

            // Panning (Constant Power)
            pan_param_value = self.params.pan.smoothed.next();
            // Interpolate the given pan_param_value
            theta = std::f32::consts::PI / 4.0 * (pan_param_value + 1.0);
            *left_ptr *= theta.cos();
            *right_ptr *= theta.sin();

            // Gain
            gain_param_value = self.params.gain.smoothed.next();
            *left_ptr *= gain_param_value;
            *right_ptr *= gain_param_value;
            
        }

        ProcessStatus::Normal
    }

    fn deactivate(&mut self) {}
}

impl ClapPlugin for MixUtility {
    const CLAP_ID: &'static str = "com.meadowlark-plugins.gain";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("mixing utility");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Stereo,
        ClapFeature::Mono,
        ClapFeature::Utility,
    ];
}

impl Vst3Plugin for MixUtility {
    const VST3_CLASS_ID: [u8; 16] = *b"MeadowMixUtility";
    const VST3_CATEGORIES: &'static str = "mixing utility";
}

nih_export_clap!(MixUtility);
nih_export_vst3!(MixUtility);
