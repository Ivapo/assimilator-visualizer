---
id: vis-001
title: visualizer
note: >
  What the visualizer is for: it renders a 3D video of one Assimilator simulation run,
  for presentations, from a CLI a harness can call. Phase 1 is the smallest surface that
  produces a video: vehicles as boxes on flat roads, top-down camera, headless Bevy to ffmpeg.
status: draft
last_updated: 2026-09-28

phases:
  - name: "Phase 1 — Moving boxes: one run to one video"
    reviewed: null
    shipped: null
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: []
reference: >
  Seed brief: ~/dev/ivapo/sim-3D-visualizer/idea.md (the idea, the stack discussion and
  the roadmap to v1). Engine: ~/dev/main/assimilator (asm-001 §10 owns the engine side of
  the harness boundary). Harness: ~/dev/main/assimilator-harness (har-001). Both are
  read-only from this repo. Out of scope from the engine: its 2D macroquad viewer
  (`crates/viz`) and its GUI frame protocol.
---

# Visualizer

## 1. Goal

Turn the output of one Assimilator simulation run into a video that a person can show in
a presentation. The consumer is a traffic engineer (or the harness acting for one) who
has a finished run and wants to show what happened in it.

**The observable is a video file rendered from one simulation run.** An MP4 of 30 s to
5 min, whatever the length of the run, in which the vehicles move as they did in the run.

Rejected candidates for the observable:
- *The CLI contract the harness calls.* It is the interface through which the video is
  requested, not a thing anyone watches. Phase 6 of the roadmap (§2.7) produces it and
  must say so.
- *A live 3D preview window.* It is a tool for writing camera paths. Its output is a
  `scene.toml`, which is an internal artifact.
- *The scene bundle (buildings, land use).* Input to a video, not the video.

### 1.1 Non-goals

- Not a real-time viewer attached to a running simulation. The engine has its own GUI.
- No simulation. The visualizer never runs the engine and never changes its output.
- No statistics. The numbers shown come from the run (FCD, `results.db`) as they are.
- No change to the engine or the harness repos. What this project needs from them is
  written down as a request (§3), not done here.
- Photo-realism in v1. The look is stylized (§2.6).

## 2. Design

### 2.1 Inputs are a run's files, read in place

The engine writes one run as (engine `rules/project-schema.md`):
- `<project>/network.yaml` — the network. Metric coordinates, `metadata.map_origin`
  `[lng, lat]`. Parsed by `crates/config/src/network.rs:NetworkConfig`.
- `<project>/results.db` — SQLite with the run tables (`runs`, `detector_intervals`,
  `trips`, …). Created in `crates/output/src/results_db.rs:ResultsDbWriter::new`.
- `<project>/fcd/<scenario>_<seed>.parquet` — FCD, zstd. Schema from
  `crates/output/src/results_db.rs:FcdParquetWriter::new`: `time` f64, `vehicle_id` u64,
  `link_id` utf8, `lane` u32, `position` f64, `speed` f64, `acceleration` f64. One batch
  per written step, rows in time order.

The visualizer reads these files and never writes into the project directory. The
legacy CSV FCD (`time, vehicle_id, link_id, lane_idx, s, speed, acceleration`) is not
supported in Phase 1.

### 2.2 FCD has no coordinates; placement uses the network

A row gives a link, a lane and a distance along the link. The world position is the
point at `position` along the link's `geometry` polyline, moved sideways by the lane
offset; the heading is the polyline's tangent there. The engine already does this for its
own GUI in `crates/geometry/src/geo_util.rs:LinkGeometryIndex::interpolate`.

**Proposed: reuse the engine's placement, do not rewrite it.** Depend on
`assimilator-config` and `assimilator-geometry` as git dependencies pinned to an engine
commit. Argument: the harness boundary's placement rule (asm-001 §10) — if getting it
wrong gives a wrong number, it belongs in the engine. A second implementation of lane
offsets would drift from the engine's. Cost: `assimilator-geometry` depends on
`assimilator-core` and `assimilator-models`, so the build grows (disk is tight, §2.5).
See OQ-1.

### 2.3 Headless render with a fixed clock

Bevy renders to an offscreen texture; no window in `render` mode. Frame `n` shows sim
time `t0 + n · speedup / fps`. The frame count is fixed by the time window, the speed-up
and the fps, never by wall-clock time, so the same inputs give the same frames. Each
frame is read back and written as raw RGBA to the stdin of an `ffmpeg` child process.
Phase 1 uses `libx264`; hardware encoders are a later option.

### 2.4 CLI

```
assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
                         [--fcd <file>] [--from <s>] [--to <s>] [--speedup <x>]
                         [--fps <n>] [--width <px>] [--height <px>]
```

- `--fcd` overrides the path in §2.1.
- Progress: one JSON object per line on stderr (`{"frame": n, "of": N}`); a final
  `{"done": "<out>"}`. Nothing else on stderr except errors.
- Exit 0 only when the MP4 is complete. Non-zero for any error, with one line naming it
  (missing file, schema mismatch, unknown link id in FCD, ffmpeg missing or failed).
- `scene.toml`, `check` and `prepare` are later phases (§2.7).

### 2.5 Build footprint

The development laptop has about 100 GB free. Bevy with `default-features = false` and
only the features the renderer uses; `debug = "line-tables-only"` for dependencies in
the dev profile; no `dynamic_linking` by default. Phase 1 records the size of `target/`
after a clean debug and release build, so later phases can see what they add.

### 2.6 Look

Stylized: flat-shaded, flat colors, no textures, no shadows in Phase 1. Visual tuning is
iteration, not specification; a spec records only what a gate can check.

### 2.7 Roadmap to v1 (not yet phases)

Each item becomes a phase appended to this spec (or a new spec, per §6.1) when it is
drafted and reviewed. Each ends with a video.

2. **Correct motion** — interpolation between FCD samples (60 fps from 1 Hz),
   junction paths, signal states from the plans, lane-change motion.
3. **Camera** — keyframes, orbit, follow-vehicle in `scene.toml`; preview window with a
   free-fly camera that saves paths.
4. **City** — `prepare`: Overture buildings and land use for the network area, chunked
   meshes, a cached scene bundle; `render` makes no network calls.
5. **Data** — links colored by speed or flow; HUD with clock, legend, one chart.
6. **Harness contract (v1)** — `check`, supported schema version ranges, stable
   `scene.toml`, harness-side docs, a Linux build check.

## 3. Open questions

- **OQ-1** — Reuse the engine's placement (`assimilator-geometry`, git-pinned) or
  re-implement it on `assimilator-config` alone? Reuse gives the engine's numbers; it
  costs build size and couples this repo to engine internals that are not a published
  surface. *(design call)*
- **OQ-2** — What does FCD hold while a vehicle is inside a junction: the incoming link
  with `position` past its end, the outgoing link, or nothing? The GUI places such a
  vehicle on a turn path (`crates/geometry/src/frame_collect.rs:interpolate_turn_path`),
  which FCD does not record. Phase 1 must at least not fail on it. *(needs-input: read
  the engine; a Phase 2 decision)*
- **OQ-3** — FCD has no `vehicle_class` or `vehicle_length`, though
  `crates/core/src/output_data.rs:VehicleSnapshot` carries both. Phase 1 draws one box
  size. Showing buses and trucks needs the engine to write the two columns — an engine
  request, not done here. *(needs-input: engine)*
- **OQ-4** — FCD is off in agent runs (`run_scenario` forces it off, asm-001 §10). How a
  harness run gets FCD for a video is a harness and engine decision. *(deferred by
  evidence: roadmap Phase 6)*
- **OQ-5** — Which engine commit do the Phase 1 test runs and any git dependency pin
  to, and who moves it? The harness pins a detached engine worktree; this repo could pin
  a git rev in `Cargo.toml` instead. *(design call)*

## 4. Implementation phases

### Phase 1 — Moving boxes: one run to one video
*Produces the observable: yes — an MP4 of one run in which each FCD vehicle is a box
moving along its link.*

- **Scope:**
  - A Rust binary crate `assimilator-video` with the `render` command of §2.4.
  - Resolve and read `network.yaml` and the FCD Parquet (§2.1). `results.db` is opened
    only to confirm the run exists (a `runs` row for scenario and seed); no KPI is read.
  - Read FCD for the time window only, using row-group statistics on `time`.
  - Place each row per §2.2 (the choice follows OQ-1). A row whose `link_id` is not in
    the network is an error. Frames use the nearest FCD sample at or before the frame
    time; no interpolation (roadmap Phase 2).
  - Scene: each link drawn as a flat grey strip of the link's total lane width; each
    vehicle as a box 4.5 × 1.8 × 1.5 m colored by speed; fixed top-down orthographic
    camera fitted to the network's bounding box; plain background.
  - Headless Bevy → raw RGBA → `ffmpeg` (`libx264`, yuv420p) per §2.3. ffmpeg must be on
    `PATH`; its absence is an error before any frame is rendered.
  - A test run: copy a bundled engine example with junctions (for example
    `configs/bundled-examples/urban_grid`) into a scratch folder outside both repos, turn
    FCD on, run it with the engine CLI, and keep the copy as this repo's fixture
    (or a script that recreates it). Never run inside the engine repo.
- **Exit gate:**
  1. `render` on the fixture writes an MP4 that `ffprobe` reports with the requested
     width, height, fps and a frame count equal to `ceil((to − from) · fps / speedup)`.
  2. Rendering the same inputs twice gives the same decoded frames (compare frame
     hashes from `ffmpeg -f framemd5`).
  3. For three FCD rows chosen by the test, the rendered vehicle's pixel position is
     within 2 px of the position computed by the placement code for that row.
  4. Missing FCD, an unknown `link_id`, and missing ffmpeg each exit non-zero with one
     error line, and write no MP4.
  5. The `target/` sizes of §2.5 are recorded in the review file.
  6. A human watches the MP4 and confirms the vehicles move along the roads.
- **Close-out:** seed `rules/inputs.md` (what is read, from where) and `rules/render.md`
  (clock, pipeline, CLI); a README with the command and its prerequisites (Rust, ffmpeg);
  write this phase's `shipped` date.
