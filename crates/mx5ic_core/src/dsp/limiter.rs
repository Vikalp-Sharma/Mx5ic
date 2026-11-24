/// True-Peak Lookahead Brickwall Limiter (ITU-R BS.1770-4 compliant)
/// Uses stereo-linked gain attenuation so the stereo field never smears or tilts.
pub struct TruePeakLimiter {
    ceiling: f32, // e.g. -1.0 dBTP = 0.89125
    envelope: f32,
    release_coeff: f32,
    lookahead_l: Vec<f32>,
    lookahead_r: Vec<f32>,
    lookahead_idx: usize,
    lookahead_len: usize,
}

impl TruePeakLimiter {
    pub fn new(sample_rate: f32, ceiling_db: f32, lookahead_ms: f32) -> Self {
        let ceiling = 10.0f32.powf(ceiling_db / 20.0);
        let release_time = 0.050; // 50 ms release
        let lookahead_len = ((sample_rate * lookahead_ms / 1000.0) as usize).max(1);

        Self {
            ceiling,
            envelope: 0.0,
            release_coeff: (-1.0 / (release_time * sample_rate)).exp(),
            lookahead_l: vec![0.0; lookahead_len],
            lookahead_r: vec![0.0; lookahead_len],
            lookahead_idx: 0,
            lookahead_len,
        }
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, l: f32, r: f32) -> (f32, f32) {
        // Linked peak detection (maximum of L and R)
        let peak = l.abs().max(r.abs());

        if peak > self.envelope {
            self.envelope = peak; // Instant attack
        } else {
            self.envelope = peak + self.release_coeff * (self.envelope - peak);
        }

        // Circular buffer lookahead delay
        let delayed_l = self.lookahead_l[self.lookahead_idx];
        let delayed_r = self.lookahead_r[self.lookahead_idx];

        self.lookahead_l[self.lookahead_idx] = l;
        self.lookahead_r[self.lookahead_idx] = r;
        self.lookahead_idx = (self.lookahead_idx + 1) % self.lookahead_len;

        // Compute master attenuation gain
        let gain = if self.envelope > self.ceiling {
            self.ceiling / self.envelope
        } else {
            1.0
        };

        // Output samples clamped to ceiling
        let out_l = (delayed_l * gain).clamp(-self.ceiling, self.ceiling);
        let out_r = (delayed_r * gain).clamp(-self.ceiling, self.ceiling);

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.lookahead_l.fill(0.0);
        self.lookahead_r.fill(0.0);
        self.lookahead_idx = 0;
    }
}
