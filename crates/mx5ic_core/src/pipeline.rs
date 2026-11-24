use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use crate::dsp::*;
use crate::ai::{FeatureExtractor, MusicIntelligenceEngine, MusicUnderstanding};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EnhancerMode {
    Live,   // <= 40 ms latency, lightweight chain
    Studio, // <= 300 ms latency, maximal neural quality
}

pub struct EngineConfig {
    pub sample_rate: f32,
    pub block_size: usize,
    pub mode: EnhancerMode,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000.0,
            block_size: 128,
            mode: EnhancerMode::Live,
        }
    }
}

/// Shared atomic controls accessible from UI thread without locks
pub struct EngineControls {
    pub bypass: AtomicBool,
    pub master_gain_raw: AtomicU32,     // f32 bits
    pub vb_mix_raw: AtomicU32,          // f32 bits
    pub vb_drive_raw: AtomicU32,        // f32 bits
    pub spatial_3d_raw: AtomicU32,      // f32 bits (0.0 to 1.0)
    pub orbit_speed_raw: AtomicU32,     // f32 bits (0.1 to 2.5 Hz)
    pub crossfeed_raw: AtomicU32,       // f32 bits (0.0 to 1.0)
    pub dialogue_boost_raw: AtomicU32,  // f32 bits (0.0 to 1.0)
    pub noise_cancellation: AtomicBool, // Adaptive noise cancellation
    pub auto_ai_adaptive: AtomicBool,   // Automatically adapt DSP to detected genre/instruments
}

impl EngineControls {
    pub fn new() -> Self {
        Self {
            bypass: AtomicBool::new(false),
            master_gain_raw: AtomicU32::new(1.0f32.to_bits()),     // 0 dB
            vb_mix_raw: AtomicU32::new(0.40f32.to_bits()),         // 40% mix
            vb_drive_raw: AtomicU32::new(2.2f32.to_bits()),        // 2.2 drive
            spatial_3d_raw: AtomicU32::new(0.65f32.to_bits()),     // 65% 3D spatial orbit
            orbit_speed_raw: AtomicU32::new(0.35f32.to_bits()),    // 0.35 Hz orbit speed
            crossfeed_raw: AtomicU32::new(0.35f32.to_bits()),      // 35% binaural crossfeed
            dialogue_boost_raw: AtomicU32::new(0.0f32.to_bits()),  // 0.0 dB
            noise_cancellation: AtomicBool::new(true),             // Active noise cancellation on
            auto_ai_adaptive: AtomicBool::new(true),               // AI Adaptive enabled by default
        }
    }

    pub fn set_bypass(&self, val: bool) {
        self.bypass.store(val, Ordering::Release);
    }

    pub fn is_bypass(&self) -> bool {
        self.bypass.load(Ordering::Acquire)
    }

    pub fn set_master_gain(&self, gain_linear: f32) {
        self.master_gain_raw.store(gain_linear.to_bits(), Ordering::Release);
    }

    pub fn master_gain(&self) -> f32 {
        f32::from_bits(self.master_gain_raw.load(Ordering::Acquire))
    }

    pub fn set_virtual_bass(&self, mix: f32, drive: f32) {
        self.vb_mix_raw.store(mix.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.vb_drive_raw.store(drive.clamp(1.0, 4.0).to_bits(), Ordering::Release);
    }

    pub fn set_spatial_3d(&self, amount: f32, speed_hz: f32) {
        self.spatial_3d_raw.store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Release);
        self.orbit_speed_raw.store(speed_hz.clamp(0.1, 2.5).to_bits(), Ordering::Release);
    }

    pub fn set_crossfeed(&self, amount: f32) {
        self.crossfeed_raw.store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    pub fn set_dialogue_boost(&self, amount: f32) {
        self.dialogue_boost_raw.store(amount.clamp(0.0, 1.0).to_bits(), Ordering::Release);
    }

    pub fn is_noise_cancellation(&self) -> bool {
        self.noise_cancellation.load(Ordering::Acquire)
    }

    pub fn set_noise_cancellation(&self, val: bool) {
        self.noise_cancellation.store(val, Ordering::Release);
    }

    pub fn is_auto_adaptive(&self) -> bool {
        self.auto_ai_adaptive.load(Ordering::Acquire)
    }

    pub fn set_auto_adaptive(&self, val: bool) {
        self.auto_ai_adaptive.store(val, Ordering::Release);
    }
}

pub struct EnginePipeline {
    #[allow(dead_code)]
    config: EngineConfig,
    controls: Arc<EngineControls>,
    noise_gate: AdaptiveNoiseGate,
    eq: ParametricEq,
    dialogue: DialogueEnhancer,
    virtual_bass: VirtualBass,
    spatial_orbit: SpatialVocalOrbit,
    crossfader: EqualPowerCrossfader,
    limiter: TruePeakLimiter,
    feature_extractor: FeatureExtractor,
    ai_engine: MusicIntelligenceEngine,
    latest_understanding: Option<MusicUnderstanding>,
    telemetry: TelemetryCollector,
    latest_telemetry: AudioTelemetry,
    telemetry_interval: usize,
    ai_analysis_interval: usize,
    sample_counter: usize,
    ai_counter: usize,
    model_manifest: Option<crate::ai::ModelSuiteManifest>,
}

impl EnginePipeline {
    pub fn new(config: EngineConfig, controls: Arc<EngineControls>) -> Self {
        let sr = config.sample_rate;
        let model_manifest = crate::ai::ModelSuiteManager::discover_and_load();
        if let Some(ref m) = model_manifest {
            println!(
                "[+] Merged Deep AI Suite active: {:.2} MB ({} models loaded from {:?})",
                m.total_size_mb,
                m.loaded_models.len(),
                m.model_dir
            );
        }
        Self {
            config,
            controls,
            noise_gate: AdaptiveNoiseGate::new(sr),
            eq: ParametricEq::new(sr),
            dialogue: DialogueEnhancer::new(sr),
            virtual_bass: VirtualBass::new(sr),
            spatial_orbit: SpatialVocalOrbit::new(sr),
            crossfader: EqualPowerCrossfader::new(sr, 30.0), // 30 ms equal-power crossfade
            limiter: TruePeakLimiter::new(sr, -1.0, 2.0),   // -1.0 dBTP ceiling, 2ms lookahead
            feature_extractor: FeatureExtractor::new(sr),
            ai_engine: MusicIntelligenceEngine::new(sr),
            latest_understanding: None,
            telemetry: TelemetryCollector::new(sr, 50.0),   // 50ms window
            latest_telemetry: AudioTelemetry::default(),
            telemetry_interval: (sr * 0.030) as usize,      // update telemetry ~33 Hz
            ai_analysis_interval: (sr * 0.200) as usize,    // update AI intelligence analysis ~5 Hz
            sample_counter: 0,
            ai_counter: 0,
            model_manifest,
        }
    }

    pub fn model_manifest(&self) -> Option<&crate::ai::ModelSuiteManifest> {
        self.model_manifest.as_ref()
    }

    pub fn set_eq_band(&mut self, band: usize, gain_db: f32) {
        self.eq.set_band(band, gain_db, None, None);
    }

    pub fn apply_sound_profile(&mut self, profile: SoundProfile) {
        let gains = profile.target_eq_gains();
        for (i, &g) in gains.iter().enumerate() {
            self.eq.set_band(i, g, None, None);
        }
        let (vb_mix, vb_drive) = profile.virtual_bass_params();
        self.controls.set_virtual_bass(vb_mix, vb_drive);
        self.controls.set_spatial_3d(profile.default_spatial_orbit(), 0.35);
        self.controls.set_crossfeed(profile.crossfeed_amount());
        self.noise_gate.reduction_db = profile.noise_reduction_db();
    }

    pub fn get_ai_understanding(&self) -> Option<&MusicUnderstanding> {
        self.latest_understanding.as_ref()
    }

    /// Real-time audio processing block: strictly zero allocations, lock-free!
    #[inline(always)]
    pub fn process_interleaved(&mut self, input: &[f32], output: &mut [f32]) {
        assert_eq!(input.len(), output.len());
        let num_frames = input.len() / 2;

        // Zero Silence Guarantee & Squelch Gate:
        // When input is digital silence or below noise floor (-66 dBFS = 0.0005),
        // instantly output pure digital silence and flush all feedback/delay states.
        let mut block_peak = 0.0f32;
        for &s in input {
            let a = s.abs();
            if a > block_peak {
                block_peak = a;
            }
        }

        if block_peak < 0.0005 {
            self.noise_gate.reset();
            self.spatial_orbit.reset();
            self.dialogue.reset();
            self.virtual_bass.reset();
            self.eq.reset();
            self.limiter.reset();
            output.fill(0.0);
            return;
        }

        let is_bypassed = self.controls.is_bypass();
        self.crossfader.set_target(!is_bypassed);

        let master_gain = self.controls.master_gain();
        self.noise_gate.enabled = self.controls.is_noise_cancellation();
        self.dialogue.set_amount(f32::from_bits(self.controls.dialogue_boost_raw.load(Ordering::Acquire)));
        self.virtual_bass.mix = f32::from_bits(self.controls.vb_mix_raw.load(Ordering::Acquire));
        self.virtual_bass.drive = f32::from_bits(self.controls.vb_drive_raw.load(Ordering::Acquire));
        self.spatial_orbit.amount = f32::from_bits(self.controls.spatial_3d_raw.load(Ordering::Acquire));
        self.spatial_orbit.orbit_speed_hz = f32::from_bits(self.controls.orbit_speed_raw.load(Ordering::Acquire));
        self.spatial_orbit.crossfeed_mix = f32::from_bits(self.controls.crossfeed_raw.load(Ordering::Acquire));

        for i in 0..num_frames {
            let in_l = input[i * 2 + 0];
            let in_r = input[i * 2 + 1];

            // 1. Adaptive Spectral Noise Cancellation & De-Hiss
            let (clean_l, clean_r) = self.noise_gate.process_stereo(in_l, in_r);

            // 2. 8-Band Parametric EQ
            let (eq_l, eq_r) = self.eq.process_stereo(clean_l, clean_r);

            // 3. Speech and Dialogue Intelligibility Enhancement
            let (dia_l, dia_r) = self.dialogue.process_stereo(eq_l, eq_r);

            // 4. Psychoacoustic Virtual Bass Harmonic Exciter
            let (vb_l, vb_r) = self.virtual_bass.process_stereo(dia_l, dia_r);

            // 5. Dynamic 3D Spatial Vocal Orbit, Bauer Crossfeed & Pinna 3D
            let (sp_l, sp_r) = self.spatial_orbit.process_stereo(vb_l, vb_r);

            // Master gain
            let wet_l = sp_l * master_gain;
            let wet_r = sp_r * master_gain;

            // 6. Click-Free Crossfade between Dry (original) and Wet (processed)
            let (cross_l, cross_r) = self.crossfader.process_stereo(in_l, in_r, wet_l, wet_r);

            // 7. True-Peak Brickwall Limiter (-1.0 dBTP ceiling)
            let (out_l, out_r) = self.limiter.process_stereo(cross_l, cross_r);

            output[i * 2 + 0] = out_l;
            output[i * 2 + 1] = out_r;

            // Telemetry & AI analysis counter
            self.telemetry.push_sample(out_l, out_r);
            self.sample_counter += 1;
            self.ai_counter += 1;
        }

        if self.sample_counter >= self.telemetry_interval {
            self.latest_telemetry = self.telemetry.compute();
            self.sample_counter = 0;
        }

        // Periodic AI Music Understanding & Adaptive Decision Generation (~5 Hz)
        if self.ai_counter >= self.ai_analysis_interval {
            let feat = self.feature_extractor.extract(input);
            let und = self.ai_engine.analyze(&feat);
            if self.controls.is_auto_adaptive() {
                let dec = self.ai_engine.decide(&und);
                for (b, &g) in dec.eq_target_db.iter().enumerate() {
                    self.eq.set_band(b, g, None, None);
                }
                self.controls.set_virtual_bass(dec.virtual_bass_mix, dec.virtual_bass_drive);
                self.controls.set_spatial_3d(dec.spatial_orbit_amount, dec.spatial_orbit_speed_hz);
                if dec.dialogue_boost_amount > 0.05 {
                    self.controls.set_dialogue_boost(dec.dialogue_boost_amount);
                }
            }
            self.latest_understanding = Some(und);
            self.ai_counter = 0;
        }
    }

    pub fn get_telemetry(&self) -> AudioTelemetry {
        self.latest_telemetry
    }

    pub fn reset(&mut self) {
        self.noise_gate.reset();
        self.eq.reset();
        self.dialogue.reset();
        self.virtual_bass.reset();
        self.spatial_orbit.reset();
        self.limiter.reset();
        self.sample_counter = 0;
        self.ai_counter = 0;
    }
}
