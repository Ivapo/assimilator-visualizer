#!/usr/bin/env bash
# vis-002 Phase 5 exit gates 10 and 11 (specs/city_spec.md), on urban_grid's fixture of
# `scripts/fixture.sh` and the Midtown fixture of `scripts/fixture.sh midtown`, and the
# renders the user watches at gate 15, into scratch/out/streets/. Gates 5–9 and 12 are
#   cargo test --release --test streets -- --include-ignored --test-threads=1 --nocapture
# Gates 1 and 2 are scripts/gates.sh (its --camera frames compared by hand),
# gates-ties.sh, gates-see-through.sh and gates-parts.sh with --no-streets; gate 3 re-runs
# the other test files and scripts; gate 4 is git diff; gate 14 is `view --bench`,
# recorded by hand. Gate 10 compares with scratch/ref-pin90b39292-*.framemd5, made by
# vis-001 Phase 7 gate 5, only to count the frames streets change.
# Offline. Needs ffmpeg/ffprobe, python3 and perl; gate 11's `view` opens a window.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/streets"
CACHE="$MID/buildings.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }

render() { # <frames> <project> <tag> [extra args…]; Midtown renders its 300–360 s window
    local n=$1 proj=$2 tag=$3 t0 t1 probe code window=()
    shift 3
    [ "$proj" = "$MID" ] && window=(--from 300 --to 360 --speedup 1)
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    t0=$(now)
    "$BIN" render --project "$proj" --scenario baseline --seed 42 ${window[@]+"${window[@]}"} \
        --out "$OUT/$tag.mp4" "$@" 2> "$OUT/$tag.stderr"
    code=$?
    t1=$(now)
    echo "gate10 $tag: exit $code in $(python3 -c "print(f'{$t1 - $t0:.1f}')") s, load $(sysctl -n vm.loadavg), last line $(tail -1 "$OUT/$tag.stderr")"
    [ "$code" = 0 ] || { fail "gate10 $tag exited $code"; return; }
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "gate10 $tag: ffprobe $probe (expected 1920,1080,30/1,$n)"
    [ "$probe" = "1920,1080,30/1,$n" ] || fail "gate10 $tag ffprobe"
    ffmpeg -y -v error -i "$OUT/$tag.mp4" -f framemd5 "$OUT/$tag.framemd5"
}
same() { # <frames> <framemd5 a> <framemd5 b>
    python3 - "$2" "$3" "$1" <<'PY' || fail "gate10 $(basename "$2") vs $(basename "$3")"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"gate10 {sys.argv[1].rsplit('/', 1)[1]} vs {sys.argv[2].rsplit('/', 1)[1]}: {eq} of {len(a)} frames equal ({len(b)})")
assert eq == len(a) == len(b) == int(sys.argv[3])
PY
}
differs() { # <reference> <framemd5>: streets change at least one frame
    python3 - "$ROOT/scratch/ref-pin90b39292-$1.framemd5" "$2" <<'PY' || fail "gate10 $(basename "$2") equals ref-pin90b39292-$1"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
d = sum(x != y for x, y in zip(a, b))
print(f"gate10 {sys.argv[2].rsplit('/', 1)[1]} vs {sys.argv[1].rsplit('/', 1)[1]}: {d} of {len(b)} frames differ")
assert len(a) == len(b) and d >= 1
PY
}
case_() { # <frames> <project> <tag> <reference> [extra args…]: twice, equal, and not today's
    local n=$1 proj=$2 tag=$3 ref=$4
    shift 4
    render "$n" "$proj" "$tag" "$@"
    render "$n" "$proj" "$tag-2" "$@"
    same "$n" "$OUT/$tag.framemd5" "$OUT/$tag-2.framemd5"
    differs "$ref" "$OUT/$tag.framemd5"
}

# ── Gate 10: on by default, deterministic ────────────────────────────────────
case_ 8700 "$UG" streets-ug-default default
case_ 8700 "$UG" streets-ug-camera camera --camera "$ROOT/tests/flight.toml"
case_ 1800 "$MID" streets-ortho-city ortho-city --buildings "$CACHE"
case_ 1800 "$MID" streets-ortho-roads ortho-roads
case_ 1800 "$MID" streets-city city-on --buildings "$CACHE" --camera "$CITY"
case_ 1800 "$MID" streets-city-roads flight-roads1 --camera "$CITY"
case_ 1800 "$MID" streets-orbit orbit-on --buildings "$CACHE" --camera "$ORBIT"

# ── Gate 11: the flag's edges ────────────────────────────────────────────────
E="$OUT/gate11"
mkdir -p "$E"
rm -f "$E"/*.mp4
"$BIN" render --project "$MID" --scenario baseline --seed 42 --from 300 --to 360 --speedup 1 \
    --buildings "$CACHE" --camera "$CITY" --out "$E/opt-in.mp4" --streets \
    > "$E/opt-in.stdout" 2> "$E/opt-in.stderr"
code=$?
echo "gate11 --streets: exit $code: $(head -1 "$E/opt-in.stderr")"
[ "$code" = 2 ] || fail "gate11 --streets exit $code"
[ ! -e "$E/opt-in.mp4" ] || fail "gate11 --streets wrote a video"
# A window that hangs fails the case: SIGALRM after 60 s.
perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
    --no-streets --bench 1 > "$E/view.stdout" 2> "$E/view.stderr"
code=$?
echo "gate11 view --no-streets --bench 1: exit $code: $(tail -1 "$E/view.stderr")"
[ "$code" = 0 ] || fail "gate11 view exit $code"
tail -1 "$E/view.stderr" | grep -q '"mean_fps"' || fail "gate11 view printed no bench JSON"

echo "gate15 videos: $OUT/streets-ortho-city.mp4 $OUT/streets-city.mp4 $OUT/streets-orbit.mp4 $OUT/streets-ug-camera.mp4"
[ $FAIL = 0 ] && echo "vis-002 Phase 5 gates 10 and 11: PASS" || echo "vis-002 Phase 5 gates 10 and 11: FAIL"
exit $FAIL
