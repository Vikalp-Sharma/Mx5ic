use std::f32::consts::PI;
use super::biquad::{Biquad, FilterType};

const ITD_MAX_SAMPLES: usize = 64;       // Max ~1.3 ms (human ear-to-ear max ITD is ~0.65 ms = 31 samples @ 48kHz)
const DIFFUSION_BUF_SIZE: usize = 1024;  // ~21 ms max for early room diffusion (strictly under Haas threshold)
const CROSSFEED_DELAY_SAMPLES: usize = 16; // ~300 microseconds for natural speaker crossfeed

/// Dynamic 3D Spatial Vocal Orbit & Binaural Surround Sound Engine
///
/// Features:
/// 1. Interaural Time Delay (ITD <= 0.55 ms = 26 samples).
/// 2. Interaural Level Difference (ILD head-shadow high-shelf filtering @ 2.8 kHz).
/// 3. Bauer/Chu Binaural Crossfeed: Simulates authentic acoustic speaker listening on earbuds.
/// 4. Pinna Anatomical Notch (7.2 kHz): Enhances front-to-back acoustic spatial localization.
/// 5. Sub-10ms Haas Early Room Diffusion: Natural acoustic spatial field without slapback echoes.
pub struct SpatialVocalOrbit {
    sample_rate: f32,
    pub enabled: bool,
    pub amount: f32,           // 0.0 (Centered stereo) to 1.0 (Full 3D orbit)
    pub orbit_speed_hz: f32,   // Orbit rotation frequency (e.g. 0.35 Hz ~ 2.8s period)
    pub crossfeed_mix: f32,    // 0.0 to 1.0 crossfeed intensity
    pub surround_width: f32,   // 0.5 to 1.5 stereo field expansion

    // Vocal presence bandpass filter (300 Hz - 3500 Hz)
    vocal_filter: Biquad,

    // Head-shadow filter for contralateral ear (attenuates > 2500 Hz on far ear)
    head_shadow_l: Biquad,
    head_shadow_r: Biquad,

    // Pinna notch filters (7.2 kHz, Q=2.5, -4 dB) for anatomical front/overhead localization
    pinna_filter_l: Biquad,
    pinna_filter_r: Biquad,

    // Chu/Bauer crossfeed lowpass filters (700 Hz gentle low shelf)
    crossfeed_filter_l: Biquad,
    crossfeed_filter_r: Biquad,

    // High-precision circular buffers for ITD micro-delay (0 - 32 samples)
    itd_buf_l: [f32; ITD_MAX_SAMPLES],
    itd_buf_r: [f32; ITD_MAX_SAMPLES],
    itd_write_idx: usize,

    // Crossfeed delay lines (~13 samples @ 48kHz ~ 0.27 ms)
    xf_buf_l: [f32; CROSSFEED_DELAY_SAMPLES],
    xf_buf_r: [f32; CROSSFEED_DELAY_SAMPLES],
    xf_write_idx: usize,

    // Haas diffusion lines (5.2 ms and 8.7 ms) for 3D room dimension without echo
    diff_buf_l: [f32; DIFFUSION_BUF_SIZE],
    diff_buf_r: [f32; DIFFUSION_BUF_SIZE],
    diff_write_idx: usize,
    diff_len_l: usize,
    diff_len_r: usize,

    // LFO orbit phase and envelope detector
    phase: f32,
    envelope: f32,
}

impl SpatialVocalOrbit {
    pub fn new(sample_rate: f32) -> Self {
        let mut vf = Biquad::new();
        vf.set_params(FilterType::BandPass, sample_rate, 1500.0, 0.0, 0.8);

        let mut hs_l = Biquad::new();
        let mut hs_r = Biquad::new();
        hs_l.set_params(FilterType::HighShelf, sample_rate, 2800.0, -5.0, 0.707);
        hs_r.set_params(FilterType::HighShelf, sample_rate, 2800.0, -5.0, 0.707);

        let mut pinna_l = Biquad::new();
        let mut pinna_r = Biquad::new();
        pinna_l.set_params(FilterType::PeakingEQ, sample_rate, 7200.0, -3.5, 2.2);
        pinna_r.set_params(FilterType::PeakingEQ, sample_rate, 7200.0, -3.5, 2.2);

        let mut xf_l = Biquad::new();
        let mut xf_r = Biquad::new();
        xf_l.set_params(FilterType::LowPass, sample_rate, 700.0, 0.0, 0.707);
        xf_r.set_params(FilterType::LowPass, sample_rate, 700.0, 0.0, 0.707);

        // Sub-10ms Haas early reflection delays: 5.2 ms and 8.7 ms
        let diff_len_l = ((sample_rate * 0.0052) as usize).clamp(1, DIFFUSION_BUF_SIZE - 1);
        let diff_len_r = ((sample_rate * 0.0087) as usize).clamp(1, DIFFUSION_BUF_SIZE - 1);

        Self {
            sample_rate,
            enabled: true,
            amount: 0.55,
            orbit_speed_hz: 0.35,
            crossfeed_mix: 0.30,
            surround_width: 1.0,
            vocal_filter: vf,
            head_shadow_l: hs_l,
            head_shadow_r: hs_r,
            pinna_filter_l: pinna_l,
            pinna_filter_r: pinna_r,
            crossfeed_filter_l: xf_l,
            crossfeed_filter_r: xf_r,
            itd_buf_l: [0.0; ITD_MAX_SAMPLES],
            itd_buf_r: [0.0; ITD_MAX_SAMPLES],
            itd_write_idx: 0,
            xf_buf_l: [0.0; CROSSFEED_DELAY_SAMPLES],
            xf_buf_r: [0.0; CROSSFEED_DELAY_SAMPLES],
            xf_write_idx: 0,
            diff_buf_l: [0.0; DIFFUSION_BUF_SIZE],
            diff_buf_r: [0.0; DIFFUSION_BUF_SIZE],
            diff_write_idx: 0,
            diff_len_l,
            diff_len_r,
            phase: 0.0,
            envelope: 0.0,
        }
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, in_l: f32, in_r: f32) -> (f32, f32) {
        if !self.enabled || (self.amount <= 0.001 && self.crossfeed_mix <= 0.001) {
            return (in_l, in_r);
        }

        // Noise gate / silence squelch check: if input and internal energy are inaudible, return input
        if in_l.abs() < 1e-5 && in_r.abs() < 1e-5 && self.envelope < 1e-5 {
            return (in_l, in_r);
        }

        // 1. Vocal Energy Extraction & Envelope
        let mid = (in_l + in_r) * 0.5;
        let side = (in_l - in_r) * 0.5 * self.surround_width;
        let vocal_band = self.vocal_filter.process(mid);

        let abs_vocal = vocal_band.abs();
        if abs_vocal > self.envelope {
            self.envelope = abs_vocal * 0.2 + self.envelope * 0.8;
        } else {
            self.envelope *= 0.9995;
        }

        // 2. Dynamic 3D Spatial Pan Orbit (azimuth angle)
        let pan_pos = (self.phase * 2.0 * PI).sin();
        self.phase += self.orbit_speed_hz / self.sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Constant-power panning law
        let pan_angle = (pan_pos + 1.0) * (PI * 0.25); // 0 (full left) to PI/2 (full right)
        let direct_gain_l = pan_angle.cos();
        let direct_gain_r = pan_angle.sin();

        // 3. Write to ITD & Crossfeed Delay Lines
        self.itd_buf_l[self.itd_write_idx] = in_l;
        self.itd_buf_r[self.itd_write_idx] = in_r;

        self.xf_buf_l[self.xf_write_idx] = in_l;
        self.xf_buf_r[self.xf_write_idx] = in_r;

        // 4. Calculate ITD (Interaural Time Difference):
        let max_itd_samples = ((self.sample_rate * 0.00055) as usize).min(30);
        let itd_offset = (pan_pos.abs() * max_itd_samples as f32) as usize;

        let (binaural_l, binaural_r) = if pan_pos < 0.0 {
            // Panned towards LEFT earbud
            let read_idx_r = (self.itd_write_idx + ITD_MAX_SAMPLES - itd_offset) % ITD_MAX_SAMPLES;
            let delayed_r = self.itd_buf_r[read_idx_r];
            let shadow_r = self.head_shadow_r.process(delayed_r);
            let elevated_l = self.pinna_filter_l.process(in_l);
            (elevated_l * (1.0 + 0.3 * pan_pos.abs()), shadow_r * (1.0 - 0.2 * pan_pos.abs()))
        } else {
            // Panned towards RIGHT earbud
            let read_idx_l = (self.itd_write_idx + ITD_MAX_SAMPLES - itd_offset) % ITD_MAX_SAMPLES;
            let delayed_l = self.itd_buf_l[read_idx_l];
            let shadow_l = self.head_shadow_l.process(delayed_l);
            let elevated_r = self.pinna_filter_r.process(in_r);
            (shadow_l * (1.0 - 0.2 * pan_pos), elevated_r * (1.0 + 0.3 * pan_pos))
        };

        self.itd_write_idx = (self.itd_write_idx + 1) % ITD_MAX_SAMPLES;

        // 5. Chu/Bauer Binaural Crossfeed: Contralateral acoustic bleed for headphone naturalness
        let xf_delay_len = ((self.sample_rate * 0.00028) as usize).clamp(1, CROSSFEED_DELAY_SAMPLES - 1);
        let xf_read_idx = (self.xf_write_idx + CROSSFEED_DELAY_SAMPLES - xf_delay_len) % CROSSFEED_DELAY_SAMPLES;
        let delayed_xf_l = self.xf_buf_l[xf_read_idx];
        let delayed_xf_r = self.xf_buf_r[xf_read_idx];
        self.xf_write_idx = (self.xf_write_idx + 1) % CROSSFEED_DELAY_SAMPLES;

        let xf_bleed_l = self.crossfeed_filter_r.process(delayed_xf_r) * 0.32;
        let xf_bleed_r = self.crossfeed_filter_l.process(delayed_xf_l) * 0.32;

        // 6. Haas Early Reflection Diffusion (3D Depth without separate echo)
        let diff_read_l = (self.diff_write_idx + DIFFUSION_BUF_SIZE - self.diff_len_l) % DIFFUSION_BUF_SIZE;
        let diff_read_r = (self.diff_write_idx + DIFFUSION_BUF_SIZE - self.diff_len_r) % DIFFUSION_BUF_SIZE;

        let early_l = self.diff_buf_l[diff_read_l];
        let early_r = self.diff_buf_r[diff_read_r];

        self.diff_buf_l[self.diff_write_idx] = side * 0.35;
        self.diff_buf_r[self.diff_write_idx] = -side * 0.35;
        self.diff_write_idx = (self.diff_write_idx + 1) % DIFFUSION_BUF_SIZE;

        // 7. Blend: Master 3D Spatial + Crossfeed + Dry Acoustic Field
        let spat_mix = self.amount.clamp(0.0, 1.0);
        let xf_mix = self.crossfeed_mix.clamp(0.0, 1.0);

        let dry_l = in_l * (1.0 - xf_mix * 0.25) + xf_bleed_l * xf_mix;
        let dry_r = in_r * (1.0 - xf_mix * 0.25) + xf_bleed_r * xf_mix;

        let out_l = dry_l * (1.0 - spat_mix * 0.45) + (binaural_l * direct_gain_l + early_l * 0.25) * spat_mix;
        let out_r = dry_r * (1.0 - spat_mix * 0.45) + (binaural_r * direct_gain_r + early_r * 0.25) * spat_mix;

        (out_l, out_r)
    }

    pub fn reset(&mut self) {
        self.vocal_filter.reset();
        self.head_shadow_l.reset();
        self.head_shadow_r.reset();
        self.pinna_filter_l.reset();
        self.pinna_filter_r.reset();
        self.crossfeed_filter_l.reset();
        self.crossfeed_filter_r.reset();
        self.itd_buf_l.fill(0.0);
        self.itd_buf_r.fill(0.0);
        self.xf_buf_l.fill(0.0);
        self.xf_buf_r.fill(0.0);
        self.diff_buf_l.fill(0.0);
        self.diff_buf_r.fill(0.0);
        self.itd_write_idx = 0;
        self.xf_write_idx = 0;
        self.diff_write_idx = 0;
        self.envelope = 0.0;
    }
}
