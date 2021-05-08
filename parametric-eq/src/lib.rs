#[macro_use]
extern crate vst;

pub mod eq_core;
mod atomic_f64;
mod parameter;
mod eq_params;
pub mod editor;
pub mod util;

use editor::{EQPluginEditor};
use eq_core::eq::FilterbandStereo;
use eq_params::{BandParameters, EQEffectParameters};

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Info, Plugin, PluginParameters};

use std::sync::Arc;

use atomic_f64::AtomicF64;

const FILTER_COUNT: usize = 4;
const FILTER_POLE_COUNT: usize = 16;

struct EQPlugin {
    params: Arc<EQEffectParameters>,
    sample_rate: Arc<AtomicF64>,    
    editor: Option<EQPluginEditor>,
    filter_bands: Vec<FilterbandStereo>,

}

impl Default for EQPlugin {
    fn default() -> Self {
        let params = Arc::new(EQEffectParameters::default());
        let sample_rate = Arc::new(AtomicF64::new(44100.0));

        let filter_bands = (0..FILTER_COUNT)
            .map(|_| FilterbandStereo::new(48000.0))
            .collect::<Vec<FilterbandStereo>>();

        Self {
            params: params.clone(),
            sample_rate: sample_rate.clone(),
            editor: Some(EQPluginEditor {
                is_open: false,
                params: params.clone(),
                sample_rate: sample_rate.clone(),
            }),
            filter_bands,
        }
    }
}

fn setup_logging() {
    let log_folder = ::dirs::home_dir().unwrap().join("tmp");

    let _ = ::std::fs::create_dir(log_folder.clone());

    let log_file = ::std::fs::File::create(log_folder.join("IMGUIBaseviewEQ.log")).unwrap();

    let log_config = ::simplelog::ConfigBuilder::new()
        .set_time_to_local(true)
        .build();

    let _ = ::simplelog::WriteLogger::init(simplelog::LevelFilter::max(), log_config, log_file);

    ::log_panics::init();

    ::log::info!("init");
}

impl Plugin for EQPlugin {
    fn get_info(&self) -> Info {
        Info {
            name: "RustyDaw EQ".to_string(),
            vendor: "RustyDaw".to_string(),
            unique_id: 1143483637,
            version: 1,
            inputs: 2,
            outputs: 2,
            // This `parameters` bit is important; without it, none of our
            // parameters will be shown!
            parameters: self.params.len() as i32,
            category: Category::Effect,
            ..Default::default()
        }
    }

    fn init(&mut self) {
        setup_logging();
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn Editor>)
        } else {
            None
        }
    }

    // Here is where the bulk of our audio processing code goes.
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {

        let (input_buffer, output_buffer) = buffer.split();
        let (input_buffer_left, input_buffer_right) = input_buffer.split_at(1);
        let (mut output_buffer_left, mut output_buffer_right) = output_buffer.split_at_mut(1);

        let inputs_stereo = input_buffer_left[0].iter().zip(input_buffer_right[0].iter());
        let outputs_stereo = output_buffer_left[0].iter_mut().zip(output_buffer_right[0].iter_mut());

        for (input_pair, output_pair) in inputs_stereo.zip(outputs_stereo) {
            
            for (i, band) in self.params.bands.iter().enumerate() {
                self.filter_bands[i].update(
                    band.get_kind(),
                    band.freq.get(),
                    band.gain.get(),
                    band.bw.get(),
                    band.get_slope(),
                    self.sample_rate.get(),
                );
            }
            
            let (input_l, input_r) = input_pair;
            let (output_l, output_r) = output_pair;

            let mut l = *input_l as f64;
            let mut r = *input_r as f64;

            for i in 0..self.filter_bands.len() {
                let [l_n, r_n] = self.filter_bands[i].process(l, r);
                l = l_n;
                r = r_n;
            }

            *output_l = l as f32;
            *output_r = r as f32;
        }
    }

    // Return the parameter object. This method can be omitted if the
    // plugin has no parameters.
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.params) as Arc<dyn PluginParameters>
    }
}

plugin_main!(EQPlugin);
