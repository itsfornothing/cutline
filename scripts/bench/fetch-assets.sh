#!/usr/bin/env bash
set -euo pipefail

# scripts/bench/fetch-assets.sh
# Deterministically generates and verifies benchmark assets for Cutline Phase 6.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ASSETS_DIR="$WORKSPACE_ROOT/bench/assets"
SLIDES_DIR="$ASSETS_DIR/slides"
LOCK_FILE="$WORKSPACE_ROOT/bench/assets.lock"

mkdir -p "$ASSETS_DIR" "$SLIDES_DIR"

echo "=== Generating & Verifying Cutline Benchmark Assets ==="

# 1. Generate bg_1080p30.mp4 (1920x1080 H.264, 30s @ 30fps with realistic entropy)
BG_VIDEO="$ASSETS_DIR/bg_1080p30.mp4"
if [ ! -f "$BG_VIDEO" ]; then
    echo "[1/4] Generating 1080p30 background video..."
    ffmpeg -v error -y -f lavfi -i "testsrc2=size=1920x1080:rate=30,noise=alls=15:allf=t+u" \
        -t 30 -c:v libx264 -preset fast -pix_fmt yuv420p "$BG_VIDEO"
fi

# 2. Generate voice_60s.mp3 (60s speech-harmonic audio)
VOICE_AUDIO="$ASSETS_DIR/voice_60s.mp3"
if [ ! -f "$VOICE_AUDIO" ]; then
    echo "[2/4] Generating 60s voice audio stream..."
    ffmpeg -v error -y -f lavfi -i "sine=frequency=220:beep_factor=4:sample_rate=48000" \
        -t 60 -c:a libmp3lame -b:a 192k "$VOICE_AUDIO"
fi

# 3. Generate 24 4K slide images (3840x2160 JPEG)
echo "[3/4] Generating 24 procedural 4K slides..."
for i in $(seq -w 01 24); do
    SLIDE_IMG="$SLIDES_DIR/slide_${i}.jpg"
    if [ ! -f "$SLIDE_IMG" ]; then
        # Seed color hue based on index
        HUE=$(( (10#$i * 15) % 360 ))
        ffmpeg -v error -y -f lavfi -i "color=c=0x112233:s=3840x2160,format=rgb24,hue=h=${HUE}" \
            -vframes 1 -q:v 2 "$SLIDE_IMG"
    fi
done

# 4. Generate LICENSES.md
cat << 'EOF' > "$ASSETS_DIR/LICENSES.md"
# Benchmark Asset Licenses & Provenance

All assets generated for Cutline Phase 6 benchmarks are procedural or CC0 public domain:
- `bg_1080p30.mp4`: Procedurally generated via FFmpeg `testsrc2` + temporal noise (CC0).
- `voice_60s.mp3`: Procedural harmonic tone sequence generated via FFmpeg `sine` (CC0).
- `slides/*.jpg`: Procedurally generated 4K gradient matrices via FFmpeg `color` + `hue` (CC0).
- Fonts: Inter (SIL Open Font License 1.1) and Amiri Arabic (SIL Open Font License 1.1).
EOF

# 5. Lock file verification / generation
echo "[4/4] Verifying assets against lock file..."
if [ -f "$LOCK_FILE" ]; then
    # Verify existing checksums
    while IFS="  " read -r expected_hash filepath; do
        if [ -n "$expected_hash" ] && [ -n "$filepath" ]; then
            ACTUAL_HASH=$(shasum -a 256 "$WORKSPACE_ROOT/$filepath" | awk '{print $1}')
            if [ "$ACTUAL_HASH" != "$expected_hash" ]; then
                echo "ERROR: Checksum mismatch for $filepath!" >&2
                echo "Expected: $expected_hash" >&2
                echo "Actual:   $ACTUAL_HASH" >&2
                exit 1
            fi
        fi
    done < "$LOCK_FILE"
    echo "✓ All benchmark asset checksums match bench/assets.lock!"
else
    echo "Creating initial bench/assets.lock..."
    (
        cd "$WORKSPACE_ROOT"
        shasum -a 256 "bench/assets/bg_1080p30.mp4"
        shasum -a 256 "bench/assets/voice_60s.mp3"
        for f in bench/assets/slides/*.jpg; do
            shasum -a 256 "$f"
        done
    ) > "$LOCK_FILE"
    echo "✓ Generated bench/assets.lock with $(wc -l < "$LOCK_FILE" | tr -d ' ') asset entries."
fi

echo "=== Benchmark Assets Ready ==="
