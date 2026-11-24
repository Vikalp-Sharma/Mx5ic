use std::f32::consts::PI;

/// Real-time audio feature extractor for music understanding
#[derive(Clone, Debug, Default)]
pub struct AudioFeatures {
    pub rms_level: f32,
    pub peak_level: f32,
    pub crest_factor: f32,          // Dynamic punch / compression indicator
    pub spectral_centroid: f32,     // Perceived brightness in Hz
    pub sub_bass_ratio: f32,        // Fraction of energy below 150 Hz
    pub mid_vocal_ratio: f32,       // Fraction of energy in vocal band (300 - 3500 Hz)
    pub high_air_ratio: f32,        // Fraction of energy above 4 kHz
    pub zero_crossing_rate: f32,    // Noisiness vs tonal harmonicity
    pub stereo_width: f32,          // Side-to-Mid ratio (0.0 = mono, 1.0 = wide stereo)
}

pub struct FeatureExtractor {
    sample_rate: f32,
}

impl FeatureExtractor {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Extract music features from an interleaved stereo buffer
    pub fn extract(&self, buffer: &[f32]) -> AudioFeatures {
        let n_frames = buffer.len() / 2;
        if n_frames < 16 {
            return AudioFeatures::default();
        }

        let mut sum_sq = 0.0f32;
        let mut diff_sq = 0.0f32;
        let mut peak = 0.0f32;
        let mut mid_sum_sq = 0.0f32;
        let mut side_sum_sq = 0.0f32;
        let mut zcr_count = 0usize;

        // 1-pole lowpass for sub-bass (< 150 Hz)
        let lp_alpha = (2.0 * PI * 150.0 / self.sample_rate).clamp(0.001, 0.99);
        // 1-pole highpass for high air (> 4 kHz)
        let hp_alpha = 1.0 / (1.0 + 2.0 * PI * 4000.0 / self.sample_rate);

        let mut lp_l = 0.0f32;
        let mut lp_r = 0.0f32;
        let mut hp_l = 0.0f32;
        let mut hp_r = 0.0f32;
        let mut sub_bass_energy = 0.0f32;
        let mut high_air_energy = 0.0f32;

        let mut prev_l = 0.0f32;
        let mut prev_r = 0.0f32;
        let mut prev_sign = buffer[0] >= 0.0;

        for i in 0..n_frames {
            let l = buffer[i * 2];
            let r = buffer[i * 2 + 1];
            let mono = 0.5 * (l + r);
            let side = 0.5 * (l - r);

            let abs_l = l.abs();
            let abs_r = r.abs();
            if abs_l > peak { peak = abs_l; }
            if abs_r > peak { peak = abs_r; }

            let frame_energy = 0.5 * (l * l + r * r);
            sum_sq += frame_energy;
            mid_sum_sq += mono * mono;
            side_sum_sq += side * side;

            // Stereo spectral forward derivative: 0.5 * (|Δl|^2 + |Δr|^2)
            let dl = l - prev_l;
            let dr = r - prev_r;
            diff_sq += 0.5 * (dl * dl + dr * dr);

            // Sub-bass 1-pole lowpass
            lp_l += lp_alpha * (l - lp_l);
            lp_r += lp_alpha * (r - lp_r);
            sub_bass_energy += 0.5 * (lp_l * lp_l + lp_r * lp_r);

            // High air 1-pole highpass: y[n] = alpha * (y[n-1] + x[n] - x[n-1])
            hp_l = hp_alpha * (hp_l + l - prev_l);
            hp_r = hp_alpha * (hp_r + r - prev_r);
            high_air_energy += 0.5 * (hp_l * hp_l + hp_r * hp_r);

            prev_l = l;
            prev_r = r;

            let cur_sign = mono >= 0.0;
            if cur_sign != prev_sign {
                zcr_count += 1;
                prev_sign = cur_sign;
            }
        }

        let total_frames = n_frames as f32;
        let rms = (sum_sq / total_frames).sqrt().max(1e-6);
        let crest = (peak / rms).clamp(1.0, 20.0);

        let mid_energy = mid_sum_sq.max(1e-7);
        let side_energy = side_sum_sq.max(1e-7);
        let stereo_width = (side_energy / (mid_energy + side_energy) * 2.0).clamp(0.0, 2.0);
        let zcr = zcr_count as f32 / total_frames;

        // Stereo Spectral Centroid: f_c = (Fs / 2pi) * sqrt(E_diff / E_total)
        let normalized_diff = (diff_sq / sum_sq.max(1e-7)).sqrt();
        let centroid = (normalized_diff * (self.sample_rate / (2.0 * PI))).clamp(40.0, 18000.0);

        let sub_bass_ratio = (sub_bass_energy / sum_sq.max(1e-7)).clamp(0.0, 1.0);
        let high_air_ratio = (high_air_energy / sum_sq.max(1e-7)).clamp(0.0, 1.0);
        let mid_vocal_ratio = (1.0 - sub_bass_ratio - high_air_ratio).clamp(0.0, 1.0);

        AudioFeatures {
            rms_level: rms,
            peak_level: peak,
            crest_factor: crest,
            spectral_centroid: centroid,
            sub_bass_ratio,
            mid_vocal_ratio,
            high_air_ratio,
            zero_crossing_rate: zcr,
            stereo_width,
        }
    }
}
