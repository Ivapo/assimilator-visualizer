---
title: inputs
sources:
  - src/inputs.rs
  - src/fcd.rs
  - src/place.rs
  - src/run.rs
  - Cargo.toml
covers: >
  What the renderer reads from a run and from where, how FCD rows become placed
  points, and which checks fail before the first frame.
max_lines: 50
generated: 2026-10-02
---

# Inputs

**What is true right now.** Corrected freely against the sources above.

## Files (read in place, never written)
- `<project>/project.yaml` → `assimilator_config::parse_and_check` →
  `serde_yaml::from_value::<ProjectConfig>` → `resolve_scenario(&project, scenario,
  project_dir).network`, as it is at render time; no `--set` overrides.
- `results.db`: default `<project>/results.db`, or `--results`. It is opened as
  `file:<abs>?mode=ro&immutable=1` with SQLite's read-only and URI flags. It must hold a
  `runs` row for (scenario, seed) with `status = 'completed'`; otherwise the error is
  "no completed run". No KPI is read.
- FCD: default `fcd/<scenario>_<seed>.parquet` beside `results.db`, or `--fcd`.
  - Columns are selected by name and their types are checked: `time` f64, `vehicle_id`
    u64, `link_id` Utf8 (LargeUtf8 is also accepted), `lane` u32, `position` f64, `speed`
    f64.
  - `vehicle_length` f64 is optional and sets the box length. Without it the length is
    4.5 m. The engine at the pin writes it (asm-020 Phase 3), and older files lack it.
    `acceleration` and `vehicle_class` (`Dictionary(Int32, Utf8)`) are not read.
  - A missing column or a wrong type is a "schema mismatch" error; CSV FCD is not supported.

## Window and snapshots
- The default window is the first to the last FCD `time` in the file.
- The whole file is read, then cut to the snapshots in `[from, to]` plus the last one at
  or before `from`. There is no row-group pruning.
- Rows are sorted by (`time`, `vehicle_id`) and grouped into snapshots by equal `time`.
- `Fcd::vehicles` keeps every row of the **whole file** by `vehicle_id`, in `time` order,
  for motion (`rules/motion.md`); an interval crossing `--from` or `--to` is built whole.
- The snapshot for sim time `t` is the latest one with `time ≤ t + 1e-6`. FCD times build
  up from 0.1 s steps (for example 60.100000000000584).

## Placement (the engine's code, engine rev pinned in `Cargo.toml`)
- `NetworkData::from_config(&network)` → `LinkGeometryIndex::from_network_config` →
  `interpolate(link_id, position, lane)`, giving x, y in metres and a heading in degrees
  (0 = north, clockwise).
- Every `link_id` in the file must be a network link, and every row of the file is
  placed up front (`place_all` and `Motion::build`). An unknown link, or a row the
  engine cannot place, is an error before the first frame.
- For motion, `Placement` also wraps the engine's junction movements and turn paths.
- `link_length` is `NetworkData::link_length`, the junction-trimmed length FCD `position`
  is measured along.

## Checks before the first frame (order)
`render`: ffmpeg on `PATH` → even, positive size; positive fps and speedup → `run::load`
(`view`: positive size, no ffmpeg → `run::load`): project/scenario → `results.db`'s run →
FCD file → schema → `to > from` → every link known and row placed → `--camera`'s file
(`rules/camera.md`) → `--buildings`' file (`rules/buildings.md`) → `render`'s credit inputs,
then its fit after the settle frames (`rules/credit.md`). An error is one `error: …` line,
exit 1, no output file or window.
