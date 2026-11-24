use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct ModelSuiteManifest {
    pub model_dir: PathBuf,
    pub total_size_bytes: u64,
    pub total_size_mb: f32,
    pub loaded_models: Vec<String>,
    pub is_complete: bool,
    pub has_05b_conformer: bool,
}

pub struct ModelSuiteManager;

impl ModelSuiteManager {
    pub const REQUIRED_MODELS: &'static [&'static str] = &[
        "bwe_restoration.pt",
        "separator_dialogue.pt",
        "tone_controller.pt",
        "spatializer.pt",
        "genre_analyzer.pt",
    ];

    pub fn discover_and_load() -> Option<ModelSuiteManifest> {
        let mut candidates = Vec::new();

        if let Ok(env_dir) = std::env::var("MX5IC_MODEL_DIR") {
            candidates.push(PathBuf::from(env_dir));
        }

        candidates.push(PathBuf::from("/usr/share/mx5ic/models"));
        candidates.push(PathBuf::from("models"));
        candidates.push(PathBuf::from("models/checkpoints"));

        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("models"));
            candidates.push(cwd.join("models/checkpoints"));
            let mut curr = cwd.clone();
            for _ in 0..4 {
                if let Some(parent) = curr.parent() {
                    candidates.push(parent.join("models"));
                    candidates.push(parent.join("models/checkpoints"));
                    curr = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }

        if let Ok(exe_path) = std::env::current_exe() {
            let mut curr = exe_path.clone();
            for _ in 0..5 {
                if let Some(parent) = curr.parent() {
                    candidates.push(parent.join("models"));
                    candidates.push(parent.join("models/checkpoints"));
                    candidates.push(parent.join("share/mx5ic/models"));
                    curr = parent.to_path_buf();
                } else {
                    break;
                }
            }
        }

        for candidate in candidates {
            if let Some(manifest) = Self::inspect_directory(&candidate) {
                if manifest.is_complete {
                    return Some(manifest);
                }
            }
        }

        // Check if models exist split across checkpoints and exported
        let checkpoints_dir = PathBuf::from("models/checkpoints");
        if let Some(manifest) = Self::inspect_directory(&checkpoints_dir) {
            if manifest.is_complete {
                return Some(manifest);
            }
        }

        None
    }

    pub fn inspect_directory(dir: &Path) -> Option<ModelSuiteManifest> {
        if !dir.exists() || !dir.is_dir() {
            return None;
        }

        let mut total_size = 0u64;
        let mut loaded = Vec::new();

        for &req in Self::REQUIRED_MODELS {
            let path = dir.join(req);
            if path.exists() {
                if let Ok(meta) = path.metadata() {
                    total_size += meta.len();
                    loaded.push(req.to_string());
                }
            }
        }

        // Also check if bwe_streaming.onnx is present in dir or exported/
        let onnx_path = dir.join("bwe_streaming.onnx");
        if onnx_path.exists() {
            if let Ok(meta) = onnx_path.metadata() {
                total_size += meta.len();
                loaded.push("bwe_streaming.onnx".to_string());
            }
        }

        // Also check if 0.5B Conformer model checkpoint is present
        let conformer_path = dir.join("mx5ic_conformer_0.5b.pt");
        let mut has_05b_conformer = false;
        if conformer_path.exists() {
            if let Ok(meta) = conformer_path.metadata() {
                total_size += meta.len();
                loaded.push("mx5ic_conformer_0.5b.pt".to_string());
                has_05b_conformer = true;
            }
        }

        if loaded.is_empty() {
            return None;
        }

        let is_complete = Self::REQUIRED_MODELS.iter().all(|&m| loaded.iter().any(|l| l == m));
        let total_mb = (total_size as f32) / (1024.0 * 1024.0);

        Some(ModelSuiteManifest {
            model_dir: dir.to_path_buf(),
            total_size_bytes: total_size,
            total_size_mb: total_mb,
            loaded_models: loaded,
            is_complete,
            has_05b_conformer,
        })
    }
}
