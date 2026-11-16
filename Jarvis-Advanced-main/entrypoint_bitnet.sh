#!/bin/bash
set -e

MODEL_DIR="/app/BitNet/models/BitNet-b1.58-2B-4T"
MODEL_PATH="${MODEL_DIR}/ggml-model-i2_s.gguf"

# If the file is missing or is just the placeholder created during build (usually size is around 12 bytes), download the real one
if [ ! -f "$MODEL_PATH" ] || [ $(wc -c <"$MODEL_PATH") -lt 1000 ]; then
    echo "[Entrypoint] Real model GGUF file not found. Downloading from Hugging Face..."
    mkdir -p "$MODEL_DIR"
    curl -L "https://huggingface.co/microsoft/bitnet-b1.58-2B-4T-gguf/resolve/main/ggml-model-i2_s.gguf" -o "$MODEL_PATH"
    echo "[Entrypoint] Model download complete."
else
    echo "[Entrypoint] Found existing model GGUF file: $(du -sh $MODEL_PATH)"
fi

echo "[Entrypoint] Launching llama-server on port 8580..."
exec /app/BitNet/build/bin/llama-server \
    --model "$MODEL_PATH" \
    --host 0.0.0.0 \
    --port 8580 \
    --parallel 1 \
    --ctx-size 2048
