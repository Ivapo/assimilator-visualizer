#!/usr/bin/env bash
# Fetch the Overture buildings around a project's network into a GeoJSON cache (vis-002
# §2.3). The only step of vis-002 that uses the network; `render` and `view` then read the
# cache offline with --buildings.
#
#   scripts/fetch-buildings.sh --project <dir> --out <file.geojson>
#                              [--scenario baseline] [--margin 250] [--release 2026-09-23.1]
#
# Needs the DuckDB CLI (>= 1.5.1; it INSTALLs httpfs and spatial on first use) and a build
# of this repo, for network-extent (the box: the network's own extent plus --margin
# metres, in lng/lat, rounded outward). Reads --project only, and writes nothing but
# --out, through a temporary directory beside it that it removes: GDAL writes the output
# file's stem as the collection's "name", so the file is always written as
# buildings.geojson and then moved, and a failed fetch leaves nothing at --out.
#
# Stdout is one JSON report line; an error is one line on stderr and a non-zero exit.
# DuckDB's own output is kept off both streams unless it fails.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="" OUT="" SCENARIO=baseline MARGIN=250 RELEASE=2026-09-23.1
die() { echo "error: $*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case $1 in
        --project|--out|--scenario|--margin|--release)
            [ $# -ge 2 ] || die "$1 needs a value"
            case $1 in
                --project) PROJECT=$2 ;;
                --out) OUT=$2 ;;
                --scenario) SCENARIO=$2 ;;
                --margin) MARGIN=$2 ;;
                --release) RELEASE=$2 ;;
            esac
            shift 2 ;;
        *) die "unknown argument: $1" ;;
    esac
done
[ -n "$PROJECT" ] || die "--project is required"
[ -n "$OUT" ] || die "--out is required"
command -v duckdb > /dev/null || die "duckdb missing: no duckdb executable on PATH"
OUT_DIR=$(dirname "$OUT")
[ -d "$OUT_DIR" ] || die "--out: no directory $OUT_DIR"

TMP=$(mktemp -d "$OUT_DIR/.fetch-buildings.XXXXXX") || die "cannot create a temporary directory in $OUT_DIR"
trap 'rm -rf "$TMP"' EXIT

# The box, W S E N.
(cd "$ROOT" && cargo build --release --quiet --bin network-extent) > "$TMP/cargo.log" 2>&1 \
    || die "cannot build network-extent: $(grep -m1 '^error' "$TMP/cargo.log")"
BOX=$("$ROOT/target/release/network-extent" --project "$PROJECT" --scenario "$SCENARIO" \
    --margin "$MARGIN" 2> "$TMP/extent.err") \
    || die "network-extent: $(sed 's/^error: //' "$TMP/extent.err" | head -1)"
read -r W S E N <<< "$BOX"

# The query (§2.3.2): intersects, not contains, so a building across the margin comes
# whole; ORDER BY id, so the same release and box give the same bytes.
sq() { printf "%s" "${1//\'/\'\'}"; }
cat > "$TMP/fetch.sql" <<SQL
INSTALL httpfs; INSTALL spatial; LOAD httpfs; LOAD spatial;
SET s3_region = 'us-west-2';
SET geometry_always_xy = true;
COPY (
  SELECT id, height, num_floors, geometry
  FROM read_parquet('s3://overturemaps-us-west-2/release/$(sq "$RELEASE")/theme=buildings/type=building/*.parquet')
  WHERE bbox.xmin <= $E AND bbox.xmax >= $W AND bbox.ymin <= $N AND bbox.ymax >= $S
  ORDER BY id
) TO '$(sq "$TMP")/buildings.geojson' WITH (FORMAT GDAL, DRIVER 'GeoJSON');
SQL
t0=$SECONDS
duckdb -bail < "$TMP/fetch.sql" > "$TMP/duckdb.log" 2>&1 \
    || die "release $RELEASE: $(grep -m1 -i 'error' "$TMP/duckdb.log")"
SECS=$((SECONDS - t0))
[ -f "$TMP/buildings.geojson" ] || die "release $RELEASE: DuckDB wrote no file"

# The report's counts, from the written file, with the height rule of §2.4.2.
COUNTS=$(duckdb -bail -noheader -csv -separator ' ' -c "LOAD spatial;
  SELECT count(*),
         count(*) FILTER (WHERE height IS NOT NULL),
         count(*) FILTER (WHERE height IS NULL AND num_floors IS NOT NULL),
         count(*) FILTER (WHERE height IS NULL AND num_floors IS NULL)
  FROM ST_Read('$(sq "$TMP")/buildings.geojson');" 2> "$TMP/count.err") \
    || die "release $RELEASE: cannot count the buildings: $(grep -m1 -i 'error' "$TMP/count.err")"
read -r NB NH NF ND <<< "$COUNTS"
BYTES=$(wc -c < "$TMP/buildings.geojson" | tr -d ' ')

mv "$TMP/buildings.geojson" "$OUT" || die "cannot move the file to $OUT"
out_json=${OUT//\\/\\\\}
out_json=${out_json//\"/\\\"}
printf '{"release": "%s", "bbox": [%s, %s, %s, %s], "buildings": %s, "height": %s, "num_floors": %s, "default": %s, "bytes": %s, "seconds": %s, "out": "%s"}\n' \
    "$RELEASE" "$W" "$S" "$E" "$N" "$NB" "$NH" "$NF" "$ND" "$BYTES" "$SECS" "$out_json"
