---
id: vis-001
title: visualizer
note: >
  What the visualizer is for: it renders a 3D video of one Assimilator simulation run,
  for presentations, from a CLI a harness can call. Phase 1 is the smallest surface that
  produces a video: vehicles as boxes on flat roads, top-down camera, headless Bevy to ffmpeg.
status: accepted
last_updated: 2026-09-29

phases:
  - name: "Phase 1 — Moving boxes: one run to one video"
    reviewed: 2026-09-28
    shipped: 2026-09-28
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

A run is described by these files (engine `rules/project-schema.md`):
- `<project>/project.yaml` and the files its scenario names — the network the run used
  is the **scenario's resolved network**: its `network:` file with `network_override`
  and inline `overrides` applied, as `crates/config/src/project.rs:resolve_scenario`
  returns it in `ResolvedScenario::network` (a `crates/config/src/network.rs:NetworkConfig`,
  metric coordinates, `metadata.map_origin` `[lng, lat]`). Harness treatments always
  carry `overrides`, so reading `<project>/network.yaml` directly would place their
  vehicles on the wrong network. The engine CLI loads it the same way
  (`crates/output/src/runner.rs:load_project_scenario`). `results.db` does not record the network
  (`runs.resolved_config` holds only the scenario name and sim params,
  `crates/output/src/runner.rs:run_to_results_db`), so the project is resolved as it is
  at render time; a project edited after the run renders against the edited network.
- `results.db` — SQLite with the run tables (`runs`, `detector_intervals`, `trips`, …),
  created in `crates/output/src/results_db.rs:ResultsDbWriter::new`, at the engine CLI's
  `-o` path (default `results.db` in the working directory). Default here:
  `<project>/results.db`; `--results` overrides it.
- `fcd/<scenario>_<seed>.parquet`, in the directory that holds `results.db`
  (`run_to_results_db` puts `fcd/` beside the database) — FCD, zstd. Schema from
  `crates/output/src/results_db.rs:FcdParquetWriter::new`: `time` f64, `vehicle_id` u64,
  `link_id` utf8, `lane` u32, `position` f64, `speed` f64, `acceleration` f64. Rows in
  time order. `position` is the vehicle **centre** along the link (asm-020 §15.4).

The visualizer reads these files and never writes into the project directory:
`results.db` is opened read-only with SQLite's `immutable=1` URI parameter, which also
keeps a WAL database from growing `-shm`/`-wal` files beside it. `immutable=1` ignores
a leftover `-wal` file, so the renderer reads a finished run only. A `-wal` file left
by a crash or by a writer that is still open is out of scope. The legacy CSV FCD
(`time, vehicle_id, link_id, lane_idx, s, speed, acceleration`) is not supported in
Phase 1.

### 2.2 FCD has no coordinates; placement uses the network

A row gives a link, a lane and a distance along the link. The world position is not
simply "`position` along the link's `geometry` polyline plus the lane offset". FCD
`position` is measured along the **junction-trimmed** link: `NetworkData::from_config`
(`crates/core/src/network_data.rs:NetworkData::from_config`) generates a polygon for
each junction and trims each link's polyline at it (waypoint ends exempt), and
`crates/core/src/network_data.rs:NetworkData::link_length` is the trimmed length. The
engine's GUI placement,
`crates/geometry/src/geo_util.rs:LinkGeometryIndex::from_network_config`, then
- places along that trimmed polyline (`NetworkData::trimmed_geometry`), falling back to
  the raw `geometry`;
- shifts a link that has an opposing link (same nodes, reversed) to the right by
  `total_width/2 + median_gap/2`, plus the link's `lateral_offset`
  (`crates/geometry/src/network_json.rs:offset_polyline`);
- rescales `s` from the centreline length to the offset polyline's length, and clamps
  it to the polyline;
- moves sideways by the lane's centre offset
  (`crates/config/src/network.rs:LinkConfig::lane_center_offsets`).

`LinkGeometryIndex::interpolate(link, s, lane)` returns `x`, `y` (metres) and a heading
(degrees, 0 = north, clockwise) at the segment's tangent.

#### 2.2.1 Placement is the engine's code, reused (decision, recorded — OQ-1)

Decided by the user, 2026-09-28. Phase 1 depends on `assimilator-config`,
`assimilator-core` and `assimilator-geometry` as git dependencies at one pinned engine
rev (§2.2.2), and places a row with exactly this chain:
`resolve_scenario(...).network` → `NetworkData::from_config(&network)` →
`LinkGeometryIndex::from_network_config(&network, &nd)` → `interpolate(link_id,
position, lane)`. It does not re-implement any part of it.

Why:
- The placement is a number, and asm-001 §10 puts numbers in the engine. Matching FCD
  `position` needs the junction polygons and the trimming, which is about 1,200–1,300
  lines of engine code (`from_network_config`, `interpolate`, `offset_polyline`, the
  lane offsets, and in `NetworkData::from_config` the junction-polygon pass,
  `generate_junction_polygon`, the trim functions and their polygon helpers; counted at
  engine `1c15330`). A port would drift from it.
- Re-implementing "on `assimilator-config` alone" buys nothing: `assimilator-config` is
  in the same private repo, so it is the same private git dependency, and it does not
  hold the trimming.
- Override resolution comes with it (`resolve_scenario`, §2.1).

Costs, accepted:
- **The repo is public and the dependency is private.** No one without engine access
  can build it, so no gate runs outside this machine (Phase 1, "Who can run the gates").
  The route to a public build is an engine request, OQ-6.
- **Build size.** Through `assimilator-core` the build also pulls
  `assimilator-demand`, `-network` (petgraph), `-models`, `hecs`, `rayon` and `rand`, all
  pure Rust and small next to Bevy. `crates/project` (git2) is not needed. §2.5 records
  the size.
- **Coupling to engine internals** that are not a published surface. Moving the pin
  (§2.2.2) can break the build; that is when it is found.

Rejected:
- Porting the placement onto a local serde model of `network.yaml`. It builds publicly,
  but it duplicates about 1.2k lines plus override resolution, and it would need a gate
  of its own against the engine's placement.
- Waiting for the engine to publish placement as data (OQ-6) before Phase 1.

#### 2.2.2 One engine rev, moved on purpose (decision, recorded — OQ-5)

Decided by the user, 2026-09-28.
- **The pin is engine `1c15330`.** That is engine main on 2026-09-28: one specs-only
  commit after `ab3e92a`, which merged asm-008 Phase 3 and re-baselined trajectories.
  It is also where the harness worktree `../assimilator-wt/harness` sits.
- **The rev is written in one place:** the `rev =` of the engine git dependencies in
  `Cargo.toml`. The fixture script (Phase 1) reads it from there and builds the engine
  CLI at that rev, so the code that places vehicles and the engine that made the FCD
  are the same commit.
- **The user moves it, on purpose,** in a commit of its own that re-runs the Phase 1
  gates. It never follows engine main or the harness pin automatically.
- **The expected move** is to the merge of asm-020 Phase 3 (the FCD `vehicle_class`
  and `vehicle_length` columns). After that, gate 7 can use the engine's column instead
  of the derived file.
- **Moved 2026-09-29 (user): `1c15330` → `df8aec0`.** The reason is the expected move
  above: asm-020 Phase 3 merged into engine main at `708d35b`, and `df8aec0` is the
  specs-only commit after it (asm-020 OQ-7 recorded). The same merge deleted
  `crates/logging`, which this repo never depended on. The build needed no code change.
  The dependency tree differs only in the rev of the six `assimilator-*` crates (config,
  core, demand, geometry, models, network). The Phase 1 gates were re-run from scratch
  at the new pin and all pass; the results are in the Phase 1 close-out and
  `specs/reviews/vis-001.md`. The last bullet's "gate 7 can use the engine's column
  instead of the derived file" held only in part: gate 7 now also measures the engine's
  column, but urban_grid is all cars at 4.5 m, so the derived file stays for the 2.0 m
  and 12.0 m cases (Phase 1 close-out).

### 2.3 Headless render with a fixed clock

Bevy renders to an offscreen texture; no window in `render` mode. Frame `n` shows sim
time `t0 + n · speedup / fps`. The frame count is fixed by the time window, the speed-up
and the fps, never by wall-clock time. With draw order fixed (links in the network's
order, vehicles by `vehicle_id`, never `HashMap` iteration), the same inputs give the
same frames **on the same machine, GPU and driver** — the claim gate 2 checks, and no
wider one. Each
frame is read back and written as raw RGBA to the stdin of an `ffmpeg` child process.
Phase 1 uses `libx264`; hardware encoders are a later option.

*Correction (2026-09-28, Phase 1 close-out).* A fixed draw order does not make frames
deterministic in Bevy. Its opaque pass bins draws and does not keep their order stable
between frames. So where boxes overlap at equal height, the box that shows varied from
frame to frame, and gate 2 failed.
- What makes frames deterministic is the depth order. Each box is lifted 0.01 m per rank
  in `vehicle_id` order among the vehicles of its frame, and the higher id wins an
  overlap.
- The camera is orthographic straight down and nothing is lit, so the lift moves no
  pixel.
- The overlapping boxes are themselves an OQ-2 artefact: a box frozen at the approach end
  is reached by the next vehicle. Phase 2 handles it.
- The measurements are in `specs/reviews/vis-001.md`.

### 2.4 CLI

```
assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
                         [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                         [--speedup <x>] [--fps <n>] [--width <px>] [--height <px>]
```

- `--results` and `--fcd` override the paths in §2.1.
- Defaults. The window runs from `--from`, the first FCD `time` in the file, to
  `--to`, the last one. `--fps` is 30. `--width`×`--height` is 1920×1080; both must be
  even (libx264 with yuv420p), and an odd value is an error. `--speedup` defaults to
  `D / clamp(D, 30, 300)` with `D = to − from` in seconds, so the default video lasts
  the run's window clamped to 30 s–5 min (§1): a 1 h run plays at 12× in 5 min, and a
  10 s run plays at 1/3 speed over 30 s. An explicit `--speedup` wins, and the video's
  length then follows from it. The frame count is `N = ceil(D · fps / speedup − 1e-6)`.
  The epsilon keeps a `D` accumulated from 0.1 s steps (a hair over a whole number)
  from adding a frame.
- Progress: one JSON object per line on stderr (`{"frame": n, "of": N}`, n from 1),
  then a final `{"done": "<out>"}`. Nothing else goes to stderr except the error line:
  Bevy's `LogPlugin` is disabled and wgpu logging is off.
- Exit 0 only when the MP4 is complete. Any error exits non-zero with one line naming it:
  missing file, schema mismatch, an unknown link id in FCD, no completed run, ffmpeg
  missing or failed. Every check that can fail on the inputs (files, schema, `runs`
  row, every `link_id` in the window, ffmpeg on `PATH`) runs **before the first frame**.
  So an input error prints no progress line and creates no output file.
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

- ~~**OQ-1** — Reuse the engine's placement (`assimilator-geometry`, git-pinned) or
  re-implement it on `assimilator-config` alone? Reuse gives the engine's numbers; it
  costs build size and couples this repo to engine internals that are not a published
  surface. This repo is public and the engine repo is private, so reuse means a git
  dependency that no one without engine access can fetch, including any future CI;
  re-implementing has no such dependency. *(design call)*~~ **RESOLVED 2026-09-28
  (user): reuse**, recorded in §2.2.1. The premise was partly wrong.
  `assimilator-config` is in the same private repo, so re-implementing on it has the
  same dependency. It also does not hold the junction trimming that FCD `position`
  depends on; that is in `assimilator-core` (review round 1, finding 1). The route to a
  build without the private dependency is OQ-6.
- ~~**OQ-2** — What does FCD hold while a vehicle is inside a junction: the incoming link
  with `position` past its end, the outgoing link, or nothing? The GUI places such a
  vehicle on a turn path (`crates/geometry/src/frame_collect.rs:interpolate_turn_path`),
  which FCD does not record. Phase 1 must at least not fail on it. *(needs-input: read
  the engine; a Phase 2 decision)*~~ **Answered by asm-020 §15.4** (engine `ad73b75`).
  While a micro vehicle is in a junction, its rows stay on the approach link and entry
  lane with `position` frozen at its value on entry: `≥ link_length − length/2 − 0.1`
  for most vehicles, and up to the stop-line offset + 5 m short of that after a
  gap-acceptance release. `speed` and `acceleration` stay live. The next row after the
  transit is on the departure link at `s = excess`. Meso has no junction interior.
  Consequence for Phase 1: a box holds at the approach end, then jumps onto the
  departure link. That is acceptable, and Phase 1 must not fail on it. Drawing turns is
  a Phase 2 decision. If Phase 2 measures that turns derived from the network are not
  good enough, it takes that evidence to the engine as a request against asm-020 OQ-4,
  which keeps turn paths out of FCD until a consumer asks.
- ~~**OQ-3** — FCD has no `vehicle_class` or `vehicle_length`, though
  `crates/core/src/output_data.rs:VehicleSnapshot` carries both. Phase 1 draws one box
  size. Showing buses and trucks needs the engine to write the two columns — an engine
  request, not done here. *(needs-input: engine)*~~ **Answered by asm-020 §15–§16,
  pending the engine build.** asm-020 Phase 3 (reviewed, not built) appends
  `vehicle_class` (Arrow `Dictionary(Int32, Utf8)`) and `vehicle_length` (`Float64`, m)
  after the seven columns of §2.1, which do not change. Micro writes each vehicle's
  sampled length; meso writes the class's representative mean (asm-020 §15.3). Files
  written before that phase lack both columns (asm-020 §15.6). The names and the shape
  are decided (asm-020 OQ-1/OQ-2, resolved at engine `2f2fa58`). The engine has no
  vehicle width (asm-020 OQ-3), so width stays this repo's constant.
  **CLOSED 2026-09-29:** asm-020 Phase 3 was built and merged at engine `708d35b`, and the
  pin moved to `df8aec0` (§2.2.2). Phase 1's reader already used `vehicle_length` when
  present, so nothing else changed.
- **OQ-4** — FCD is off in agent runs (`run_scenario` forces it off, asm-001 §10). How a
  harness run gets FCD for a video is a harness and engine decision. *(deferred by
  evidence: roadmap Phase 6)*
- ~~**OQ-5** — Which engine commit do the Phase 1 test runs and any git dependency pin
  to, and who moves it? The harness pins a detached engine worktree; this repo could pin
  a git rev in `Cargo.toml` instead. *(design call)*~~ **RESOLVED 2026-09-28 (user):
  engine `1c15330`**, written once as the `rev =` in `Cargo.toml`. The fixture script
  builds the engine CLI at the same rev. The user moves the pin on purpose, in a commit
  that re-runs the Phase 1 gates. Recorded in §2.2.2.
- **OQ-6** — Can the engine publish its placement as data, so that a build of this repo
  needs no private dependency? For example: per-link trimmed and offset polylines, the
  centreline length and the lane centre offsets, written beside the FCD or by a CLI
  command. This would be an engine request (asm-001 §10 owns that side), not done here.
  It is not needed for Phase 1, which reuses the code (§2.2.1). It blocks a public build
  and any CI. *(needs-input: engine; deferred by evidence to roadmap Phase 6, the Linux
  build check)*

## 4. Implementation phases

### Phase 1 — Moving boxes: one run to one video
*Produces the observable: yes — an MP4 of one run in which each FCD vehicle is a box
moving along its link.*

- **Scope:**
  - **The crate.** A Rust crate `assimilator-video`, a library with a thin binary that
    runs the `render` command of §2.4. The engine crates `assimilator-config`,
    `assimilator-core` and `assimilator-geometry` are git dependencies at the rev of
    §2.2.2. The library exposes three things:
    - rendering one frame to its RGBA buffer, the lossless readback before encoding;
    - the camera's world-to-pixel transform;
    - its metres per pixel, `k`.

    Gates 3 and 7 measure through these, not through the MP4. The crate's `serde_yaml`
    must be the engine's version (0.9), because a `ProjectConfig` is deserialized from
    the engine's `serde_yaml::Value`.
  - **Inputs (§2.1).**
    - Load the project as `load_project_scenario` does, without `--set` overrides
      (which reach no network path) and without depending on `assimilator-output`:
      - `crates/config/src/version.rs:parse_and_check` on `<project>/project.yaml`;
      - deserialize a `ProjectConfig`;
      - `resolve_scenario(&project, scenario, project_dir)`.
    - Take its `.network`.
    - Open `results.db` read-only and immutable. Require a `runs` row for scenario and
      seed whose `status` is `completed`; any other status is the "no completed run"
      error. No KPI is read.
  - **Reading FCD.**
    - Select columns by name, never by position.
    - Read the rows in `[from, to]`, plus the last snapshot at or before `from`. Row-group
      statistics on `time` may skip row groups outside the window. This is an
      optimisation only: the fixture's FCD is a single row group, so no gate observes it.
    - `vehicle_length`, when present, sets each box's length; otherwise the length is
      4.5 m (OQ-3). `vehicle_class` is ignored.
  - **Placement.**
    - Place each row with the chain of §2.2.1.
    - Every `link_id` in the window is checked against the network before the first
      frame. One that is missing is an error.
    - Frame `n` (from 0) shows sim time `t_n = from + n · speedup / fps`, using the
      **snapshot** at the latest FCD `time ≤ t_n`: exactly the vehicles in that
      snapshot, each at its row. A vehicle absent from that snapshot is not drawn, so a
      vehicle that has left the network disappears.
    - There is no interpolation (roadmap Phase 2).
    - A vehicle in a junction holds at the end of its approach link, then jumps onto the
      departure link (OQ-2). That is not an error.
  - **Scene.**
    - **Roads.** Each link is a flat grey strip of `LinkConfig::total_width`, centred on
      the **same trimmed and offset polyline that placement uses**. The engine keeps
      that polyline private (`LinkPolyline::points`), so the strip is sampled through
      `LinkGeometryIndex::interpolate_with_lateral(link, s, 0.0)`, at `s` from 0 to
      `NetworkData::link_length(link)` in steps of at most 1 m, plus the end point.
      Drawing on the raw `geometry` would put the boxes of an opposing pair's outer lane
      off the road: urban_grid's pairs share one centreline and sit 3.75 m to either
      side of it.
    - **Vehicles.** Each vehicle is a box of its length × 1.8 m (width, for every
      vehicle) × 1.5 m (height), centred on the placed point (`position` is the centre,
      §2.1), turned to the placed heading, and coloured by speed. The speed colours are
      never the road grey or the background colour.
    - **Camera.** Top-down orthographic, fitted to the bounding box of the drawn strips
      plus a 20 m margin on every side, so `k = max(bw / width, bh / height)`.
    - **Background.** Plain.
    - **Draw order** is fixed (§2.3).
  - **Output.**
    - Headless Bevy → raw RGBA → `ffmpeg` (`libx264`, yuv420p) per §2.3.
    - ffmpeg writes `<out>.partial`, which is renamed to `<out>` only after ffmpeg exits
      0. So a failure leaves no file at `<out>`.
    - Inputs are validated before the first frame (§2.4), and that includes ffmpeg being
      on `PATH`.
  - **The fixture script,** `scripts/fixture.sh`. It writes only under `scratch/`
    (gitignored) and never inside the engine repo. No engine data is committed here.
    It:
    1. reads the engine rev and git URL from `Cargo.toml`;
    2. builds the engine CLI at that rev with
       `cargo install --git <url> --rev <rev> --locked assimilator-cli --root scratch/engine`.
       Cargo builds in a temporary target directory and keeps only the binary, so there
       is no engine worktree and no `target/` in `../assimilator`;
    3. exports `configs/bundled-examples/urban_grid` at the same rev with `git -C
       ${ENGINE_CHECKOUT:-../assimilator} archive <rev>` into `scratch/urban_grid`
       (9 signalized junctions, 2 lanes × 3.5 m per link, every link in an opposing
       pair). `git archive` only reads the checkout;
    4. runs `scratch/engine/bin/assimilator run -c scratch/urban_grid -s baseline -o
       scratch/urban_grid/results.db --set simulation.output.fcd.enabled=true`.
       `--set` builds the nested mapping (`crates/config/src/project.rs:apply_set_overrides`);
       urban_grid's `simulation:` block has no `output:` key. FCD lands in
       `scratch/urban_grid/fcd/`;
    5. derives the test FCD files with a small Rust binary in this repo, using the
       arrow and parquet crates the renderer already needs (no Python). The binary
       writes:
       - `vehicle_length` appended as `Float64`, 2.0, 8.0 or 12.0 m by
         `vehicle_id mod 3` (for gate 7);
       - one row's `link_id` replaced by an id absent from the network (for gate 4);
       - single-vehicle subsets on request (for gates 3 and 7).

    Once asm-020 Phase 3 ships and the pin moves (§2.2.2), the engine writes
    `vehicle_length` itself.
  - **Who can run the gates.** Every gate needs this machine: the engine is private, and
    the build depends on it (§2.2.1), so without engine access the crate does not
    compile and no gate runs, not even 4 or 5. The README says so. OQ-6 is the route
    to a public build.
- **Exit gate.** All gates run on the fixture above, on the development machine.
  - **Scale for gates 3 and 7.** These gates render at 3840×2160 and first assert
    `k ≤ 0.6 m/px`.
    - Why 0.6 (re-derived 2026-09-28, review round 2): urban_grid's nodes span
      1200 m × 1200 m. The drawn strips' bounding box is also 1200 m, because each
      offset is perpendicular to its own link and never pushes a strip past the outer
      endpoints. With the margin that is 1240 m, and at a height of 2160 px
      `k ≈ 0.574`.
    - So 2 px ≤ 1.2 m, about half the smallest error these gates exist to catch:
      drawing the box's front instead of its centre (2.25 m, ≈3.9 px). A wrong lane
      (3.5 m, ≈6.1 px) and a missing opposing-pair offset (3.75 m, ≈6.5 px) are caught
      with more margin.
    - Positions are measured on the lossless readback: the pixels that differ from the
      same frame rendered with no vehicles, weighted by their difference. So neither
      MSAA edges nor yuv420p and x264 enter the error.
  1. `render` on the fixture with defaults writes an MP4. `ffprobe` reports
     1920×1080, 30 fps and `N` frames, with `N`, `D` and the default speedup per §2.4.
     For the fixture, FCD runs from 0.1 s to 299.1 s, so the default speedup is 1 and
     `N = 8970`. Stderr is exactly `N` progress lines (`frame` 1…`N`, `of`
     `N`) and then the `done` line. The same holds for one run with explicit `--from`,
     `--to`, `--speedup`, `--fps`, `--width` and `--height`.
  2. Rendering the same inputs twice on this machine gives the same decoded frames: the
     frame hashes from `ffmpeg -f framemd5` are equal.
  3. The test chooses three FCD rows: one on lane 0, one on lane 1, and one frozen at the
     end of an approach link during a junction transit (OQ-2). For each:
     - render frame 0 with `from` set to that row's `time`, from a single-vehicle subset
       of the FCD;
     - the weighted centroid of the vehicle's pixels must be within 2 px of
       `LinkGeometryIndex::interpolate(link_id, position, lane)` projected by the
       camera.
  4. Each of these cases exits non-zero, writes exactly one stderr line and no progress
     line, and leaves no file at `--out`:
     - the FCD file is missing;
     - the derived file has an unknown `link_id`;
     - ffmpeg is missing from `PATH`.
  5. The `target/` sizes of §2.5 are recorded in the review file.
  6. A human watches the default MP4 and confirms that the vehicles move along the
     roads, in their lanes, on the right-hand strip of each pair.
  7. Box length, on single-vehicle subsets, measuring each box's extent along its
     heading:
     - **Run's FCD** (no `vehicle_class` or `vehicle_length`): for two vehicles chosen
       by the test, the extent is within 2 px of 4.5 m.
     - **Derived file:** for a 2.0 m vehicle and a 12.0 m vehicle, the extent is within
       2 px of each vehicle's own length.

     The lengths differ from 4.5 m and from each other by at least 2.5 m (≈4.3 px),
     more than twice the 2 px tolerance. So an implementation that ignores
     `vehicle_length`, or confuses the two lengths, fails.
- **Close-out:**
  - Seed `rules/inputs.md` (what is read, and from where) and `rules/render.md` (clock,
    pipeline, CLI).
  - Write a README with the command and its prerequisites: Rust, ffmpeg and ffprobe,
    engine access and how cargo fetches the private git dependency, and
    `scripts/fixture.sh`. It must say that the gates run only with engine access.
  - Write this phase's `shipped` date.
  - *Correction (2026-09-28, close-out) to gate 1's prediction.* The fixture's FCD runs
    from **9.1 s**, when the first vehicle spawns, to 299.1 s, not from 0.1 s; that span
    was never measured. So D = 290 s, the default speedup is 1 and **N = 8700**, not 8970.
    Gate 1 checks N against §2.4's formula, with D taken from the fixture FCD's own
    min/max `time`. The gate text above is left as written.
  - *Gate 6 passed (2026-09-28): a human check, by the user.* The user watched
    `scratch/out/default.mp4` at full frame, and a 4× close-up of the centre junction for
    t = 150–190 s.
    - Boxes stay on the roads and in their lanes: eastbound on the south strip,
      northbound on the east strip, both lanes used, and one lane on the one-lane link
      `L_J11_J01`. There is no flicker at queues.
    - Known and expected for Phase 1:
      - only links are drawn, with no junction surfaces;
      - motion steps once per second (1 Hz FCD, no interpolation until Phase 2);
      - boxes hold at the approach end and jump across junctions (OQ-2);
      - the fixed full-network camera makes cars about 8 px long at 1080p.
    - The fixture description in step 3 ("2 lanes × 3.5 m per link") is slightly off:
      47 of urban_grid's 48 links have 2 lanes, and `L_J11_J01` has 1. No gate depended
      on it.
  - *Gates re-run at the new pin (2026-09-29, engine `df8aec0`, §2.2.2).* Every gate was
    re-run from scratch: clean builds, the engine CLI re-installed, the fixture re-run. All
    pass.
    - The fixture FCD now has nine columns. Its seven Phase 1 columns are identical to the
      run at `1c15330` under DuckDB `EXCEPT ALL` in both directions (0 rows each way,
      21 557 rows each). Every vehicle is `car` at `vehicle_length` 4.5.
    - Gates 1, 3, 4 and 7 give Phase 1's values exactly: N = 8700; gate 3 errors 0.050,
      0.016 and 0.053 px; gate 7 errors 0.281 and 0.090 px at 4.5 m, and 0.102 and
      0.244 px at 2.0 and 12.0 m.
    - Gate 2: the default render's decoded frames equal Phase 1's shipped render hash for
      hash (8700 frames), so gate 6's human check carries over unchanged.
    - Gate 5: release 2 m 58 s and 1.2 GB, debug 1 m 27 s and 3.5 GB.
    - **Missed prediction 1: the test tooling broke on the new columns.** `fcd-derive
      subset`'s hand-written column copy did not handle `vehicle_class`'s
      `Dictionary(Int32, Utf8)`, so gates 3 and 7 failed. `fcd-derive lengths` refused
      a file that already had `vehicle_length`, so fixture step 5 stopped. Cause: the
      tooling was written against the seven-column file and was never exercised on the
      nine-column schema asm-020 §15.2 had already fixed. Fix (user-approved):
      `subset` uses arrow's `filter_record_batch`, which handles any column type, and
      `lengths` overwrites an existing `vehicle_length`.
    - **Missed prediction 2: gate 7 still needs a derived length file.** §2.2.2 expected
      the engine's column to replace it. urban_grid has only `car`, so the column is 4.5 m
      everywhere and cannot show 2.0 m or 12.0 m. Gate 7 now measures three cases:
      (a) the engine's column as written (4.5 m); (b) the derived 2.0 m and 12.0 m file;
      (c) a copy without `vehicle_class` and `vehicle_length` (new `fcd-derive
      drop-columns`), which must render at the 4.5 m fallback. The gate 7 text above is
      left as written.
    - `scripts/gates.sh` now deletes the old framemd5 files before gate 2 and runs ffmpeg
      with `-y`. Without both, a rerun could compare stale hash files.
