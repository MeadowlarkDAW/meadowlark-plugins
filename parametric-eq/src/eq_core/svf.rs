use std::f64::consts::PI;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Errors {
    OutsideNyquist,
    NegativeQ,
    NegativeFrequency,
}

#[derive(Clone, Copy, Debug)]
pub enum Type<DBGain> {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    AllPass,
    LowShelf(DBGain),
    HighShelf(DBGain),
    PeakingEQ(DBGain),
}

#[derive(Copy, Clone, Debug)]
pub struct SVFCoefficients<T> {
    pub g: T,
    pub k: T,
    pub a1: T,
    pub a2: T,
    pub a3: T,
    pub m0: T,
    pub m1: T,
    pub m2: T,
}

impl SVFCoefficients<f64> {
    /// Creates a SVF from a set of filter coefficients
    pub fn from_params(
        filter: Type<f64>,
        fs: f64,
        f0: f64,
        q_value: f64,
    ) -> Result<SVFCoefficients<f64>, Errors> {
        if 2.0 * f0 > fs {
            return Err(Errors::OutsideNyquist);
        }

        if q_value < 0.0 {
            return Err(Errors::NegativeQ);
        }
        match filter {
            Type::LowPass => {
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 0.0;
                let m1 = 0.0;
                let m2 = 1.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::HighPass => {
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 1.0;
                let m1 = -k;
                let m2 = -1.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::BandPass => {
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 0.0;
                let m1 = 1.0;
                let m2 = 0.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::Notch => {
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 1.0;
                let m1 = -k;
                let m2 = 0.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::AllPass => {
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 1.0;
                let m1 = -2.0 * k;
                let m2 = 0.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::LowShelf(db_gain) => {
                let a = 10.0f64.powf(db_gain / 40.0);
                let g = (PI * f0 / fs).tan() / (a).sqrt();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 1.0;
                let m1 = k * (a - 1.0);
                let m2 = a * a - 1.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::HighShelf(db_gain) => {
                let a = 10.0f64.powf(db_gain / 40.0);
                let g = (PI * f0 / fs).tan() * (a).sqrt();
                let k = 1.0 / q_value;
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = a * a;
                let m1 = k * (1.0 - a) * a;
                let m2 = 1.0 - a * a;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
            Type::PeakingEQ(db_gain) => {
                let a = 10.0f64.powf(db_gain / 40.0);
                let g = (PI * f0 / fs).tan();
                let k = 1.0 / (q_value * a);
                let a1 = 1.0 / (1.0 + g * (g + k));
                let a2 = g * a1;
                let a3 = g * a2;
                let m0 = 1.0;
                let m1 = k * (a * a - 1.0);
                let m2 = 0.0;
                Ok(SVFCoefficients {
                    g,
                    k,
                    a1,
                    a2,
                    a3,
                    m0,
                    m1,
                    m2,
                })
            }
        }
    }
}

/// Internal states and coefficients of the SVF form
#[derive(Copy, Clone, Debug)]
pub struct SVF<T> {
    ic1eq: T,
    ic2eq: T,
    pub coeffs: SVFCoefficients<T>,
}

impl SVF<f64> {
    /// Creates a SVF from a set of filter coefficients
    pub fn new(coefficients: SVFCoefficients<f64>) -> Self {
        SVF {
            ic1eq: 0.0,
            ic2eq: 0.0,
            coeffs: coefficients,
        }
    }

    pub fn run(&mut self, input: f64) -> f64 {
        let v3 = input - self.ic2eq;
        let v1 = self.coeffs.a1 * self.ic1eq + self.coeffs.a2 * v3;
        let v2 = self.ic2eq + self.coeffs.a2 * self.ic1eq + self.coeffs.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        self.coeffs.m0 * input + self.coeffs.m1 * v1 + self.coeffs.m2 * v2
    }

    pub fn update_coefficients(&mut self, new_coefficients: SVFCoefficients<f64>) {
        self.coeffs = new_coefficients;
    }
}
