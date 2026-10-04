#!/usr/bin/env bash
# vis-002 Phase 2 exit gates 4 (its checks), 5, 9 (the CLI cases) and 14
# (specs/city_spec.md), on the Midtown fixture of `scripts/fixture.sh midtown` and
# urban_grid's of `scripts/fixture.sh`. Gates 3 (the `Buildings` equality), 6–8, 10 and 11 are
#   cargo test --release --test credit -- --include-ignored --test-threads=1 --nocapture
# Gates 12 and 13 are Phase 1's gate 11 and 12 tests in tests/buildings.rs. Gates 1–3 and
# 15 are recorded by hand in specs/reviews/vis-002.md, and gate 16 is the user's.
#
# Gate 4's first two steps run by hand, once, before this script: Phase 1's cache copied
# to scratch/buildings-vis002p1.geojson, then `REFETCH=1 scripts/fixture.sh midtown`.
# Offline, except gate 4's second fetch, which uses the network and runs only with
# REFETCH=1. Needs ffmpeg/ffprobe, python3 and the duckdb CLI (for that fetch).
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/credit"
CACHE="$MID/buildings.geojson"
P1="$ROOT/scratch/buildings-vis002p1.geojson"
CACHE_SHA=f241ccbd9a2fba2e3c80fa28ec7522ca2a33b84628c18954952ad6bc1d4a1dfd
P1_SHA=561a615d6fa77bbef482b99f9f771b8b12f5891a46e661015e034b833d2bce47
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video || exit 1
BIN="$ROOT/target/release/assimilator-video"
FAIL=0
fail() { echo "FAIL: $*"; FAIL=1; }

# One-line error check: exit code 1, exactly one stderr line starting with $prefix, no
# progress line, nothing on stdout.
one_error() { # <label> <code> <stderr file> <stdout file> <prefix>
    local label=$1 code=$2 err=$3 out=$4 prefix=$5 nl
    nl=$(wc -l < "$err" | tr -d ' ')
    echo "$label: exit $code, $nl stderr line(s): $(head -c 300 "$err")"
    [ "$code" = 1 ] || fail "$label exit $code"
    [ "$nl" = 1 ] || fail "$label stderr has $nl lines"
    [[ "$(head -1 "$err")" == "$prefix"* ]] || fail "$label stderr does not begin with $prefix"
    grep -q '"frame"' "$err" && fail "$label printed progress"
    [ ! -s "$out" ] || fail "$label wrote to stdout"
}

# ── Gate 4: the re-fetch ─────────────────────────────────────────────────────
sha=$(shasum -a 256 "$P1" | cut -d' ' -f1)
echo "gate4 Phase 1's cache, kept: sha256 $sha"
[ "$sha" = "$P1_SHA" ] || fail "gate4 scratch/buildings-vis002p1.geojson hash"
python3 - "$MID/fetch.log" <<'PY' || fail "gate4 report"
import json, sys
lines = open(sys.argv[1]).read().splitlines()
assert len(lines) == 1, f"{len(lines)} report lines"
r = json.loads(lines[0])
print(f"gate4 report: {lines[0]}")
assert r["release"] == "2026-09-23.1", r["release"]
assert r["bbox"] == [-73.9938413, 40.7544211, -73.9643086, 40.7740495], r["bbox"]
got = (r["buildings"], r["height"], r["num_floors"], r["default"], r["bytes"])
assert got == (4336, 4277, 8, 51, 2284830), got
want = {"Microsoft ML Buildings": 29, "OpenStreetMap": 4327, "USGS Lidar": 316}
assert r["sources"] == want, r["sources"]
assert list(r["sources"]) == sorted(r["sources"]), "sources keys not sorted"
assert r["no_sources"] == 0, r["no_sources"]
print(f"gate4 fetch time: {r['seconds']} s (recorded)")
PY
python3 - "$CACHE" <<'PY' || fail "gate4 datasets per building"
import collections, json, sys
doc = json.load(open(sys.argv[1]))
assert doc["description"] == "Overture Maps buildings, release 2026-09-23.1", doc.get("description")
n = collections.Counter(len(f["properties"]["sources"] or []) for f in doc["features"])
print(f"gate4 datasets per building: {dict(sorted(n.items()))}, {sum(n.values())} buildings")
assert dict(n) == {1: 4000, 2: 336}, n
PY
bytes=$(wc -c < "$CACHE" | tr -d ' ')
sha=$(shasum -a 256 "$CACHE" | cut -d' ' -f1)
echo "gate4 cache: $bytes bytes, sha256 $sha"
[ "$bytes" = 2284830 ] || fail "gate4 cache bytes"
[ "$sha" = "$CACHE_SHA" ] || fail "gate4 cache hash"
if [ "${REFETCH:-0}" = 1 ]; then
    mkdir -p "$OUT/refetch"
    "$ROOT/scripts/fetch-buildings.sh" --project "$MID" --out "$OUT/refetch/buildings.geojson" --no-parts \
        > "$OUT/refetch/fetch.log" 2> "$OUT/refetch/fetch.stderr" || fail "gate4 second fetch exited $?"
    echo "gate4 second fetch: $(cat "$OUT/refetch/fetch.log")"
    sha2=$(shasum -a 256 "$OUT/refetch/buildings.geojson" | cut -d' ' -f1)
    echo "gate4 second fetch sha256 $sha2"
    [ "$sha2" = "$CACHE_SHA" ] || fail "gate4 second fetch hash"
else
    echo "gate4: the second fetch uses the network; run with REFETCH=1"
fi

# ── Gate 5: Phase 1's content, unchanged ─────────────────────────────────────
python3 - "$CACHE" "$P1" <<'PY' || fail "gate5 strip-and-compare"
import re, sys
new = open(sys.argv[1], "rb").read()
old = open(sys.argv[2], "rb").read()
desc = re.compile(rb'^"description": "[^"\n]*",\n', re.M)
src = re.compile(rb', "sources": (?:null|\[[^\]\n]*\])')
assert len(desc.findall(new)) == 1, "one description line"
n = len(src.findall(new))
stripped = src.sub(b"", desc.sub(b"", new, count=1))
print(f"gate5: removed the description line and {n} sources members: {len(stripped)} bytes against {len(old)}")
assert stripped == old, "differs from Phase 1's cache"
print("gate5: byte-identical to scratch/buildings-vis002p1.geojson")
PY

# ── Gate 9: errors through render ────────────────────────────────────────────
CASES="$OUT/gate9"
COPY="$CASES/midtown"
rm -rf "$CASES"
mkdir -p "$CASES"
cp -R "$MID" "$COPY"
REPORT="$COPY/import_report.json"
cp "$REPORT" "$CASES/import_report.orig.json"
python3 - "$CASES" "$ROOT/tests/shapes.geojson" <<'PY' || fail "gate9 cases"
import json, os, sys
d, shapes = sys.argv[1], sys.argv[2]
orig = json.load(open(os.path.join(d, "import_report.orig.json")))
open(os.path.join(d, "report_not_json.json"), "w").write("{ not json")
here = json.loads(json.dumps(orig)); here["source"]["type"] = "Here"
json.dump(here, open(os.path.join(d, "report_here.json"), "w"))
norel = json.loads(json.dumps(orig)); del norel["source"]["release"]
json.dump(norel, open(os.path.join(d, "report_no_release.json"), "w"))
# The made-up *long* file: ten made-up datasets on one shape (241.28 em).
doc = json.load(open(shapes))
doc["description"] = "Overture Maps buildings, release test"
doc["features"][0]["properties"]["sources"] = [
    f"Made-up dataset name number {i:02} for tests" for i in range(10)]
json.dump(doc, open(os.path.join(d, "long.geojson"), "w"))
print("gate9: 3 reports and the long file written")
PY
gate9() { # <label> <project> <prefix> [extra args…]
    local label=$1 proj=$2 prefix=$3 code
    shift 3
    local out="$CASES/$label.mp4"
    rm -f "$out" "$out.partial"
    "$BIN" render --project "$proj" --scenario baseline --seed 42 --out "$out" "$@" \
        > "$CASES/$label.stdout" 2> "$CASES/$label.stderr"
    code=$?
    one_error "gate9 $label" $code "$CASES/$label.stderr" "$CASES/$label.stdout" "$prefix"
    [ ! -e "$out" ] || fail "gate9 $label left $out"
    [ ! -e "$out.partial" ] || fail "gate9 $label left $out.partial"
}
for r in not_json here no_release; do
    cp "$CASES/report_$r.json" "$REPORT"
    gate9 "report_$r" "$COPY" "error: $REPORT: "
done
cp "$CASES/import_report.orig.json" "$REPORT"
grep -q 'has no credit (vis-002 OQ-11)' "$CASES/report_here.stderr" || fail "gate9 report_here does not name OQ-11"
gate9 phase1_cache "$MID" "error: --buildings $P1: no release" --buildings "$P1"
gate9 long "$MID" "error: the credit line would be " --buildings "$CASES/long.geojson"
s=$(sed -nE 's/^error: the credit line would be ([0-9]+) px in a 1920×1080 frame, under 10 px; render a larger frame$/\1/p' "$CASES/long.stderr")
echo "gate9 long: S' = ${s:-?} (predicted 7, or 8 with kerning)"
[ -n "$s" ] && [ "$s" -lt 10 ] || fail "gate9 long: S' ${s:-missing} is not under 10, or the message differs"
gate9 urban_grid_shapes "$UG" "error: --buildings" --buildings "$ROOT/tests/shapes.geojson"
grep -q 'metadata.map_origin' "$CASES/urban_grid_shapes.stderr" || fail "gate9 urban_grid does not name metadata.map_origin"

# ── Gate 14: deterministic, orthographic ─────────────────────────────────────
render14() { # <tag> [extra args…]
    local tag=$1 t0=$SECONDS probe code
    shift
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    "$BIN" render --project "$MID" --scenario baseline --seed 42 --from 300 --to 360 \
        --speedup 1 --out "$OUT/$tag.mp4" "$@" 2> "$OUT/$tag.stderr"
    code=$?
    echo "gate14 $tag: exit $code in $((SECONDS - t0)) s, last line $(tail -1 "$OUT/$tag.stderr")"
    [ "$code" = 0 ] || { fail "gate14 $tag exited $code"; return; }
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "gate14 $tag: ffprobe $probe (expected 1920,1080,30/1,1800)"
    [ "$probe" = "1920,1080,30/1,1800" ] || fail "gate14 $tag ffprobe"
    ffmpeg -y -v error -i "$OUT/$tag.mp4" -f framemd5 "$OUT/$tag.framemd5"
}
same14() { # <a> <b>
    python3 - "$OUT/$1.framemd5" "$OUT/$2.framemd5" <<'PY' || fail "gate14 $1 vs $2"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"gate14 {sys.argv[1].rsplit('/', 1)[1]} vs {sys.argv[2].rsplit('/', 1)[1]}: {eq} of {len(a)} frames equal ({len(b)})")
assert eq == len(a) == len(b) == 1800
PY
}
render14 city1 --buildings "$CACHE"
render14 city2 --buildings "$CACHE"
same14 city1 city2
render14 roads1
render14 roads2
same14 roads1 roads2

[ $FAIL = 0 ] && echo "vis-002 Phase 2 gates 4, 5, 9 (CLI) and 14: PASS" || echo "vis-002 Phase 2 gates 4, 5, 9 (CLI) and 14: FAIL"
exit $FAIL
