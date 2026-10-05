#!/usr/bin/env bash
# vis-002 Phase 4 exit gates 5, 2, 9, 11 (its CLI half) and 13 (specs/city_spec.md), on the
# Midtown fixture of `scripts/fixture.sh midtown`, and the renders the user watches at gate
# 15, into scratch/out/parts/. Gates 6, 7, 8, 10, 11 and 12 are
#   cargo test --release --test parts -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh; gate 3 re-runs the other test files and scripts; gate 4 is
# git diff; gate 14 is `view --bench`, recorded by hand.
# Gate 5 reads the release saved while drafting through its mirror,
# scratch/overture-2026-09-23.1/theme=buildings (type=building/ and type=building_part/),
# laid out by hand; it ends with `REFETCH=1 scripts/fixture.sh midtown`, after copying the
# fixture's caches and logs to scratch/out/parts/backup/, and stops the script if
# buildings.geojson is not today's cache after it. Gate 2 compares against
# scratch/ref-pin90b39292-*.framemd5, made by vis-001 Phase 7 gate 5.
# Offline. Needs duckdb, ffmpeg/ffprobe and python3.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
OUT="$ROOT/scratch/out/parts"
MIRROR="$ROOT/scratch/overture-2026-09-23.1/theme=buildings"
TODAY="$MID/buildings.geojson"
PARTS="$MID/buildings-parts.geojson"
CITY="$ROOT/tests/city-flight.toml"
ORBIT="$ROOT/tests/see-through-flight.toml"
SHA_TODAY=f241ccbd9a2fba2e3c80fa28ec7522ca2a33b84628c18954952ad6bc1d4a1dfd
SHA_PARTS=714e2f3a3ab612618100b82b698cc08d49a9badb96caa7afc31e8dbbdf159b14
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }
now() { python3 -c 'import time; print(f"{time.time():.3f}")'; }
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }

# ── Gate 5: the fetch from the saved data ────────────────────────────────────
F="$OUT/fetch"
rm -rf "$F"
mkdir -p "$F/one" "$F/two" "$F/np-one" "$F/np-two" "$F/none"
[ -d "$MIRROR/type=building" ] && [ -d "$MIRROR/type=building_part" ] \
    || { echo "FAIL: gate5 no mirror at $MIRROR"; exit 1; }
# The report line apart from "seconds" and "out".
report() { python3 -c 'import re, sys; print(re.sub(r", \"seconds\": \d+, \"out\": \".*\"\}$", "}", sys.stdin.read().strip()))' < "$1"; }
WANT='{"release": "2026-09-23.1", "bbox": [-73.9938413, 40.7544211, -73.9643086, 40.7740495], "buildings": 4336, "height": 4277, "num_floors": 8, "default": 51, "sources": {"Microsoft ML Buildings": 29, "OpenStreetMap": 4327, "USGS Lidar": 316}, "no_sources": 0, "raised": 3, "parts": {"count": 4209, "buildings": 487, "height": 4152, "num_floors": 17, "default": 40, "raised": 695, "sources": {"OpenStreetMap": 4209}, "no_sources": 0}, "bytes": 4944032}'
fetch() { # <dir> [extra args…]
    local d=$1 t0 code
    shift
    t0=$SECONDS
    scripts/fetch-buildings.sh --project "$MID" --out "$F/$d/a.geojson" --source "$MIRROR" "$@" \
        > "$F/$d.report" 2> "$F/$d.stderr"
    code=$?
    echo "gate5 fetch $d ${*:-(parts)}: exit $code in $((SECONDS - t0)) s: $(cat "$F/$d.report" "$F/$d.stderr")"
    [ "$code" = 0 ] || fail "gate5 fetch $d exited $code"
}
for d in one two; do
    fetch $d
    got=$(report "$F/$d.report")
    [ "$got" = "$WANT" ] || fail "gate5 $d report: $got"
    bytes=$(wc -c < "$F/$d/a.geojson" | tr -d ' ')
    echo "gate5 $d: $bytes bytes, SHA-256 $(sha "$F/$d/a.geojson")"
    [ "$bytes" = 4944032 ] || fail "gate5 $d bytes"
    [ "$(sha "$F/$d/a.geojson")" = "$SHA_PARTS" ] || fail "gate5 $d SHA-256"
done
python3 - "$F/one/a.geojson" "$TODAY" <<'PY' || fail "gate5 the buildings' lines are not today's"
import re, sys
new = open(sys.argv[1]).read().split("\n")
old = open(sys.argv[2]).read().split("\n")
head = old.index('"features": [') + 1
nb = sum(1 for l in old if l.startswith('{ "type": "Feature"'))
assert new[:head] == old[:head], "header lines"
bad = 0
for i in range(nb):
    l = new[head + i].replace('"building_id": null, ', "", 1)
    l = re.sub(r', "min_height": [^,]+, "min_floor": [^,]+(?=, "sources")', "", l, count=1)
    if i == nb - 1:
        assert l.endswith(","), "the last building's line ends in a comma"
        l = l[:-1]
    bad += l != old[head + i]
print(f"gate5 strip-and-compare: {nb} building lines, {bad} differ from today's; header {head} lines equal")
assert bad == 0
PY
echo "gate5 fixture step 6: $PARTS SHA-256 $(sha "$PARTS")"
[ "$(sha "$PARTS")" = "$SHA_PARTS" ] || fail "gate5 buildings-parts.geojson SHA-256"
scripts/fetch-buildings.sh --project "$MID" --out "$F/none/a.geojson" --source "$ROOT/scratch/no-such-dir" \
    > "$F/none.report" 2> "$F/none.stderr"
code=$?
nl=$(wc -l < "$F/none.stderr" | tr -d ' ')
echo "gate5 --source scratch/no-such-dir: exit $code, $nl line(s): $(cat "$F/none.stderr")"
[ "$code" != 0 ] || fail "gate5 no-such-dir exit 0"
[ "$nl" = 1 ] || fail "gate5 no-such-dir stderr has $nl lines"
[ -z "$(ls -A "$F/none")" ] || fail "gate5 no-such-dir left something beside --out"
WANT_NP=$(report "$MID/fetch.log")
for d in np-one np-two; do
    fetch $d --no-parts
    got=$(report "$F/$d.report")
    [ "$got" = "$WANT_NP" ] || fail "gate5 $d report: $got (fetch.log: $WANT_NP)"
    bytes=$(wc -c < "$F/$d/a.geojson" | tr -d ' ')
    echo "gate5 $d: $bytes bytes, SHA-256 $(sha "$F/$d/a.geojson")"
    [ "$bytes" = 2284830 ] || fail "gate5 $d bytes"
    [ "$(sha "$F/$d/a.geojson")" = "$SHA_TODAY" ] || fail "gate5 $d SHA-256"
done
# Last in gate 5: the fixture with REFETCH=1, the caches and logs kept aside first.
B="$OUT/backup"
rm -rf "$B"
mkdir -p "$B"
cp -p "$TODAY" "$PARTS" "$MID/fetch.log" "$MID/fetch-parts.log" "$B/" || { echo "FAIL: gate5 cannot back up the fixture"; exit 1; }
REFETCH=1 scripts/fixture.sh midtown > "$F/refetch.log" 2>&1
code=$?
echo "gate5 REFETCH=1 scripts/fixture.sh midtown: exit $code; buildings.geojson $(sha "$TODAY"), buildings-parts.geojson $(sha "$PARTS")"
if [ "$(sha "$TODAY")" != "$SHA_TODAY" ]; then
    cp -p "$B/buildings.geojson" "$TODAY"
    cp -p "$B/fetch.log" "$MID/fetch.log"
    echo "FAIL: gate5 buildings.geojson changed after REFETCH=1; restored from $B. Stopping."
    exit 1
fi
[ "$code" = 0 ] || fail "gate5 fixture exit $code"
[ "$(sha "$PARTS")" = "$SHA_PARTS" ] || fail "gate5 buildings-parts.geojson after REFETCH=1"
for log in fetch.log fetch-parts.log; do
    [ "$(report "$MID/$log")" = "$(report "$B/$log")" ] || fail "gate5 $log changed beyond seconds and out"
    echo "gate5 $log after REFETCH=1: $(cat "$MID/$log")"
done

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

# ── Gate 2: Midtown with today's cache, byte-identical ───────────────────────
render gate2 today-ortho-city --buildings "$TODAY"
same gate2 "$(ref ortho-city)" "$OUT/today-ortho-city.framemd5"
render gate2 today-city-on --buildings "$TODAY" --camera "$CITY"
same gate2 "$(ref city-on)" "$OUT/today-city-on.framemd5"
render gate2 today-city-off --buildings "$TODAY" --camera "$CITY" --no-see-through
same gate2 "$(ref city-off)" "$OUT/today-city-off.framemd5"
render gate2 today-orbit-on --buildings "$TODAY" --camera "$ORBIT"
same gate2 "$(ref orbit-on)" "$OUT/today-orbit-on.framemd5"
render gate2 today-orbit-off --buildings "$TODAY" --camera "$ORBIT" --no-see-through
same gate2 "$(ref orbit-off)" "$OUT/today-orbit-off.framemd5"

# ── Gates 9, 11 and 13: Midtown with parts, deterministic; render times ──────
render gate9 parts-city-on1 --buildings "$PARTS" --camera "$CITY"
render gate9 parts-city-on2 --buildings "$PARTS" --camera "$CITY"
same gate9 "$OUT/parts-city-on1.framemd5" "$OUT/parts-city-on2.framemd5"
render gate9 parts-orbit-on1 --buildings "$PARTS" --camera "$ORBIT"
render gate9 parts-orbit-on2 --buildings "$PARTS" --camera "$ORBIT"
same gate9 "$OUT/parts-orbit-on1.framemd5" "$OUT/parts-orbit-on2.framemd5"
render gate9 parts-city-off --buildings "$PARTS" --camera "$CITY" --no-see-through
render gate9 parts-orbit-off --buildings "$PARTS" --camera "$ORBIT" --no-see-through
python3 - "$OUT/parts-city-on1.framemd5" "$(ref city-on)" "$OUT/parts-orbit-on1.framemd5" "$(ref orbit-on)" \
    "$OUT/parts-city-off.framemd5" "$(ref city-off)" "$OUT/parts-orbit-off.framemd5" "$(ref orbit-off)" <<'PY'
import sys
def frames(p): return [l for l in open(p) if not l.startswith("#")]
args = sys.argv[1:]
for new, old in zip(args[::2], args[1::2]):
    a, b = frames(new), frames(old)
    print(f"gate9 {new.rsplit('/', 1)[1]} vs {old.rsplit('/', 1)[1]}: {sum(x != y for x, y in zip(a, b))} of {len(a)} frames differ (recorded)")
PY

echo "gate15 videos (today's cache, then with parts):"
echo "  city flight, cut: $OUT/today-city-on.mp4 $OUT/parts-city-on1.mp4"
echo "  orbit, cut: $OUT/today-orbit-on.mp4 $OUT/parts-orbit-on1.mp4"
echo "  city flight, --no-see-through: $OUT/today-city-off.mp4 $OUT/parts-city-off.mp4"
[ $FAIL = 0 ] && echo "vis-002 Phase 4 gates 2, 5, 9 and 11: PASS" || echo "vis-002 Phase 4 gates 2, 5, 9 and 11: FAIL"
exit $FAIL
