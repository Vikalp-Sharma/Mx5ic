use std::f32::consts::PI;

/// Click-free Equal-Power Crossfader
/// Prevents pop transients and volume drops when toggling bypass or switching presets.
pub struct EqualPowerCrossfader {
    #[allow(dead_code)] current_state: bool, // true = active, false = bypassed
    target_state: bool,
    progress: f32,       // 0.0 (bypassed) to 1.0 (active)
    step: f32,
}

impl EqualPowerCrossfader {
    pub fn new(sample_rate: f32, fade_duration_ms: f32) -> Self {
        let fade_samples = (sample_rate * fade_duration_ms / 1000.0).max(1.0);
        Self {
            current_state: true,
            target_state: true,
            progress: 1.0,
            step: 1.0 / fade_samples,
        }
    }

    pub fn set_target(&mut self, active: bool) {
        self.target_state = active;
    }

    #[inline(always)]
    pub fn is_fading(&self) -> bool {
        (self.target_state && self.progress < 1.0) || (!self.target_state && self.progress > 0.0)
    }

    #[inline(always)]
    pub fn process_stereo(&mut self, dry_l: f32, dry_r: f32, wet_l: f32, wet_r: f32) -> (f32, f32) {
        if self.target_state && self.progress < 1.0 {
            self.progress = (self.progress + self.step).min(1.0);
        } else if !self.target_state && self.progress > 0.0 {
            self.progress = (self.progress - self.step).max(0.0);
        }

        if self.progress >= 1.0 {
            return (wet_l, wet_r);
        } else if self.progress <= 0.0 {
            return (dry_l, dry_r);
        }

        // Equal-power sinusoidal crossfade: cos^2(theta) + sin^2(theta) = 1.0
        let angle = self.progress * (PI * 0.5);
        let wet_gain = angle.sin();
        let dry_gain = angle.cos();

        (
            dry_l * dry_gain + wet_l * wet_gain,
            dry_r * dry_gain + wet_r * wet_gain,
        )
    }
}
