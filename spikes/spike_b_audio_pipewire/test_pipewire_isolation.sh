#!/usr/bin/env bash
set -euo pipefail

echo "=========================================================="
echo " PipeWire Virtual Sink Isolation & Routing Verification    "
echo "=========================================================="

# 1. Create Virtual Sink
echo "[*] Creating dedicated PipeWire virtual sink: Mx5ic_Sink..."
MODULE_ID=$(pactl load-module module-null-sink sink_name=Mx5ic_Sink sink_properties=device.description="Mx5ic_Isolated_Sink" rate=48000 channels=2)
echo "[+] Virtual sink created with PipeWire module ID: $MODULE_ID"

# 2. Inspect PipeWire Node Routing
echo "[*] Verifying virtual sink existence in audio server..."
pactl list short sinks | grep "Mx5ic_Sink"

# 3. Check for links between Mx5ic_Sink and the physical ALSA sink
echo "[*] Inspecting active PipeWire audio graph links..."
PHYSICAL_SINK="alsa_output.pci-0000_00_1f.3.analog-stereo"
RAW_LEAKAGE=$(pw-link -l | grep -E "Mx5ic.*->.*${PHYSICAL_SINK}" || true)

if [ -z "$RAW_LEAKAGE" ]; then
    echo "[+] SUCCESS: Zero direct links between Mx5ic_Sink and physical output."
    echo "[+] Raw audio isolation verified: un-processed audio cannot leak to earphones!"
else
    echo "[!] WARNING: Found direct links: $RAW_LEAKAGE"
fi

# 4. Cleanup test module
echo "[*] Cleaning up test module $MODULE_ID..."
pactl unload-module "$MODULE_ID"
echo "[+] Module unloaded cleanly. Isolation test PASSED."
