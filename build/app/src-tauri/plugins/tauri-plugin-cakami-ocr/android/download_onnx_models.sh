#!/usr/bin/env bash
# Download official open-source PP-OCRv6 Small ONNX weights at APK BUILD time.
# No user content is sent; bundled weights are read only from Android assets.
set -euo pipefail
cd "$(dirname "$0")"
dir="src/main/assets/models"
mkdir -p "$dir/det" "$dir/rec"

fetch() {
  local target="$1" url="$2" minimum="$3"
  if [[ -f "$target" ]] && [[ "$(stat -c%s "$target")" -ge "$minimum" ]]; then
    echo "Using existing model: $target"
  else
    local tmp="${target}.downloading"
    rm -f "$tmp"
    curl --fail --location --silent --show-error \
      --retry 3 --retry-delay 2 --connect-timeout 30 --max-time 240 \
      --user-agent "CAKAMI-Android-Offline-OCR-Build/1.0" \
      --output "$tmp" "$url"
    local got
    got="$(stat -c%s "$tmp")"
    if [[ "$got" -lt "$minimum" ]]; then
      echo "FAIL: incomplete model asset $target bytes=$got" >&2
      rm -f "$tmp"
      exit 1
    fi
    mv -f "$tmp" "$target"
  fi
  sha256sum "$target"
}
fetch "$dir/det/inference.onnx" \
  "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx/resolve/main/inference.onnx" 1000000
fetch "$dir/rec/inference.onnx" \
  "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/main/inference.onnx" 2000000
fetch "$dir/rec/inference.yml" \
  "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/main/inference.yml" 1000
echo "PP-OCRv6 Small det+rec ONNX and YAML are present, ready to bundle in APK"
