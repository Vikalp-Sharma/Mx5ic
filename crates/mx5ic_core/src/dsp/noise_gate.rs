use crate::dsp::biquad::{Biquad, FilterType};

/// Adaptive Spectral Noise Cancellation & De-Hiss Filter
///
/// Features:
/// 1. 20 Hz Subsonic Rumble Filter (High-pass).
/// 2. Adaptive Noise Floor Tracking (Continuous Minimum-Energy Estimation).
/// 3. Soft-Knee Downward Expander: Smoothly eliminates background microphone hiss,
///    preamp buzz, and codec quantization noise during quieter passages.
/// 4. 8 kHz Dynamic De-Hiss High Shelf for whisper-clean air without harshness.
#[derive(Debug, Clone)]
pub struct AdaptiveNoiseGate {
    _sample_rate: f32,
    pub enabled: bool,
    pub threshold_db: f32, // User/AI threshold offset (default ~ -54 dBFS)
    pub reduction_db: f32, // Max noise attenuation in dB (e.g. 18 dB)

    // Dynamic noise floor estimate (slow adaptation)
    noise_floor_estimate: f32,
    
    // Envelope follower (fast attack, smooth release)
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
    gain_smooth: f32,

    // Subsonic Rumble Blockers (20 Hz HPF)
    rumble_filter_l: Biquad,
    rumble_filter_r: Biquad,

    // High-frequency Hiss Damper (8 kHz High Shelf)
    dehiss_filter_l: Biquad,
    dehiss_filter_r: Biquad,
}

impl AdaptiveNoiseGate {
    pub fn new(sample_rate: f32) -> Self {
        let attack_time = 0.0015; // 1.5 ms attack
        let release_time = 0.0450; // 45 ms release

        let attack_coeff = (-1.0 / (attack_time * sample_rate)).exp();
        let release_coeff = (-1.0 / (release_time * sample_rate)).exp();

        let mut rumble_l = Biquad::new();
        let mut rumble_r = Biquad::new();
        rumble_l.set_params(FilterType::HighPass, sample_rate, 22.0, 0.0, 0.707);
        rumble_r.set_params(FilterType::HighPass, sample_rate, 22.0, 0.0, 0.707);

        let mut dehiss_l = Biquad::new();
        let mut dehiss_r = Biquad::new();
        dehiss_l.set_params(FilterType::HighShelf, sample_rate, 8500.0, -3.0, 0.707);
        dehiss_r.set_params(FilterType::HighShelf, sample_rate, 8500.0, -3.0, 0.707);

        Self {
            _sample_rate: sample_rate,
            enabled: true,
            threshold_db: -54.0,
            reduction_db: 18.0,
            noise_floor_estimate: 0.002, // ~ -54 dBFS initial
            envelope: 0.0,
            attack_coeff,
            release_coeff,
            gain_smooth: 1.0,
            rumble_filter_l: rumble_l,
            rumble_filter_r: rumble_r,
            dehiss_filter_l: dehiss_l,
            dehiss_filter_r: dehiss_r,
        }
    }

    /// Process a stereo frame (in_l, in_r) -> (out_l, out_r)
    #[inline]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if !self.enabled {
            return (in_l, in_r);
        }

        // 1. Strip Subsonic Rumble (< 22 Hz)
        let clean_l = self.rumble_filter_l.process(in_l);
        let clean_r = self.rumble_filter_r.process(in_r);

        // 2. Measure instantaneous peak energy
        let peak = clean_l.abs().max(clean_r.abs());

        // Fast attack, slow release envelope
        if peak > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * peak;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * peak;
        }

        // 3. Adaptive Noise Floor Tracking (slow decay towards minimum observed energy)
        if self.envelope > 0.00001 && self.envelope < self.noise_floor_estimate * 1.5 {
            self.noise_floor_estimate = 0.9995 * self.noise_floor_estimate + 0.0005 * self.envelope;
        }

        // 4. Calculate Soft-Knee Expansion Gain
        let env_db = if self.envelope > 1e-6 {
            20.0 * self.envelope.log10()
        } else {
            -120.0
        };

        // Knee width = 10 dB around threshold
        let thresh = self.threshold_db.max(20.0 * self.noise_floor_estimate.max(1e-5).log10() + 6.0);
        let knee_half = 5.0;

        let target_gain = if env_db > thresh + knee_half {
            // Far above noise floor: 100% transparent unity gain
            1.0f32
        } else if env_db < thresh - knee_half {
            // Well below noise floor: apply maximum downward expansion reduction
            let min_gain = 10.0f32.powf(-self.reduction_db / 20.0);
            min_gain
        } else {
            // Soft-knee transition region (smooth polynomial interpolation)
            let x = (env_db - (thresh - knee_half)) / (2.0 * knee_half);
            let factor = x * x * (3.0 - 2.0 * x); // Smoothstep curve
            let min_gain = 10.0f32.powf(-self.reduction_db / 20.0);
            min_gain + factor * (1.0 - min_gain)
        };

        // Smooth gain changes to eliminate clicks
        self.gain_smooth = 0.85 * self.gain_smooth + 0.15 * target_gain;

        // When deep in noise region, apply gentle de-hiss high shelf to eradicate tape/mic hiss
        let (out_l, out_r) = if self.gain_smooth < 0.6 {
            let hissed_l = self.dehiss_filter_l.process(clean_l);
            let hissed_r = self.dehiss_filter_r.process(clean_r);
            (hissed_l * self.gain_smooth, hissed_r * self.gain_smooth)
        } else {
            (clean_l * self.gain_smooth, clean_r * self.gain_smooth)
        };

        (out_l, out_r)
    }

    pub fn set_threshold_db(&mut self, thresh: f32) {
        self.threshold_db = thresh;
    }

    pub fn set_reduction_db(&mut self, red: f32) {
        self.reduction_db = red.clamp(0.0, 40.0);
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.gain_smooth = 1.0;
        self.rumble_filter_l.reset();
        self.rumble_filter_r.reset();
        self.dehiss_filter_l.reset();
        self.dehiss_filter_r.reset();
    }
}
