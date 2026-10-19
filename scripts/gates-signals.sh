#!/usr/bin/env bash
# vis-003 Phase 1 exit gates 2, 10 and 15's CLI cases (specs/data_spec.md), on urban_grid's
# fixture of `scripts/fixture.sh` and the Midtown fixture of `scripts/fixture.sh midtown`,
# and the renders the user watches at gate 14, into scratch/out/signals/. Gates 5–9 and
# 15's `L` are
#   cargo test --release --test signals -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh (its --camera frames compared by hand), gates-ties.sh,
# gates-see-through.sh and gates-parts.sh with --no-signals; gate 3 re-runs the other test
# files and scripts; gate 4 is git diff; gate 11 is read by hand; gates 12 and 13 are
# recorded by hand. Gates 2 and 10 compare with Phase 6's renders of the same cases,
# copied to scratch/out/look-p6/ before Phase 1 was built.
# Offline. Needs ffmpeg/ffprobe, python3, perl, and the sqlite3 CLI (gate 15 edits a copy
# of urban_grid's results.db; /usr/bin/sqlite3 on macOS).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/signals"
P6="$ROOT/scratch/out/look-p6"
CACHE="$MID/buildings.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
command -v sqlite3 > /dev/null || { echo "FAIL: no sqlite3 CLI on PATH"; exit 1; }
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
differs() { # <earlier framemd5> <framemd5>: the dots change at least one frame
    python3 - "$1" "$2" <<'PY' || fail "gate10 $(basename "$2") equals $(basename "$1")"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
d = sum(x != y for x, y in zip(a, b))
print(f"gate10 {sys.argv[2].rsplit('/', 1)[1]} vs {sys.argv[1].rsplit('/', 1)[1]}: {d} of {len(b)} frames differ")
assert len(a) == len(b) and d >= 1
PY
}
case_() { # <frames> <project> <case> [extra args…]: off (--no-signals --no-hud) against Phase 6; on twice, equal, not Phase 6's
    local n=$1 proj=$2 c=$3
    shift 3
    render gate2 "$n" "$proj" "off-$c" --no-signals --no-hud "$@"
    same gate2 "$n" "$P6/look-$c.framemd5" "$OUT/off-$c.framemd5"
    render gate10 "$n" "$proj" "signals-$c" "$@"
    render gate10 "$n" "$proj" "signals-$c-2" "$@"
    same gate10 "$n" "$OUT/signals-$c.framemd5" "$OUT/signals-$c-2.framemd5"
    differs "$P6/look-$c.framemd5" "$OUT/signals-$c.framemd5"
}

# ── Gates 2 and 10: off is Phase 6's; on, deterministic, not Phase 6's ───────
case_ 8700 "$UG" ug-default
case_ 8700 "$UG" ug-camera --camera "$ROOT/tests/flight.toml"
case_ 1800 "$MID" ortho-city --buildings "$CACHE"
case_ 1800 "$MID" ortho-roads
case_ 1800 "$MID" city --buildings "$CACHE" --camera "$CITY"
case_ 1800 "$MID" city-roads --camera "$CITY"
case_ 1800 "$MID" orbit --buildings "$CACHE" --camera "$ORBIT"

# ── Gate 15: the flag's edges, through the CLI ───────────────────────────────
E="$OUT/gate15"
mkdir -p "$E"
rm -f "$E"/*.mp4
"$BIN" render --project "$UG" --scenario baseline --seed 42 --out "$E/opt-in.mp4" --signals \
    > "$E/opt-in.stdout" 2> "$E/opt-in.stderr"
code=$?
echo "gate15 render --signals: exit $code: $(head -1 "$E/opt-in.stderr")"
[ "$code" = 2 ] || fail "gate15 --signals exit $code"
[ ! -e "$E/opt-in.mp4" ] || fail "gate15 --signals wrote a video"
# A window that hangs fails the case: SIGALRM after 60 s.
view() { # <tag> <project> [extra args…]: exit code in $code
    local tag=$1 proj=$2
    shift 2
    perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$proj" --scenario baseline --seed 42 \
        "$@" --bench 1 > "$E/$tag.stdout" 2> "$E/$tag.stderr"
    code=$?
    echo "gate15 $tag: exit $code: $(tail -1 "$E/$tag.stderr")"
}
view view-ug-no-signals "$UG" --no-signals
[ "$code" = 0 ] || fail "gate15 view --no-signals exit $code"
tail -1 "$E/view-ug-no-signals.stderr" | grep -q '"mean_fps"' || fail "gate15 view --no-signals printed no bench JSON"

# Two copies of urban_grid, each results.db edited: a meso runner's row (less its `meso`
# key, which nothing reads), and a micro row with no timestep. The fixture is in WAL mode,
# and an immutable=1 reader does not read a -wal, so the journal goes to DELETE first.
for c in meso noclock; do
    rm -rf "${E:?}/$c"
    cp -R "$UG" "$E/$c"
done
sqlite3 "$E/meso/results.db" "PRAGMA journal_mode = DELETE;" \
    "UPDATE runs SET mode = 'meso', resolved_config = '{\"scenario\":\"baseline\",\"mode\":\"meso\",\"sim_params\":{\"duration\":300.0,\"warm_up\":30.0,\"random_seed\":42}}';" \
    > /dev/null || fail "gate15 sqlite3 meso"
sqlite3 "$E/noclock/results.db" "PRAGMA journal_mode = DELETE;" \
    "UPDATE runs SET resolved_config = '{\"scenario\":\"baseline\",\"sim_params\":{\"duration\":300.0,\"warm_up\":30.0,\"random_seed\":42}}';" \
    > /dev/null || fail "gate15 sqlite3 noclock"
for c in meso noclock; do
    echo "gate15 $c row: $(sqlite3 "$E/$c/results.db" "SELECT mode, status, resolved_config FROM runs;")"
done

MODE_ERR="error: $E/meso/results.db: the run is meso; signals are replayed only for micro runs (vis-003 OQ-3); --no-signals renders without signals"
CLOCK_ERR="error: $E/noclock/results.db: the run's timestep is not readable (runs.resolved_config): "
refused() { # <tag> <project> <expected head> <expected tail>
    local tag=$1 proj=$2 head=$3 tail=$4 line
    "$BIN" render --project "$proj" --scenario baseline --seed 42 --out "$E/$tag.mp4" \
        > "$E/$tag.stdout" 2> "$E/$tag.stderr"
    code=$?
    line=$(cat "$E/$tag.stderr")
    echo "gate15 $tag: exit $code: $line"
    [ "$code" = 1 ] || fail "gate15 render $tag exit $code"
    [ "$(wc -l < "$E/$tag.stderr" | tr -d ' ')" = 1 ] || fail "gate15 render $tag: not one stderr line"
    case "$line" in "$head"*"$tail") ;; *) fail "gate15 render $tag: message" ;; esac
    [ ! -e "$E/$tag.mp4" ] || fail "gate15 render $tag wrote a video"
}
refused render-meso "$E/meso" "$MODE_ERR" ""
refused render-noclock "$E/noclock" "$CLOCK_ERR" "; --no-signals renders without signals"

"$BIN" render --project "$E/meso" --scenario baseline --seed 42 --from 10 --to 20 --speedup 1 \
    --no-signals --out "$E/render-meso-no-signals.mp4" 2> "$E/render-meso-no-signals.stderr"
code=$?
echo "gate15 render --no-signals on meso: exit $code: $(tail -1 "$E/render-meso-no-signals.stderr")"
[ "$code" = 0 ] || fail "gate15 render --no-signals on meso exit $code"
probe=$(ffprobe -v error -select_streams v:0 -count_frames \
    -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$E/render-meso-no-signals.mp4")
echo "gate15 render --no-signals on meso: ffprobe $probe (expected 1920,1080,30/1,300)"
[ "$probe" = "1920,1080,30/1,300" ] || fail "gate15 render --no-signals on meso ffprobe"

view view-meso "$E/meso"
[ "$code" = 1 ] || fail "gate15 view on meso exit $code"
[ "$(cat "$E/view-meso.stderr")" = "$MODE_ERR" ] || fail "gate15 view on meso: message"
view view-meso-no-signals "$E/meso" --no-signals
[ "$code" = 0 ] || fail "gate15 view --no-signals on meso exit $code"
tail -1 "$E/view-meso-no-signals.stderr" | grep -q '"mean_fps"' || fail "gate15 view --no-signals on meso printed no bench JSON"

echo "gate14 videos: $OUT/signals-ortho-city.mp4 $OUT/signals-city.mp4 $OUT/signals-orbit.mp4 $OUT/signals-ug-camera.mp4"
[ $FAIL = 0 ] && echo "vis-003 Phase 1 gates 2, 10 and 15 (CLI): PASS" || echo "vis-003 Phase 1 gates 2, 10 and 15 (CLI): FAIL"
exit $FAIL
