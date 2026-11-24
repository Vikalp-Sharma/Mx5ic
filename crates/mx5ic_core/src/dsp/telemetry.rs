use serde::{Serialize, Deserialize};

pub const NUM_SPECTRUM_BANDS: usize = 16;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AudioTelemetry {
    pub rms_db: f32,
    pub peak_db: f32,
    pub true_peak_db: f32,
    pub spectrum: [f32; NUM_SPECTRUM_BANDS],
}

impl Default for AudioTelemetry {
    fn default() -> Self {
        Self {
            rms_db: -90.0,
            peak_db: -90.0,
            true_peak_db: -90.0,
            spectrum: [0.0; NUM_SPECTRUM_BANDS],
        }
    }
}

pub struct TelemetryCollector {
    sum_squares: f32,
    max_peak: f32,
    sample_count: usize,
    #[allow(dead_code)] window_samples: usize,
    spectrum_energies: [f32; NUM_SPECTRUM_BANDS],
    decay: f32,
}

impl TelemetryCollector {
    pub fn new(sample_rate: f32, window_ms: f32) -> Self {
        let window_samples = ((sample_rate * window_ms / 1000.0) as usize).max(64);
        Self {
            sum_squares: 0.0,
            max_peak: 0.0,
            sample_count: 0,
            window_samples,
            spectrum_energies: [0.0; NUM_SPECTRUM_BANDS],
            decay: 0.92,
        }
    }

    #[inline(always)]
    pub fn push_sample(&mut self, l: f32, r: f32) {
        let mono = 0.5 * (l + r);
        let abs_val = mono.abs();

        self.sum_squares += mono * mono;
        if abs_val > self.max_peak {
            self.max_peak = abs_val;
        }

        self.sample_count += 1;
    }

    pub fn compute(&mut self) -> AudioTelemetry {
        let rms = if self.sample_count > 0 {
            (self.sum_squares / self.sample_count as f32).sqrt()
        } else {
            0.0
        };

        let rms_db = if rms > 1e-5 {
            20.0 * rms.log10()
        } else {
            -90.0
        };

        let peak_db = if self.max_peak > 1e-5 {
            20.0 * self.max_peak.log10()
        } else {
            -90.0
        };

        // Smooth decaying pseudo-spectrum bars for fluid liquid glass visualizer
        for i in 0..NUM_SPECTRUM_BANDS {
            let target = (self.max_peak * (1.0 - (i as f32 * 0.04))).max(0.0);
            self.spectrum_energies[i] = self.spectrum_energies[i] * self.decay + target * (1.0 - self.decay);
        }

        let result = AudioTelemetry {
            rms_db: rms_db.clamp(-90.0, 6.0),
            peak_db: peak_db.clamp(-90.0, 6.0),
            true_peak_db: peak_db.clamp(-90.0, 6.0),
            spectrum: self.spectrum_energies,
        };

        // Reset accumulation
        self.sum_squares = 0.0;
        self.max_peak = 0.0;
        self.sample_count = 0;

        result
    }
}
