use super::biquad::{Biquad, FilterType};

/// Stereo Virtual Bass Harmonic Exciter (MaxxBass principle)
pub struct VirtualBass {
    crossover_l: Biquad,
    crossover_r: Biquad,
    dc_blocker_l: Biquad,
    dc_blocker_r: Biquad,
    pub drive: f32,
    pub mix: f32,
    pub enabled: bool,
}

impl VirtualBass {
    pub fn new(sample_rate: f32) -> Self {
        let mut xo_l = Biquad::new();
        let mut xo_r = Biquad::new();
        xo_l.set_params(FilterType::LowPass, sample_rate, 80.0, 0.0, 0.707);
        xo_r.set_params(FilterType::LowPass, sample_rate, 80.0, 0.0, 0.707);

        let mut dc_l = Biquad::new();
        let mut dc_r = Biquad::new();
        dc_l.set_params(FilterType::HighPass, sample_rate, 25.0, 0.0, 0.707);
        dc_r.set_params(FilterType::HighPass, sample_rate, 25.0, 0.0, 0.707);

        Self {
            crossover_l: xo_l,
            crossover_r: xo_r,
            dc_blocker_l: dc_l,
            dc_blocker_r: dc_r,
            drive: 2.2,
            mix: 0.40,
            enabled: true,
        }
    }

    #[inline(always)]
    pub fn process_sample(&mut self, sample: f32, is_left: bool) -> f32 {
        if !self.enabled || self.mix <= 0.001 {
            return sample;
        }

        let xo = if is_left { &mut self.crossover_l } else { &mut self.crossover_r };
        let sub = xo.process(sample);

        // Harmonic generation:
        // 2nd harmonic: asymmetric squaring
        // 3rd harmonic: symmetric tanh saturation
        let driven = sub * self.drive;
        let h2 = 0.5 * (driven * driven).min(1.0);
        let h3 = (driven * 1.2).tanh();
        let raw_harmonics = (h2 + h3) * 0.5;

        // Block any DC drift from squaring
        let dc = if is_left { &mut self.dc_blocker_l } else { &mut self.dc_blocker_r };
        let clean_harmonics = dc.process(raw_harmonics);

        sample + clean_harmonics * self.mix
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        (self.process_sample(l, true), self.process_sample(r, false))
    }

    pub fn reset(&mut self) {
        self.crossover_l.reset();
        self.crossover_r.reset();
        self.dc_blocker_l.reset();
        self.dc_blocker_r.reset();
    }
}
