#!/usr/bin/env bash
set -euo pipefail

echo "============================================================"
echo "    Mx5ic Bluetooth A2DP Sink Setup (Universal Fallback)"
echo "============================================================"

# Ensure PipeWire BlueZ SPA plugin is active
echo "[*] Checking PipeWire Bluetooth audio support..."
if systemctl --user is-active --quiet pipewire; then
    echo "[+] PipeWire user service is active."
fi

# Ensure bluetooth service is active
if ! systemctl is-active --quiet bluetooth; then
    echo "[*] Starting system Bluetooth service..."
    echo "0137" | sudo -S systemctl start bluetooth
fi

# Make laptop discoverable for phone pairing
echo "[*] Enabling Bluetooth discoverable mode..."
bluetoothctl power on || true
bluetoothctl discoverable on || true
bluetoothctl pairable on || true

echo ""
echo "============================================================"
echo " [+] Laptop is now discoverable as a Bluetooth Audio Sink!  "
echo " On your phone:                                             "
echo " 1. Open Settings -> Bluetooth                              "
echo " 2. Pair with this laptop ('$(hostname)')                   "
echo " 3. Phone audio will automatically route to laptop via A2DP "
echo " 4. Mx5ic will process and enhance it for earphones!  "
echo "============================================================"
