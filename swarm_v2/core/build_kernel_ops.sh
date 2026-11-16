#!/usr/bin/env bash
set -e

echo "[+] Compiling TRM Native Kernel Operations Library..."

cd "$(dirname "$0")"

g++ -O3 -shared -fPIC \
    -march=native -mtune=native -mavx2 -mfma \
    -flto -fno-rtti -fno-exceptions \
    -Wall -Wextra \
    swarm_kernel_ops.cpp \
    -o libswarm_kernel_ops.so

echo "[+] Build complete: libswarm_kernel_ops.so"
echo "[+] Size: $(ls -lh libswarm_kernel_ops.so | awk '{print $5}')"

# Verify symbols
echo "[+] Exported symbols:"
nm -D libswarm_kernel_ops.so | grep " T " | grep -E "c_mamba|c_instella|c_nvme"
