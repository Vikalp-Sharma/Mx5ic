# Mx5ic Known Technical Limitations & Mitigation Guide

In adherence to honest engineering, this document details platform constraints, Android audio capture boundaries, and established mitigations.

---

## 1. Android Audio Capture Boundaries

### Limitation 1: App Opt-Out Policy (`allowAudioPlaybackCapture`)
- **Context**: On Android 10+ (API 29+), applications can specify `android:allowAudioPlaybackCapture="false"` in their manifest (often enabled by default in banking apps or high-security video players).
- **Impact**: While apps with `USAGE_MEDIA`, `USAGE_GAME`, or `USAGE_UNKNOWN` (Spotify, YouTube, SoundCloud, local players) can be captured freely, opt-out apps produce silence in standard `AudioPlaybackCapture`.
- **Engineering Mitigation**: Mx5ic provides **Path B: Universal Bluetooth A2DP Sink** via [`scripts/setup_bluetooth_sink.sh`](scripts/setup_bluetooth_sink.sh). Pairing the phone as an audio source to the laptop captures 100% of all audio streams at the operating system hardware level, bypassing software manifest restrictions completely.

### Limitation 2: Post-Volume Capture & Double Playback Prevention
- **Context**: In some OEM Android distributions (Samsung OneUI, Xiaomi MIUI/HyperOS), `AudioRecord` from `AudioPlaybackCapture` captures audio post-master volume. Muting the phone's volume slider to 0 may also silence the capture buffer.
- **Impact**: If phone volume is set high, the user might hear the phone speaker alongside the laptop headphones.
- **Engineering Mitigation**:
  1. Reduce phone volume to the minimum audible tick (1/15), which maintains a clean PCM signal while silencing the phone speaker in the room.
  2. Or use **Path B (Bluetooth A2DP Sink)**, which automatically redirects phone audio to the laptop and natively silences the phone's built-in speaker.

---

## 2. Linux Desktop Graphics & Audio

### Limitation 3: Legacy GPU Drivers without OpenGL 3.3 Core
- **Context**: The Liquid Glass interface utilizes modern GLSL 330 core shaders (raymarched SDFs, smooth polynomial unions, and chromatic dispersion).
- **Impact**: Systems running software-only LLVMpipe or graphics drivers older than 2011 may experience lower frame rates (< 60 FPS).
- **Engineering Mitigation**: The UI includes an automatic fallback tier that simplifies refraction passes if frame time exceeds 16 ms, and provides a `--headless` verification mode for continuous background daemon operation.

### Limitation 4: FAT32/exFAT Umask File Permissions for Debian Packaging
- **Context**: Building Debian packages with `dpkg-deb` requires standard POSIX permissions (`0755` for directories/binaries, `0644` for control files). Removable media formatted with exFAT/FAT32 mounts with uniform `0777` permissions.
- **Engineering Mitigation**: The [`setup.sh`](setup.sh) script automatically constructs Debian packages in `/tmp/mx5ic_deb` where POSIX permissions are fully enforceable, and then copies the verified `.deb` into `dist/`.
