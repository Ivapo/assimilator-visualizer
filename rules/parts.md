---
title: parts
sources:
  - scripts/fetch-buildings.sh
  - scripts/fixture.sh
  - src/buildings.rs
  - src/see_through.rs
  - src/render.rs
  - src/view/mod.rs
covers: >
  Building parts and bases: the cache's parts and its form without them, the query by
  `building_id`, `--source`, `--no-parts`, the report's `raised` and `parts`, the reader's
  checks, the volumes and their mesh, `Building::top` and the cut.
max_lines: 40
generated: 2026-10-04
---

# Parts

**What is true right now.** Corrected freely against the sources above.

## The fetch (`scripts/fetch-buildings.sh`)
- Default form: the buildings, by `id`, each with `"building_id": null`, `min_height` and
  `min_floor`; then every `building_part` whose `building_id` is one of them, by
  `building_id` and `id`, with `id, building_id, height, num_floors, min_height, min_floor,
  sources, geometry`. One file; the same `description`.
- Two DuckDB runs: run 1 writes the box's buildings, every column, to `$TMP/b.parquet` and
  prints their own extent; run 2 selects the parts over that extent (as printed) and by
  `building_id`, not by the box. No building: no parts `SELECT`.
- `--no-parts`: the form before parts (`rules/buildings.md`), its query and report.
- `--source <dir>`: a directory laid out like `theme=buildings/` (`type=building/`,
  `type=building_part/`) read in place of S3; not a directory, `s3://` included, is
  `error: --source: no directory <dir>` with nothing at `--out`. `--release` still names
  the release; the files are not checked against it.
- The report's existing keys count the buildings only; `raised` the buildings with a base
  above 0; `parts` an object of `count`, `buildings` (with a part), `height`,
  `num_floors`, `default`, `raised`, `sources`, `no_sources`, over the parts.
- `scripts/fixture.sh midtown`: step 5 `--no-parts` into `buildings.geojson`, step 6 the
  parts form into `buildings-parts.geojson` (`fetch-parts.log`), each through
  `--source scratch/overture-2026-09-23.1/theme=buildings` when it exists; `FORCE=1` keeps both.

## Reading (`buildings::read`)
- A feature with a string `building_id` is a `Part` of that building, in file order; any
  other is a `Building`. A file without `building_id`s reads as before.
- Base: `min_height`, else `min_floor` × 3.5 m, else 0; top: `rules/buildings.md`'s rule.
- Per feature, after the existing checks: `building_id` not absent, null or a string; a
  used `min_height` not finite ≥ 0, or `min_floor` not an integer ≥ 0 (checked on a
  building with parts too); `base … m is not below its top … m`, on parts and on buildings
  without parts. After every feature: `part of <bid>, which is not a building of the file`.
- `Building::top()` is its highest part's `height`, else its own; `tallest` is the highest top.

## Mesh and cut
- `building_mesh`: without parts, its own polygons from `base` to `height`; with parts,
  each part from its base to its height, its walls then its roof. No underside.
- See-through, `render` and `view` take `top()` for a building's height (`rules/see-through.md`);
  a part above `h′` is clamped to a lid at `h′`.
