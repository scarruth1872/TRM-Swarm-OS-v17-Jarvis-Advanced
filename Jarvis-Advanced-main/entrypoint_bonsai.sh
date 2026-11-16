#!/bin/bash
set -e

MODEL_SIZE="${MODEL_SIZE:-1.7B}"
MODEL_DIR="/app/llama.cpp/models"
MODEL_PATH="${MODEL_DIR}/Ternary-Bonsai-${MODEL_SIZE}-Q2_0.gguf"

if [ ! -f "$MODEL_PATH" ]; then
    echo "[Bonsai Entrypoint] Model Ternary-Bonsai-${MODEL_SIZE}-Q2_0 not found. Downloading from Hugging Face..."
    mkdir -p "$MODEL_DIR"
    
    if [ "$MODEL_SIZE" = "1.7B" ]; then
        URL="https://huggingface.co/prism-ml/Ternary-Bonsai-1.7B-gguf/resolve/main/Ternary-Bonsai-1.7B-Q2_0.gguf"
    elif [ "$MODEL_SIZE" = "4B" ]; then
        URL="https://huggingface.co/prism-ml/Ternary-Bonsai-4B-gguf/resolve/main/Ternary-Bonsai-4B-Q2_0.gguf"
    elif [ "$MODEL_SIZE" = "8B" ]; then
        URL="https://huggingface.co/prism-ml/Ternary-Bonsai-8B-gguf/resolve/main/Ternary-Bonsai-8B-Q2_0.gguf"
    else
        echo "[Bonsai Entrypoint] Invalid MODEL_SIZE: ${MODEL_SIZE}."
        exit 1
    fi
    
    curl -L "$URL" -o "$MODEL_PATH"
    echo "[Bonsai Entrypoint] Model Ternary-Bonsai-${MODEL_SIZE}-Q2_0 download complete."
else
    echo "[Bonsai Entrypoint] Found existing model GGUF file: $(du -sh $MODEL_PATH)"
fi

echo "[Bonsai Entrypoint] Launching llama-server for size ${MODEL_SIZE} on port 8580..."
exec /app/llama.cpp/build/bin/llama-server \
    --model "$MODEL_PATH" \
    --host 0.0.0.0 \
    --port 8580 \
    --parallel 1 \
    --ctx-size 8192
