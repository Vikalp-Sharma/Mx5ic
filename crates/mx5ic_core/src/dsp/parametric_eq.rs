use super::biquad::{Biquad, FilterType};

pub const NUM_BANDS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub struct EqBandConfig {
    pub filter_type: FilterType,
    pub frequency: f32,
    pub gain_db: f32,
    pub q: f32,
    pub enabled: bool,
}

pub struct ParametricEq {
    sample_rate: f32,
    bands_l: [Biquad; NUM_BANDS],
    bands_r: [Biquad; NUM_BANDS],
    configs: [EqBandConfig; NUM_BANDS],
}

impl ParametricEq {
    pub fn new(sample_rate: f32) -> Self {
        let default_configs = [
            // Band 0: Sub Bass
            EqBandConfig { filter_type: FilterType::LowShelf,  frequency: 60.0,    gain_db: 0.0, q: 0.707, enabled: true },
            // Band 1: Bass Punch
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 120.0,   gain_db: 0.0, q: 1.0,   enabled: true },
            // Band 2: Warmth / Body
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 350.0,   gain_db: 0.0, q: 1.2,   enabled: true },
            // Band 3: Core Midrange
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 1000.0,  gain_db: 0.0, q: 1.2,   enabled: true },
            // Band 4: Vocal Clarity
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 2500.0,  gain_db: 0.0, q: 1.4,   enabled: true },
            // Band 5: Presence
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 5000.0,  gain_db: 0.0, q: 1.4,   enabled: true },
            // Band 6: Brilliance
            EqBandConfig { filter_type: FilterType::PeakingEQ, frequency: 10000.0, gain_db: 0.0, q: 1.0,   enabled: true },
            // Band 7: Air
            EqBandConfig { filter_type: FilterType::HighShelf, frequency: 14000.0, gain_db: 0.0, q: 0.707, enabled: true },
        ];

        let mut eq = Self {
            sample_rate,
            bands_l: [Biquad::new(); NUM_BANDS],
            bands_r: [Biquad::new(); NUM_BANDS],
            configs: default_configs,
        };

        for i in 0..NUM_BANDS {
            eq.recalculate_band(i);
        }

        eq
    }

    pub fn set_band(&mut self, index: usize, gain_db: f32, freq: Option<f32>, q: Option<f32>) {
        if index < NUM_BANDS {
            self.configs[index].gain_db = gain_db.clamp(-18.0, 18.0);
            if let Some(f) = freq {
                self.configs[index].frequency = f;
            }
            if let Some(new_q) = q {
                self.configs[index].q = new_q;
            }
            self.recalculate_band(index);
        }
    }

    fn recalculate_band(&mut self, index: usize) {
        let cfg = self.configs[index];
        self.bands_l[index].set_params(cfg.filter_type, self.sample_rate, cfg.frequency, cfg.gain_db, cfg.q);
        self.bands_r[index].set_params(cfg.filter_type, self.sample_rate, cfg.frequency, cfg.gain_db, cfg.q);
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, mut l: f32, mut r: f32) -> (f32, f32) {
        for i in 0..NUM_BANDS {
            if self.configs[i].enabled {
                l = self.bands_l[i].process(l);
                r = self.bands_r[i].process(r);
            }
        }
        (l, r)
    }

    pub fn reset(&mut self) {
        for i in 0..NUM_BANDS {
            self.bands_l[i].reset();
            self.bands_r[i].reset();
        }
    }
}
