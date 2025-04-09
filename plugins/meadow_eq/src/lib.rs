use meadow_eq_dsp::{BandParams, BandType, DEFAULT_Q, EqParams, FilterOrder, MeadowEqDsp};
use nih_plug::prelude::*;
use std::sync::Arc;

const NUM_BANDS: usize = 1;

struct MeadowEq {
    params: Arc<MeadowEqParams>,
    dsp: MeadowEqDsp<NUM_BANDS>,
}

#[derive(Params)]
struct MeadowEqParams {
    #[id = "lp_enabled"]
    pub lp_enabled: BoolParam,
    #[id = "lp_cutoff_hz"]
    pub lp_cutoff_hz: FloatParam,
    #[id = "lp_q"]
    pub lp_q: FloatParam,
    #[id = "lp_order"]
    pub lp_order: IntParam,

    #[id = "hp_enabled"]
    pub hp_enabled: BoolParam,
    #[id = "hp_cutoff_hz"]
    pub hp_cutoff_hz: FloatParam,
    #[id = "hp_q"]
    pub hp_q: FloatParam,
    #[id = "hp_order"]
    pub hp_order: IntParam,

    #[id = "band_1_enabled"]
    pub band_1_enabled: BoolParam,
    #[id = "band_1_type"]
    pub band_1_type: IntParam,
    #[id = "band_1_cutoff_hz"]
    pub band_1_cutoff_hz: FloatParam,
    #[id = "band_1_q"]
    pub band_1_q: FloatParam,
    #[id = "band_1_gain_db"]
    pub band_1_gain_db: FloatParam,
}

impl Default for MeadowEq {
    fn default() -> Self {
        Self {
            params: Arc::new(MeadowEqParams::default()),
            dsp: MeadowEqDsp::new(44_100.0),
        }
    }
}

impl Default for MeadowEqParams {
    fn default() -> Self {
        let cutoff_range = FloatRange::Skewed {
            min: 20.0,
            max: 21_480.0,
            factor: FloatRange::skew_factor(-2.0),
        };
        let q_range_1 = FloatRange::SymmetricalSkewed {
            min: 0.3,
            max: 8.0,
            factor: 0.85,
            center: 1.5,
        };
        let q_range_2 = FloatRange::SymmetricalSkewed {
            min: 0.02,
            max: 40.0,
            factor: 0.85,
            center: 2.5,
        };

        Self {
            lp_enabled: BoolParam::new("LP enabled", false),
            lp_cutoff_hz: FloatParam::new("LP cutoff", 21_480.0, cutoff_range.clone()),
            lp_q: FloatParam::new("LP Q", DEFAULT_Q, q_range_1.clone()),
            lp_order: IntParam::new("LP order", 1, IntRange::Linear { min: 0, max: 4 })
                .with_value_to_string(Arc::new(|v| match v {
                    0 => String::from("x1"),
                    1 => String::from("x2"),
                    2 => String::from("x4"),
                    3 => String::from("x6"),
                    _ => String::from("x8"),
                })),

            hp_enabled: BoolParam::new("HP enabled", false),
            hp_cutoff_hz: FloatParam::new("HP cutoff", 20.0, cutoff_range.clone()),
            hp_q: FloatParam::new("HP Q", DEFAULT_Q, q_range_1.clone()),
            hp_order: IntParam::new("HP order", 1, IntRange::Linear { min: 0, max: 4 })
                .with_value_to_string(Arc::new(|v| match v {
                    0 => String::from("x1"),
                    1 => String::from("x2"),
                    2 => String::from("x4"),
                    3 => String::from("x6"),
                    _ => String::from("x8"),
                })),

            band_1_enabled: BoolParam::new("Band 1 enabled", false),
            band_1_type: IntParam::new("Band 1 type", 0, IntRange::Linear { min: 0, max: 4 })
                .with_value_to_string(Arc::new(|v| match v {
                    0 => String::from("bell"),
                    1 => String::from("low shelf"),
                    2 => String::from("high shelf"),
                    3 => String::from("notch"),
                    _ => String::from("allpass"),
                })),
            band_1_cutoff_hz: FloatParam::new("Band 1 cutoff", 1000.0, cutoff_range.clone()),
            band_1_q: FloatParam::new("Band 1 Q", DEFAULT_Q, q_range_2.clone()),
            band_1_gain_db: FloatParam::new(
                "Band 1 Gain",
                0.0,
                FloatRange::SymmetricalSkewed {
                    min: -30.0,
                    max: 30.0,
                    factor: 0.4,
                    center: 0.0,
                },
            )
            .with_unit(" dB"),
        }
    }
}

impl Plugin for MeadowEq {
    const NAME: &'static str = "Meadow Eq";
    const VENDOR: &'static str = "Billy Messenger";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "60663878+BillyDM@users.noreply.github.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.dsp = MeadowEqDsp::new(config.sample_rate as f64);

        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let params = EqParams {
            lp_enabled: self.params.lp_enabled.value(),
            lp_cutoff_hz: self.params.lp_cutoff_hz.value(),
            lp_q: self.params.lp_q.value(),
            lp_order: FilterOrder::from_u32(self.params.lp_order.value() as u32),

            hp_enabled: self.params.hp_enabled.value(),
            hp_cutoff_hz: self.params.hp_cutoff_hz.value(),
            hp_q: self.params.hp_q.value(),
            hp_order: FilterOrder::from_u32(self.params.hp_order.value() as u32),

            bands: [BandParams {
                enabled: self.params.band_1_enabled.value(),
                band_type: BandType::from_u32(self.params.band_1_type.value() as u32),
                cutoff_hz: self.params.band_1_cutoff_hz.value(),
                q: self.params.band_1_q.value(),
                gain_db: self.params.band_1_gain_db.value(),
            }],
        };

        self.dsp.set_params(params);

        let out = buffer.as_slice();
        let (out_l, out_r) = out.split_first_mut().unwrap();

        self.dsp.process(out_l, out_r[0]);

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MeadowEq {
    const CLAP_ID: &'static str = "app.meadowlark.meadow-eq";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A high quality open source parametric EQ");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MeadowEq {
    const VST3_CLASS_ID: [u8; 16] = *b"Meadowlark.ParEQ";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(MeadowEq);
nih_export_vst3!(MeadowEq);
