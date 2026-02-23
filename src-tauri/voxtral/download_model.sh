#!/bin/bash
# Download Voxtral Realtime 4B model from HuggingFace
#
# Usage: ./download_model.sh [--dir DIR]
#   --dir DIR   Download to DIR (default: voxtral-model)

set -e

MODEL_ID="mistralai/Voxtral-Mini-4B-Realtime-2602"
MODEL_DIR="voxtral-model"

while [[ $# -gt 0 ]]; do
    case $1 in
        --dir) MODEL_DIR="$2"; shift 2 ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "Downloading Voxtral Realtime 4B to ${MODEL_DIR}/"
echo "Model: ${MODEL_ID}"
echo ""

mkdir -p "${MODEL_DIR}"

# Files to download
FILES=(
    "consolidated.safetensors"
    "params.json"
    "tekken.json"
)

BASE_URL="https://huggingface.co/${MODEL_ID}/resolve/main"

for file in "${FILES[@]}"; do
    dest="${MODEL_DIR}/${file}"
    if [ -f "${dest}" ]; then
        echo "  [skip] ${file} (already exists)"
    else
        echo "  [download] ${file}..."
        curl -L -o "${dest}" "${BASE_URL}/${file}" --progress-bar
        echo "  [done] ${file}"
    fi
done

echo ""
echo "Download complete. Model files in ${MODEL_DIR}/"
ls -lh "${MODEL_DIR}/"
