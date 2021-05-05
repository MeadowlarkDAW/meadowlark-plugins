use core::fmt;

//use crate::eq_core::{
//    svf::{SVFCoefficients, Type, SVF},
//    units::{butterworth_cascade_q, Units},
//};

use num_complex::Complex;

use crate::FILTER_POLE_COUNT;

use super::{
    svf::{SVFCoefficients, Type, SVF},
    units::{butterworth_cascade_q, Units},
};

const Q_BUTTERWORTH_F64: f64 = 0.70710678118654757f64;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FilterKind {
    Bell,
    LowPass,
    HighPass,
    LowShelf,
    HighShelf,
    Notch,
    BandPass,
    Tilt,
    Mesa,
    AllPass,
}

impl FilterKind {
    pub fn from_u32(value: u32) -> FilterKind {
        match value {
            1 => FilterKind::Bell,
            2 => FilterKind::LowPass,
            3 => FilterKind::HighPass,
            4 => FilterKind::LowShelf,
            5 => FilterKind::HighShelf,
            6 => FilterKind::Notch,
            7 => FilterKind::BandPass,
            8 => FilterKind::Tilt,
            9 => FilterKind::Mesa,
            10 => FilterKind::AllPass,
            _ => panic!("Unknown value: {}", value),
        }
    }

    pub fn from_f32(n: f32) -> FilterKind {
        return FilterKind::from_u32(n as u32);
    }
}

impl fmt::Display for FilterKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_slope(slope: f32) -> f64 {
    let u_slope = slope as u32;
    if !u_slope % 2 == 0 {
        //Only supporting even currently
        (u_slope + 1) as f64
    } else {
        u_slope as f64
    }
}

#[derive(Copy, Clone, Debug)]
pub struct SVFCoefficientsSet {
    pub a: [SVFCoefficients<f64>; FILTER_POLE_COUNT],
    pub b: [SVFCoefficients<f64>; FILTER_POLE_COUNT],
}

impl SVFCoefficientsSet {
    pub fn new(sample_rate: f64) -> SVFCoefficientsSet {
        let coeffs =
            SVFCoefficients::<f64>::from_params(Type::PeakingEQ(0.0f64), sample_rate, 1000.0, 1.0)
                .unwrap();
        SVFCoefficientsSet {
            a: [coeffs; FILTER_POLE_COUNT],
            b: [coeffs; FILTER_POLE_COUNT],
        }
    }

    pub fn get_bode_sample(&self, f_hz: f64) -> Complex<f64> {
        //Use y.norm() for amplitude and -y.arg().to_degrees() for phase. Add to combine phase.
        let mut y = self.a[0].get_bode_sample(f_hz) * self.b[0].get_bode_sample(f_hz);
        for (band_a, band_b) in self.a.iter().skip(1).zip(self.b.iter().skip(1)) {
            y *= band_a.get_bode_sample(f_hz);
            y *= band_b.get_bode_sample(f_hz);
        }
        y
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FilterbandStereo {
    svf_l: [SVF<f64>; FILTER_POLE_COUNT],
    svf_r: [SVF<f64>; FILTER_POLE_COUNT],
    svfb_l: [SVF<f64>; FILTER_POLE_COUNT],
    svfb_r: [SVF<f64>; FILTER_POLE_COUNT],
    pub coeffs: SVFCoefficientsSet,
    kind: FilterKind,
    freq: f64,
    gain: f64,
    bw_value: f64,
    slope: f64,
    sample_rate: f64,
}

impl FilterbandStereo {
    pub fn new(sample_rate: f64) -> FilterbandStereo {
        let coeffs = SVFCoefficientsSet::new(sample_rate);
        FilterbandStereo {
            svf_l: [SVF::<f64>::new(coeffs.a[0]); FILTER_POLE_COUNT],
            svf_r: [SVF::<f64>::new(coeffs.a[0]); FILTER_POLE_COUNT],
            svfb_l: [SVF::<f64>::new(coeffs.a[0]); FILTER_POLE_COUNT],
            svfb_r: [SVF::<f64>::new(coeffs.a[0]); FILTER_POLE_COUNT],
            coeffs,
            kind: FilterKind::Bell,
            freq: 1000.0,
            gain: 0.0,
            bw_value: 1.0,
            slope: 2.0,
            sample_rate: 48000.0,
        }
    }

    pub fn get_bode_sample(&self, f_hz: f64) -> Complex<f64> {
        //Use y.norm() for amplitude and -y.arg().to_degrees() for phase. Add to combine phase.
        let mut y = self.coeffs.get_bode_sample(f_hz);
        if self.kind == FilterKind::Mesa {
            let gain = self.gain.db_to_lin();
            y *= gain;
        } else if self.kind == FilterKind::Tilt {
            let gain = (self.gain * -1.0).db_to_lin();
            y *= gain;
        }
        y
    }

    pub fn process(&mut self, l: f64, r: f64) -> [f64; 2] {
        let mut l = l;
        let mut r = r;

        if self.kind == FilterKind::Bell
            || self.kind == FilterKind::Notch
            || self.kind == FilterKind::AllPass
        {
            l = self.svf_l[0].run(l);
            r = self.svf_r[0].run(r);
        } else {
            for i in 0..(self.slope * 0.5) as usize {
                if self.kind == FilterKind::Mesa || self.kind == FilterKind::BandPass {
                    l = self.svf_l[i].run(self.svfb_l[i].run(l));
                    r = self.svf_r[i].run(self.svfb_r[i].run(r));
                } else {
                    l = self.svf_l[i].run(l);
                    r = self.svf_r[i].run(r);
                }
            }
        }
        if self.kind == FilterKind::Mesa {
            let gain = self.gain.db_to_lin();
            l *= gain;
            r *= gain;
        } else if self.kind == FilterKind::Tilt {
            let gain = (self.gain * -1.0).db_to_lin();
            l *= gain;
            r *= gain;
        }
        [l, r]
    }

    pub fn update(
        &mut self,
        kind: FilterKind,
        in_freq: f64,
        in_gain: f64,
        in_bw_value: f64,
        slope: f64,
        sample_rate: f64,
    ) {
        if kind == self.kind
            && in_freq == self.freq
            && in_gain == self.gain
            && in_bw_value == self.bw_value
            && slope == self.slope
            && sample_rate == self.sample_rate
        {
            return;
        }

        self.kind = kind;
        self.freq = in_freq;
        self.gain = in_gain;
        self.bw_value = in_bw_value;
        self.slope = slope;
        self.sample_rate = sample_rate;

        let freq = self.freq;
        let gain = self.gain;
        let bw_value = self.bw_value;

        let u_slope = slope as u32;

        let slope_gain = ((self.slope * 0.5) as u32) as f64;

        let partial_gain = gain / slope_gain;

        let q_value = bw_value.bw_to_q(freq, sample_rate);
        let q_offset = q_value * Q_BUTTERWORTH_F64;

        match self.kind {
            FilterKind::Bell => {
                self.coeffs.a[0] = SVFCoefficients::<f64>::from_params(
                    Type::PeakingEQ(gain),
                    sample_rate,
                    freq,
                    q_value,
                )
                .unwrap();
                self.svf_l[0].update_coefficients(self.coeffs.a[0]);
                self.svf_r[0].update_coefficients(self.coeffs.a[0]);
            }
            FilterKind::LowPass => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::LowPass,
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                }
            }
            FilterKind::HighPass => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::HighPass,
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                }
            }
            FilterKind::LowShelf => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::LowShelf(partial_gain),
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                }
            }
            FilterKind::HighShelf => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::HighShelf(partial_gain),
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                }
            }
            FilterKind::Notch => {
                self.coeffs.a[0] =
                    SVFCoefficients::<f64>::from_params(Type::Notch, sample_rate, freq, q_value)
                        .unwrap();
                self.svf_l[0].update_coefficients(self.coeffs.a[0]);
                self.svf_r[0].update_coefficients(self.coeffs.a[0]);
            }
            FilterKind::BandPass => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::HighPass,
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                    self.coeffs.b[i] = SVFCoefficients::<f64>::from_params(
                        Type::LowPass,
                        sample_rate,
                        freq,
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svfb_l[i].update_coefficients(self.coeffs.b[i]);
                    self.svfb_r[i].update_coefficients(self.coeffs.b[i]);
                }
            }
            FilterKind::Tilt => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::HighShelf(partial_gain * 2.0),
                        sample_rate,
                        freq, // * (partial_gain * 0.495).db_to_lin()
                        q_value * q_offset,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                }
            }
            FilterKind::Mesa => {
                for i in 0..(self.slope * 0.5) as usize {
                    let q_value = butterworth_cascade_q(u_slope, i as u32);
                    self.coeffs.a[i] = SVFCoefficients::<f64>::from_params(
                        Type::LowShelf(-partial_gain),
                        sample_rate,
                        (freq / (self.bw_value + 0.5)).min(20000.0).max(20.0),
                        q_value,
                    )
                    .unwrap();
                    self.svf_l[i].update_coefficients(self.coeffs.a[i]);
                    self.svf_r[i].update_coefficients(self.coeffs.a[i]);
                    self.coeffs.b[i] = SVFCoefficients::<f64>::from_params(
                        Type::HighShelf(-partial_gain),
                        sample_rate,
                        (freq * (self.bw_value + 0.5)).min(20000.0).max(20.0),
                        q_value,
                    )
                    .unwrap();
                    self.svfb_l[i].update_coefficients(self.coeffs.b[i]);
                    self.svfb_r[i].update_coefficients(self.coeffs.b[i]);
                }
            }
            FilterKind::AllPass => {
                self.coeffs.a[0] =
                    SVFCoefficients::<f64>::from_params(Type::AllPass, sample_rate, freq, q_value)
                        .unwrap();
                self.svf_l[0].update_coefficients(self.coeffs.a[0]);
                self.svf_r[0].update_coefficients(self.coeffs.a[0]);
            }
        }
    }
}
