use crate::FILTER_POLE_COUNT;

use super::eq_core::eq::FilterKind;
use super::parameter::Parameter;

use vst::plugin::{PluginParameters};


pub struct BandParameters {
    pub kind: Parameter,
    pub freq: Parameter,
    pub gain: Parameter,
    pub bw: Parameter,
    pub slope: Parameter,
}

impl BandParameters {
    pub fn get_kind(&self) -> FilterKind {
        return FilterKind::from_u32(self.kind.get() as u32);
    }

    pub fn get_slope(&self) -> f64 {
        let u_slope = self.slope.get() as u32;
        if !u_slope % 2 == 0 {
            //Only supporting even currently
            (u_slope + 1) as f64
        } else {
            u_slope as f64
        }
    }
}

pub struct EQEffectParameters {
    pub bands: Vec<Arc<BandParameters>>,
}

use std::{ops::Index, sync::Arc};

impl Index<usize> for EQEffectParameters {
    type Output = Parameter;
    fn index(&self, i: usize) -> &Self::Output {
        match i {
            0 => &self.bands[0].kind,
            1 => &self.bands[0].freq,
            2 => &self.bands[0].gain,
            3 => &self.bands[0].bw,
            4 => &self.bands[0].slope,
            5 => &self.bands[1].kind,
            6 => &self.bands[1].freq,
            7 => &self.bands[1].gain,
            8 => &self.bands[1].bw,
            9 => &self.bands[1].slope,
            10 => &self.bands[2].kind,
            11 => &self.bands[2].freq,
            12 => &self.bands[2].gain,
            13 => &self.bands[2].bw,
            14 => &self.bands[2].slope,
            15 => &self.bands[3].kind,
            16 => &self.bands[3].freq,
            17 => &self.bands[3].gain,
            18 => &self.bands[3].bw,
            19 => &self.bands[3].slope,
            _ => &self.bands[3].kind,
        }
    }
}

impl EQEffectParameters {
    pub fn len(&self) -> usize {
        16
    }
}

fn new_band_pram_set(n: usize) -> BandParameters {
    BandParameters {
        kind: Parameter::new(
            &format!("Band {} Type", n),
            1.0,
            1.0,
            10.0,
            |x| format!("Type {:.2}", x),
            |x| x,
            |x| x,
        ),
        freq: Parameter::new(
            &format!("Band {} hz", n),
            1000.0,
            20.0,
            20000.0,
            |x| format!("hz {:.2}", x),
            |x| x.powf(4.0),
            |x| x.powf(0.25),
        ),
        gain: Parameter::new(
            &format!("Band {} dB", n),
            0.0,
            -12.0,
            12.0,
            |x| format!("dB {:.2}", x),
            |x| x,
            |x| x,
        ),
        bw: Parameter::new(
            &format!("Band {} BW", n),
            1.0,
            0.1,
            24.0,
            |x| format!("BW {:.2}", x),
            |x| x,
            |x| x,
        ),
        slope: Parameter::new(
            &format!("Band {} Slope", n),
            1.0,
            1.0,
            FILTER_POLE_COUNT as f64,
            |x| format!("Slope {:.2}", x),
            |x| x,
            |x| x,
        ),
    }
}

impl Default for EQEffectParameters {
    fn default() -> EQEffectParameters {
        EQEffectParameters {
            bands: (0..4)
                .map(|_| Arc::new(new_band_pram_set(1)))
                .collect::<Vec<Arc<BandParameters>>>(),
        }
    }
}

impl PluginParameters for EQEffectParameters {
    // the `get_parameter` function reads the value of a parameter.
    fn get_parameter(&self, index: i32) -> f32 {
        if (index as usize) < self.len() {
            self[index as usize].get_normalized() as f32
        } else {
            0.0
        }
    }

    // the `set_parameter` function sets the value of a parameter.
    fn set_parameter(&self, index: i32, val: f32) {
        #[allow(clippy::single_match)]
        if (index as usize) < self.len() {
            self[index as usize].set_normalized(val as f64);
        }
    }

    // This is what will display underneath our control.  We can
    // format it into a string that makes the most since.

    fn get_parameter_text(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_display()
        } else {
            "".to_string()
        }
    }

    // This shows the control's name.
    fn get_parameter_name(&self, index: i32) -> String {
        if (index as usize) < self.len() {
            self[index as usize].get_name()
        } else {
            "".to_string()
        }
    }
}