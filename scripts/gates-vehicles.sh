#!/usr/bin/env bash
# vis-004 Phase 1 exit gates 1's and 2's hud-p2 cases, 8 and 13's CLI cases
# (specs/vehicles_spec.md), on urban_grid's fixture of `scripts/fixture.sh`, the Midtown
# fixture of `scripts/fixture.sh midtown` and the class mix of `scripts/fixture.sh mix`
# (drawn with Midtown's buildings), and the renders the user watches at gate 12, into
# scratch/out/vehicles/. Gates 5–7, 9 and 13's `V` are
#   cargo test --release --test vehicles -- --include-ignored --test-threads=1 --nocapture
# The rest of gates 1 and 2 are scripts/gates.sh (its --camera frames compared by hand),
# gates-ties.sh, gates-see-through.sh, gates-parts.sh, gates-signals.sh, gates-hud.sh and
# gates-links.sh; gate 3 re-runs the other test files and scripts; gate 4 is git diff;
# gates 10, 11 and 14 are recorded by hand (gate 10 from the times below). Gates 1, 2 and
# 8 compare with vis-003 Phase 2's renders of the same cases in scratch/out/hud-p2/.
# Offline. Needs ffmpeg/ffprobe, python3 and perl; gate 13's `view` opens a window.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
MIX="$ROOT/scratch/mix"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/vehicles"
P2="$ROOT/scratch/out/hud-p2"
CACHE="$MID/buildings.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
TRUCK="$ROOT/tests/truck-flight.toml"
WINDOW=(--from 300 --to 360 --speedup 1)
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video --bin fcd-derive || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }

render() { # <gate> <frames> <project> <tag> [extra args…]
    local g=$1 n=$2 proj=$3 tag=$4 t0 t1 probe code
    shift 4
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    t0=$(now)
    "$BIN" render --project "$proj" --scenario baseline --seed 42 \
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
all_differ() { # <hud-p2's framemd5> <framemd5>: a vehicle in every frame, and no shape is its box
    python3 - "$1" "$2" <<'PY' || fail "gate8 $(basename "$2") equals $(basename "$1") in some frame"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
d = sum(x != y for x, y in zip(a, b))
print(f"gate8 {sys.argv[2].rsplit('/', 1)[1]} vs {sys.argv[1].rsplit('/', 1)[1]}: {d} of {len(b)} frames differ")
assert len(a) == len(b) and d == len(b)
PY
}

# ── Gate 1: the orthographic render, as today and with --no-shapes ───────────
ortho() { # <frames> <project> <case> [extra args…]
    local n=$1 proj=$2 c=$3
    shift 3
    render gate1 "$n" "$proj" "ortho-$c" "$@"
    same gate1 "$n" "$P2/hud-$c.framemd5" "$OUT/ortho-$c.framemd5"
    render gate1 "$n" "$proj" "ortho-$c-noshapes" "$@" --no-shapes
    same gate1 "$n" "$P2/hud-$c.framemd5" "$OUT/ortho-$c-noshapes.framemd5"
}
ortho 8700 "$UG" ug-default
ortho 1800 "$MID" ortho-city "${WINDOW[@]}" --buildings "$CACHE"
ortho 1800 "$MID" ortho-roads "${WINDOW[@]}"

# ── Gate 2: --no-shapes in 3D is today's ─────────────────────────────────────
off() { # <frames> <project> <case> [extra args…]
    local n=$1 proj=$2 c=$3
    shift 3
    render gate2 "$n" "$proj" "off-$c" "$@" --no-shapes
    same gate2 "$n" "$P2/hud-$c.framemd5" "$OUT/off-$c.framemd5"
}
off 8700 "$UG" ug-camera --camera "$ROOT/tests/flight.toml"
off 1800 "$MID" city "${WINDOW[@]}" --buildings "$CACHE" --camera "$CITY"
off 1800 "$MID" city-roads "${WINDOW[@]}" --camera "$CITY"
off 1800 "$MID" orbit "${WINDOW[@]}" --buildings "$CACHE" --camera "$ORBIT"

# ── Gate 8: shapes on, twice each, equal; the look changed ───────────────────
on() { # <frames> <project> <case> [extra args…]
    local n=$1 proj=$2 c=$3
    shift 3
    render gate8 "$n" "$proj" "vehicles-$c" "$@"
    render gate8 "$n" "$proj" "vehicles-$c-2" "$@"
    same gate8 "$n" "$OUT/vehicles-$c.framemd5" "$OUT/vehicles-$c-2.framemd5"
}
on 1800 "$MID" city "${WINDOW[@]}" --buildings "$CACHE" --camera "$CITY"
all_differ "$P2/hud-city.framemd5" "$OUT/vehicles-city.framemd5"
on 1800 "$MIX" mix-city "${WINDOW[@]}" --buildings "$CACHE" --camera "$CITY"
on 1800 "$MIX" mix-orbit "${WINDOW[@]}" --buildings "$CACHE" --camera "$ORBIT"
on 900 "$MIX" truck --from 315 --to 345 --speedup 1 --buildings "$CACHE" --camera "$TRUCK"
on 8700 "$UG" ug-camera --camera "$ROOT/tests/flight.toml"
all_differ "$P2/hud-ug-camera.framemd5" "$OUT/vehicles-ug-camera.framemd5"
# Gate 12's twin of the close-up, in boxes.
render gate12 900 "$MIX" vehicles-truck-boxes --from 315 --to 345 --speedup 1 --buildings "$CACHE" \
    --camera "$TRUCK" --no-shapes

# ── Gate 13: the flag's edges and the schema error, through the CLI ──────────
E="$OUT/gate13"
mkdir -p "$E"
rm -f "$E"/*.mp4
INT="$OUT/class_int.parquet"
"$ROOT/target/release/fcd-derive" class-int --input "$UG/fcd/baseline_42.parquet" --output "$INT" \
    || fail "gate13 fcd-derive class-int"
SCHEMA="error: schema mismatch: $INT column \"vehicle_class\" is Int32, expected Dictionary(Int32, Utf8) or Utf8"
cli() { # <tag> <want exit> <want stderr line, or -> [render args…]: a 10 s render of urban_grid
    local tag=$1 want=$2 line=$3 code
    shift 3
    "$BIN" render --project "$UG" --scenario baseline --seed 42 --from 10 --to 20 --speedup 1 \
        --out "$E/$tag.mp4" "$@" > "$E/$tag.stdout" 2> "$E/$tag.stderr"
    code=$?
    echo "gate13 render $*: exit $code: $(tail -1 "$E/$tag.stderr")"
    [ "$code" = "$want" ] || fail "gate13 $tag exit $code, expected $want"
    if [ "$want" != 0 ]; then
        [ ! -e "$E/$tag.mp4" ] || fail "gate13 $tag wrote a video"
        [ "$(wc -l < "$E/$tag.stderr" | tr -d ' ')" = 1 ] || fail "gate13 $tag: stderr is not one line"
    fi
    [ "$line" = - ] || [ "$(cat "$E/$tag.stderr")" = "$line" ] || fail "gate13 $tag: stderr"
}
frames() { # <tag>: the video's ffprobe line against 300 frames, and its framemd5
    local probe
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$E/$1.mp4")
    echo "gate13 $1: ffprobe $probe (expected 1920,1080,30/1,300)"
    [ "$probe" = "1920,1080,30/1,300" ] || fail "gate13 $1 ffprobe"
    ffmpeg -y -v error -i "$E/$1.mp4" -f framemd5 "$E/$1.framemd5"
}
cli ortho 0 -
frames ortho
cli ortho-noshapes 0 - --no-shapes
frames ortho-noshapes
same gate13 300 "$E/ortho.framemd5" "$E/ortho-noshapes.framemd5"
cli shapes 2 - --shapes
cli class-int 1 "$SCHEMA" --camera "$ROOT/tests/flight.toml" --fcd "$INT"
cli class-int-noshapes 0 - --camera "$ROOT/tests/flight.toml" --fcd "$INT" --no-shapes
frames class-int-noshapes
view() { # <tag> <want exit> <want stderr line, or bench> [view args…]; a window that hangs fails: SIGALRM after 60 s
    local tag=$1 want=$2 line=$3 code
    shift 3
    perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
        "$@" --bench 1 > "$E/view-$tag.stdout" 2> "$E/view-$tag.stderr"
    code=$?
    echo "gate13 view $* --bench 1: exit $code: $(tail -1 "$E/view-$tag.stderr")"
    [ "$code" = "$want" ] || fail "gate13 view $tag exit $code, expected $want"
    if [ "$line" = bench ]; then
        tail -1 "$E/view-$tag.stderr" | grep -q '"mean_fps"' || fail "gate13 view $tag printed no bench JSON"
    else
        [ "$(cat "$E/view-$tag.stderr")" = "$line" ] || fail "gate13 view $tag: stderr"
    fi
}
view shapes 0 bench
view noshapes 0 bench --no-shapes
view class-int 1 "$SCHEMA" --fcd "$INT"
view class-int-noshapes 0 bench --fcd "$INT" --no-shapes

echo "gate12 videos: $OUT/vehicles-truck.mp4 $OUT/vehicles-truck-boxes.mp4 $OUT/vehicles-mix-orbit.mp4 $OUT/vehicles-city.mp4 $OUT/vehicles-ug-camera.mp4"
echo "gate12 vis-003 Phase 2's: $P2/hud-city.mp4 $P2/hud-ug-camera.mp4"
echo "gate12 line-up (tests/vehicles.rs gate 7): $OUT/vehicles-lineup-rear.ppm $OUT/vehicles-lineup-front.ppm"
[ $FAIL = 0 ] && echo "vis-004 Phase 1 gates 1, 2, 8 and 13 (CLI): PASS" || echo "vis-004 Phase 1 gates 1, 2, 8 and 13 (CLI): FAIL"
exit $FAIL
