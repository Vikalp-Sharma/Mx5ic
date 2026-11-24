use crate::ai::features::AudioFeatures;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicGenre {
    Pop,
    HipHop,
    EDM,
    Rock,
    Metal,
    Classical,
    Jazz,
    LoFi,
    Acoustic,
    Bollywood,
    Ambient,
}

impl MusicGenre {
    pub fn as_str(&self) -> &'static str {
        match self {
            MusicGenre::Pop => "Modern Pop",
            MusicGenre::HipHop => "Hip-Hop / Rap",
            MusicGenre::EDM => "Electronic / Dance",
            MusicGenre::Rock => "Rock / Alternative",
            MusicGenre::Metal => "Heavy Metal",
            MusicGenre::Classical => "Classical / Orchestral",
            MusicGenre::Jazz => "Jazz / Soul",
            MusicGenre::LoFi => "Lo-Fi Beats",
            MusicGenre::Acoustic => "Acoustic / Folk",
            MusicGenre::Bollywood => "Bollywood / Indian Pop",
            MusicGenre::Ambient => "Ambient / Cinematic",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InstrumentPresence {
    pub lead_vocals: bool,
    pub background_harmonies: bool,
    pub sub_bass: bool,
    pub kick_drum: bool,
    pub acoustic_or_electric_guitar: bool,
    pub synthesizer_layers: bool,
    pub acoustic_strings: bool,
}

#[derive(Clone, Debug)]
pub struct MusicUnderstanding {
    pub primary_genre: MusicGenre,
    pub genre_confidence: f32,
    pub instruments: InstrumentPresence,
    pub vocal_presence: f32,        // 0.0 to 1.0
    pub vocal_clarity_score: f32,   // 0.0 (muffled) to 1.0 (clear)
    pub bass_state: &'static str,   // "Weak", "Muddy", "Balanced", "Heavy"
    pub treble_state: &'static str, // "Dull", "Balanced", "Harsh"
}

#[derive(Clone, Debug)]
pub struct AdaptiveDecision {
    pub eq_target_db: [f32; 8],     // 8 bands
    pub virtual_bass_mix: f32,      // 0.0 to 1.0
    pub virtual_bass_drive: f32,    // 1.0 to 4.0
    pub spatial_orbit_amount: f32,  // 0.0 to 1.0
    pub spatial_orbit_speed_hz: f32,// 0.1 to 2.0 Hz
    pub dialogue_boost_amount: f32, // 0.0 to 1.0
}

pub struct MusicIntelligenceEngine {
    _sample_rate: f32,
}

impl MusicIntelligenceEngine {
    pub fn new(sample_rate: f32) -> Self {
        Self { _sample_rate: sample_rate }
    }

    /// Analyze incoming acoustic features and generate music understanding
    pub fn analyze(&self, feat: &AudioFeatures) -> MusicUnderstanding {
        // 1. Hierarchical genre classification
        let (primary_genre, confidence) = if feat.sub_bass_ratio > 0.35 {
            if feat.high_air_ratio > 0.03 || feat.spectral_centroid > 2500.0 {
                (MusicGenre::EDM, 0.92)
            } else {
                (MusicGenre::HipHop, 0.88)
            }
        } else if feat.spectral_centroid > 3200.0 && feat.crest_factor < 2.8 {
            (MusicGenre::Metal, 0.84)
        } else if feat.spectral_centroid > 2600.0 && feat.stereo_width > 0.6 {
            (MusicGenre::Rock, 0.81)
        } else if feat.crest_factor > 5.5 && feat.stereo_width > 0.8 {
            (MusicGenre::Classical, 0.94)
        } else if feat.mid_vocal_ratio > 0.40 && feat.stereo_width < 0.4 {
            (MusicGenre::Acoustic, 0.86)
        } else if feat.spectral_centroid < 1200.0 && feat.crest_factor < 3.2 && feat.high_air_ratio < 0.05 {
            (MusicGenre::LoFi, 0.89)
        } else if feat.mid_vocal_ratio > 0.45 && feat.stereo_width > 0.55 {
            (MusicGenre::Bollywood, 0.87)
        } else {
            (MusicGenre::Pop, 0.85)
        };

        // 2. Instrument presence detection
        let has_vocals = feat.mid_vocal_ratio > 0.28;
        let has_bg_harmonies = has_vocals && feat.stereo_width > 0.65;
        let has_sub_bass = feat.sub_bass_ratio > 0.22;
        let has_kick = feat.crest_factor > 3.2;
        let has_guitar = (primary_genre == MusicGenre::Rock || primary_genre == MusicGenre::Acoustic) && feat.spectral_centroid > 1500.0;
        let has_synths = (primary_genre == MusicGenre::EDM || primary_genre == MusicGenre::Pop) && feat.high_air_ratio > 0.08;
        let has_strings = primary_genre == MusicGenre::Classical || primary_genre == MusicGenre::Bollywood;

        let instruments = InstrumentPresence {
            lead_vocals: has_vocals,
            background_harmonies: has_bg_harmonies,
            sub_bass: has_sub_bass,
            kick_drum: has_kick,
            acoustic_or_electric_guitar: has_guitar,
            synthesizer_layers: has_synths,
            acoustic_strings: has_strings,
        };

        // 3. Acoustic evaluations
        let bass_state = if feat.sub_bass_ratio < 0.15 {
            "Weak"
        } else if feat.sub_bass_ratio > 0.55 && feat.crest_factor < 2.5 {
            "Muddy"
        } else if feat.sub_bass_ratio > 0.38 {
            "Heavy"
        } else {
            "Balanced"
        };

        let treble_state = if feat.high_air_ratio < 0.04 {
            "Dull"
        } else if feat.high_air_ratio > 0.35 && feat.spectral_centroid > 4500.0 {
            "Harsh"
        } else {
            "Balanced"
        };

        let vocal_presence = if has_vocals { (feat.mid_vocal_ratio * 1.8).clamp(0.4, 1.0) } else { 0.1 };
        let vocal_clarity = (feat.spectral_centroid / 4000.0).clamp(0.2, 0.95);

        MusicUnderstanding {
            primary_genre,
            genre_confidence: confidence,
            instruments,
            vocal_presence,
            vocal_clarity_score: vocal_clarity,
            bass_state,
            treble_state,
        }
    }

    /// Compute adaptive enhancement decisions based on the music understanding
    pub fn decide(&self, und: &MusicUnderstanding) -> AdaptiveDecision {
        let (mut eq, mut vb_mix, mut vb_drive, spatial_orbit, orbit_speed, dialogue_boost) = match und.primary_genre {
            MusicGenre::Pop => (
                [2.0, 1.0, 0.0, 0.5, 2.2, 0.0, 1.5, 1.0],
                0.35, 2.2, 0.55, 0.45, 0.0
            ),
            MusicGenre::HipHop => (
                [4.5, 2.0, -1.0, 0.0, 1.8, 0.0, 0.8, 0.5],
                0.48, 2.6, 0.35, 0.35, 0.0
            ),
            MusicGenre::EDM => (
                [4.0, 1.5, -0.5, 0.0, 1.0, 0.5, 2.0, 1.8],
                0.42, 2.4, 0.65, 0.60, 0.0
            ),
            MusicGenre::Rock => (
                [1.5, 2.0, 0.0, 1.0, 1.5, -0.5, 1.0, 0.8],
                0.25, 1.8, 0.30, 0.30, 0.0
            ),
            MusicGenre::Metal => (
                [2.0, 1.0, -1.5, 0.5, 1.2, -1.5, 0.5, 0.0],
                0.20, 1.6, 0.25, 0.25, 0.0
            ),
            MusicGenre::Classical => (
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5],
                0.05, 1.1, 0.20, 0.15, 0.0
            ),
            MusicGenre::Jazz => (
                [1.5, 1.0, 0.5, 0.0, 0.8, 0.0, 0.8, 0.5],
                0.22, 1.5, 0.35, 0.25, 0.0
            ),
            MusicGenre::LoFi => (
                [2.5, 1.5, 0.5, 0.0, -0.5, -1.0, -1.5, -2.0],
                0.40, 2.2, 0.30, 0.20, 0.0
            ),
            MusicGenre::Acoustic => (
                [1.0, 1.5, 0.0, 0.5, 1.8, 0.0, 1.2, 1.0],
                0.18, 1.4, 0.40, 0.30, 0.0
            ),
            MusicGenre::Bollywood => (
                [3.0, 1.8, -0.5, 1.0, 2.5, 0.0, 1.5, 1.2],
                0.38, 2.3, 0.70, 0.42, 0.25
            ),
            MusicGenre::Ambient => (
                [2.0, 0.5, 0.0, 0.0, 0.5, 0.5, 1.5, 2.0],
                0.28, 1.8, 0.75, 0.18, 0.0
            ),
        };

        // Adaptive fine-tuning based on detected acoustic states
        if und.bass_state == "Weak" {
            eq[0] += 2.0;
            vb_mix = (vb_mix + 0.15f32).min(0.8f32);
        } else if und.bass_state == "Muddy" {
            eq[1] -= 2.0; // Clean up 150 Hz mud
            vb_drive = (vb_drive - 0.5f32).max(1.0f32);
        }

        if und.treble_state == "Dull" {
            eq[6] += 2.0;
            eq[7] += 2.5;
        } else if und.treble_state == "Harsh" {
            eq[5] -= 2.0; // Soften 5 kHz sibilance
            eq[6] -= 1.5;
        }

        AdaptiveDecision {
            eq_target_db: eq,
            virtual_bass_mix: vb_mix,
            virtual_bass_drive: vb_drive,
            spatial_orbit_amount: spatial_orbit,
            spatial_orbit_speed_hz: orbit_speed,
            dialogue_boost_amount: dialogue_boost,
        }
    }
}
