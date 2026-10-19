#!/usr/bin/env bash
# vis-003 Phase 2 exit gates 2, 8 and 13's CLI cases (specs/data_spec.md), on urban_grid's
# fixture of `scripts/fixture.sh` and the Midtown fixture of `scripts/fixture.sh midtown`,
# and the renders the user watches at gate 12, into scratch/out/hud/. Gates 5–7, 9 and
# 13's `H` are
#   cargo test --release --test hud -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh (its --camera frames compared by hand), gates-ties.sh,
# gates-see-through.sh and gates-parts.sh with --no-hud; gate 2's second half is
# gates-signals.sh's off- renders; gate 3 re-runs the other test files and scripts; gate 4
# is git diff; gates 10 and 11 are recorded by hand (gate 10 from the times below). Gates 2
# and 8 compare with Phase 1's renders of the same cases, copied to scratch/out/signals-p1/
# before Phase 2 was built.
# Offline. Needs ffmpeg/ffprobe, python3 and perl.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/hud"
P1="$ROOT/scratch/out/signals-p1"
CACHE="$MID/buildings.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }

render() { # <gate> <frames> <project> <tag> [extra args…]; Midtown renders its 300–360 s window
    local g=$1 n=$2 proj=$3 tag=$4 t0 t1 probe code window=()
    shift 4
    [ "$proj" = "$MID" ] && window=(--from 300 --to 360 --speedup 1)
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    t0=$(now)
    "$BIN" render --project "$proj" --scenario baseline --seed 42 ${window[@]+"${window[@]}"} \
        --out "$OUT/$tag.mp4" "$@" 2> "$OUT/$tag.stderr"
    code=$?
    t1=$(now)
    echo "$g $tag: exit $code in $(python3 -c "print(f'{$t1 - $t0:.1f}')") s, load $(sysctl -n vm.loadavg), last line $(tail -1 "$OUT/$tag.stderr")"
    [ "$code" = 0 ] || { fail "$g $tag exited $code"; return; }
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "$g $tag: ffprobe $probe (expected 1920,1080,30/1,$n)"
    [ "$probe" = "1920,1080,30/1,$n" ] || fail "$g $tag ffprobe"
    ffmpeg -y -v error -i "$OUT/$tag.mp4" -f framemd5 "$OUT/$tag.framemd5"
}
same() { # <gate> <frames> <framemd5 a> <framemd5 b>
    python3 - "$3" "$4" "$2" "$1" <<'PY' || fail "$1 $(basename "$3") vs $(basename "$4")"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"{sys.argv[4]} {sys.argv[1].rsplit('/', 1)[1]} vs {sys.argv[2].rsplit('/', 1)[1]}: {eq} of {len(a)} frames equal ({len(b)})")
assert eq == len(a) == len(b) == int(sys.argv[3])
PY
}
all_differ() { # <Phase 1's framemd5> <framemd5>: the HUD changes every frame
    python3 - "$1" "$2" <<'PY' || fail "gate8 $(basename "$2") equals $(basename "$1") in some frame"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
d = sum(x != y for x, y in zip(a, b))
print(f"gate8 {sys.argv[2].rsplit('/', 1)[1]} vs {sys.argv[1].rsplit('/', 1)[1]}: {d} of {len(b)} frames differ")
assert len(a) == len(b) and d == len(b)
PY
}
case_() { # <frames> <project> <case> [extra args…]: --no-hud against Phase 1; on twice, equal, every frame not Phase 1's
    local n=$1 proj=$2 c=$3
    shift 3
    render gate2 "$n" "$proj" "nohud-$c" --no-hud "$@"
    same gate2 "$n" "$P1/signals-$c.framemd5" "$OUT/nohud-$c.framemd5"
    render gate8 "$n" "$proj" "hud-$c" "$@"
    render gate8 "$n" "$proj" "hud-$c-2" "$@"
    same gate8 "$n" "$OUT/hud-$c.framemd5" "$OUT/hud-$c-2.framemd5"
    all_differ "$P1/signals-$c.framemd5" "$OUT/hud-$c.framemd5"
}

# ── Gates 2 and 8: --no-hud is Phase 1's; on, deterministic, every frame not Phase 1's ──
case_ 8700 "$UG" ug-default
case_ 8700 "$UG" ug-camera --camera "$ROOT/tests/flight.toml"
case_ 1800 "$MID" ortho-city --buildings "$CACHE"
case_ 1800 "$MID" ortho-roads
case_ 1800 "$MID" city --buildings "$CACHE" --camera "$CITY"
case_ 1800 "$MID" city-roads --camera "$CITY"
case_ 1800 "$MID" orbit --buildings "$CACHE" --camera "$ORBIT"

# ── Gate 13: the flag's edges, through the CLI ───────────────────────────────
E="$OUT/gate13"
mkdir -p "$E"
rm -f "$E"/*.mp4
"$BIN" render --project "$UG" --scenario baseline --seed 42 --out "$E/opt-in.mp4" --hud \
    > "$E/opt-in.stdout" 2> "$E/opt-in.stderr"
code=$?
echo "gate13 render --hud: exit $code: $(head -1 "$E/opt-in.stderr")"
[ "$code" = 2 ] || fail "gate13 --hud exit $code"
[ ! -e "$E/opt-in.mp4" ] || fail "gate13 --hud wrote a video"
# A window that hangs fails the case: SIGALRM after 60 s.
perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
    --no-hud --bench 1 > "$E/view-ug-no-hud.stdout" 2> "$E/view-ug-no-hud.stderr"
code=$?
echo "gate13 view --no-hud: exit $code: $(tail -1 "$E/view-ug-no-hud.stderr")"
[ "$code" = 0 ] || fail "gate13 view --no-hud exit $code"
tail -1 "$E/view-ug-no-hud.stderr" | grep -q '"mean_fps"' || fail "gate13 view --no-hud printed no bench JSON"
"$BIN" render --project "$UG" --scenario baseline --seed 42 --from 10 --to 12 --speedup 1 \
    --width 64 --height 36 --out "$E/tiny.mp4" 2> "$E/tiny.stderr"
code=$?
echo "gate13 render 64x36: exit $code: $(tail -1 "$E/tiny.stderr")"
[ "$code" = 0 ] || fail "gate13 render 64x36 exit $code"
probe=$(ffprobe -v error -select_streams v:0 -count_frames \
    -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$E/tiny.mp4")
echo "gate13 render 64x36: ffprobe $probe (expected 64,36,30/1,60)"
[ "$probe" = "64,36,30/1,60" ] || fail "gate13 render 64x36 ffprobe"

echo "gate12 videos: $OUT/hud-ortho-city.mp4 $OUT/hud-city.mp4 $OUT/hud-orbit.mp4 $OUT/hud-ug-camera.mp4"
echo "gate12 Phase 1's: $P1/signals-ortho-city.mp4 $P1/signals-city.mp4 $P1/signals-orbit.mp4 $P1/signals-ug-camera.mp4"
[ $FAIL = 0 ] && echo "vis-003 Phase 2 gates 2, 8 and 13 (CLI): PASS" || echo "vis-003 Phase 2 gates 2, 8 and 13 (CLI): FAIL"
exit $FAIL
