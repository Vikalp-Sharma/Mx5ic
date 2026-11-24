use crate::dsp::biquad::{Biquad, FilterType};

/// Dedicated Speech and Dialogue Intelligibility Enhancer
/// Focuses on center-channel vocal clarity, dynamic de-essing,
/// and masking reduction for movies, podcasts, and spoken dialogue.
pub struct DialogueEnhancer {
    sample_rate: f32,
    pub amount: f32,         // 0.0 (off) to 1.0 (full dialogue boost)
    pub de_ess_amount: f32,  // 0.0 (off) to 1.0 (harsh sibilance clamp)
    formant_filter_l: Biquad,
    formant_filter_r: Biquad,
    sibilance_detector_l: Biquad,
    sibilance_detector_r: Biquad,
    de_ess_gain: f32,
}

impl DialogueEnhancer {
    pub fn new(sample_rate: f32) -> Self {
        // Speech formant peak at ~2.4 kHz (Q=1.2) for consonant intelligibility
        let mut formant_l = Biquad::new();
        formant_l.set_params(FilterType::PeakingEQ, sample_rate, 2400.0, 4.5, 1.2);
        let mut formant_r = Biquad::new();
        formant_r.set_params(FilterType::PeakingEQ, sample_rate, 2400.0, 4.5, 1.2);

        // High-frequency bandpass detector around 6.5 kHz for sibilance detection (t, s, sh)
        let mut sib_l = Biquad::new();
        sib_l.set_params(FilterType::BandPass, sample_rate, 6500.0, 0.0, 2.0);
        let mut sib_r = Biquad::new();
        sib_r.set_params(FilterType::BandPass, sample_rate, 6500.0, 0.0, 2.0);

        Self {
            sample_rate,
            amount: 0.0,
            de_ess_amount: 0.5,
            formant_filter_l: formant_l,
            formant_filter_r: formant_r,
            sibilance_detector_l: sib_l,
            sibilance_detector_r: sib_r,
            de_ess_gain: 1.0,
        }
    }

    pub fn set_amount(&mut self, amount: f32) {
        self.amount = amount.clamp(0.0, 1.0);
        let boost_db = self.amount * 6.0; // up to +6 dB intelligibility boost
        self.formant_filter_l.set_params(FilterType::PeakingEQ, self.sample_rate, 2400.0, boost_db, 1.2);
        self.formant_filter_r.set_params(FilterType::PeakingEQ, self.sample_rate, 2400.0, boost_db, 1.2);
    }

    /// Process a stereo sample frame (strictly real-time safe, no heap allocations)
    #[inline(always)]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if self.amount <= 0.001 {
            return (in_l, in_r);
        }

        // 1. Mid-Side decomposition: Dialogue is predominantly in the center (Mid)
        let mid = 0.5 * (in_l + in_r);
        let side = 0.5 * (in_l - in_r);

        // 2. Enhance Mid formant intelligibility
        let mid_enhanced = self.formant_filter_l.process(mid);

        // 3. Dynamic Sibilance Detection & Attenuation (De-Essing)
        let sib_energy_l = self.sibilance_detector_l.process(mid).abs();
        let sib_energy_r = self.sibilance_detector_r.process(side).abs();
        let sib_energy = sib_energy_l.max(sib_energy_r);
        let sib_threshold = 0.18;
        let target_de_ess = if sib_energy > sib_threshold {
            let excess = (sib_energy - sib_threshold) * 2.5 * self.de_ess_amount;
            (1.0 - excess).clamp(0.5, 1.0)
        } else {
            1.0
        };

        // Smooth de-ess gain response (instant attack, gentle release)
        if target_de_ess < self.de_ess_gain {
            self.de_ess_gain = target_de_ess; // fast attack
        } else {
            self.de_ess_gain += (1.0 - self.de_ess_gain) * 0.005; // release
        }

        let mid_processed = mid_enhanced * self.de_ess_gain;

        // 4. Subtle side-channel attenuation during heavy dialogue to reduce acoustic masking
        let side_duck = 1.0 - 0.25 * self.amount;
        let side_processed = side * side_duck;

        // 5. Recombine to Stereo: Left = Mid + Side, Right = Mid - Side
        let out_l = mid_processed + side_processed;
        let out_r = mid_processed - side_processed;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.formant_filter_l.reset();
        self.formant_filter_r.reset();
        self.sibilance_detector_l.reset();
        self.sibilance_detector_r.reset();
        self.de_ess_gain = 1.0;
    }
}
