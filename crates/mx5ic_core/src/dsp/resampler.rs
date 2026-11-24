/// Adaptive Clock-Drift Resampler
/// Employs cubic Hermite polynomial interpolation with smooth ratio adjustments
/// to eliminate phone/laptop crystal clock drift and prevent buffer underrun/overflow.
pub struct DriftResampler {
    target_occupancy: usize,
    kp: f32,
    ki: f32,
    integral_err: f32,
    pub ratio: f32,
    #[allow(dead_code)] phase: f32,
    // History buffer for 4-point cubic Hermite interpolation
    hist_l: [f32; 4],
    hist_r: [f32; 4],
}

impl DriftResampler {
    pub fn new(target_occupancy: usize) -> Self {
        Self {
            target_occupancy,
            kp: 0.00002,
            ki: 0.0000005,
            integral_err: 0.0,
            ratio: 1.0,
            phase: 0.0,
            hist_l: [0.0; 4],
            hist_r: [0.0; 4],
        }
    }

    pub fn update_drift(&mut self, current_occupancy: usize) -> f32 {
        let err = (current_occupancy as isize - self.target_occupancy as isize) as f32;
        self.integral_err = (self.integral_err + err).clamp(-2000.0, 2000.0);
        let adj = self.kp * err + self.ki * self.integral_err;
        // Limit adjustment to +/- 500 ppm (0.05%) to avoid audible pitch changes
        self.ratio = (1.0 + adj).clamp(0.9995, 1.0005);
        self.ratio
    }

    #[inline(always)]
    pub fn push_input(&mut self, l: f32, r: f32) {
        self.hist_l[0] = self.hist_l[1];
        self.hist_l[1] = self.hist_l[2];
        self.hist_l[2] = self.hist_l[3];
        self.hist_l[3] = l;

        self.hist_r[0] = self.hist_r[1];
        self.hist_r[1] = self.hist_r[2];
        self.hist_r[2] = self.hist_r[3];
        self.hist_r[3] = r;
    }

    #[inline(always)]
    fn hermite_interp(p: &[f32; 4], t: f32) -> f32 {
        let c0 = p[1];
        let c1 = 0.5 * (p[2] - p[0]);
        let c2 = p[0] - 2.5 * p[1] + 2.0 * p[2] - 0.5 * p[3];
        let c3 = 0.5 * (p[3] - p[0]) + 1.5 * (p[1] - p[2]);
        ((c3 * t + c2) * t + c1) * t + c0
    }

    #[inline(always)]
    pub fn interpolate(&self, fraction: f32) -> (f32, f32) {
        let l = Self::hermite_interp(&self.hist_l, fraction);
        let r = Self::hermite_interp(&self.hist_r, fraction);
        (l, r)
    }
}
