# DATASETS.md: Training & Evaluation Dataset Registry

All datasets used for training, fine-tuning, and evaluating the real-time AI music enhancer are cataloged here. Licenses are strictly checked and verified prior to inclusion. Non-commercial datasets are quarantined for research and evaluation only and excluded from production release builds.

| Dataset Name | Domain / Target | URL / Source | License | Commercial Use Permitted? | Role in Pipeline | Quarantined from Release Build? |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Free Music Archive (FMA) Small/Medium** | Music genre classification, full mixes | https://github.com/mdeff/fma | Creative Commons (per-track: CC-BY, CC-BY-SA, CC0 filtered) | Yes (filtered subsets) | Analyzer training, DDSP inverse degradation training | No (only CC-commercial tracks included) |
| **MTG-Jamendo** | Music audio tagging, genre priors | https://github.com/MTG/mtg-jamendo-dataset | Creative Commons (CC-BY / CC0 filtered) | Yes (filtered subsets) | Genre target spectral stats, DDSP conditioning | No (only CC-commercial tracks included) |
| **MUSDB18-HQ** | 4-stem multitrack music (Vocals, Drums, Bass, Other) | https://sigsep.github.io/datasets/musdb.html | Creative Commons BY-NC-SA 4.0 | **No (Non-Commercial)** | Benchmark evaluation only, held-out validation | **Yes (Quarantined, never shipped in weights)** |
| **LibriSpeech (clean-100)** | Clean spoken dialogue | http://www.openslr.org/12 | CC-BY 4.0 | Yes | Dialogue enhancer & speech-in-music masking training | No |
| **AutoEQ Database** | Headphone frequency response compensation | https://github.com/jaakkopasanen/AutoEq | MIT License | Yes | Headphone profile correction parametric EQ curves | No |
| **SADIE II HRIR Database** | Binaural room impulse responses (SOFA format) | https://www.york.ac.uk/sadie-project/ | CC-BY 4.0 | Yes | Spatializer HRTF direct convolution filters | No |
| **ESC-50** | Environmental background noise | https://github.com/karolpiczak/ESC-50 | CC-BY-NC 3.0 | **No (Non-Commercial)** | Dialogue intelligibility stress testing | **Yes (Quarantined)** |

## License Compliance Protocols

1. **Filtering Pipeline**:
   - The dataset ingestion scripts automatically query track metadata.
   - Any audio file tagged with `NC` (Non-Commercial) or `ND` (No Derivatives) is placed strictly in `data/eval_quarantine/` and is rejected by the production training export script.
2. **Artist-Level Splitting**:
   - Data splits are executed strictly at the Artist / Album / Track boundary.
   - Zero clip overlap exists between train, validation, and the frozen held-out test set.
