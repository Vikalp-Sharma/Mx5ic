# Mx5ic Deep AI Model Suite — Model Card

## Model Overview

- **Model Suite Name**: Mx5ic Deep Music Intelligence Suite (v1.0.0)
- **Total Model Suite Footprint**: **$142.98\text{ MB}$**
- **Total Parameters**: **$37,451,275$ parameters**
- **Architecture Class**: Fully Causal Temporal Convolutional & Recurrent Networks
- **Target Hardware**: Edge CPU (Intel Core / AMD Ryzen / ARM64 Mobile), optional CUDA/ROCm
- **Inference Runtime**: ONNX Runtime (Streaming Opset 18) / Native PyTorch Checkpoints

---

## Model Suite Breakdown

### 1. Neural Bandwidth Extension & Audio Restoration Net (BWE)
- **Filename**: `models/checkpoints/bwe_restoration.pt`
- **File Size**: **$42.89\text{ MB}$** ($11,231,922$ parameters)
- **Architecture**: 20-layer Causal Dilated Gated Residual Network (TCN).
- **Hidden Channels**: 280 channels.
- **Receptive Field**: 512 samples ($\approx 10.7\text{ ms}$ context at 48 kHz).
- **Function**: Reconstructs lost high-frequency acoustic harmonics ($12\text{ kHz}$ to $24\text{ kHz}$) and attenuates lossy MP3/AAC compression artifacts.
- **Loss Function**: Multi-Resolution STFT Loss (Spectral Convergence + Log-Magnitude Distance across FFT sizes 512 and 1024).
- **Training Loss**: $1.1774$.

### 2. Streaming 4-Stem Separator & Dialogue Enhancer
- **Filename**: `models/checkpoints/separator_dialogue.pt`
- **File Size**: **$47.78\text{ MB}$** ($12,515,530$ parameters)
- **Architecture**: 16-block Dilated Causal Convolutional Sub-Band Separator with dual heads.
- **Hidden Channels**: 320 channels.
- **Outputs**: 4 stereo stems (Vocals, Drums, Bass, Other) + 2-channel speech intelligibility mask.
- **Function**: Real-time stem isolation and foreground speech isolation for dialogue enhancement.
- **Loss Function**: Scale-Invariant Signal-to-Distortion Ratio (SI-SDR) + Mask Regularization.
- **Training Loss**: $1.0252$.

### 3. Deep DDSP Tone & Dynamics Controller
- **Filename**: `models/checkpoints/tone_controller.pt`
- **File Size**: **$21.30\text{ MB}$** ($5,580,978$ parameters)
- **Architecture**: 4-layer Causal GRU (hidden dimension 480).
- **Inputs**: 64-dimensional acoustic feature stream (spectral centroid, flux, RMS, sub-bass ratio).
- **Outputs**: Continuous parameters for 8-band parametric EQ, multiband compression, and harmonic saturation.
- **Function**: Predicts smooth, click-free DSP parameter curves dynamically responding to musical changes.
- **Loss Function**: Mean Squared Error (MSE) Parameter Regression.
- **Training Loss**: $1.3994$.

### 4. 3D Spatializer & Binaural Positioning Network
- **Filename**: `models/checkpoints/spatializer.pt`
- **File Size**: **$25.22\text{ MB}$** ($6,604,928$ parameters)
- **Architecture**: 12-layer Causal Residual Network (hidden dimension 280).
- **Function**: Estimates per-stem 3D spatial azimuth ($-90^\circ$ to $+90^\circ$), distance depth, and interaural time difference (ITD).
- **Target**: Powers the dynamic earbud ping-pong vocal orbit effect.
- **Loss Function**: Binaural Angular Divergence & Distance Depth Loss.
- **Training Loss**: $0.1766$.

### 5. Multi-Genre & Acoustic Quality Classifier
- **Filename**: `models/checkpoints/genre_analyzer.pt`
- **File Size**: **$5.80\text{ MB}$** ($1,515,917$ parameters)
- **Architecture**: 3-stage 1D Causal CNN + 2-layer LSTM (hidden dimension 288).
- **Classes**: 42 world music genres (Pop, Hip-Hop, EDM, Rock, Metal, Classical, Jazz, Lo-Fi, Bollywood, Ambient, etc.).
- **Outputs**: Multi-label genre probabilities, LUFS loudness estimate, crest factor, and codec quality.
- **Loss Function**: Multi-Class Cross-Entropy Loss + Quality MSE.
- **Training Loss**: $3.9211$.

---

## Strict 3 GB Footprint Compliance

- **Permitted Model Budget**: $\le 500\text{ MB}$ (Total installed footprint $\le 3\text{ GB}$).
- **Actual Model Suite Size**: **$142.98\text{ MB}$** ($28.6\%$ of model budget, $> 95\%$ overall system headroom).
- **Runtime Dependency**: Fully self-contained on-device inference without external cloud APIs.
