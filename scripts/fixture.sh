#!/usr/bin/env bash
# Build a test fixture under scratch/ (gitignored): vis-001 Phase 1's urban_grid with no
# argument, or vis-002's Midtown with `midtown` (below).
#
# 1. Reads the engine git URL and rev from Cargo.toml (the one place the pin lives).
# 2. Builds the engine CLI at that rev with `cargo install` into scratch/engine; cargo
#    builds in a temporary target directory, so nothing is written to ../assimilator.
# 3. Exports urban_grid at the same rev with `git archive` (read-only on the checkout).
# 4. Runs the engine there with FCD on.
# 5. Derives the test FCD files with this repo's fcd-derive binary. The engine's FCD has
#    vehicle_class and vehicle_length (asm-020 Phase 3); `lengths` overwrites the latter.
#
# Needs read access to the private engine repo. ENGINE_CHECKOUT names a local clone for
# step 3 (default ../assimilator). Steps already done are skipped; FORCE=1 redoes them.
#
# `scripts/fixture.sh midtown` (vis-002 §2.9) builds the Midtown 2850 fixture in
# scratch/midtown instead. After steps 1–2 (the engine):
# 2. Exports the user's project (MIDTOWN_PROJECT, default
#    ~/assimilator/projects/midtown-section) at e2de274 with `git archive`, read-only.
# 3. Halves the demand: each `rate: r` becomes r/2 rounded to a multiple of 10, ties to
#    even; the 65 rates must sum to 2,850.
# 4. Runs the engine there with FCD on.
# 5. Fetches buildings.geojson with scripts/fetch-buildings.sh, only if it is absent or
#    REFETCH=1: the fixture's one use of the network. The report line goes to fetch.log.
# FORCE=1 redoes steps 1–4 and keeps buildings.geojson and fetch.log: the release they
# came from may be gone (vis-002 §2.3.3). Every gate after step 5 runs offline.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SCRATCH="$ROOT/scratch"
ENGINE_CHECKOUT="${ENGINE_CHECKOUT:-$ROOT/../assimilator}"
FIXTURE="${1:-urban_grid}"
case "$FIXTURE" in
    urban_grid | midtown) ;;
    *) echo "fixture: unknown fixture $FIXTURE (urban_grid or midtown)" >&2; exit 1 ;;
esac

url=$(grep -E '^assimilator-(config|core|geometry) *=' Cargo.toml | sed -E 's/.*git *= *"([^"]+)".*/\1/' | sort -u)
rev=$(grep -E '^assimilator-(config|core|geometry) *=' Cargo.toml | sed -E 's/.*rev *= *"([^"]+)".*/\1/' | sort -u)
if [ "$(printf '%s\n' "$url" | wc -l)" -ne 1 ] || [ "$(printf '%s\n' "$rev" | wc -l)" -ne 1 ] || [ -z "$rev" ]; then
    echo "fixture: the engine dependencies in Cargo.toml disagree on git/rev:" >&2
    printf '  url: %s\n  rev: %s\n' "$url" "$rev" >&2
    exit 1
fi
echo "fixture: engine $url @ $rev"

mkdir -p "$SCRATCH"

# 2. Engine CLI at the pin.
if [ "${FORCE:-0}" = 1 ] || [ ! -x "$SCRATCH/engine/bin/assimilator" ]; then
    CARGO_NET_GIT_FETCH_WITH_CLI=true \
        cargo install --force --git "$url" --rev "$rev" --locked assimilator-cli --root "$SCRATCH/engine"
fi

midtown() {
    local src="${MIDTOWN_PROJECT:-$HOME/assimilator/projects/midtown-section}"
    local proj="$SCRATCH/midtown"
    mkdir -p "$proj"

    # 2–3. The project at e2de274, with the demand halved.
    if [ "${FORCE:-0}" = 1 ] || [ ! -f "$proj/project.yaml" ]; then
        find "$proj" -mindepth 1 -maxdepth 1 ! -name buildings.geojson ! -name fetch.log \
            -exec rm -rf {} +
        git -C "$src" archive e2de274 | tar -x -C "$proj"
        python3 - "$proj/demand.yaml" <<'PY'
import re, sys
path = sys.argv[1]
lines = open(path, newline="").read().split("\n")
total = 0
for i, l in enumerate(lines):
    m = re.fullmatch(r"(\s*rate:\s*)(\d+)", l)
    if m:
        r = int(m.group(2))
        assert r % 2 == 0, f"line {i + 1}: odd rate {r}"
        h = round(r // 2, -1)  # ties to even: 15 -> 20, 25 -> 20, 115 -> 120, 145 -> 140
        lines[i] = f"{m.group(1)}{h}"
        total += h
assert total == 2850, f"the halved rates sum to {total}, not 2850"
open(path, "w", newline="").write("\n".join(lines))
print(f"fixture: demand halved, the rates sum to {total}")
PY
    fi

    # 4. The run, FCD on.
    local fcd="$proj/fcd/baseline_42.parquet"
    if [ "${FORCE:-0}" = 1 ] || [ ! -f "$fcd" ]; then
        rm -rf "$proj/results.db" "$proj/fcd"
        local t0=$SECONDS
        "$SCRATCH/engine/bin/assimilator" run -c "$proj" -s baseline -o "$proj/results.db" \
            --set simulation.output.fcd.enabled=true > "$proj/run.log" 2>&1 \
            || { echo "fixture: the engine run failed; see $proj/run.log" >&2; exit 1; }
        echo "fixture: engine run took $((SECONDS - t0)) s"
    fi
    [ -f "$fcd" ] || { echo "fixture: engine run wrote no $fcd" >&2; exit 1; }

    # 5. The buildings, once.
    if [ "${REFETCH:-0}" = 1 ] || [ ! -f "$proj/buildings.geojson" ]; then
        "$ROOT/scripts/fetch-buildings.sh" --project "$proj" --out "$proj/buildings.geojson" \
            > "$proj/fetch.log.new" || { rm -f "$proj/fetch.log.new"; exit 1; }
        mv "$proj/fetch.log.new" "$proj/fetch.log"
        cat "$proj/fetch.log"
    fi
    echo "fixture: ready in $proj"
}
if [ "$FIXTURE" = midtown ]; then
    midtown
    exit 0
fi

# 3. urban_grid at the same rev.
PROJ="$SCRATCH/urban_grid"
if [ "${FORCE:-0}" = 1 ] || [ ! -f "$PROJ/project.yaml" ]; then
    rm -rf "$PROJ" && mkdir -p "$PROJ"
    git -C "$ENGINE_CHECKOUT" archive "$rev" configs/bundled-examples/urban_grid \
        | tar -x -C "$PROJ" --strip-components=3
fi

# 4. The run, FCD on.
FCD="$PROJ/fcd/baseline_42.parquet"
if [ "${FORCE:-0}" = 1 ] || [ ! -f "$FCD" ]; then
    rm -rf "$PROJ/results.db" "$PROJ/fcd"
    "$SCRATCH/engine/bin/assimilator" run -c "$PROJ" -s baseline -o "$PROJ/results.db" \
        --set simulation.output.fcd.enabled=true
fi
[ -f "$FCD" ] || { echo "fixture: engine run wrote no $FCD" >&2; exit 1; }

# 5. Derived FCD files.
cargo build --release --quiet --bin fcd-derive
DERIVE="$ROOT/target/release/fcd-derive"
mkdir -p "$SCRATCH/derived"
"$DERIVE" lengths --input "$FCD" --output "$SCRATCH/derived/lengths.parquet"
"$DERIVE" unknown-link --input "$FCD" --output "$SCRATCH/derived/unknown_link.parquet"
# A pre-asm-020 file (no class or length), for gate 7's fallback case.
"$DERIVE" drop-columns --input "$FCD" --output "$SCRATCH/derived/no_class_length.parquet" \
    --column vehicle_class --column vehicle_length
echo "fixture: ready in $SCRATCH"
