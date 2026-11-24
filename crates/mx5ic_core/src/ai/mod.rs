pub mod features;
pub mod intelligence;
pub mod models;

pub use features::{AudioFeatures, FeatureExtractor};
pub use intelligence::{
    AdaptiveDecision, InstrumentPresence, MusicGenre, MusicIntelligenceEngine, MusicUnderstanding,
};
pub use models::{ModelSuiteManager, ModelSuiteManifest};

