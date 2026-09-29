#!/usr/bin/env bash
# Build the vis-001 Phase 1 test fixture under scratch/ (gitignored).
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
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
SCRATCH="$ROOT/scratch"
ENGINE_CHECKOUT="${ENGINE_CHECKOUT:-$ROOT/../assimilator}"

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
