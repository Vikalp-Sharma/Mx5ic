#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# Mx5ic Audio Enhancer - Unified Setup & Build Script
# Supports Pop!_OS / Ubuntu (apt) and Fedora (dnf)
# Builds Linux application (.deb, portable bundle) and Android APK
# ==============================================================================

echo "========================================================"
echo "         Mx5ic Real-Time AI Music Enhancer Setup        "
echo "========================================================"

# 1. Detect Package Manager and Install System Dependencies (optional / skippable)
if [ "${SKIP_DEPS:-0}" != "1" ] && [ "${1:-}" != "--no-deps" ]; then
    if command -v apt-get &> /dev/null; then
        echo "[*] Detected Debian / Ubuntu / Pop!_OS (apt)"
        export DEBIAN_FRONTEND=noninteractive
        sudo apt-get update -y
        sudo apt-get install -y \
            build-essential \
            clang \
            cmake \
            pkg-config \
            libpipewire-0.3-dev \
            libspa-0.2-dev \
            libasound2-dev \
            libopus-dev \
            libgl1-mesa-dev \
            libegl1-mesa-dev \
            libglfw3-dev \
            pulseaudio-utils \
            ffmpeg \
            python3 \
            python3-pip \
            openjdk-17-jdk \
            git
    elif command -v dnf &> /dev/null; then
        echo "[*] Detected Fedora / RHEL (dnf)"
        sudo dnf install -y \
            gcc \
            gcc-c++ \
            clang \
            cmake \
            pkgconf-pkg-config \
            pipewire-devel \
            alsa-lib-devel \
            opus-devel \
            mesa-libGL-devel \
            mesa-libEGL-devel \
            glfw-devel \
            pulseaudio-utils \
            ffmpeg \
            python3 \
            python3-pip \
            java-25-openjdk-devel \
            dpkg-dev \
            git
    fi
else
    echo "[*] Skipping system package manager installation (--no-deps or SKIP_DEPS=1)."
fi

# 2. Build Rust Audio Core & Liquid Glass Desktop UI
echo "[*] Building Mx5ic Audio Core and Desktop UI (Release)..."
cargo build --release --workspace

# 3. Package Linux Application
echo "[*] Assembling Linux Distribution Packages..."
mkdir -p dist/portable/mx5ic/shaders
mkdir -p dist/portable/mx5ic/models

cp target/release/mx5ic_ui dist/portable/mx5ic/mx5ic-ui
cp target/release/mx5ic_daemon dist/portable/mx5ic/mx5ic-daemon
cp target/release/mx5ic_enhance dist/portable/mx5ic/mx5ic-enhance
cp target/release/libmx5ic_core.so dist/portable/mx5ic/libmx5ic_core.so
cp ui/src/shaders/liquid_glass.frag dist/portable/mx5ic/shaders/
if [ -f dist/mx5ic_icon.png ]; then
    cp dist/mx5ic_icon.png dist/portable/mx5ic/mx5ic.png
fi

# Bundle 0.58B Conformer (>500 MB) Deep AI Model Suite into portable distribution
echo "[*] Bundling 0.58B Parameter Conformer AI Models into Portable Package..."
cp models/checkpoints/*.pt dist/portable/mx5ic/models/
if [ -d models/exported ]; then
    cp -r models/exported/* dist/portable/mx5ic/models/ 2>/dev/null || true
fi
if [ -f models/model_suite_report.json ]; then
    cp models/model_suite_report.json dist/portable/mx5ic/models/
fi

cat << 'RUN_EOF' > dist/portable/mx5ic/run.sh
#!/bin/bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LD_LIBRARY_PATH="$DIR:$LD_LIBRARY_PATH"
export MX5IC_MODEL_DIR="$DIR/models"

if ! pactl list short sinks | grep -q "Mx5ic_Sink"; then
    echo "[*] Creating isolated virtual sink: Mx5ic_Sink..."
    pactl load-module module-null-sink sink_name=Mx5ic_Sink sink_properties=device.description="Mx5ic_Isolated_Audio_Processor" rate=48000 channels=2 > /dev/null 2>&1 || true
fi

echo "[*] Launching Mx5ic Liquid Glass Interface..."
exec "$DIR/mx5ic-ui" "$@"
RUN_EOF
chmod +x dist/portable/mx5ic/run.sh

tar -czf dist/mx5ic-linux-x86_64.tar.gz -C dist/portable mx5ic

# Assemble Debian Package via temporary directory to enforce standard POSIX file permissions
rm -rf /tmp/mx5ic_deb
mkdir -p /tmp/mx5ic_deb/DEBIAN
mkdir -p /tmp/mx5ic_deb/usr/bin
mkdir -p /tmp/mx5ic_deb/usr/lib
mkdir -p /tmp/mx5ic_deb/usr/share/mx5ic/shaders
mkdir -p /tmp/mx5ic_deb/usr/share/mx5ic/models
mkdir -p /tmp/mx5ic_deb/usr/share/applications
mkdir -p /tmp/mx5ic_deb/usr/share/pixmaps

cat << 'DEB_EOF' > /tmp/mx5ic_deb/DEBIAN/control
Package: mx5ic
Version: 0.1.0
Section: sound
Priority: optional
Architecture: amd64
Maintainer: Mx5ic Project <dev@mx5ic.local>
Depends: libpipewire-0.3-0 | pipewire, libasound2, libgl1, libglfw3
Description: Mx5ic Real-time AI Music Enhancer with Liquid Glass UI
 Advanced on-device real-time neural audio enhancer featuring
 bundled 0.58B Conformer AI Suite (>500 MB), dynamic 3D spatial vocal orbit,
 psychoacoustic virtual bass, and a zero-dropout PipeWire audio pipeline.
DEB_EOF

cat << 'DEB_LAUNCHER' > /tmp/mx5ic_deb/usr/bin/mx5ic
#!/bin/bash
set -e
export MX5IC_MODEL_DIR="/usr/share/mx5ic/models"
if ! pactl list short sinks | grep -q "Mx5ic_Sink"; then
    pactl load-module module-null-sink sink_name=Mx5ic_Sink sink_properties=device.description="Mx5ic_Isolated_Audio_Processor" rate=48000 channels=2 > /dev/null 2>&1 || true
fi
exec /usr/bin/mx5ic-ui "$@"
DEB_LAUNCHER

cat << 'DESK_EOF' > /tmp/mx5ic_deb/usr/share/applications/mx5ic.desktop
[Desktop Entry]
Name=Mx5ic
Comment=Real-Time AI Music Enhancer with Liquid Glass UI
Exec=mx5ic
Icon=mx5ic
Terminal=false
Type=Application
Categories=AudioVideo;Audio;AudioVideoEditing;
Keywords=Audio;Music;Enhancer;AI;Spatial;DSP;
StartupWMClass=mx5ic_ui
DESK_EOF

cp target/release/mx5ic_ui /tmp/mx5ic_deb/usr/bin/mx5ic-ui
cp target/release/mx5ic_daemon /tmp/mx5ic_deb/usr/bin/mx5ic-daemon
cp target/release/mx5ic_enhance /tmp/mx5ic_deb/usr/bin/mx5ic-enhance
cp target/release/libmx5ic_core.so /tmp/mx5ic_deb/usr/lib/libmx5ic_core.so
cp ui/src/shaders/liquid_glass.frag /tmp/mx5ic_deb/usr/share/mx5ic/shaders/
if [ -f dist/mx5ic_icon.png ]; then
    cp dist/mx5ic_icon.png /tmp/mx5ic_deb/usr/share/pixmaps/mx5ic.png
fi

# Bundle 0.58B Conformer (>500 MB) Deep AI Model Suite into Debian package
echo "[*] Bundling 0.58B Parameter Conformer AI Models into Debian Package..."
cp models/checkpoints/*.pt /tmp/mx5ic_deb/usr/share/mx5ic/models/
if [ -d models/exported ]; then
    cp -r models/exported/* /tmp/mx5ic_deb/usr/share/mx5ic/models/ 2>/dev/null || true
fi
if [ -f models/model_suite_report.json ]; then
    cp models/model_suite_report.json /tmp/mx5ic_deb/usr/share/mx5ic/models/
fi

chmod 0755 /tmp/mx5ic_deb/DEBIAN
chmod 0644 /tmp/mx5ic_deb/DEBIAN/control
chmod 0755 /tmp/mx5ic_deb/usr/bin/*
chmod 0644 /tmp/mx5ic_deb/usr/lib/*
chmod 0644 /tmp/mx5ic_deb/usr/share/applications/*
chmod 0644 /tmp/mx5ic_deb/usr/share/pixmaps/*
chmod 0644 /tmp/mx5ic_deb/usr/share/mx5ic/models/*

dpkg-deb --build --root-owner-group /tmp/mx5ic_deb /tmp/mx5ic_0.1.0_amd64.deb
cp /tmp/mx5ic_0.1.0_amd64.deb dist/mx5ic_0.1.0_amd64.deb

# 4. Build & Sign Android APK (if SDK tools exist)
SDK_TOOLS="/home/kairo/.build-tools/android-sdk/build-tools/34.0.0"
PLATFORM_JAR="/home/kairo/.build-tools/android-sdk/platforms/android-34/android.jar"

if [ -f "$SDK_TOOLS/aapt2" ] && [ -f "$PLATFORM_JAR" ]; then
    echo "[*] Bundling Deep AI Models into Android Assets..."
    mkdir -p android/assets/models
    cp models/checkpoints/*.pt android/assets/models/
    if [ -d models/exported ]; then
        cp -r models/exported/* android/assets/models/ 2>/dev/null || true
    fi
    if [ -f models/model_suite_report.json ]; then
        cp models/model_suite_report.json android/assets/models/
    fi

    echo "[*] Compiling and Signing Merged Android APK..."
    rm -rf android/build
    mkdir -p android/build/compiled android/build/classes android/build/dex

    $SDK_TOOLS/aapt2 compile --dir android/res -o android/build/compiled_res.zip
    $SDK_TOOLS/aapt2 link \
        -I $PLATFORM_JAR \
        --manifest android/AndroidManifest.xml \
        -A android/assets \
        -0 pt -0 onnx -0 data \
        -o android/build/base.apk \
        --java android/src \
        --auto-add-overlay \
        android/build/compiled_res.zip

    javac -source 8 -target 8 -bootclasspath $PLATFORM_JAR -d android/build/classes android/src/com/mx5ic/enhancer/*.java
    $SDK_TOOLS/d8 --lib "$PLATFORM_JAR" --release --min-api 29 --output android/build/dex android/build/classes/com/mx5ic/enhancer/*.class

    (cd android/build/dex && zip -u ../base.apk classes.dex)
    $SDK_TOOLS/zipalign -f -v -p 4 android/build/base.apk android/build/aligned.apk

    if [ ! -f android/mx5ic_release.keystore ]; then
        keytool -genkeypair -v -keystore android/mx5ic_release.keystore \
            -alias mx5ic_release -keyalg RSA -keysize 2048 -sigalg SHA256withRSA -validity 10000 \
            -storepass mx5icrelease2026 -keypass mx5icrelease2026 \
            -dname "CN=Mx5ic Audio, OU=Audio Intelligence Engineering, O=Mx5ic Technologies Inc., L=San Francisco, ST=California, C=US"
    fi

    $SDK_TOOLS/apksigner sign \
        --ks android/mx5ic_release.keystore \
        --ks-key-alias mx5ic_release \
        --ks-pass pass:mx5icrelease2026 \
        --key-pass pass:mx5icrelease2026 \
        --v1-signing-enabled true \
        --v2-signing-enabled true \
        --v3-signing-enabled true \
        --v4-signing-enabled false \
        --out dist/Mx5ic.apk \
        android/build/aligned.apk
    echo "[+] Merged Android APK signed successfully (Schemes v1, v2, v3): dist/Mx5ic.apk"
fi

echo "========================================================"
echo "    Build & Packaging Complete! Artifacts in dist/     "
echo "  - Linux Debian: dist/mx5ic_0.1.0_amd64.deb           "
echo "  - Linux Portable: dist/mx5ic-linux-x86_64.tar.gz     "
echo "  - Android App: dist/Mx5ic.apk                       "
echo "========================================================"
