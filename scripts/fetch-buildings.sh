#!/usr/bin/env bash
# Fetch the Overture buildings around a project's network into a GeoJSON cache (vis-002
# §2.3). The only step of vis-002 that uses the network; `render` and `view` then read the
# cache offline with --buildings.
#
#   scripts/fetch-buildings.sh --project <dir> --out <file.geojson>
#                              [--scenario baseline] [--margin 250] [--release 2026-09-23.1]
#                              [--source <dir>] [--no-parts]
#
# Needs the DuckDB CLI (>= 1.5.1; it INSTALLs httpfs and spatial on first use) and a build
# of this repo, for network-extent (the box: the network's own extent plus --margin
# metres, in lng/lat, rounded outward). Reads --project only, and writes nothing but
# --out, through a temporary directory beside it that it removes: GDAL writes the output
# file's stem as the collection's "name", so the file is always written as
# buildings.geojson and then moved, and a failed fetch leaves nothing at --out.
#
# The file carries its provenance (vis-002 §2.14.3): the release in the collection's
# "description" ("Overture Maps buildings, release <release>"), and each feature's
# datasets in properties.sources, distinct and sorted, which `render` credits.
#
# Building parts (vis-002 §2.16.3): the buildings come first, by id, each with a null
# building_id and its min_height and min_floor; then every building_part of those
# buildings, by building_id and id. Parts are selected by building_id, not by the box (a
# part of a building across the box's edge may lie outside it), over the fetched
# buildings' own extent; so DuckDB runs twice, the buildings handed over as a parquet file.
# --no-parts writes the form before parts (vis-002 Phase 2's query and report).
#
# --source <dir> reads a local directory laid out like the release's theme=buildings/
# (<dir>/type=building/*.parquet, <dir>/type=building_part/*.parquet) instead of S3.
# --release still names the release in "description": the script cannot check that the
# files are that release.
#
# Stdout is one JSON report line; an error is one line on stderr and a non-zero exit.
# DuckDB's own output is kept off both streams unless it fails.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROJECT="" OUT="" SCENARIO=baseline MARGIN=250 RELEASE=2026-09-23.1 SOURCE="" PARTS=1
die() { echo "error: $*" >&2; exit 1; }

while [ $# -gt 0 ]; do
    case $1 in
        --project|--out|--scenario|--margin|--release|--source)
            [ $# -ge 2 ] || die "$1 needs a value"
            case $1 in
                --project) PROJECT=$2 ;;
                --out) OUT=$2 ;;
                --scenario) SCENARIO=$2 ;;
                --margin) MARGIN=$2 ;;
                --release) RELEASE=$2 ;;
                --source) SOURCE=$2 ;;
            esac
            shift 2 ;;
        --no-parts) PARTS=0; shift ;;
        *) die "unknown argument: $1" ;;
    esac
done
[ -n "$PROJECT" ] || die "--project is required"
[ -n "$OUT" ] || die "--out is required"
command -v duckdb > /dev/null || die "duckdb missing: no duckdb executable on PATH"
OUT_DIR=$(dirname "$OUT")
[ -d "$OUT_DIR" ] || die "--out: no directory $OUT_DIR"
if [ -n "$SOURCE" ]; then
    [ -d "$SOURCE" ] || die "--source: no directory $SOURCE"
    BASE=$(cd "$SOURCE" && pwd)
else
    BASE="s3://overturemaps-us-west-2/release/$RELEASE/theme=buildings"
fi

TMP=$(mktemp -d "$OUT_DIR/.fetch-buildings.XXXXXX") || die "cannot create a temporary directory in $OUT_DIR"
trap 'rm -rf "$TMP"' EXIT

# The box, W S E N.
(cd "$ROOT" && cargo build --release --quiet --bin network-extent) > "$TMP/cargo.log" 2>&1 \
    || die "cannot build network-extent: $(grep -m1 '^error' "$TMP/cargo.log")"
BOX=$("$ROOT/target/release/network-extent" --project "$PROJECT" --scenario "$SCENARIO" \
    --margin "$MARGIN" 2> "$TMP/extent.err") \
    || die "network-extent: $(sed 's/^error: //' "$TMP/extent.err" | head -1)"
read -r W S E N <<< "$BOX"

# The query (§2.3.2, §2.14.3): intersects, not contains, so a building across the margin
# comes whole; ORDER BY id, so the same release and box give the same bytes. The two HTTP
# settings change no byte; DuckDB's defaults (30 s, 3 retries) failed on a slow link.
sq() { printf "%s" "${1//\'/\'\'}"; }
PREAMBLE="INSTALL httpfs; INSTALL spatial; LOAD httpfs; LOAD spatial;
SET s3_region = 'us-west-2';
SET geometry_always_xy = true;
SET http_timeout = 120;
SET http_retries = 8;"
WRITE="TO '$(sq "$TMP")/buildings.geojson' WITH (FORMAT GDAL, DRIVER 'GeoJSON',
      LAYER_CREATION_OPTIONS 'DESCRIPTION=Overture Maps buildings, release $(sq "$RELEASE")');"
t0=$SECONDS
if [ "$PARTS" = 0 ]; then
    cat > "$TMP/fetch.sql" <<SQL
$PREAMBLE
COPY (
  SELECT id, height, num_floors,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources,
         geometry
  FROM read_parquet('$(sq "$BASE")/type=building/*.parquet')
  WHERE bbox.xmin <= $E AND bbox.xmax >= $W AND bbox.ymin <= $N AND bbox.ymax >= $S
  ORDER BY id
) $WRITE
SQL
    duckdb -bail < "$TMP/fetch.sql" > "$TMP/duckdb.log" 2>&1 \
        || die "release $RELEASE: $(grep -m1 -i 'error' "$TMP/duckdb.log")"
else
    # Run 1: the buildings, every column, and their own extent, W2 S2 E2 N2.
    cat > "$TMP/fetch1.sql" <<SQL
$PREAMBLE
COPY (SELECT * FROM read_parquet('$(sq "$BASE")/type=building/*.parquet')
      WHERE bbox.xmin <= $E AND bbox.xmax >= $W AND bbox.ymin <= $N AND bbox.ymax >= $S)
  TO '$(sq "$TMP")/b.parquet';
SELECT min(bbox.xmin), min(bbox.ymin), max(bbox.xmax), max(bbox.ymax)
  FROM read_parquet('$(sq "$TMP")/b.parquet');
SQL
    duckdb -bail -noheader -csv -separator ' ' < "$TMP/fetch1.sql" > "$TMP/extent.txt" 2> "$TMP/duckdb.log" \
        || die "release $RELEASE: $(cat "$TMP/duckdb.log" "$TMP/extent.txt" | grep -m1 -i 'error')"
    read -r W2 S2 E2 N2 < "$TMP/extent.txt"
    # Run 2: the buildings, then their parts. No building, no extent: no parts.
    PARTS_SELECT=""
    if [ -n "${W2:-}" ]; then
        PARTS_SELECT="
  UNION ALL
  SELECT id, building_id, height, num_floors, min_height, min_floor,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources, geometry
  FROM read_parquet('$(sq "$BASE")/type=building_part/*.parquet')
  WHERE bbox.xmin <= $E2 AND bbox.xmax >= $W2 AND bbox.ymin <= $N2 AND bbox.ymax >= $S2
    AND building_id IN (SELECT id FROM b)"
    fi
    cat > "$TMP/fetch2.sql" <<SQL
$PREAMBLE
CREATE TEMP TABLE b AS SELECT * FROM read_parquet('$(sq "$TMP")/b.parquet');
COPY (
  SELECT id, NULL::VARCHAR AS building_id, height, num_floors, min_height, min_floor,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources, geometry
  FROM b$PARTS_SELECT
  ORDER BY building_id NULLS FIRST, id
) $WRITE
SQL
    duckdb -bail < "$TMP/fetch2.sql" > "$TMP/duckdb.log" 2>&1 \
        || die "release $RELEASE: $(grep -m1 -i 'error' "$TMP/duckdb.log")"
fi
SECS=$((SECONDS - t0))
[ -f "$TMP/buildings.geojson" ] || die "release $RELEASE: DuckDB wrote no file"

# The report's counts, from the written file, with the height rule of §2.4.2. With parts,
# today's keys count the buildings (no building_id), and "raised" those with a base above
# 0 (§2.16.5); the parts are counted apart.
if [ "$PARTS" = 0 ]; then
    COUNT_SQL="SELECT count(*),
         count(*) FILTER (WHERE height IS NOT NULL),
         count(*) FILTER (WHERE height IS NULL AND num_floors IS NOT NULL),
         count(*) FILTER (WHERE height IS NULL AND num_floors IS NULL)"
else
    COUNT_SQL="SELECT $(for w in 'building_id IS NULL' 'building_id IS NOT NULL'; do
        printf "count(*) FILTER (WHERE %s),
         count(*) FILTER (WHERE %s AND height IS NOT NULL),
         count(*) FILTER (WHERE %s AND height IS NULL AND num_floors IS NOT NULL),
         count(*) FILTER (WHERE %s AND height IS NULL AND num_floors IS NULL),
         count(*) FILTER (WHERE %s AND coalesce(min_height, min_floor * 3.5, 0) > 0),
         " "$w" "$w" "$w" "$w" "$w"; done) count(DISTINCT building_id)"
fi
COUNTS=$(duckdb -bail -noheader -csv -separator ' ' -c "LOAD spatial;
  $COUNT_SQL
  FROM ST_Read('$(sq "$TMP")/buildings.geojson');" 2> "$TMP/count.err") \
    || die "release $RELEASE: cannot count the buildings: $(grep -m1 -i 'error' "$TMP/count.err")"
read -r NB NH NF ND NR PC PH PF PD PR PB <<< "$COUNTS"

# The datasets (§2.14.3): each feature counts once under each dataset it names, and
# "no_sources" counts those with null or []. Read as JSON, not through GDAL, whose field
# type turns to text when null or [] sit beside arrays. With parts, the buildings and the
# parts are counted apart. Prints "<no_sources>" then "<dataset>": <n>, ….
sources() { # <filter on b, the feature's building_id>
    local out ns="" src="" name n
    out=$(duckdb -bail -noheader -list -separator $'\t' -c "
  CREATE TEMP TABLE s AS SELECT json_extract(f, '\$.properties.sources') AS j,
         json_extract_string(f, '\$.properties.building_id') AS b
    FROM (SELECT unnest(features) AS f FROM read_json('$(sq "$TMP")/buildings.geojson',
          columns = {'features': 'JSON[]'}, maximum_object_size = 1073741824));
  DELETE FROM s WHERE NOT ($1);
  SELECT 'none', count(*) FILTER (WHERE j IS NULL OR json_type(j) = 'NULL' OR json_array_length(j) = 0) FROM s;
  SELECT d, count(*) FROM (SELECT unnest(from_json(j, '[\"VARCHAR\"]')) AS d FROM s
    WHERE json_type(j) = 'ARRAY') GROUP BY d ORDER BY d;" 2> "$TMP/sources.err") \
        || die "release $RELEASE: cannot count the sources: $(grep -m1 -i 'error' "$TMP/sources.err")"
    while IFS=$'\t' read -r name n; do
        if [ -z "$ns" ]; then ns=$n; continue; fi
        name=${name//\\/\\\\}
        name=${name//\"/\\\"}
        src="$src${src:+, }\"$name\": $n"
    done <<< "$out"
    printf '%s\n%s\n' "$ns" "$src"
}
if [ "$PARTS" = 0 ]; then
    SOURCES=$(sources true) || exit 1
else
    SOURCES=$(sources 'b IS NULL') || exit 1
    PSOURCES=$(sources 'b IS NOT NULL') || exit 1
    { read -r PNS; read -r PSRC; } <<< "$PSOURCES"
fi
{ read -r NS; read -r SRC; } <<< "$SOURCES"
BYTES=$(wc -c < "$TMP/buildings.geojson" | tr -d ' ')

mv "$TMP/buildings.geojson" "$OUT" || die "cannot move the file to $OUT"
out_json=${OUT//\\/\\\\}
out_json=${out_json//\"/\\\"}
PARTS_JSON=""
if [ "$PARTS" = 1 ]; then
    PARTS_JSON=$(printf ', "raised": %s, "parts": {"count": %s, "buildings": %s, "height": %s, "num_floors": %s, "default": %s, "raised": %s, "sources": {%s}, "no_sources": %s}' \
        "$NR" "$PC" "$PB" "$PH" "$PF" "$PD" "$PR" "$PSRC" "$PNS")
fi
printf '{"release": "%s", "bbox": [%s, %s, %s, %s], "buildings": %s, "height": %s, "num_floors": %s, "default": %s, "sources": {%s}, "no_sources": %s%s, "bytes": %s, "seconds": %s, "out": "%s"}\n' \
    "$RELEASE" "$W" "$S" "$E" "$N" "$NB" "$NH" "$NF" "$ND" "$SRC" "$NS" "$PARTS_JSON" "$BYTES" "$SECS" "$out_json"
