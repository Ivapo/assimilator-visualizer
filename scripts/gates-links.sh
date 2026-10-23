#!/usr/bin/env bash
# vis-003 Phase 3 exit gates 2, 8 and 13's CLI cases (specs/data_spec.md), on urban_grid's
# fixture of `scripts/fixture.sh` and the Midtown fixture of `scripts/fixture.sh midtown`,
# and the renders the user watches at gate 12, into scratch/out/links/. Gates 5–7, 9 and
# 13's `C` are
#   cargo test --release --test links -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh (its --camera frames compared by hand), gates-ties.sh,
# gates-see-through.sh and gates-parts.sh, unedited; gate 3 re-runs the other test files
# and scripts; gate 4 is git diff; gates 10 and 11 are recorded by hand (gate 10 from the
# times below). Gates 2 and 8 compare with Phase 2's renders of the same cases, copied to
# scratch/out/hud-p2/ before Phase 3 was built.
# Offline. Needs ffmpeg/ffprobe, python3 and perl.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/links"
P2="$ROOT/scratch/out/hud-p2"
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
all_differ() { # <Phase 2's framemd5> <framemd5>: the legend and the grey boxes change every frame
    python3 - "$1" "$2" <<'PY' || fail "gate8 $(basename "$2") equals $(basename "$1") in some frame"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
d = sum(x != y for x, y in zip(a, b))
print(f"gate8 {sys.argv[2].rsplit('/', 1)[1]} vs {sys.argv[1].rsplit('/', 1)[1]}: {d} of {len(b)} frames differ")
assert len(a) == len(b) and d == len(b)
PY
}
case_() { # <frames> <project> <case> <link args> [extra args…]: off is Phase 2's; on twice, equal, every frame not Phase 2's
    local n=$1 proj=$2 c=$3 on=()
    read -r -a on <<< "$4"
    shift 4
    render gate2 "$n" "$proj" "off-$c" "$@"
    same gate2 "$n" "$P2/hud-$c.framemd5" "$OUT/off-$c.framemd5"
    render gate8 "$n" "$proj" "links-$c" "$@" "${on[@]}"
    render gate8 "$n" "$proj" "links-$c-2" "$@" "${on[@]}"
    same gate8 "$n" "$OUT/links-$c.framemd5" "$OUT/links-$c-2.framemd5"
    all_differ "$P2/hud-$c.framemd5" "$OUT/links-$c.framemd5"
}

# ── Gates 2 and 8: off is Phase 2's; on, deterministic, every frame not Phase 2's ──
case_ 8700 "$UG" ug-default "--link-colours density"
case_ 8700 "$UG" ug-camera "--link-colours" --camera "$ROOT/tests/flight.toml"
case_ 1800 "$MID" ortho-city "--link-colours" --buildings "$CACHE"
case_ 1800 "$MID" ortho-roads "--link-colours speed --link-lanes"
case_ 1800 "$MID" city "--link-colours speed --link-lanes" --buildings "$CACHE" --camera "$CITY"
case_ 1800 "$MID" city-roads "--link-colours stopped" --camera "$CITY"
case_ 1800 "$MID" orbit "--link-colours stopped --link-window 1" --buildings "$CACHE" --camera "$ORBIT"

# ── Gate 13: the flags' edges, through the CLI ───────────────────────────────
E="$OUT/gate13"
mkdir -p "$E"
rm -f "$E"/*.mp4
SHORT="error: --link-window 0.5 s is shorter than the run's FCD interval (1.000 s); --link-window 1.000 colours by each frame's own FCD row"
cli() { # <tag> <want exit> <want stderr line, or -> [render args…]: a 10 s render of urban_grid
    local tag=$1 want=$2 line=$3 code
    shift 3
    "$BIN" render --project "$UG" --scenario baseline --seed 42 --from 10 --to 20 --speedup 1 \
        --out "$E/$tag.mp4" "$@" > "$E/$tag.stdout" 2> "$E/$tag.stderr"
    code=$?
    echo "gate13 render $*: exit $code: $(head -1 "$E/$tag.stderr")"
    [ "$code" = "$want" ] || fail "gate13 $tag exit $code, expected $want"
    if [ "$want" != 0 ]; then
        [ ! -e "$E/$tag.mp4" ] || fail "gate13 $tag wrote a video"
        [ "$(wc -l < "$E/$tag.stderr" | tr -d ' ')" = 1 ] || fail "gate13 $tag: stderr is not one line"
    fi
    [ "$line" = - ] || [ "$(cat "$E/$tag.stderr")" = "$line" ] || fail "gate13 $tag: stderr"
}
cli short 1 "$SHORT" --link-colours --link-window 0.5
cli interval 0 - --link-colours --link-window 1
probe=$(ffprobe -v error -select_streams v:0 -count_frames \
    -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$E/interval.mp4")
echo "gate13 render --link-window 1: ffprobe $probe (expected 1920,1080,30/1,300)"
[ "$probe" = "1920,1080,30/1,300" ] || fail "gate13 --link-window 1 ffprobe"
cli zero 1 "error: --link-window must be a positive number of seconds (got 0)" --link-colours --link-window 0
cli negative 1 "error: --link-window must be a positive number of seconds (got -5)" --link-colours --link-window -5
cli nan 1 "error: --link-window must be a positive number of seconds (got NaN)" --link-colours --link-window nan
cli window-alone 2 - --link-window 30
cli lanes-alone 2 - --link-lanes
cli flow 2 - --link-colours flow
# A window that hangs fails the case: SIGALRM after 60 s.
perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
    --link-window 0.5 --bench 1 > "$E/view-short.stdout" 2> "$E/view-short.stderr"
code=$?
echo "gate13 view --link-window 0.5: exit $code: $(tail -1 "$E/view-short.stderr")"
[ "$code" = 1 ] || fail "gate13 view --link-window 0.5 exit $code"
[ "$(cat "$E/view-short.stderr")" = "$SHORT" ] || fail "gate13 view --link-window 0.5: stderr"
perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$UG" --scenario baseline --seed 42 \
    --link-colours stopped --link-lanes --bench 1 > "$E/view-stopped.stdout" 2> "$E/view-stopped.stderr"
code=$?
echo "gate13 view --link-colours stopped --link-lanes: exit $code: $(tail -1 "$E/view-stopped.stderr")"
[ "$code" = 0 ] || fail "gate13 view --link-colours stopped --link-lanes exit $code"
tail -1 "$E/view-stopped.stderr" | grep -q '"mean_fps"' || fail "gate13 view --link-colours stopped printed no bench JSON"

echo "gate12 videos: $OUT/links-ortho-city.mp4 $OUT/links-city.mp4 $OUT/links-orbit.mp4 $OUT/links-ug-camera.mp4"
echo "gate12 Phase 2's: $P2/hud-ortho-city.mp4 $P2/hud-city.mp4 $P2/hud-orbit.mp4 $P2/hud-ug-camera.mp4"
[ $FAIL = 0 ] && echo "vis-003 Phase 3 gates 2, 8 and 13 (CLI): PASS" || echo "vis-003 Phase 3 gates 2, 8 and 13 (CLI): FAIL"
exit $FAIL
