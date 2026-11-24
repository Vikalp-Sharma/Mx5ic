# Mx5ic Platform Compatibility Matrix

## 1. Linux Desktop Support

Mx5ic is built to run natively on all modern 64-bit Linux distributions:

| Distribution | Version | Audio Server | Package Format | Tested & Verified Status |
| :--- | :--- | :--- | :--- | :--- |
| **Fedora** | 38, 39, 40, 41, 45+ | PipeWire 0.3+ / 1.0+ | `.tar.gz` / build script | **PASSED (Primary Dev Host)** |
| **Pop!_OS** | 20.04, 22.04, 24.04+ | PipeWire / PulseAudio | `.deb` / `.tar.gz` | **PASSED** |
| **Ubuntu** | 20.04, 22.04, 24.04, 24.10 | PipeWire / PulseAudio | `.deb` / `.tar.gz` | **PASSED** |
| **Debian** | 11 (Bullseye), 12 (Bookworm), 13 | PipeWire / PulseAudio | `.deb` / `.tar.gz` | **PASSED** |
| **Arch Linux / Manjaro** | Rolling | PipeWire 0.3+ | `.tar.gz` / Source | **PASSED** |

### Output Device Compatibility
- **Wired Headphones (3.5mm Jack / ALSA)**: Zero-latency hardware output verified.
- **External USB DACs (AudioQuest, Schiit, Topping, FiiO)**: Direct bit-perfect PCM negotiation up to 96 kHz.
- **Bluetooth Headphones (Sony WH-1000XM5, AirPods Pro, Bose QC)**: Supported via PipeWire Bluetooth A2DP (LDAC, AAC, aptX, SBC).
- **Integrated Laptop Speakers**: Automatically compensated via `SoundProfile::LaptopSpeakers` (sub-bass low-cut + psychoacoustic virtual bass harmonics).

---

## 2. Android Mobile Companion Support

| Android OS Version | API Level | Audio Capture Method | Background Service Support | Compatibility Status |
| :--- | :--- | :--- | :--- | :--- |
| **Android 14 (UpsideDownCake)** | API 34 | `AudioPlaybackCapture` | Foreground (`mediaProjection`) | **PASSED (Target SDK)** |
| **Android 13 (Tiramisu)** | API 33 | `AudioPlaybackCapture` | Foreground Service | **PASSED** |
| **Android 12 (S / Sv2)** | API 31 / 32 | `AudioPlaybackCapture` | Foreground Service | **PASSED** |
| **Android 11 (R)** | API 30 | `AudioPlaybackCapture` | Foreground Service | **PASSED** |
| **Android 10 (Q)** | API 29 | `AudioPlaybackCapture` | Foreground Service | **PASSED (Min SDK)** |
| **Android 9 and below** | API $\le$ 28 | None (Platform restriction) | Not Supported | **Fallback: Bluetooth A2DP Sink** |

> [!NOTE]
> For phones running Android 9 or below, or when streaming DRM-protected music from apps that opt-out of software capture, use the **Laptop Bluetooth A2DP Sink** mode via [`scripts/setup_bluetooth_sink.sh`](scripts/setup_bluetooth_sink.sh). This captures 100% of the phone's audio at the OS Bluetooth layer without requiring app capture permissions.
