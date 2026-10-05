#!/usr/bin/env bash
# vis-002 Phase 3 exit gates 2, 7, 10 (the CLI cases) and 12 (specs/city_spec.md), on the
# Midtown fixture of `scripts/fixture.sh midtown` and urban_grid's of `scripts/fixture.sh`,
# and the renders the user watches at gate 14. Gates 5, 6, 8, 9 and 11 are
#   cargo test --release --test see_through -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh, its --camera frames compared by hand; gate 3 re-runs the
# other test files and scripts; gate 13 is `view --bench`, recorded by hand.
# Gate 2 compares against scratch/ref-pin90b39292-*.framemd5, made by vis-001 Phase 7
# gate 5.
# Offline. Needs ffmpeg/ffprobe, python3 and perl; gate 10's `view` opens a window.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/see-through"
CACHE="$MID/buildings.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }

render() { # <gate> <tag> [extra args…]
    local gate=$1 tag=$2 t0 t1 probe code
    shift 2
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    t0=$(now)
    "$BIN" render --project "$MID" --scenario baseline --seed 42 --from 300 --to 360 \
        --speedup 1 --out "$OUT/$tag.mp4" "$@" 2> "$OUT/$tag.stderr"
    code=$?
    t1=$(now)
    echo "$gate $tag: exit $code in $(python3 -c "print(f'{$t1 - $t0:.1f}')") s, last line $(tail -1 "$OUT/$tag.stderr")"
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
ref() { echo "$ROOT/scratch/ref-pin90b39292-$1.framemd5"; }

# ── Gate 2: `--no-see-through`, and where nothing is cut ─────────────────────
render gate2 nost-ortho-city --buildings "$CACHE" --no-see-through
same gate2 "$(ref ortho-city)" "$OUT/nost-ortho-city.framemd5"
render gate2 nost-ortho-roads --no-see-through
same gate2 "$(ref ortho-roads)" "$OUT/nost-ortho-roads.framemd5"
render gate2 see-through-city-off --buildings "$CACHE" --camera "$CITY" --no-see-through
same gate2 "$(ref flight-city1)" "$OUT/see-through-city-off.framemd5"
render gate2 nost-flight-roads --camera "$CITY" --no-see-through
same gate2 "$(ref flight-roads1)" "$OUT/nost-flight-roads.framemd5"
render gate2 default-ortho-city --buildings "$CACHE"
same gate2 "$(ref ortho-city)" "$OUT/default-ortho-city.framemd5"
render gate2 default-ortho-roads
same gate2 "$(ref ortho-roads)" "$OUT/default-ortho-roads.framemd5"
render gate2 default-flight-roads --camera "$CITY"
same gate2 "$(ref flight-roads1)" "$OUT/default-flight-roads.framemd5"

# ── Gates 7 and 12: Midtown cut, deterministic; render times ─────────────────
render gate7 see-through-city-on1 --buildings "$CACHE" --camera "$CITY"
render gate7 see-through-city-on2 --buildings "$CACHE" --camera "$CITY"
same gate7 "$OUT/see-through-city-on1.framemd5" "$OUT/see-through-city-on2.framemd5"
render gate7 see-through-orbit-on1 --buildings "$CACHE" --camera "$ORBIT"
render gate7 see-through-orbit-on2 --buildings "$CACHE" --camera "$ORBIT"
same gate7 "$OUT/see-through-orbit-on1.framemd5" "$OUT/see-through-orbit-on2.framemd5"
render gate7 see-through-orbit-off --buildings "$CACHE" --camera "$ORBIT" --no-see-through
python3 - "$OUT/see-through-city-on1.framemd5" "$OUT/see-through-city-off.framemd5" \
    "$OUT/see-through-orbit-on1.framemd5" "$OUT/see-through-orbit-off.framemd5" <<'PY'
import sys
def frames(p): return [l for l in open(p) if not l.startswith("#")]
for on, off in [(sys.argv[1], sys.argv[2]), (sys.argv[3], sys.argv[4])]:
    a, b = frames(on), frames(off)
    print(f"gate7 {on.rsplit('/', 1)[1]} vs {off.rsplit('/', 1)[1]}: {sum(x != y for x, y in zip(a, b))} of {len(a)} frames differ (recorded)")
PY

# ── Gate 10: the flag's edges ────────────────────────────────────────────────
E="$OUT/gate10"
mkdir -p "$E"
rm -f "$E"/*.mp4
"$BIN" render --project "$MID" --scenario baseline --seed 42 --buildings "$CACHE" \
    --camera "$CITY" --out "$E/opt-in.mp4" --see-through > "$E/opt-in.stdout" 2> "$E/opt-in.stderr"
code=$?
echo "gate10 --see-through: exit $code: $(head -1 "$E/opt-in.stderr")"
[ "$code" = 2 ] || fail "gate10 --see-through exit $code"
[ ! -e "$E/opt-in.mp4" ] || fail "gate10 --see-through wrote a video"
for flag in "" --no-see-through; do
    tag=ug${flag:+-nost}
    "$BIN" render --project "$UG" --scenario baseline --seed 42 --buildings "$ROOT/tests/shapes.geojson" \
        --camera "$ROOT/tests/flight.toml" --out "$E/$tag.mp4" $flag > "$E/$tag.stdout" 2> "$E/$tag.stderr"
    code=$?
    nl=$(wc -l < "$E/$tag.stderr" | tr -d ' ')
    echo "gate10 urban_grid --buildings ${flag:-(no flag)}: exit $code, $nl line(s): $(head -c 300 "$E/$tag.stderr")"
    [ "$code" = 1 ] || fail "gate10 $tag exit $code"
    [ "$nl" = 1 ] || fail "gate10 $tag stderr has $nl lines"
    grep -q '^error: --buildings .*metadata.map_origin' "$E/$tag.stderr" || fail "gate10 $tag does not name metadata.map_origin"
    [ ! -e "$E/$tag.mp4" ] || fail "gate10 $tag wrote a video"
done
# A window that hangs fails the case: SIGALRM after 60 s.
perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
    --no-see-through --bench 1 > "$E/view.stdout" 2> "$E/view.stderr"
code=$?
echo "gate10 view --no-see-through --bench 1: exit $code: $(tail -1 "$E/view.stderr")"
[ "$code" = 0 ] || fail "gate10 view exit $code"
tail -1 "$E/view.stderr" | grep -q '"mean_fps"' || fail "gate10 view printed no bench JSON"

echo "gate14 videos: $OUT/see-through-orbit-on1.mp4 $OUT/see-through-orbit-off.mp4 $OUT/see-through-city-on1.mp4 $OUT/see-through-city-off.mp4"
[ $FAIL = 0 ] && echo "vis-002 Phase 3 gates 2, 7 and 10: PASS" || echo "vis-002 Phase 3 gates 2, 7 and 10: FAIL"
exit $FAIL
