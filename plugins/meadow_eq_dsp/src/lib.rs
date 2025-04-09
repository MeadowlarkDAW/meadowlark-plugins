use arrayvec::ArrayVec;
use std::f64::consts::PI;

pub const DEFAULT_Q: f32 = Q_BUTTERWORTH_ORD2 as f32;

const Q_BUTTERWORTH_ORD2: f64 = 0.70710678118654752440;
const Q_BUTTERWORTH_ORD4: [f64; 2] = [0.54119610014619698440, 1.3065629648763765279];
const Q_BUTTERWORTH_ORD6: [f64; 3] = [
    0.51763809020504152470,
    0.70710678118654752440,
    1.9318516525781365735,
];
const Q_BUTTERWORTH_ORD8: [f64; 4] = [
    0.50979557910415916894,
    0.60134488693504528054,
    0.89997622313641570464,
    2.5629154477415061788,
];

const ORD4_Q_SCALE: f64 = 0.35;
const ORD6_Q_SCALE: f64 = 0.2;
const ORD8_Q_SCALE: f64 = 0.14;

const MAX_ONE_POLE_FILTERS: usize = 2;
const MAX_SVF_FILTERS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FilterOrder {
    X1 = 0,
    X2,
    X4,
    X6,
    X8,
}

impl FilterOrder {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::X1,
            1 => Self::X2,
            2 => Self::X4,
            3 => Self::X6,
            _ => Self::X8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BandType {
    Bell = 0,
    LowShelf,
    HighShelf,
    Notch,
    Allpass,
}

impl BandType {
    pub fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Bell,
            1 => Self::LowShelf,
            2 => Self::HighShelf,
            3 => Self::Notch,
            _ => Self::Allpass,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandParams {
    pub enabled: bool,
    pub band_type: BandType,
    pub cutoff_hz: f32,
    pub q: f32,
    pub gain_db: f32,
}

impl Default for BandParams {
    fn default() -> Self {
        Self {
            enabled: false,
            band_type: BandType::Notch,
            cutoff_hz: 1000.0,
            q: DEFAULT_Q,
            gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqParams<const NUM_BANDS: usize> {
    pub lp_enabled: bool,
    pub lp_cutoff_hz: f32,
    pub lp_q: f32,
    pub lp_order: FilterOrder,

    pub hp_enabled: bool,
    pub hp_cutoff_hz: f32,
    pub hp_q: f32,
    pub hp_order: FilterOrder,

    pub bands: [BandParams; NUM_BANDS],
}

impl<const NUM_BANDS: usize> Default for EqParams<NUM_BANDS> {
    fn default() -> Self {
        Self {
            lp_enabled: false,
            lp_cutoff_hz: 21_480.0,
            lp_q: DEFAULT_Q,
            lp_order: FilterOrder::X2,

            hp_enabled: false,
            hp_cutoff_hz: 20.0,
            hp_q: DEFAULT_Q,
            hp_order: FilterOrder::X2,

            bands: [BandParams::default(); NUM_BANDS],
        }
    }
}

pub struct MeadowEqDsp<const NUM_BANDS: usize> {
    params: EqParams<NUM_BANDS>,

    lp_band: MultiOrderBand<2>,
    hp_band: MultiOrderBand<2>,

    bands: [SecondOrderBand<2>; NUM_BANDS],

    has_first_order_filter: bool,

    sample_rate_recip: f64,
}

impl<const NUM_BANDS: usize> MeadowEqDsp<NUM_BANDS> {
    pub fn new(sample_rate: f64) -> Self {
        let sample_rate_recip = sample_rate.recip();

        let params = EqParams::default();

        Self {
            params,
            lp_band: MultiOrderBand::new(FilterOrder::X2),
            hp_band: MultiOrderBand::new(FilterOrder::X2),
            bands: [SecondOrderBand::new(); NUM_BANDS],
            has_first_order_filter: false,
            sample_rate_recip,
        }
    }

    pub fn set_params(&mut self, params: EqParams<NUM_BANDS>) {
        if self.params != params {
            self.params = params;
            self.has_first_order_filter = false;

            self.lp_band.enabled = params.lp_enabled;
            if params.lp_enabled {
                match params.lp_order {
                    FilterOrder::X1 => self.lp_band.set_ord1(OnePoleCoeff::lowpass(
                        params.lp_cutoff_hz as f64,
                        self.sample_rate_recip,
                    )),
                    FilterOrder::X2 => self.lp_band.set_ord2(SvfCoeff::lowpass_ord2(
                        params.lp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.lp_q as f64,
                    )),
                    FilterOrder::X4 => self.lp_band.set_ord4(SvfCoeff::lowpass_ord4(
                        params.lp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.lp_q as f64,
                    )),
                    FilterOrder::X6 => self.lp_band.set_ord6(SvfCoeff::lowpass_ord6(
                        params.lp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.lp_q as f64,
                    )),
                    FilterOrder::X8 => self.lp_band.set_ord8(SvfCoeff::lowpass_ord8(
                        params.lp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.lp_q as f64,
                    )),
                }
            }

            self.hp_band.enabled = params.hp_enabled;
            if params.hp_enabled {
                match params.hp_order {
                    FilterOrder::X1 => self.hp_band.set_ord1(OnePoleCoeff::highpass(
                        params.hp_cutoff_hz as f64,
                        self.sample_rate_recip,
                    )),
                    FilterOrder::X2 => self.hp_band.set_ord2(SvfCoeff::highpass_ord2(
                        params.hp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.hp_q as f64,
                    )),
                    FilterOrder::X4 => self.hp_band.set_ord4(SvfCoeff::highpass_ord4(
                        params.hp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.hp_q as f64,
                    )),
                    FilterOrder::X6 => self.hp_band.set_ord6(SvfCoeff::highpass_ord6(
                        params.hp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.hp_q as f64,
                    )),
                    FilterOrder::X8 => self.hp_band.set_ord8(SvfCoeff::highpass_ord8(
                        params.hp_cutoff_hz as f64,
                        self.sample_rate_recip,
                        params.hp_q as f64,
                    )),
                }
            }

            for (band_params, band) in params.bands.iter().zip(self.bands.iter_mut()) {
                band.enabled = band_params.enabled;
                if band_params.enabled {
                    match band_params.band_type {
                        BandType::Bell => band.set(SvfCoeff::bell(
                            band_params.cutoff_hz as f64,
                            self.sample_rate_recip,
                            band_params.q as f64,
                            band_params.gain_db as f64,
                        )),
                        BandType::LowShelf => band.set(SvfCoeff::low_shelf(
                            band_params.cutoff_hz as f64,
                            self.sample_rate_recip,
                            band_params.q as f64,
                            band_params.gain_db as f64,
                        )),
                        BandType::HighShelf => band.set(SvfCoeff::high_shelf(
                            band_params.cutoff_hz as f64,
                            self.sample_rate_recip,
                            band_params.q as f64,
                            band_params.gain_db as f64,
                        )),
                        BandType::Notch => band.set(SvfCoeff::notch(
                            band_params.cutoff_hz as f64,
                            self.sample_rate_recip,
                            band_params.q as f64,
                        )),
                        BandType::Allpass => band.set(SvfCoeff::allpass(
                            band_params.cutoff_hz as f64,
                            self.sample_rate_recip,
                            band_params.q as f64,
                        )),
                    }
                }
            }
        }
    }

    pub fn process(&mut self, buf_l: &mut [f32], buf_r: &mut [f32]) {
        let mut one_pole_filters: ArrayVec<
            (OnePoleCoeff, [OnePoleState; 2]),
            MAX_ONE_POLE_FILTERS,
        > = ArrayVec::new();
        let mut svf_filters: ArrayVec<(SvfCoeff, [SvfState; 2]), MAX_SVF_FILTERS> = ArrayVec::new();

        if self.lp_band.enabled {
            self.lp_band
                .add_filter_states(&mut one_pole_filters, &mut svf_filters);
        }
        if self.hp_band.enabled {
            self.hp_band
                .add_filter_states(&mut one_pole_filters, &mut svf_filters);
        }
        for band in self.bands.iter().filter(|b| b.enabled) {
            band.add_filter_states(&mut svf_filters);
        }

        if one_pole_filters.is_empty() && svf_filters.is_empty() {
            return;
        }

        match one_pole_filters.len() {
            0 => {}
            1 => {
                let (coeff, state) = &mut one_pole_filters[0];

                for (buf_l, buf_r) in buf_l.iter_mut().zip(buf_r.iter_mut()) {
                    *buf_l = state[0].tick(*buf_l, coeff);
                    *buf_r = state[1].tick(*buf_r, coeff);
                }
            }
            2 => {
                let (f0, f1) = one_pole_filters.split_first_mut().unwrap();
                let (coeff_0, state_0) = f0;
                let (coeff_1, state_1) = &mut f1[0];

                for (buf_l, buf_r) in buf_l.iter_mut().zip(buf_r.iter_mut()) {
                    let l = state_0[0].tick(*buf_l, coeff_0);
                    let r = state_0[1].tick(*buf_r, coeff_0);

                    *buf_l = state_1[0].tick(l, coeff_1);
                    *buf_r = state_1[1].tick(r, coeff_1);
                }
            }
            _ => unreachable!(),
        }

        if !svf_filters.is_empty() {
            for (buf_l, buf_r) in buf_l.iter_mut().zip(buf_r.iter_mut()) {
                let mut l = *buf_l;
                let mut r = *buf_r;

                for (coeff, state) in svf_filters.iter_mut() {
                    l = state[0].tick(l, coeff);
                    r = state[1].tick(r, coeff);
                }

                *buf_l = l;
                *buf_r = r;
            }
        }

        let mut one_pole_filter_i = 0;
        let mut svf_filter_i = 0;

        if self.lp_band.enabled {
            self.lp_band.sync_filter_states(
                &mut one_pole_filter_i,
                &mut svf_filter_i,
                &one_pole_filters,
                &svf_filters,
            );
        }
        if self.hp_band.enabled {
            self.hp_band.sync_filter_states(
                &mut one_pole_filter_i,
                &mut svf_filter_i,
                &one_pole_filters,
                &svf_filters,
            );
        }
        for band in self.bands.iter_mut().filter(|b| b.enabled) {
            band.sync_filter_states(&mut svf_filter_i, &svf_filters);
        }
    }
}

#[derive(Default, Clone, Copy)]
struct SvfCoeff {
    a1: f32,
    a2: f32,
    a3: f32,

    m0: f32,
    m1: f32,
    m2: f32,
}

impl SvfCoeff {
    fn lowpass_ord2(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> Self {
        let g = g(cutoff_hz, sample_rate_recip);
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, 0.0, 0.0, 1.0)
    }

    fn lowpass_ord4(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 2] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD4_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD4[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 0.0, 0.0, 1.0)
        })
    }

    fn lowpass_ord6(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 3] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD6_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD6[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 0.0, 0.0, 1.0)
        })
    }

    fn lowpass_ord8(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 4] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD8_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD8[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 0.0, 0.0, 1.0)
        })
    }

    fn highpass_ord2(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> Self {
        let g = g(cutoff_hz, sample_rate_recip);
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, 1.0, -k, -1.0)
    }

    fn highpass_ord4(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 2] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD4_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD4[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 1.0, -k, -1.0)
        })
    }

    fn highpass_ord6(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 3] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD6_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD6[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 1.0, -k, -1.0)
        })
    }

    fn highpass_ord8(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> [Self; 4] {
        let g = g(cutoff_hz, sample_rate_recip);
        let q_norm = scale_q_norm_for_order(q_norm(q), ORD8_Q_SCALE);

        std::array::from_fn(|i| {
            let q = q_norm * Q_BUTTERWORTH_ORD8[i];
            let k = 1.0 / q;

            Self::from_g_and_k(g, k, 1.0, -k, -1.0)
        })
    }

    fn notch(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> Self {
        let g = g(cutoff_hz, sample_rate_recip);
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, 1.0, -k, 0.0)
    }

    fn bell(cutoff_hz: f64, sample_rate_recip: f64, q: f64, gain_db: f64) -> Self {
        let a = gain_db_to_a(gain_db);

        let g = g(cutoff_hz, sample_rate_recip);
        let k = 1.0 / (q * a);

        Self::from_g_and_k(g, k, 1.0, k * (a * a - 1.0), 0.0)
    }

    fn low_shelf(cutoff_hz: f64, sample_rate_recip: f64, q: f64, gain_db: f64) -> Self {
        let a = gain_db_to_a(gain_db);

        let g = (PI * cutoff_hz * sample_rate_recip).tan() / a.sqrt();
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, 1.0, k * (a - 1.0), a * a - 1.0)
    }

    fn high_shelf(cutoff_hz: f64, sample_rate_recip: f64, q: f64, gain_db: f64) -> Self {
        let a = gain_db_to_a(gain_db);

        let g = (PI * cutoff_hz * sample_rate_recip).tan() / a.sqrt();
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, a * a, k * (1.0 - a) * a, 1.0 - a * a)
    }

    fn allpass(cutoff_hz: f64, sample_rate_recip: f64, q: f64) -> Self {
        let g = g(cutoff_hz, sample_rate_recip);
        let k = 1.0 / q;

        Self::from_g_and_k(g, k, 1.0, -2.0 * k, 0.0)
    }

    fn from_g_and_k(g: f64, k: f64, m0: f64, m1: f64, m2: f64) -> Self {
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;

        Self {
            a1: a1 as f32,
            a2: a2 as f32,
            a3: a3 as f32,
            m0: m0 as f32,
            m1: m1 as f32,
            m2: m2 as f32,
        }
    }
}

fn g(cutoff_hz: f64, sample_rate_recip: f64) -> f64 {
    (PI * cutoff_hz * sample_rate_recip).tan()
}

fn q_norm(q: f64) -> f64 {
    q * (1.0 / Q_BUTTERWORTH_ORD2)
}

fn gain_db_to_a(gain_db: f64) -> f64 {
    10.0f64.powf(gain_db.clamp(-30.0, 30.0) / 40.0)
}

fn scale_q_norm_for_order(q_norm: f64, scale: f64) -> f64 {
    if q_norm > 1.0 {
        1.0 + ((q_norm - 1.0) * scale)
    } else {
        q_norm
    }
}

#[derive(Default, Clone, Copy)]
struct SvfState {
    ic1eq: f32,
    ic2eq: f32,
}

impl SvfState {
    #[inline(always)]
    fn tick(&mut self, input: f32, coeff: &SvfCoeff) -> f32 {
        let v3 = input - self.ic2eq;
        let v1 = coeff.a1 * self.ic1eq + coeff.a2 * v3;
        let v2 = self.ic2eq + coeff.a2 * self.ic1eq + coeff.a3 * v3;
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;

        coeff.m0 * input + coeff.m1 * v1 + coeff.m2 * v2
    }
}

#[derive(Default, Clone, Copy)]
struct OnePoleCoeff {
    a0: f32,
    b1: f32,

    m0: f32,
    m1: f32,
}

impl OnePoleCoeff {
    fn lowpass(cutoff_hz: f64, sample_rate_recip: f64) -> Self {
        let b1 = ((-2.0 * PI) * cutoff_hz * sample_rate_recip).exp();
        let a0 = 1.0 - b1;

        Self {
            a0: a0 as f32,
            b1: b1 as f32,
            m0: 0.0,
            m1: 1.0,
        }
    }

    fn highpass(cutoff_hz: f64, sample_rate_recip: f64) -> Self {
        let b1 = ((-2.0 * PI) * cutoff_hz * sample_rate_recip).exp();
        let a0 = 1.0 - b1;

        Self {
            a0: a0 as f32,
            b1: b1 as f32,
            m0: 1.0,
            m1: -1.0,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct OnePoleState {
    z1: f32,
}

impl OnePoleState {
    #[inline(always)]
    fn tick(&mut self, input: f32, coeff: &OnePoleCoeff) -> f32 {
        self.z1 = (coeff.a0 * input) + (coeff.b1 * self.z1);
        coeff.m0 * input + coeff.m1 * self.z1
    }
}

#[derive(Clone, Copy)]
struct SecondOrderBand<const NUM_CHANNELS: usize> {
    enabled: bool,
    coeff: SvfCoeff,
    state: [SvfState; NUM_CHANNELS],
}

impl<const NUM_CHANNELS: usize> SecondOrderBand<NUM_CHANNELS> {
    fn set(&mut self, coeff: SvfCoeff) {
        self.coeff = coeff;
    }

    fn add_filter_states(
        &self,
        svf_filters: &mut ArrayVec<(SvfCoeff, [SvfState; NUM_CHANNELS]), MAX_SVF_FILTERS>,
    ) {
        svf_filters.push((self.coeff, self.state));
    }

    fn sync_filter_states(
        &mut self,
        svf_filter_i: &mut usize,
        svf_filters: &ArrayVec<(SvfCoeff, [SvfState; NUM_CHANNELS]), MAX_SVF_FILTERS>,
    ) {
        self.state = svf_filters[*svf_filter_i].1;
        *svf_filter_i += 1;
    }
}

impl<const NUM_CHANNELS: usize> SecondOrderBand<NUM_CHANNELS> {
    fn new() -> Self {
        Self {
            enabled: false,
            coeff: SvfCoeff::default(),
            state: [SvfState::default(); NUM_CHANNELS],
        }
    }
}

struct MultiOrderBand<const NUM_CHANNELS: usize> {
    enabled: bool,
    order: FilterOrder,

    one_pole_coeff: OnePoleCoeff,
    one_pole_state: [OnePoleState; NUM_CHANNELS],

    coeff_0: SvfCoeff,
    coeff_1: SvfCoeff,
    coeff_2: SvfCoeff,
    coeff_3: SvfCoeff,

    state_0: [SvfState; NUM_CHANNELS],
    state_1: [SvfState; NUM_CHANNELS],
    state_2: [SvfState; NUM_CHANNELS],
    state_3: [SvfState; NUM_CHANNELS],
}

impl<const NUM_CHANNELS: usize> MultiOrderBand<NUM_CHANNELS> {
    fn new(order: FilterOrder) -> Self {
        Self {
            enabled: false,

            order,

            one_pole_coeff: OnePoleCoeff::default(),
            one_pole_state: [OnePoleState::default(); NUM_CHANNELS],

            coeff_0: SvfCoeff::default(),
            coeff_1: SvfCoeff::default(),
            coeff_2: SvfCoeff::default(),
            coeff_3: SvfCoeff::default(),

            state_0: [SvfState::default(); NUM_CHANNELS],
            state_1: [SvfState::default(); NUM_CHANNELS],
            state_2: [SvfState::default(); NUM_CHANNELS],
            state_3: [SvfState::default(); NUM_CHANNELS],
        }
    }

    fn add_filter_states(
        &self,
        one_pole_filters: &mut ArrayVec<
            (OnePoleCoeff, [OnePoleState; NUM_CHANNELS]),
            MAX_ONE_POLE_FILTERS,
        >,
        svf_filters: &mut ArrayVec<(SvfCoeff, [SvfState; NUM_CHANNELS]), MAX_SVF_FILTERS>,
    ) {
        match self.order {
            FilterOrder::X1 => one_pole_filters.push((self.one_pole_coeff, self.one_pole_state)),
            FilterOrder::X2 => svf_filters.push((self.coeff_0, self.state_0)),
            FilterOrder::X4 => {
                svf_filters.push((self.coeff_0, self.state_0));
                svf_filters.push((self.coeff_1, self.state_1));
            }
            FilterOrder::X6 => {
                svf_filters.push((self.coeff_0, self.state_0));
                svf_filters.push((self.coeff_1, self.state_1));
                svf_filters.push((self.coeff_2, self.state_2));
            }
            FilterOrder::X8 => {
                svf_filters.push((self.coeff_0, self.state_0));
                svf_filters.push((self.coeff_1, self.state_1));
                svf_filters.push((self.coeff_2, self.state_2));
                svf_filters.push((self.coeff_3, self.state_3))
            }
        }
    }

    fn sync_filter_states(
        &mut self,
        one_pole_filter_i: &mut usize,
        svf_filter_i: &mut usize,
        one_pole_filters: &ArrayVec<
            (OnePoleCoeff, [OnePoleState; NUM_CHANNELS]),
            MAX_ONE_POLE_FILTERS,
        >,
        svf_filters: &ArrayVec<(SvfCoeff, [SvfState; NUM_CHANNELS]), MAX_SVF_FILTERS>,
    ) {
        match self.order {
            FilterOrder::X1 => {
                self.one_pole_state = one_pole_filters[*one_pole_filter_i].1;
                *one_pole_filter_i += 1;
            }
            FilterOrder::X2 => {
                self.state_0 = svf_filters[*svf_filter_i].1;
                *svf_filter_i += 1;
            }
            FilterOrder::X4 => {
                self.state_0 = svf_filters[*svf_filter_i].1;
                self.state_1 = svf_filters[*svf_filter_i + 1].1;
                *svf_filter_i += 2;
            }
            FilterOrder::X6 => {
                self.state_0 = svf_filters[*svf_filter_i].1;
                self.state_1 = svf_filters[*svf_filter_i + 1].1;
                self.state_2 = svf_filters[*svf_filter_i + 2].1;
                *svf_filter_i += 3;
            }
            FilterOrder::X8 => {
                self.state_0 = svf_filters[*svf_filter_i].1;
                self.state_1 = svf_filters[*svf_filter_i + 1].1;
                self.state_2 = svf_filters[*svf_filter_i + 2].1;
                self.state_3 = svf_filters[*svf_filter_i + 3].1;
                *svf_filter_i += 4;
            }
        }
    }

    fn set_ord1(&mut self, coeff: OnePoleCoeff) {
        self.order = FilterOrder::X1;
        self.one_pole_coeff = coeff;
    }

    fn set_ord2(&mut self, coeffs: SvfCoeff) {
        self.order = FilterOrder::X2;
        self.coeff_0 = coeffs;
    }

    fn set_ord4(&mut self, coeffs: [SvfCoeff; 2]) {
        self.order = FilterOrder::X4;
        self.coeff_0 = coeffs[0];
        self.coeff_1 = coeffs[1];
    }

    fn set_ord6(&mut self, coeffs: [SvfCoeff; 3]) {
        self.order = FilterOrder::X6;
        self.coeff_0 = coeffs[0];
        self.coeff_1 = coeffs[1];
        self.coeff_2 = coeffs[2];
    }

    fn set_ord8(&mut self, coeffs: [SvfCoeff; 4]) {
        self.order = FilterOrder::X8;
        self.coeff_0 = coeffs[0];
        self.coeff_1 = coeffs[1];
        self.coeff_2 = coeffs[2];
        self.coeff_3 = coeffs[3];
    }
}
