#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundProfile {
    CinemaSurround3D,   // 1. Expansive 3D cinematic soundstage, subwoofer drive, immersive Haas diffusion
    ConcertHallLive,    // 2. Spherical binaural crossfeed, elevated hall air, natural live acoustic space
    VocalIntimacy,      // 3. Dialogue & vocal clarity front-centered, subtle orbiting backing field
    ClubBassReflex,     // 4. Punchy psychoacoustic sub-bass harmonics & sharpened dynamic transients
    NightAudiophile,    // 5. Smart evening loudness compression, sibilance de-essing, transparent de-noise
    StudioReference,    // 6. Bit-perfect flat target with subtle Bauer crossfeed for zero headphone fatigue
    HarmanConsumer,     // 7. Standard Harman acoustic target for mainstream earphones
}

impl SoundProfile {
    pub const ALL: [SoundProfile; 7] = [
        SoundProfile::CinemaSurround3D,
        SoundProfile::ConcertHallLive,
        SoundProfile::VocalIntimacy,
        SoundProfile::ClubBassReflex,
        SoundProfile::NightAudiophile,
        SoundProfile::StudioReference,
        SoundProfile::HarmanConsumer,
    ];

    pub fn from_index(idx: usize) -> Self {
        Self::ALL[idx % Self::ALL.len()]
    }

    pub fn to_index(&self) -> usize {
        match self {
            SoundProfile::CinemaSurround3D => 0,
            SoundProfile::ConcertHallLive => 1,
            SoundProfile::VocalIntimacy => 2,
            SoundProfile::ClubBassReflex => 3,
            SoundProfile::NightAudiophile => 4,
            SoundProfile::StudioReference => 5,
            SoundProfile::HarmanConsumer => 6,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            SoundProfile::CinemaSurround3D => "Cinema 3D Surround",
            SoundProfile::ConcertHallLive => "Concert Hall Live 3D",
            SoundProfile::VocalIntimacy => "Vocal Intimacy & Center",
            SoundProfile::ClubBassReflex => "Club Bass Reflex",
            SoundProfile::NightAudiophile => "Night Audiophile & Dialogue",
            SoundProfile::StudioReference => "Studio Reference Flat",
            SoundProfile::HarmanConsumer => "Harman Target (Consumer)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SoundProfile::CinemaSurround3D => "Expansive 3D cinematic soundstage with deep sub-bass crossover",
            SoundProfile::ConcertHallLive => "Spherical binaural crossfeed with natural live concert acoustics",
            SoundProfile::VocalIntimacy => "Front-anchored lead vocal presence with smooth stereo diffusion",
            SoundProfile::ClubBassReflex => "Maximum sub-harmonic punch and transient bass impact",
            SoundProfile::NightAudiophile => "Evening listening mode with low noise floor and dynamic control",
            SoundProfile::StudioReference => "Neutral acoustic response with gentle Bauer crossfeed",
            SoundProfile::HarmanConsumer => "Warm low-end and airy treble tailored for modern earphones",
        }
    }

    /// Returns 8-band EQ target gains in dB for the profile
    /// Bands: 60Hz, 150Hz, 400Hz, 1kHz, 2.4kHz, 5kHz, 10kHz, 16kHz
    pub fn target_eq_gains(&self) -> [f32; 8] {
        match self {
            SoundProfile::CinemaSurround3D => [4.2, 2.0, 0.0, -0.5, 1.2, 1.8, 2.5, 3.0],
            SoundProfile::ConcertHallLive => [1.8, 0.5, 0.0, 0.5, 1.0, 2.2, 3.2, 4.0],
            SoundProfile::VocalIntimacy => [1.0, -1.0, 0.5, 2.0, 3.8, 1.5, 0.5, 0.0],
            SoundProfile::ClubBassReflex => [6.5, 3.2, 0.5, -1.0, 0.0, 1.2, 2.0, 2.5],
            SoundProfile::NightAudiophile => [2.0, 1.0, 0.5, 1.2, 2.0, -1.5, -2.0, -3.0],
            SoundProfile::StudioReference => [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            SoundProfile::HarmanConsumer => [3.2, 1.5, 0.0, 0.5, 1.8, 0.0, 1.2, 0.8],
        }
    }

    /// Virtual bass tuning: (mix [0.0, 1.0], drive [1.0, 4.0])
    pub fn virtual_bass_params(&self) -> (f32, f32) {
        match self {
            SoundProfile::CinemaSurround3D => (0.65, 2.8),
            SoundProfile::ConcertHallLive => (0.35, 1.8),
            SoundProfile::VocalIntimacy => (0.25, 1.5),
            SoundProfile::ClubBassReflex => (0.85, 3.6),
            SoundProfile::NightAudiophile => (0.40, 1.8),
            SoundProfile::StudioReference => (0.00, 1.0),
            SoundProfile::HarmanConsumer => (0.40, 2.0),
        }
    }

    /// Default 3D spatial orbit amount [0.0, 1.0]
    pub fn default_spatial_orbit(&self) -> f32 {
        match self {
            SoundProfile::CinemaSurround3D => 0.85,
            SoundProfile::ConcertHallLive => 0.75,
            SoundProfile::VocalIntimacy => 0.45,
            SoundProfile::ClubBassReflex => 0.50,
            SoundProfile::NightAudiophile => 0.35,
            SoundProfile::StudioReference => 0.15,
            SoundProfile::HarmanConsumer => 0.55,
        }
    }

    /// Binaural Chu/Bauer crossfeed intensity [0.0, 1.0]
    pub fn crossfeed_amount(&self) -> f32 {
        match self {
            SoundProfile::CinemaSurround3D => 0.60,
            SoundProfile::ConcertHallLive => 0.75,
            SoundProfile::VocalIntimacy => 0.30,
            SoundProfile::ClubBassReflex => 0.25,
            SoundProfile::NightAudiophile => 0.40,
            SoundProfile::StudioReference => 0.35,
            SoundProfile::HarmanConsumer => 0.30,
        }
    }

    /// Noise gate suppression level in dB
    pub fn noise_reduction_db(&self) -> f32 {
        match self {
            SoundProfile::NightAudiophile => 24.0, // High de-noise for quiet environments
            SoundProfile::VocalIntimacy => 20.0,
            SoundProfile::StudioReference => 12.0,
            _ => 16.0,
        }
    }
}
