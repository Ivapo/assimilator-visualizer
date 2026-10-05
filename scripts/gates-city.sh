#!/usr/bin/env bash
# vis-002 Phase 1 exit gates 3, 4, 5, 7 (the CLI cases) and 13 (specs/city_spec.md), on
# the Midtown fixture of `scripts/fixture.sh midtown` and urban_grid's of
# `scripts/fixture.sh`. Gates 6–12 and 14 are
#   cargo test --release --test buildings -- --include-ignored --test-threads=1 --nocapture
# Gate 1 is scripts/gates.sh, and its --camera frames compared by hand; gates 2 and 15
# are recorded by hand in specs/reviews/vis-002.md. Gate 16 is the user's.
# Offline, except gate 5's second fetch and missing-release case, which use the network
# and run only with REFETCH=1. Needs ffmpeg/ffprobe, duckdb, python3, perl, sandbox-exec
# and read access (git show) to the user's project.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
MID="$ROOT/scratch/midtown"
UG="$ROOT/scratch/urban_grid"
OUT="$ROOT/scratch/out/city"
SRC="${MIDTOWN_PROJECT:-$HOME/assimilator/projects/midtown-section}"
COMMIT=e2de274
CACHE="$MID/buildings.geojson"
mkdir -p "$OUT"
cargo build --release --quiet --bin assimilator-video --bin network-extent || exit 1
BIN="$ROOT/target/release/assimilator-video"
EXTENT="$ROOT/target/release/network-extent"
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
    head -1 "$err" | grep -q "^$prefix" || fail "$label stderr does not begin with $prefix"
    grep -q '"frame"' "$err" && fail "$label printed progress"
    [ ! -s "$out" ] || fail "$label wrote to stdout"
}

# ── Gate 3: the fixture ──────────────────────────────────────────────────────
for f in project.yaml network.yaml import_report.json; do
    if cmp -s <(git -C "$SRC" show "$COMMIT:$f") "$MID/$f"; then
        echo "gate3 $f: byte-identical to $COMMIT"
    else
        fail "gate3 $f differs from $COMMIT"
    fi
done
python3 - <(git -C "$SRC" show "$COMMIT:demand.yaml") "$MID/demand.yaml" <<'PY' || fail "gate3 demand.yaml"
import re, sys
a = open(sys.argv[1]).read().split("\n")
b = open(sys.argv[2]).read().split("\n")
rate = re.compile(r"\s*rate:\s*\d+")
assert len(a) == len(b), f"{len(a)} lines vs {len(b)}"
diff = [(x, y) for x, y in zip(a, b) if x != y]
assert all(rate.fullmatch(x) and rate.fullmatch(y) for x, y in diff), "a line other than rate: differs"
rates = [int(l.split(":")[1]) for l in b if rate.fullmatch(l)]
print(f"gate3 demand.yaml: {len(diff)} lines differ, all rate:; {len(rates)} rates sum to {sum(rates)}")
assert len(rates) == 65 and sum(rates) == 2850
PY
sha=$(shasum -a 256 "$MID/demand.yaml" | cut -d' ' -f1)
echo "gate3 demand.yaml sha256 $sha"
[ "$sha" = 66f54820d3981aa7e6deb8569b59558d137230bdbad09a4a6f827e2068d85d9a ] || fail "gate3 demand.yaml hash"

FCD="$MID/fcd/baseline_42.parquet"
read -r ROWS VEH TMIN TMAX < <(duckdb -noheader -csv -separator ' ' \
    -c "select count(*), count(distinct vehicle_id), min(time), max(time) from '$FCD'") \
    || { echo "FAIL: cannot read $FCD with duckdb"; exit 1; }
echo "gate3 FCD: $ROWS rows, $VEH vehicles, time $TMIN … $TMAX"
[ "$ROWS" = 202241 ] || fail "gate3 FCD rows"
[ "$VEH" = 985 ] || fail "gate3 FCD vehicles"
python3 -c "assert round($TMIN, 1) == 1.1 and round($TMAX, 1) == 1199.1" || fail "gate3 FCD time span"
REF="$ROOT/scratch/midtown-2850-90b39292/fcd/baseline_42.parquet"
if [ -f "$REF" ]; then
    read -r HERE THERE < <(duckdb -noheader -csv -separator ' ' -c "
        select (select count(*) from (select * from '$FCD' except all select * from '$REF')),
               (select count(*) from (select * from '$REF' except all select * from '$FCD'))")
    echo "gate3 EXCEPT ALL against scratch/midtown-2850-90b39292: $HERE rows only here, $THERE only there"
    if [ "$HERE" != 0 ] || [ "$THERE" != 0 ]; then
        fail "gate3 FCD differs from scratch/midtown-2850-90b39292"
        # The difference's shape: its vehicles, its time span, and which columns differ
        # on rows paired by (vehicle_id, time).
        duckdb -c "
            create temp table here as select * from '$FCD' except all select * from '$REF';
            create temp table there as select * from '$REF' except all select * from '$FCD';
            select count(distinct vehicle_id) as vehicles, list(distinct vehicle_id order by vehicle_id) as ids,
                   min(time) as first_time, max(time) as last_time
            from (select * from here union all select * from there);
            select vehicle_id, count(*) as rows, min(time) as first_time, max(time) as last_time
            from (select * from here union all select * from there) group by vehicle_id order by vehicle_id;
            select count(*) as paired,
                   count(*) filter (where h.link_id is distinct from t.link_id) as link_id,
                   count(*) filter (where h.lane is distinct from t.lane) as lane,
                   count(*) filter (where h.position is distinct from t.position) as position,
                   count(*) filter (where h.speed is distinct from t.speed) as speed,
                   count(*) filter (where h.acceleration is distinct from t.acceleration) as acceleration,
                   count(*) filter (where h.vehicle_class is distinct from t.vehicle_class) as vehicle_class,
                   count(*) filter (where h.vehicle_length is distinct from t.vehicle_length) as vehicle_length
            from here h join there t using (vehicle_id, time);"
    fi
else
    echo "gate3: no scratch/midtown-2850-90b39292, EXCEPT ALL skipped"
fi

# ── Gate 4: the extent ───────────────────────────────────────────────────────
got=$("$EXTENT" --project "$MID")
echo "gate4 margin 250: $got"
[ "$got" = "-73.9938413 40.7544211 -73.9643086 40.7740495" ] || fail "gate4 margin 250"
got=$("$EXTENT" --project "$MID" --margin 0)
echo "gate4 margin 0: $got"
[ "$got" = "-73.9908762 40.7566669 -73.9672737 40.7718037" ] || fail "gate4 margin 0"
"$EXTENT" --project "$UG" > "$OUT/gate4_ug.stdout" 2> "$OUT/gate4_ug.stderr"
one_error "gate4 urban_grid" $? "$OUT/gate4_ug.stderr" "$OUT/gate4_ug.stdout" "error: "
grep -q 'metadata.map_origin' "$OUT/gate4_ug.stderr" || fail "gate4 urban_grid does not name metadata.map_origin"

# ── Gate 5: the fetch ────────────────────────────────────────────────────────
python3 - "$MID/fetch.log" <<'PY' || fail "gate5 report"
import json, sys
lines = open(sys.argv[1]).read().splitlines()
assert len(lines) == 1, f"{len(lines)} report lines"
r = json.loads(lines[0])
print(f"gate5 report: {lines[0]}")
assert r["release"] == "2026-09-23.1", r["release"]
assert r["bbox"] == [-73.9938413, 40.7544211, -73.9643086, 40.7740495], r["bbox"]
got = (r["buildings"], r["height"], r["num_floors"], r["default"], r["bytes"])
assert got == (4336, 4277, 8, 51, 2284830), got
print(f"gate5 fetch time: {r['seconds']} s (recorded)")
PY
CACHE_SHA=f241ccbd9a2fba2e3c80fa28ec7522ca2a33b84628c18954952ad6bc1d4a1dfd
bytes=$(wc -c < "$CACHE" | tr -d ' ')
sha=$(shasum -a 256 "$CACHE" | cut -d' ' -f1)
echo "gate5 cache: $bytes bytes, sha256 $sha"
[ "$bytes" = 2284830 ] || fail "gate5 cache bytes"
[ "$sha" = "$CACHE_SHA" ] || fail "gate5 cache hash"
if [ "${REFETCH:-0}" = 1 ]; then
    rm -rf "$OUT/refetch" "$OUT/missing" && mkdir -p "$OUT/refetch" "$OUT/missing"
    scripts/fetch-buildings.sh --project "$MID" --out "$OUT/refetch/buildings.geojson" --no-parts \
        > "$OUT/refetch/fetch.log" 2> "$OUT/refetch/fetch.stderr" || fail "gate5 second fetch exited $?"
    echo "gate5 second fetch: $(cat "$OUT/refetch/fetch.log")"
    sha2=$(shasum -a 256 "$OUT/refetch/buildings.geojson" 2> /dev/null | cut -d' ' -f1)
    echo "gate5 second fetch sha256 $sha2"
    [ "$sha2" = "$CACHE_SHA" ] || fail "gate5 second fetch hash"
    scripts/fetch-buildings.sh --project "$MID" --out "$OUT/missing/buildings.geojson" \
        --release 2020-01-01.0 > "$OUT/missing/fetch.stdout" 2> "$OUT/missing/fetch.stderr"
    code=$?
    nl=$(wc -l < "$OUT/missing/fetch.stderr" | tr -d ' ')
    echo "gate5 missing release: exit $code, $nl stderr line(s): $(cat "$OUT/missing/fetch.stderr")"
    [ "$code" != 0 ] || fail "gate5 missing release exit 0"
    [ "$nl" = 1 ] || fail "gate5 missing release stderr has $nl lines"
    grep -q '2020-01-01.0' "$OUT/missing/fetch.stderr" || fail "gate5 missing release not named"
    [ -z "$(ls -A "$OUT/missing" | grep -v '^fetch\.std')" ] || fail "gate5 missing release left $(ls -A "$OUT/missing")"
else
    echo "gate5: the second fetch and the missing release use the network; run with REFETCH=1"
fi

# ── Gate 7: reading errors, through the CLI ──────────────────────────────────
CASES="$OUT/gate7"
rm -rf "$CASES" && mkdir -p "$CASES"
python3 - "$CASES" <<'PY' || fail "gate7 cases"
import json, math, os, sys
d = sys.argv[1]
OLNG, OLAT = -73.9775152177763, 40.76472024499192
def ll(x, y):
    return [OLNG + x / (111320.0 * math.cos(OLAT * math.pi / 180.0)), OLAT + y / 111320.0]
sq = [ll(120, -5), ll(130, -5), ll(130, 5), ll(120, 5), ll(120, -5)]
def poly(*rings):
    return {"type": "Polygon", "coordinates": list(rings)}
def feat(props, geom):
    return {"type": "Feature", "properties": props, "geometry": geom}
def fc(*features):
    return json.dumps({"type": "FeatureCollection", "features": list(features)})
cases = {
    "not_json": "{ not json",
    "array": "[]",
    "top_feature": json.dumps(feat({"id": "x", "height": 10}, poly(sq))),
    "no_id": fc(feat({"height": 10}, poly(sq))),
    "point": fc(feat({"id": "p"}, {"type": "Point", "coordinates": sq[0]})),
    "ring3": fc(feat({"id": "r3"}, poly([sq[0], sq[1], sq[0]]))),
    "unclosed": fc(feat({"id": "open"}, poly(sq[:4]))),
    "position_a": fc(feat({"id": "pa"}, poly([sq[0], ["a", 40.7], sq[2], sq[3], sq[0]]))),
    "lat91": fc(feat({"id": "l91"}, poly([sq[0], sq[1], [sq[2][0], 91], [sq[3][0], 91], sq[0]]))),
    "metres": fc(feat({"id": "m"}, poly([[294.2, -240.2], [304.2, -240.2], [304.2, -230.2], [294.2, -240.2]]))),
    "height0": fc(feat({"id": "h0", "height": 0}, poly(sq))),
    "height_neg": fc(feat({"id": "h-5", "height": -5}, poly(sq))),
    "floors0": fc(feat({"id": "f0", "num_floors": 0}, poly(sq))),
    "floors2.5": fc(feat({"id": "f2.5", "num_floors": 2.5}, poly(sq))),
    "empty": fc(),
    "far": fc(feat({"id": "far"}, poly([[0, 0], [0.0001, 0], [0.0001, 0.0001], [0, 0]]))),
}
for k, v in cases.items():
    open(os.path.join(d, k + ".geojson"), "w").write(v)
print(f"gate7: {len(cases)} case files")
PY
gate7() { # <label> <cmd: render|view> <project> <buildings file>
    local label=$1 cmd=$2 proj=$3 file=$4 code
    local out="$CASES/$label.$cmd.mp4"
    rm -f "$out" "$out.partial"
    if [ "$cmd" = render ]; then
        "$BIN" render --project "$proj" --scenario baseline --seed 42 --out "$out" \
            --buildings "$file" > "$CASES/$label.$cmd.stdout" 2> "$CASES/$label.$cmd.stderr"
        code=$?
    else
        # A window that opens fails the case instead of hanging: SIGALRM after 60 s.
        perl -e 'alarm 60; exec @ARGV' "$BIN" view --project "$proj" --scenario baseline \
            --seed 42 --buildings "$file" > "$CASES/$label.$cmd.stdout" 2> "$CASES/$label.$cmd.stderr"
        code=$?
    fi
    one_error "gate7 $label ($cmd)" $code "$CASES/$label.$cmd.stderr" "$CASES/$label.$cmd.stdout" \
        "error: --buildings"
    [ ! -e "$out" ] || fail "gate7 $label left $out"
    [ ! -e "$out.partial" ] || fail "gate7 $label left $out.partial"
}
gate7 missing render "$MID" "$CASES/does_not_exist.geojson"
for f in "$CASES"/*.geojson; do
    gate7 "$(basename "$f" .geojson)" render "$MID" "$f"
done
gate7 no_map_origin render "$UG" "$ROOT/tests/shapes.geojson"
gate7 missing view "$MID" "$CASES/does_not_exist.geojson"
gate7 no_map_origin view "$UG" "$ROOT/tests/shapes.geojson"
gate7 far view "$MID" "$CASES/far.geojson"

# ── Gate 13: deterministic and offline ───────────────────────────────────────
render13() { # <tag> <sandboxed: 0|1> [extra args…]
    local tag=$1 sb=$2 t0=$SECONDS probe code
    shift 2
    local cmd=("$BIN" render --project "$MID" --scenario baseline --seed 42 --buildings "$CACHE"
        --from 300 --to 360 --speedup 1 --out "$OUT/$tag.mp4" "$@")
    rm -f "$OUT/$tag.mp4" "$OUT/$tag.framemd5"
    if [ "$sb" = 1 ]; then
        sandbox-exec -p '(version 1)(allow default)(deny network*)' "${cmd[@]}" 2> "$OUT/$tag.stderr"
    else
        "${cmd[@]}" 2> "$OUT/$tag.stderr"
    fi
    code=$?
    echo "gate13 $tag: exit $code in $((SECONDS - t0)) s, last line $(tail -1 "$OUT/$tag.stderr")"
    [ "$code" = 0 ] || { fail "gate13 $tag exited $code"; return; }
    probe=$(ffprobe -v error -select_streams v:0 -count_frames \
        -show_entries stream=width,height,r_frame_rate,nb_read_frames -of csv=p=0 "$OUT/$tag.mp4")
    echo "gate13 $tag: ffprobe $probe (expected 1920,1080,30/1,1800)"
    [ "$probe" = "1920,1080,30/1,1800" ] || fail "gate13 $tag ffprobe"
    ffmpeg -y -v error -i "$OUT/$tag.mp4" -f framemd5 "$OUT/$tag.framemd5"
}
same13() { # <a> <b>
    python3 - "$OUT/$1.framemd5" "$OUT/$2.framemd5" <<'PY' || fail "gate13 $1 vs $2"
import sys
a = [l for l in open(sys.argv[1]) if not l.startswith("#")]
b = [l for l in open(sys.argv[2]) if not l.startswith("#")]
eq = sum(x == y for x, y in zip(a, b))
print(f"gate13 {sys.argv[1].rsplit('/', 1)[1]} vs {sys.argv[2].rsplit('/', 1)[1]}: {eq} of {len(a)} frames equal ({len(b)})")
assert eq == len(a) == len(b) == 1800
PY
}
render13 flight1 0 --camera "$ROOT/tests/city-flight.toml"
render13 flight2 0 --camera "$ROOT/tests/city-flight.toml"
render13 flight3 1 --camera "$ROOT/tests/city-flight.toml"
same13 flight1 flight2
same13 flight1 flight3
render13 ortho1 0
render13 ortho2 0
same13 ortho1 ortho2

[ $FAIL = 0 ] && echo "vis-002 gates 3, 4, 5, 7 (CLI) and 13: PASS" || echo "vis-002 gates 3, 4, 5, 7 (CLI) and 13: FAIL"
exit $FAIL
