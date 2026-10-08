#!/usr/bin/env bash
# vis-001 exit gates 1, 2 and 4 (Phases 1 and 2), on the fixture of scripts/fixture.sh.
# Gates 3, 6–11 (Phase 2; 3 and 7 in Phase 1), and the determinism regression check
# behind gate 2:
#   cargo test --release --test gates -- --ignored --test-threads=1 --nocapture
# Needs ffmpeg/ffprobe, duckdb (the FCD's time span, read independently of the renderer)
# and python3.
# Gate 5 is recorded by hand in specs/reviews/vis-001.md. The human check is Phase 1's
# gate 6 and Phase 2's gate 12.
# Phase 3 (view): gate 1's reference comparison is below; gates 2 (view) and 4–9 are
#   cargo test --release --test view -- --include-ignored --test-threads=1 --nocapture
# Phase 5 (3D camera): gate 10, the keyframed render, is below; gates 4–9, 11 and 12 are
#   cargo test --release --test camera -- --include-ignored --test-threads=1 --nocapture
# vis-002 Phase 5: the renders compared with a reference pass --no-streets.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
PROJ="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }

render() { # <tag> [extra args…]; writes $OUT/<tag>.mp4 and $OUT/<tag>.stderr
    local tag=$1; shift
    rm -f "$OUT/$tag.mp4"
    "$BIN" render --project "$PROJ" --scenario baseline --seed 42 --out "$OUT/$tag.mp4" "$@" \
        2> "$OUT/$tag.stderr"
}

check_video() { # <tag> <width> <height> <fps> <N>
    local tag=$1 w=$2 h=$3 fps=$4 n=$5
    local probe
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "gate1 $tag: ffprobe $probe (expected $w,$h,$fps/1,$n)"
    [ "$probe" = "$w,$h,$fps/1,$n" ] || fail "gate1 $tag ffprobe"
    python3 - "$OUT/$tag.stderr" "$n" "$OUT/$tag.mp4" <<'PY' || fail "gate1 $tag stderr"
import json, sys
lines = open(sys.argv[1]).read().splitlines()
n = int(sys.argv[2])
assert len(lines) == n + 1, f"{len(lines)} stderr lines, expected {n + 1}"
for i, l in enumerate(lines[:-1], 1):
    assert json.loads(l) == {"frame": i, "of": n}, f"line {i}: {l!r}"
assert json.loads(lines[-1]) == {"done": sys.argv[3]}, f"last line {lines[-1]!r}"
print(f"gate1 stderr: {n} progress lines then done — ok")
PY
}

# Gate 1 — defaults. N is checked against §2.4's formula with D taken from the fixture
# FCD's own min and max `time`, read with duckdb (independent of the renderer's reader).
# The spec's prediction was N = 8970 (FCD "0.1 … 299.1 s"); that span was never measured.
read -r T_MIN T_MAX < <(duckdb -noheader -csv -separator ' ' \
    -c "select min(time), max(time) from '$PROJ/fcd/baseline_42.parquet'") \
    || { echo "FAIL: cannot read the FCD time span with duckdb"; exit 1; }
N_DEFAULT=$(python3 -c "
import math
d = $T_MAX - $T_MIN
s = d / min(max(d, 30.0), 300.0)
print(math.ceil(d * 30 / s - 1e-6))")
echo "gate1 formula: FCD time $T_MIN … $T_MAX, D = $(python3 -c "print($T_MAX - $T_MIN)"), N = $N_DEFAULT"
t0=$(date +%s)
render default --no-streets || fail "gate1 default render exited $?"
dt=$(( $(date +%s) - t0 ))
echo "gate1 default: rendered $N_DEFAULT frames in $dt s ($(python3 -c "print(round($N_DEFAULT / max($dt, 1), 1))") frames/s)"
check_video default 1920 1080 30 "$N_DEFAULT"

# Gate 1 — explicit arguments: D = 60, N = ceil(60·24/2) = 720.
render explicit --from 60 --to 120 --speedup 2 --fps 24 --width 1280 --height 720 \
    || fail "gate1 explicit render exited $?"
check_video explicit 1280 720 24 720

# Gate 2 — the same inputs twice give the same decoded frames.
render default2 --no-streets || fail "gate2 second render exited $?"
# No stale hash file can be compared: delete both first, and -y in case one reappears.
rm -f "$OUT/default.framemd5" "$OUT/default2.framemd5"
ffmpeg -y -v error -i "$OUT/default.mp4" -f framemd5 "$OUT/default.framemd5"
ffmpeg -y -v error -i "$OUT/default2.mp4" -f framemd5 "$OUT/default2.framemd5"
if cmp -s "$OUT/default.framemd5" "$OUT/default2.framemd5"; then
    echo "gate2: $(grep -vc '^#' "$OUT/default.framemd5") frame hashes equal"
else
    fail "gate2 frame hashes differ"
fi

# Phase 3 gate 1 — the default render's frames equal the pin90b39292 reference's, when the
# reference hash file is present (built as vis-001 Phase 7 gate 5 says, then kept).
REF="$ROOT/scratch/ref-pin90b39292-default.framemd5"
if [ -f "$REF" ]; then
    python3 - "$REF" "$OUT/default.framemd5" <<'PY' || fail "phase3 gate1 frames differ from the reference"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"phase3 gate1: {eq} of {len(a)} frames equal to the pin90b39292 reference ({len(b)} rendered)")
assert eq == len(a) == len(b)
PY
else
    echo "phase3 gate1: no $REF, skipped"
fi

# Phase 5 gate 10 — a keyframed render (vis-001 §2.11.5): the defaults with
# --camera tests/flight.toml, twice; the frames of both runs must be equal.
t0=$(date +%s)
render camera --camera "$ROOT/tests/flight.toml" --no-streets || fail "phase5 gate10 render exited $?"
dt=$(( $(date +%s) - t0 ))
echo "phase5 gate10: rendered $N_DEFAULT frames with --camera in $dt s"
check_video camera 1920 1080 30 "$N_DEFAULT"
render camera2 --camera "$ROOT/tests/flight.toml" --no-streets || fail "phase5 gate10 second render exited $?"
rm -f "$OUT/camera.framemd5" "$OUT/camera2.framemd5"
ffmpeg -y -v error -i "$OUT/camera.mp4" -f framemd5 "$OUT/camera.framemd5"
ffmpeg -y -v error -i "$OUT/camera2.mp4" -f framemd5 "$OUT/camera2.framemd5"
if cmp -s "$OUT/camera.framemd5" "$OUT/camera2.framemd5"; then
    echo "phase5 gate10: $(grep -vc '^#' "$OUT/camera.framemd5") frame hashes equal"
else
    fail "phase5 gate10 frame hashes differ"
fi

# Gate 4 — input errors: non-zero exit, exactly one stderr line, no progress, no file.
gate4() { # <label> <env PATH> [args…]
    local label=$1 path=$2; shift 2
    local out="$OUT/gate4_$label.mp4"
    rm -f "$out" "$out.partial"
    PATH="$path" "$BIN" render --project "$PROJ" --scenario baseline --seed 42 --out "$out" "$@" \
        2> "$OUT/gate4_$label.stderr"
    local code=$?
    local nl
    nl=$(wc -l < "$OUT/gate4_$label.stderr" | tr -d ' ')
    echo "gate4 $label: exit $code, $nl stderr line(s): $(cat "$OUT/gate4_$label.stderr")"
    [ "$code" -ne 0 ] || fail "gate4 $label exit 0"
    [ "$nl" = 1 ] || fail "gate4 $label stderr has $nl lines"
    grep -q '"frame"' "$OUT/gate4_$label.stderr" && fail "gate4 $label printed progress"
    [ ! -e "$out" ] || fail "gate4 $label left $out"
    [ ! -e "$out.partial" ] || fail "gate4 $label left $out.partial"
}
gate4 missing-fcd "$PATH" --fcd "$ROOT/scratch/derived/does_not_exist.parquet"
gate4 unknown-link "$PATH" --fcd "$ROOT/scratch/derived/unknown_link.parquet"
gate4 no-ffmpeg "/usr/bin:/bin"

[ $FAIL = 0 ] && echo "gates 1, 2, 4 (and Phase 5 gate 10): PASS" || echo "gates 1, 2, 4 (and Phase 5 gate 10): FAIL"
exit $FAIL
