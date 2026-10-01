#!/usr/bin/env bash
# vis-002 Phase 1 exit gates 3, 4 and 5 (specs/city_spec.md), on the Midtown fixture of
# `scripts/fixture.sh midtown` and urban_grid's of `scripts/fixture.sh`. Gate 6 is
#   cargo test --release --test buildings -- --include-ignored --test-threads=1 --nocapture
# Offline, except gate 5's second fetch and missing-release case, which use the network
# and run only with REFETCH=1. Needs duckdb, python3 and read access (git show) to the
# user's project.
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
cargo build --release --quiet --bin network-extent || exit 1
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
[ "$ROWS" = 280872 ] || fail "gate3 FCD rows"
[ "$VEH" = 985 ] || fail "gate3 FCD vehicles"
python3 -c "assert round($TMIN, 1) == 1.1 and round($TMAX, 1) == 1199.1" || fail "gate3 FCD time span"
REF="$ROOT/scratch/midtown-2850/fcd/baseline_42.parquet"
if [ -f "$REF" ]; then
    read -r HERE THERE < <(duckdb -noheader -csv -separator ' ' -c "
        select (select count(*) from (select * from '$FCD' except all select * from '$REF')),
               (select count(*) from (select * from '$REF' except all select * from '$FCD'))")
    echo "gate3 EXCEPT ALL against scratch/midtown-2850: $HERE rows only here, $THERE only there"
    if [ "$HERE" != 0 ] || [ "$THERE" != 0 ]; then
        fail "gate3 FCD differs from scratch/midtown-2850"
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
    echo "gate3: no scratch/midtown-2850, EXCEPT ALL skipped"
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
assert got == (4336, 4277, 8, 51, 2140989), got
print(f"gate5 fetch time: {r['seconds']} s (recorded)")
PY
CACHE_SHA=561a615d6fa77bbef482b99f9f771b8b12f5891a46e661015e034b833d2bce47
bytes=$(wc -c < "$CACHE" | tr -d ' ')
sha=$(shasum -a 256 "$CACHE" | cut -d' ' -f1)
echo "gate5 cache: $bytes bytes, sha256 $sha"
[ "$bytes" = 2140989 ] || fail "gate5 cache bytes"
[ "$sha" = "$CACHE_SHA" ] || fail "gate5 cache hash"
if [ "${REFETCH:-0}" = 1 ]; then
    rm -rf "$OUT/refetch" "$OUT/missing" && mkdir -p "$OUT/refetch" "$OUT/missing"
    scripts/fetch-buildings.sh --project "$MID" --out "$OUT/refetch/buildings.geojson" \
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

[ $FAIL = 0 ] && echo "vis-002 gates 3, 4 and 5: PASS" || echo "vis-002 gates 3, 4 and 5: FAIL"
exit $FAIL
