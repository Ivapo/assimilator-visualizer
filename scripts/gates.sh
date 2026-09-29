#!/usr/bin/env bash
# vis-001 Phase 1 exit gates 1, 2 and 4, on the fixture of scripts/fixture.sh.
# Gates 3 and 7: cargo test --test gates -- --ignored --test-threads=1 --nocapture
# Gate 5 is recorded by hand in specs/reviews/vis-001.md; gate 6 is a human watching.
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

# Gate 1 — defaults. The spec predicts N = 8970 for FCD 0.1 … 299.1 s at speedup 1.
N_DEFAULT=${N_DEFAULT:-8970}
t0=$(date +%s)
render default || fail "gate1 default render exited $?"
echo "gate1 default: rendered in $(( $(date +%s) - t0 )) s"
check_video default 1920 1080 30 "$N_DEFAULT"

# Gate 1 — explicit arguments: D = 60, N = ceil(60·24/2) = 720.
render explicit --from 60 --to 120 --speedup 2 --fps 24 --width 1280 --height 720 \
    || fail "gate1 explicit render exited $?"
check_video explicit 1280 720 24 720

# Gate 2 — the same inputs twice give the same decoded frames.
render default2 || fail "gate2 second render exited $?"
ffmpeg -v error -i "$OUT/default.mp4" -f framemd5 "$OUT/default.framemd5"
ffmpeg -v error -i "$OUT/default2.mp4" -f framemd5 "$OUT/default2.framemd5"
if cmp -s "$OUT/default.framemd5" "$OUT/default2.framemd5"; then
    echo "gate2: $(grep -vc '^#' "$OUT/default.framemd5") frame hashes equal"
else
    fail "gate2 frame hashes differ"
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

[ $FAIL = 0 ] && echo "gates 1, 2, 4: PASS" || echo "gates 1, 2, 4: FAIL"
exit $FAIL
