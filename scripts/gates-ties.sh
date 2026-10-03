#!/usr/bin/env bash
# vis-001 Phase 6 exit gates 2 and 9 (specs/visualizer_spec.md), on the Midtown fixture of
# `scripts/fixture.sh midtown`: the orthographic render unchanged, and the `--camera`
# flight deterministic (vis-002 OQ-8). Gates 6–8 are
#   cargo test --release --test ties -- --include-ignored --test-threads=1 --nocapture
# Gate 2 compares against scratch/ref-ortho-{city,roads}-92d09a0.framemd5, copied from
# scripts/gates-credit.sh's gate 14 output before Phase 6 changed any code.
# Offline. Needs ffmpeg/ffprobe and python3.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
OUT="$ROOT/scratch/out/ties"
CACHE="$MID/buildings.geojson"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }

render() { # <gate> <tag> [extra args…]
    local gate=$1 tag=$2 t0=$SECONDS probe code
    shift 2
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    "$BIN" render --project "$MID" --scenario baseline --seed 42 --from 300 --to 360 \
        --speedup 1 --out "$OUT/$tag.mp4" "$@" 2> "$OUT/$tag.stderr"
    code=$?
    echo "$gate $tag: exit $code in $((SECONDS - t0)) s, last line $(tail -1 "$OUT/$tag.stderr")"
    [ "$code" = 0 ] || { fail "$gate $tag exited $code"; return; }
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "$gate $tag: ffprobe $probe (expected 1920,1080,30/1,1800)"
    [ "$probe" = "1920,1080,30/1,1800" ] || fail "$gate $tag ffprobe"
    ffmpeg -y -v error -i "$OUT/$tag.mp4" -f framemd5 "$OUT/$tag.framemd5"
}
same() { # <gate> <framemd5 a> <framemd5 b>
    python3 - "$2" "$3" "$1" <<'PY' || fail "$1 $(basename "$2") vs $(basename "$3")"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"{sys.argv[3]} {sys.argv[1].rsplit('/', 1)[1]} vs {sys.argv[2].rsplit('/', 1)[1]}: {eq} of {len(a)} frames equal ({len(b)})")
assert eq == len(a) == len(b) == 1800
PY
}

# ── Gate 2: the orthographic render, unchanged ───────────────────────────────
render gate2 ortho-city --buildings "$CACHE"
same gate2 "$ROOT/scratch/ref-ortho-city-92d09a0.framemd5" "$OUT/ortho-city.framemd5"
render gate2 ortho-roads
same gate2 "$ROOT/scratch/ref-ortho-roads-92d09a0.framemd5" "$OUT/ortho-roads.framemd5"

# ── Gate 9: Midtown's flight, deterministic ──────────────────────────────────
render gate9 flight-city1 --buildings "$CACHE" --camera "$ROOT/tests/city-flight.toml"
render gate9 flight-city2 --buildings "$CACHE" --camera "$ROOT/tests/city-flight.toml"
same gate9 "$OUT/flight-city1.framemd5" "$OUT/flight-city2.framemd5"
render gate9 flight-roads1 --camera "$ROOT/tests/city-flight.toml"
render gate9 flight-roads2 --camera "$ROOT/tests/city-flight.toml"
same gate9 "$OUT/flight-roads1.framemd5" "$OUT/flight-roads2.framemd5"

[ $FAIL = 0 ] && echo "vis-001 Phase 6 gates 2 and 9: PASS" || echo "vis-001 Phase 6 gates 2 and 9: FAIL"
exit $FAIL
