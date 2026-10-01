---
id: vis-001
title: visualizer
note: >
  What the visualizer is for: it renders a 3D video of one Assimilator simulation run,
  for presentations, from a CLI a harness can call. Phase 1 is the smallest surface that
  produces a video: vehicles as boxes on flat roads, top-down camera, headless Bevy to ffmpeg.
status: accepted
last_updated: 2026-09-30

phases:
  - name: "Phase 1 — Moving boxes: one run to one video"
    reviewed: 2026-09-28
    shipped: 2026-09-28
    cut: null
    by: null
  - name: "Phase 2 — Correct motion: smooth video from 1 Hz FCD"
    reviewed: 2026-09-29
    shipped: 2026-09-29
    cut: null
    by: null
  - name: "Phase 3 — View: a window over a finished run"
    reviewed: 2026-09-29
    shipped: 2026-09-29
    cut: null
    by: null
  - name: "Phase 4 — Time slider: click and drag through the run in view"
    reviewed: 2026-09-30
    shipped: 2026-09-30
    cut: null
    by: null
  - name: "Phase 5 — 3D camera: orbit and tilt in view, keyframed flights in render"
    reviewed: 2026-09-30
    shipped: 2026-09-30
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: []
reference: >
  Seed brief: ~/dev/ivapo/Orchtr-assimilator-visualizer/idea.md (the idea, the stack discussion and
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
  `scene.toml`, which is an internal artifact. *Note (2026-09-29):* Phase 3 builds it as
  `view` (§2.9): 2D top-down for now, and its output is keyframe lines on stdout.
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

*Note (2026-09-29, Phase 2).* A row is still placed with exactly this chain. Phase 2 adds
the walk along a turn path, also the engine's code: the public functions of
`assimilator-core`'s `spatial_conflict` module, reused and not copied (§2.8.3).

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

*Note (2026-09-30, Phase 5):* "the lift moves no pixel" holds for the orthographic path
only. Under Phase 5's perspective the lift is visible, so that path lifts 0.001 m per rank
instead (§2.11.2); the orthographic path is unchanged.

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
- *Phase 5 (2026-09-30):* `render` gains `[--camera <file.toml>]`, a keyframe file
  (§2.11.4). Without it, everything above is unchanged.

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

2. **Correct motion** — now Phase 2 (§2.8, §4). *Revised 2026-09-29 (user):* the bar
   is at least 30 fps and visibly smooth, not 60 fps; `--fps 60` stays available.
   Signal states moved to item 5.
3. **Camera** — two tools that share the reading, placement, interpolation (§2.8) and
   drawing code:
   - scripted camera moves for `render`, in `scene.toml`: keyframes, orbit, follow a
     vehicle, zoom to a junction, pan;
     *(note 2026-09-30, Phase 5)* the keyframes are read from a file given as `render
     --camera <file.toml>` (§2.11.4), not from `scene.toml`, which stays roadmap item 6's;
   - *(added 2026-09-29)* an interactive `view` command. It opens a Bevy window over a
     **finished** run, with a free camera and time scrubbing, and can save camera paths
     for `render`. Like the preview window of §1, it is a tool and not the observable.

   Watching a run live is out of scope: it needs the engine to stream (§1.1).

   *Split 2026-09-29 (user) into two phases:*
   - **Phase 3 — `view`** (§2.9, §4): the window, top-down 2D like `render`, and the
     keyframe line (§2.9.5). It ends with no video, which Phase 3 argues for.
   - ~~**Phase 4 — scripted camera moves for `render`**, not yet drafted. Its keyframes are
     `view`'s lines (§2.9.5); the file that holds them, and the moves, are Phase 4's.~~

   *Revised 2026-09-30 (user), after Phase 3's gate 11 answered OQ-9:*
   - **Phase 4 — time slider** (§2.10, §4): a bar in `view` to click and drag through the
     run. It ends with no video, which Phase 4 argues for.
   - ~~**Phase 5 — scripted camera moves for `render`**, not yet drafted. Its keyframes are
     `view`'s lines (§2.9.5); the file that holds them, and the moves, are Phase 5's.~~

   *Revised 2026-09-30 (user), before drafting Phase 5:*
   - **Phase 5 — 3D camera** (§2.11, §4): orbit and tilt in `view`, a perspective camera
     in `view` and in keyframed renders, and flights through `K`'s keyframes in `render`.
     It comes before item 4, the city, so the camera is built once, and it works on
     today's roads and boxes. It produces the observable.
4. **City** — `prepare`: Overture buildings and land use for the network area, chunked
   meshes, a cached scene bundle; `render` makes no network calls.
   *Note (2026-10-01):* narrowed by vis-002 (`specs/city_spec.md`), a spec of its own:
   real buildings from Overture only, fetched once by `scripts/fetch-buildings.sh` into a
   cached `buildings.geojson` that `render` and `view` read with `--buildings`, so neither
   makes a network call. `prepare`, land use, chunked meshes and the scene bundle are not
   in it; vis-002's roadmap (§2.13 there) carries what remains.
5. **Data** — links colored by speed or flow; HUD with clock, legend, one chart;
   signal states from the plans (moved from item 2, 2026-09-29).
6. **Harness contract (v1)** — `check`, supported schema version ranges, stable
   `scene.toml`, harness-side docs, a Linux build check.

### 2.8 Motion between samples (Phase 2)

FCD comes at 1 Hz and the video runs at 30 fps or more, so a frame almost never falls on
a sample. Phase 1 drew the last snapshot. Phase 2 draws each vehicle where the samples
around the frame's time say it is. Every rule below works on one vehicle's rows, in
`time` order, taken from the **whole file**, so an interval or junction that crosses
`--from` or `--to` is built as one inside the window; the window only picks the frames.
`Δt` is the time between two rows: 1.0 s in the fixture, assumed by no rule.

#### 2.8.1 Along a link: monotone cubic Hermite

Two consecutive rows `i`, `i+1` on the same link, outside a junction span (§2.8.2), give
`position` `s_i → s_{i+1}` and `speed` `v_i → v_{i+1}`. With `u = (t − t_i)/Δt`, the
position is the cubic Hermite curve through the two positions with end tangents
`m_i = v_i·Δt` and `m_{i+1} = v_{i+1}·Δt`, so the box leaves and reaches each sample at
its speed, and braking and acceleration show. A linear blend moves at `Δs/Δt`
throughout.

Monotonicity is guaranteed by clamping the tangents (Fritsch and Carlson, 1980):
- `Δs = s_{i+1} − s_i`. FCD speeds are never negative, so `m ≥ 0`.
- **`Δs = 0`:** both tangents are taken as 0, and the box stands still for the interval.
  This is a stop, whatever the two speeds say.
- **`Δs > 0`:** with `α = m_i/Δs` and `β = m_{i+1}/Δs`, if `α² + β² > 9` both tangents are
  scaled by `3/√(α² + β²)`. For `α, β ≥ 0` and `α² + β² ≤ 9`, the cubic is non-decreasing
  on `[0, 1]`, so the box never moves backwards.
- **Zero speed at one end** is a zero tangent: the box eases into or out of a stop.
  A clamped interval meets its samples exactly, at a lower speed than FCD's.
- **`Δs < 0`** (never seen in the fixture) is drawn as `Δs = 0`: the box holds at `s_i`
  and jumps back to `s_{i+1}` at `t_{i+1}`, so the next sample is still shown exactly.
  That jump is the one backward move the design allows, and each is counted.

The world position is the engine's placement at the interpolated `s`:
`LinkGeometryIndex::interpolate_with_lateral(link, s, lateral)` with the lateral offset
of §2.8.4, which with no lane change equals `interpolate(link, s, lane)`. A frame at a
sample's time shows the sample, except the frozen rows inside a junction span, which
§2.8.2 replaces on purpose.

#### 2.8.2 Through a junction: the engine's turn path, timed by the live speed

**The span.** When a vehicle's `link_id` changes between rows `D−1` and `D`, the rows
before `D` on the approach link `a` end frozen at `p_f`, the position of `D−1` (OQ-2,
asm-020 §15.4).
- The anchor `A` is the last row on `a`, before `D`, whose position is below `p_f`. If
  there is none, `A` is the first row of that visit to `a`.
- Rows `A … D` are one **junction span**. §2.8.1 does not apply inside it.
- The **entry lane** is the lane of row `D−1`. The **departure lane** is the lane of `D`.
- A transit cut off by the end of the file has no row `D`, so it is not a span. Its
  frozen rows are `Δs = 0` intervals under §2.8.1, and the box stands at the approach
  end, as in Phase 1.

**The movement.** With `n` = `a`'s `to_node` and `b` = `D`'s link:
`crates/core/src/network_data.rs:NetworkData::resolve_route_pair(n, a, b)`. That is the
engine's own resolution of a route pair (asm-045 §2.1).
- `Junction(m)` or `Waypoint(m)`: the turn path is found in this order, and the first
  that exists is used:
  1. `NetworkData::lane_turn_path(n, m.id, entry lane, departure lane)`;
  2. `lane_turn_path` to the departure lane the engine itself picks on entry,
     `NetworkData::resolve_departure_lane(m.id, entry lane, m.from_lanes, m.to_lanes)`.
     The engine does the same in
     `crates/core/src/systems/junction_transit.rs:run_junction_transit_execution`. The
     difference to the FCD departure lane is then a lateral slide (§2.8.4);
  3. `NetworkData::turn_path(n, m.id)`, the movement's centreline. The box slides from
     the entry lane to the centreline over `[t_A, t_{A+1}]`, and from the centreline to
     `D`'s lane over `[t_{D−1}, t_D]` (§2.8.4). A slide that shares an interval with a
     lane slide is added to it.
- `Transparent` (no `JunctionConfig`): no turn path. The box goes straight from the
  approach end to the departure start.
- `None`, or no path in 1–3: **no movement matches.** The box follows the straight chord
  from `interpolate(a, L_a, entry lane)` to `interpolate(b, 0, departure lane)`, timed as
  below. It does not hold and it does not jump. Each such span is counted.

**The geometry** is one path: the approach link from `s_A` to its end `L_a`
(`NetworkData::link_length`, §2.2) in the entry lane, then the turn path, then the
departure link from 0 to `s_D`. Its length is `G = (L_a − s_A) + len(turn path) + s_D`.
In the fixture the per-lane paths meet the engine's placement of the link ends with a
0.000 m gap (421 of 421), and the heading turns by at most 3.3° at a join (measured).

**The timing** comes from the live speed. `speed` stays live while `position` is frozen
(asm-020 §15.4).
- `I(t)` is the integral of the FCD speed from `t_A` to `t`, linear between rows (the
  trapezoid rule). `I = I(t_D)`.
- The distance along the path at time `t` is `d(t) = G · I(t) / I`. Where `I` is under
  1 mm, time is used instead: `d(t) = G · (t − t_A)/(t_D − t_A)`.
- So the span starts at row `A`'s placement, ends exactly at row `D`'s, and waits where
  the vehicle waits (for a gap, or in a queue inside the junction).

**When `G` and `I` disagree**, continuity wins. The ratio `r = G / I` scales the
displayed speed for the whole span, so the box never jumps at `A` or `D`. In the fixture
`r` is 1.024–1.196, median 1.094 (422 spans), and `G − I` is 1.0–3.7 m.

Most of that gap is the frozen shortfall `L_a − p_f`, which the engine itself jumps
on entry and never drives (OQ-8). The box still covers it, so `r` stays the display
scale. But the shortfall grows with vehicle length (a 12 m truck on a short span gives
`r` near 1.3 on correct geometry), so the band test takes it out:
- `r′ = (G − (L_a − p_f)) / I` compares the rebuilt path with the distance the engine
  drove. The residual `e = G − (L_a − p_f) − I` is −1.24 to +1.39 m in the fixture; its
  top is one engine step at the speed limit (13.89 m/s × 0.1 s), which the approach
  clamp can swallow, and the rest is the 1 Hz trapezoid.
- `r′` outside `[0.8, 1.25]` is **out of band**: drawn the same way, counted, and
  evidence for §2.8.6. The edges are a judgement, not derived from the data: a path
  20–25 % off the distance driven is a wrong path, not noise.

The heading along the path is the tangent of the piece the box is on: the link's heading
from `interpolate_with_lateral`, the turn path's segment tangent, or the chord.

#### 2.8.3 The turn-path walk is reused, not copied (decision, recorded)

Decided by the user, 2026-09-29, replacing the draft's decision to copy it.
- A point on a turn path at arc length `s` is
  `crates/core/src/spatial_conflict.rs:interpolate_pos(&path.path, s)`, and its heading is
  `crates/core/src/spatial_conflict.rs:interpolate_heading(&path.path, s)`, on the path
  data of `LaneTurnPath` or `TurnPathInfo` (`crates/core/src/network_data.rs`). Both are
  public at `df8aec0` and pick the segment by accumulated length the same way. Past
  either end, the position clamps to the path and the heading is the end segment's.
- **The one conversion.** `interpolate_heading` returns radians, `atan2(dy, dx)`, counted
  anticlockwise from east. Phase 2 uses the placement's convention (degrees, 0 = north,
  clockwise, §2.2), so the heading is `(90° − θ·180/π)`, wrapped to 0–360°.
- **Dependency: none new.** It is `assimilator-core`, already a git dependency at the
  pinned rev (§2.2.1), and `spatial_conflict` is a `pub mod` of it. `Cargo.toml` does
  not change.
- On a segment shorter than 1e-9 m, `atan2(0, 0)` gives 90° after conversion, where
  the engine GUI's walk gives 0°. The `>=` segment test never selects such a segment
  inside a path, only as the first or last one, at the joins. The Phase 2 test records
  how many first and last segments are that short.
- **Rejected: copying the walk** (the draft's decision). It rested on the walk being
  private, and it was only private in one place:
  `crates/geometry/src/frame_collect.rs:interpolate_turn_path` is a private `fn`, but
  `spatial_conflict` has the same position walk, line for line, as public functions
  (review round 1). The copy would have been about 50 lines to keep in step with the
  engine and an engine request (OQ-7), for no gain.

#### 2.8.4 Lane changes: a sideways slide

A lane difference is a lateral offset that changes over one sample interval, by
smoothstep in time (`3u² − 2u³`). So it starts and ends with no sideways speed, and it
never snaps.
- **Along a link** (rows `i`, `i+1`, lanes `k ≠ k'`): the offset goes from
  `LinkGeometryIndex::lane_offset(link, k)` to `lane_offset(link, k')`.
- **In a span:**
  - a lane difference between `A` and `A+1` slides over `[t_A, t_{A+1}]`;
  - a difference between the turn path's departure lane and `D`'s slides over
    `[t_{D−1}, t_D]`.
  - Whatever part of a slide falls on the turn path is applied perpendicular to its
    tangent, as the engine does for a mid-manoeuvre entry
    (`crates/geometry/src/geo_util.rs:apply_perpendicular_offset`).
- The box keeps the lane's heading. Yaw during a lane change is not drawn in Phase 2.
- The peak sideways speed is `1.5 · Δoffset / Δt`: 5.25 m/s for one 3.5 m lane in 1 s,
  so 0.175 m per frame at 30 fps.

#### 2.8.5 When a vehicle is drawn

A vehicle is drawn at time `t` when `t_first − 1e-6 ≤ t ≤ t_last + 1e-6`, where those are
its first and last rows in the file. Phase 1 kept a vehicle until the next snapshot;
Phase 2 removes it at its last row and does not extrapolate. The depth lift of §2.3 still
ranks the vehicles drawn in each frame by `vehicle_id`.

#### 2.8.6 Fallback: turn paths from the engine (not a request now)

The engine keeps turn paths out of FCD until a consumer asks (asm-020 OQ-4). This design
rebuilds each transit from the network instead. Evidence goes to the engine as a request
against asm-020 OQ-4 **only if this proves not good enough**, namely if:
- the user's visual check (Phase 2 gate 12) rejects the junction motion; or
- a real run gives spans with no matching movement, or `r′` out of band often enough
  to show.

The evidence is the counts and the `r′` distribution that Phase 2 records. Phase 2
prints none of it (stderr carries only progress, §2.4), so on a real run it is read
through `Job::motion_report()`; showing it without the library is roadmap item 6's
`check`.

### 2.9 The viewer (Phase 3)

`view` opens a window over a **finished** run: the same roads and boxes as `render`,
moving as §2.8 places them, under a camera and a clock that the user drives. It is a
tool, not the observable (§2.7 item 3): what it hands on is a keyframe line (§2.9.5),
which Phase 4's scripted camera will read.

*Note (2026-09-30):* the scripted camera is now **Phase 5** (§2.7). Where §2.9 and
Phase 3 say "Phase 4" they mean it; Phase 4 is the time slider (§2.10).

```
assimilator-video view --project <dir> --scenario <name> --seed <n>
                       [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                       [--width <px>] [--height <px>]
```

- **Inputs and checks are `render`'s** (§2.1, §2.4): the same files, the same
  completed-run check, every `link_id` known and every row placed, all before any Bevy
  `App` is built. So an input error prints one `error: …` line on stderr, exits 1, and
  cannot open a window. `view` does not need ffmpeg and does not look for it.
- `--width`×`--height` is the window's size in **logical** pixels (the OS's points,
  2×2 physical on this machine's Retina display), 1280×720 by default; any positive size
  is accepted.
- **Stdout carries only keyframe lines** (§2.9.5), one per press, flushed. Stderr carries
  only the error line (Bevy's logger is off, as in `render`), or the hidden `--bench`
  flag's JSON (Phase 3). Closing the window exits 0.
- **The look is `render`'s** (§2.3): top-down, orthographic, north up. 3D and a tilted
  camera wait for the city phase (§2.7 item 4).
  *Revised 2026-09-30 (user):* Phase 5 brings them before the city. `view` becomes
  perspective, starts straight down and north up, and orbits and tilts (§2.11.3).

#### 2.9.1 The view clock: wall-clock driven

`render`'s clock is fixed by the frame count (§2.3). `view`'s is driven by wall-clock
time, because it has to keep up with a person.
- The state is the sim time `t`, playing or paused, and a speed `s`.
- `t` starts at `from`, paused, at `s` = 1.
- Each window frame, when playing: `t ← t + s · min(Δ, 0.1 s)`, where `Δ` is the real
  time since the last frame. The cap keeps a stall (a window drag, a slow first frame)
  from skipping more than 0.1 s of wall time.
- `t` is clamped to `[from, to]`. Reaching `to` while playing pauses. Play from `to`
  restarts at `from`.
- **Speed** is a ladder of powers of two, `s` ∈ {1/8, 1/4, 1/2, 1, 2, 4, 8, 16, 32, 64}.
  `+` doubles it and `−` halves it, and each stops at its end of the ladder. `+` is
  the `=` key with or without Shift, or keypad `+`; `−` is `-` or keypad `−`.
  *Gate 11 note (2026-09-29, the user's check):* `+` did nothing on the user's keyboard,
  so the user could go down to 1/8× but not back up. The build matched the key's
  position (`KeyCode::Equal`, the US `=` key), not what it types. Keys are now matched by
  their logical character (Bevy's `ButtonInput<Key>`): `+` or `=` doubles, `-` halves, on
  any layout; the keypad keys are matched by position, as before.
- **Step** pauses and moves `t` by a fixed amount, one per key press (held keys do not
  repeat):
  - `←`/`→`: by exactly `1/30` s of sim time, one frame of `render` at its defaults
    (30 fps, 1× on the fixture). Motion is continuous (§2.8), so every step shows a new
    position;
  - `Shift+←`/`Shift+→`: to the previous or next FCD sample, the latest snapshot time
    below `t − 1e-6` or the earliest above `t + 1e-6` (§2.4's epsilon), where the data is
    exact. Past the first or last snapshot `t` does not move.
- Frames are drawn with `boxes_at(t)` (§2.8), the positions `render` draws at `t`.

#### 2.9.2 The view camera

The camera is a centre `(cx, cy)` in world metres and a zoom `k` in metres per logical
pixel, over a window `W × H` logical pixels. A cursor at `(px, py)` (logical, origin top
left, `y` down) is over the world point
`(cx + (px − W/2)·k, cy − (py − H/2)·k)`.
- **Start:** `render`'s fit (Phase 1 "Camera"), at the window's logical size at launch:
  the strips' bounding box plus 20 m, so the whole network shows. Its zoom is `k_fit`.
  `k_fit`, `k_max` and the fitted rectangle are fixed at launch; a resize does not
  change them.
- **Pan:** a left-button drag by `(dx, dy)` logical pixels, measured from the press,
  moves the centre by `(−dx·k, +dy·k)` from where it was at the press, so the world
  point under the cursor stays under it. `W`, `A`, `S`, `D` pan north, west, south and
  east at half the window's width per second of real time, `0.5·W·k` metres per second
  (the same on-screen speed at any zoom), over the same capped `Δ` as the clock (§2.9.1).
- **Zoom:** a scroll of `n` lines sets `k ← clamp(k · 1.1^(−n), k_min, k_max)`, scrolling
  up to zoom in. A trackpad's pixel scroll counts 20 pixels as a line. The world point
  under the cursor stays under it: `c ← p + (c − p)·k′/k`. The limits:
  - `k_min` = 0.02 m per logical pixel, so a 4.5 m car is 225 logical pixels long;
  - `k_max` = 2·`k_fit`, so the network can shrink to half the window and no further.
- **Click or drag:** a press becomes a drag the first frame the cursor is 4 logical
  pixels or more from where it was pressed (net distance, not path length); from then
  until release the centre follows the pan rule above. A press released without
  becoming a drag is a click (§2.9.3), and the camera did not move during it.
- **Centre bound:** the centre is clamped to the fitted rectangle (the strips' bounding
  box plus 20 m), so the network cannot be panned out of sight. Where the clamp applies,
  it wins over keeping the point under the cursor.

`k` and the centre are the whole camera. The window is drawn at its physical resolution,
and Bevy's orthographic projection is set each frame to `W·k × H·k` metres, so a resize
keeps the centre and the zoom and shows more or less around them. The road mesh is baked
relative to the fit's centre `(fx, fy)` (`src/draw.rs:road_mesh`: world `(x, y)` → Bevy
`(x − fx, ·, −(y − fy))`), so the camera sits at `(cx − fx, ·, −(cy − fy))`.

*Note (2026-09-30, Phase 5):* the camera gains a yaw and a pitch, and the projection
becomes perspective (§2.11.3). The formulas above are what §2.11.3's reduce to, to
rounding, at the start pose (yaw 0, pitch 90) (§2.11.1).

#### 2.9.3 Picking and following a vehicle

**A click picks** the vehicle drawn at `t` whose box is nearest the cursor:
- the distance is from the world point under the cursor to the box's footprint, the
  `length × 1.8 m` rectangle at its heading (Phase 1 "Vehicles"): 0 inside it;
- the nearest box within **8 logical pixels**, `8·k` m, is picked. On a tie, including a
  cursor inside two overlapping boxes, the higher `vehicle_id` wins: it is the box drawn
  on top (§2.3);
- a click with no box within 8 pixels changes nothing, including a follow in progress.

**Following** a vehicle `v` sets the centre to `v`'s placed point in every frame where it
is drawn, after `t` has moved, so the box stays at the window's centre.
- Zooming while following zooms about the centre, not the cursor.
- A drag or a `WASD` pan stops following; so does `Esc`. Clicking another vehicle
  follows it instead.
- **When `v` is not drawn** at `t` (before its first row or after its last, §2.8.5), the
  camera holds the last centre it had, and the follow stays armed: stepping or playing
  back into `v`'s drawn interval re-centres on it. The readout says so (§2.9.4).

#### 2.9.4 The readout

One small line of text in the window's top-left corner shows `t` (to 0.01 s, so each
1/30 s step shows), the window `[from, to]`, the speed, playing or paused, and the
vehicle followed, if any, with "(not drawn)" when §2.9.3's hold applies. For example:
`t 64.10 s [9.1–299.1]  ×2  paused  following 103`. It is drawn on the window and never
enters a keyframe or a `render` frame.
*Gate 11 note (2026-09-29, the user's check):* the user followed a vehicle and did not
know how to stop. While following, the readout now adds "— Esc to stop" after
"following <id>", and after "(not drawn)" on hold: `… following 103 — Esc to stop`. The
behaviour is unchanged: `Esc`, a drag and `WASD` stop following, and a click that hits no
box does not.
*Phase 4 (2026-09-30):* the readout moves from the top-left corner to just above the
time slider (§2.10.1). Its text does not change.

#### 2.9.5 The keyframe line

`K` prints the current camera as one line on stdout. It is a TOML inline table, so a later
file can hold the lines verbatim as an array:

```
{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }
{ t = 200.000, follow = 103, height_m = 60.00 }
```

- `t` is the sim time in seconds, 3 decimals.
- `x`, `y` are the centre in world metres (the network's frame, not the camera's), 2
  decimals.
- `follow` (the `vehicle_id`) replaces `x`, `y` while following **and** the vehicle is
  drawn at `t`. The line then means: the centre is that vehicle's placed point at `t`
  (§2.8). While the follow is on hold (§2.9.3) the vehicle has no placed point at `t`, so
  `K` prints the held centre as `x`, `y`: the line is always the camera on screen.
- `height_m` is the zoom as the visible height in metres, `H·k`, 2 decimals. It fixes
  the vertical extent only; the width follows from the aspect of whatever displays it. It
  is the height, not `k`, so it does not depend on the window's size or on the resolution
  a later `render` uses.
- Numbers are written by Rust's `format!("{:.N}")` (the exact binary value, rounded to
  nearest, ties to even). A value that rounds to zero from below prints as `-0.00`, which
  is valid TOML and parses to 0.
- Keys come in exactly this order, separated by `, `, with one space inside each brace.

It is **minimal on purpose**: a time and a 2D camera, nothing about easing, rotation,
tilt or duration. Phase 4 decides the file that holds the lines (for example as the
elements of a TOML array, a comma after each), what a `t` outside its render's window
means, and adds what its scripted moves need as new keys; Phase 3 designs none of that.
`view` also parses its own line, for the round-trip gate: it accepts exactly the two forms
above and rejects anything else.

*Note (2026-09-30, Phase 5):* `K` now prints `yaw_deg` and `pitch_deg` as well, lines are
read as TOML, and the two forms above stay valid unchanged (§2.11.4). The file is
`render --camera` (§2.11.4).

#### 2.9.6 Sharing code with `render`, whose output does not change

`view` reuses reading (§2.1), placement (§2.2), motion (§2.8) and drawing; only the loop
around them differs. The Phase 3 scope (§4) names the modules.
- **Loading and drawing are moved, not rewritten,** so `render`'s frames cannot change.
  `tests/gates.rs` compiles unedited: every public path it names stays where it is or is
  re-exported there (`Job`, its fields and methods, `RenderOptions`, `inputs`,
  `fcd::{self, Row}`, `place::Placement`, `motion::{How, Piece}`, `render::VehicleBox`).
- **`render` stays headless.** With the windowing feature on, `DefaultPlugins` includes
  `WinitPlugin`, whose `build` creates the OS event loop: on macOS that panics off the
  main thread, and libtest runs every test on a thread of its own, even with
  `--test-threads=1`. A second event loop in one process also panics. So `render`
  disables `WinitPlugin` and creates no window (bevy_winit 0.19.1, `WinitPlugin::build`).
- **The input → state logic is separate from the window.** The camera, the clock, picking,
  following and the keyframe line are plain Rust with no Bevy types: a frame's input goes
  in, and the new state comes out. The windowed app only translates Bevy's input into that
  struct and the state into a camera transform, a projection, the boxes and the readout.
  So tests drive the state with scripted input on the fixture, with no window and no GPU:
  the boxes come from the loaded run's `boxes_at(t)`, which needs no `Renderer`.

#### 2.9.7 Build cost

Windowing adds Bevy's `bevy_winit` feature (it brings `bevy_window`, which the build
already has). The readout's text adds `bevy_text`, `default_font`, `bevy_ui` and
`bevy_ui_render`. Measured on 2026-09-29 before building (clean builds, sccache off):

| Bevy features added | crates compiled | release wall | release CPU (user+sys) | `target/` release, then + debug |
|---|---|---|---|---|
| none (Phase 2 as shipped) | 284 | 311, 329, 289 s | 1 654, 1 889 s | 1.2 GB, 4.7 GB |
| `bevy_winit` | 307 | 310, 306 s | 2 060 s | 1.3 GB, 5.0 GB |
| + `bevy_text`, `default_font`, `bevy_ui`, `bevy_ui_render` | 360 | 605, 448 s | 2 740 s | 1.6 GB, 6.3 GB |

- The window alone costs almost nothing; the readout's text (font shaping and the UI
  renderer) is most of the cost. Text wall times are not clean: other sessions were
  compiling (load average 10–50), and the two runs differ by 157 s where the base's three
  spread 40 s. Text is kept (§2.9.8, decision 1); the alternative was the window's title
  bar, which costs nothing.
- **What `target/` becomes.** The working `target/` is 5.6 GB (2026-09-29). A feature
  change rebuilds most Bevy crates under new hashes and cargo keeps the old ones, so
  without a clean it grows to about 12 GB (5.6 + 6.3). The build runs `cargo clean` once
  after the `Cargo.toml` change, which brings it to about 6.3 GB plus incremental growth.
  Gate 1's reference build adds about 1.2 GB in `scratch/`, deleted after the gate. With
  about 100 GB free, none of this is gated.
- Measured on a throwaway copy with its own `target/`, deleted afterwards; the method is
  in `specs/reviews/vis-001.md`.

`x11` and `wayland` are Linux-only winit back ends and compile nothing here; a Linux
build needs one of them, which is roadmap item 6's Linux build check.

#### 2.9.8 Four calls on the draft (decision, recorded)

Decided by the user, 2026-09-29, on the Phase 3 draft:
1. **The readout is on-screen text**, at §2.9.7's cost, because roadmap item 5's HUD
   (clock, legend, chart) needs the same Bevy features in `render` anyway.
2. **The frame rate is recorded, not gated.** The predicted 16.67 ms median is the
   display's vsync, so a gate would measure the display, not `view`; and load averages
   of 50–75 while other sessions compile would make it flaky. `--bench` stays to record it.
3. **Steps** are 1/30 s, and one FCD sample with Shift (§2.9.1), as drafted.
4. **Follow** holds and stays armed when the vehicle is not drawn (§2.9.3), as drafted.

### 2.10 The time slider (Phase 4)

A thin bar along the bottom of `view`'s window spans the run's window `[from, to]`, with a
handle at `t`. A click anywhere on it sets `t` there; dragging scrubs `t` continuously.
It answers OQ-9. It is in `view` only: the keys, the keyframe line (§2.9.5) and `render`
do not change. Agreed with the user on 2026-09-30.

#### 2.10.1 Geometry

In logical pixels (§2.9.2's units: origin top left, `y` down), in a window `W × H`:

| Part | Where |
|---|---|
| track | from `x0 = 16` to `x1 = W − 16`, so `L = W − 32`; 4 px tall, centred on `y_bar = H − 14` |
| handle | 12 × 12 px, centred on `(x(t), y_bar)`; at either end it overhangs the track by 6 px, inside the 16 px inset |
| ticks | 1 × 10 px, centred on `y_bar`, drawn under the handle (§2.10.4) |
| hit area | `x ∈ [x0 − 8, x1 + 8]`, `y ∈ [H − 28, H]`: the bottom 28 px, 8 px wider than the track at each end |
| readout | its bottom edge 30 px above the window's bottom, left edge at `x0`: just above the hit area (moved from §2.9.4's top-left corner) |

At the default 1280×720: `x0 = 16`, `x1 = 1264`, `L = 1248`, `y_bar = 706`, the hit area
`[8, 1272] × [692, 720]`, and one pixel of track is `290/1248` ≈ 0.232 s of the fixture.
- **Resize.** The geometry is a function of the window's current logical size, recomputed
  every frame from `ViewInput::size`. `t` does not change; the handle is redrawn at `x(t)`
  in the new geometry. A scrub in progress maps the cursor through the new geometry.
- **Too small.** With `W < 64` or `H < 64` logical pixels there is no bar: nothing is
  drawn, no hit area exists, and a scrub in progress ends as on release (§2.10.3), with
  `t` held where the last scrub frame left it. The readout keeps its place.
- **Over the map.** The bar is drawn over the scene, and the fit (§2.9.2) does not change
  for it, so Phase 3's `k_fit` and gates stand. At the launch fit the hit area covers
  world `y` below about −272 m in the fixture, the south ends of its southern entry links;
  a box there is picked after a pan or zoom.

#### 2.10.2 x ↔ t

With `u` the fraction along the track:
- `x(t) = (1 − u)·x0 + u·x1`, with `u = (t − from)/(to − from)`;
- `t(x) = (1 − u)·from + u·to`, with `u = clamp((x − x0)/(x1 − x0), 0, 1)`.

Both are written as a two-sided blend, not `from + u·(to − from)`, so each end is exact in
floating point: `u = 0` gives `from` (or `x0`) and `u = 1` gives `to` (or `x1`) bit for
bit, whatever rounding `to − from` carries. `to > from` always (`src/run.rs:load`
rejects an empty window), so `u` is defined. The clamp is the only clamping: a cursor
left of `x0` gives `from`, and right of `x1` gives `to`. There is no snapping to samples,
frames or ticks.

#### 2.10.3 Pressing, scrubbing, releasing

- **The bar owns its input.** A left press with the cursor in the hit area starts a
  **scrub**, and that press never becomes a pan (§2.9.2) or a click (§2.9.3): no
  `Press` is made for it. A press anywhere else is Phase 3's, even if it later drags over
  the bar. Scroll with the cursor in the hit area, or during a scrub, does nothing; it
  zooms everywhere else as before.
- **Scrubbing.** From the press frame until release, every frame sets `t ← t(x)` with the
  cursor's `x`. So a click on the bar (press and release in one frame) sets `t` there,
  and a drag from anywhere on the bar, the handle included, scrubs. The cursor's `x`
  decides alone; `y` may leave the bar.
- **Off the window.** Bevy's `Window::cursor_position` is `None` outside the window, so
  a fast drag past an end could stop short of it. While the button is held, winit on macOS
  keeps sending `CursorMoved` outside the view (winit 0.30.13,
  `src/platform_impl/macos/view.rs`, `mouse_motion`), and Bevy passes its position on
  without bounding it (bevy_winit 0.19.1, `WindowEvent::CursorMoved` in `state.rs`). So
  `ViewInput` gains `pointer`, the last `CursorMoved` position of the frame, unbounded,
  and a scrub reads `pointer`, else `cursor`, else holds `t`. Phase 3's pan keeps using
  `cursor`.
- **Playback.** The press remembers whether the clock was playing and pauses it. While
  scrubbing, `t` comes from the cursor only: space, `←` and `→` are ignored; `+` and `−`
  still change the speed. On release, playing resumes if it was playing and `t < to`;
  at `to` it stays paused, as reaching `to` does (§2.9.1).
- **Follow.** A scrub does not stop a follow. Each frame it re-centres on the vehicle
  when it is drawn at the new `t`, and holds the last centre when it is not, with the
  follow still armed: §2.9.3's rule for stepping, unchanged. The readout's "(not drawn)"
  follows it.

#### 2.10.4 Ticks

A tick marks each whole minute of sim time in the window: `t = m·P` for integer `m`, with
`from ≤ t ≤ to`, at `x(t)`, and `P` = 60 s. When minute ticks would be closer than 8 px
(`P·L/(to − from) < 8`), `P` is the first of 5, 10, 15, 30 and 60 min, then 2, 3, 6, 12
and 24 h, that spaces them 8 px or more; if none does, there are no ticks. `P` is
recomputed with the geometry, so a resize can change it. The ticks are unlabelled.

For the fixture at 1280×720 (`from` = 9.099999999999984, `to` = 299.0999999999995): `P` =
60 s, 258.2 px apart, four ticks at 60, 120, 180 and 240 s. A 1 h run at the same size
keeps 60 s (20.8 px), a 3 h run gets 5 min (34.7 px), and a 24 h run 10 min (8.7 px).

#### 2.10.5 Where it lives, and the frame order

As §2.9.6 set for the rest of `view`, the slider's logic is plain Rust with no Bevy
types, so tests drive it headless; `src/view/mod.rs` only draws it. The modules are named
in Phase 4's scope. `ViewState::frame` applies one frame in this order:
1. the window size, and from it the bar;
2. **bar press:** a press in the hit area starts a scrub and pauses;
3. clock (§2.9.1), with space, `←` and `→` ignored while scrubbing;
4. **scrub:** `t ← t(x)`; on release (or when the bar disappears) the scrub ends and
   playing resumes per §2.10.3;
5. `Esc`; 6. pan and zoom, making no `Press` for a bar press and ignoring scroll per
   §2.10.3; 7. click; 8. follow, at the new `t`; 9. `K`.

Steps 5–9 are Phase 3's order (§2.9, Phase 3 scope). Scrubbing after the clock and before
the follow is what makes the follow re-centre at the scrubbed `t` in the same frame, and
makes a `K` in that frame print it.

#### 2.10.6 Build cost: none

The bar is `bevy_ui` nodes (`Node`, `BackgroundColor`), and `bevy_ui` and
`bevy_ui_render` are already on for the readout (§2.9.7). No feature, crate or
dependency is added (`bevy_egui`, OQ-9's example, is not used), the release build stays
at 360 crates, and nothing is measured beforehand.

#### 2.10.7 Not in Phase 4

- **A hover readout** (the time under the cursor, shown before a click): deferred. It
  needs a second text node and a rule for where it sits; the tick marks and the readout
  above the bar carry the time for now.
- Tick labels, keyboard focus on the bar, and any slider in `render`'s video (the HUD is
  roadmap item 5).

#### 2.10.8 The user's calls on the draft (decision, recorded)

Decided by the user, 2026-09-30, on the Phase 4 draft, before review round 1:
1. **A press on the handle jumps `t` to the cursor**, like a press anywhere on the bar:
   there is no grab offset (§2.10.3).
2. **Space, `←` and `→` are ignored** while the button is held on the bar (§2.10.3).
3. **Scroll over the bar or during a scrub is ignored** (§2.10.3).
4. **Ticks thin out on long runs**: 1, 5, 10, 15, 30 and 60 min, then 2–24 h (§2.10.4), as
   drafted.
5. **The launch fit is not shrunk for the bar**, which may cover the south edge of the map
   (§2.10.1).
6. **The close-out adds `rules/slider.md`** rather than raising `rules/view.md`'s cap.

Decided by the user, 2026-09-30, after review round 1 (finding F6), not a scope change:

7. **A window made too small mid-drag ends the drag**: the bar disappears, `t` holds where
   the last scrub frame left it, and playing resumes if it was playing (§2.10.1, gate 8).

### 2.11 The 3D camera (Phase 5)

Key cameras in the style of VISSIM and Aimsun. `view` gains a camera that orbits and tilts
over the run, and `render` gains a camera that flies through keyframes written from
`view`'s `K` lines. It comes before the city (roadmap item 4), so the camera is built
once, and it works on today's roads and boxes. The user's calls that shape it are
§2.11.8; everything else here is the draft's proposal.

#### 2.11.1 The pose and the projection

A camera **pose** is five numbers:
- `(cx, cy)`, the **look-at point**, on the ground (`z` = 0) in world metres;
- `height_m`, the world height visible at the look-at point, as in Phase 3 (§2.9.5);
- `yaw_deg`, the compass direction the camera faces, 0 = north, clockwise (§2.2's heading
  convention). It is the direction at the top of the image; 0 is north up;
- `pitch_deg`, the angle between the view axis and the ground: 90 is straight down, and
  smaller values tilt toward the horizon. The range is **[25, 90]**.

The projection is perspective with a **vertical field of view `φ` = 45°** (Bevy's
default, a named constant). The camera sits on the view axis at distance
`d = height_m / (2·tan(φ/2))` = 1.2071067811865475 × `height_m` from the look-at point.
In world coordinates (`x` east, `y` north, `z` up), with `s`/`c` the sine and cosine:
- view axis `a = (s(yaw)·c(pitch), c(yaw)·c(pitch), −s(pitch))`; image up
  `u = (s(yaw)·s(pitch), c(yaw)·s(pitch), c(pitch))`; image right `r = (c(yaw), −s(yaw), 0)`;
- eye `E = (cx, cy, 0) − d·a`;
- a point `X` is at `(x_c, y_c, z_c) = ((X−E)·r, (X−E)·u, (X−E)·a)`, and at pixel
  `(W/2 + f·x_c/z_c, H/2 − f·y_c/z_c)` in a `W × H` image (origin top left, `y` down),
  `f = (H/2)/tan(φ/2)`.

What follows from it:
- **`height_m` keeps its Phase 3 meaning, exactly.** At `pitch_deg` = 90 the ground is the
  plane at distance `d`, perpendicular to the axis, so a perspective camera shows the same
  ground rectangle as Phase 3's orthographic one: `height_m` tall and `height_m·W/H` wide.
  Only the boxes differ, since a box top (1.55 m) is nearer the camera than the road: at
  the launch fit (`d` = 1 497 m) it is 0.1 % larger, and at `height_m` = 60 m 2.2 %. So a
  Phase 3 line frames the same area it did.
- **Degrees have exact multiples of 90°.** Sines and cosines are taken of degrees by one
  helper that returns 0, ±1 exactly at multiples of 90°. So at the default pose (yaw 0,
  pitch 90) every formula in §2.11.3 reduces to Phase 3's **to rounding**, not bit for
  bit: `d·(px − W/2)/f` is not `(px − W/2)·k` in floating point, and a direct
  implementation differs from Phase 3's `world` in the last bit (up to 4.5e-13 m at the
  launch fit). Every gate that compares the two does so to 1e-9, and no special case for
  pitch 90 is added. What *is* exact at yaw 0 or 90 is anything that multiplies by the
  helper's 0 or 1: an axis a move does not touch stays bit-exact (Phase 5 gate 11's `W`).
- **Every pixel sees the ground.** The floor of 25° is above `φ/2` = 22.5°, so the top
  edge of the image looks at least 2.5° below the horizon and the horizon never shows. A
  cursor always has a ground point, which pan, zoom and pick need (§2.11.3). Lower shots
  would show a horizon over empty background; they wait for the city (OQ-12).
- **The far plane** is `20·d`. Bevy's default `far` of 1 000 m would cull the whole
  network at the launch fit (`d` = 1 497 m). The farthest ground any pixel sees is along
  the top edge at 25° pitch, `d·sin 25°/sin 2.5°` = 9.69·`d` away, so `20·d` keeps all of
  it with a margin of two. Bevy's perspective is reverse-Z with an infinite far plane, so
  `far` only culls; `near` stays at Bevy's 0.1 m.

The pose is plain Rust with no Bevy types (`src/camera.rs`): `project(pose, W, H, X)` and
its inverse onto a horizontal plane, `ray_to_plane(pose, W, H, pixel, z)`. The Bevy camera
is built from the pose: a `Transform` at `E` looking at the look-at point with up `u`, and
a `PerspectiveProjection` of `φ` and `far`, in the scene frame baked relative to the fit's
centre (§2.9.2: world `(x, y, z)` → Bevy `(x − fx, z, −(y − fy))`).

#### 2.11.2 What each path draws

- **`render` with no `--camera` is Phase 4's path, unchanged:** orthographic, straight
  down, the fit of Phase 1. Its frames stay byte-identical (§2.11.8 c). It is the only
  orthographic path left.
- **`view`, and `render` with `--camera`, are perspective,** at every pose (§2.11.8 b).
- **The rank lift** (§2.3) moves pixels under perspective, and 0.01 m per rank would float
  urban_grid's 110th box 1.09 m over the road at a tilt. The perspective path lifts
  **0.001 m per rank** (0.109 m at 110). Near the look-at point at the launch fit, one
  step of Bevy's reverse-Z `Depth32Float` is about 0.16 mm, so 0.001 m is about 6 steps;
  it coarsens with distance, to about 1.6 mm at the top edge at the 25° floor, where two
  overlapping boxes one rank apart may tie. A tie is still resolved the same way on every
  run, so determinism does not rest on the lift. The orthographic path keeps 0.01 m. A
  keyframed render is gated for determinism the same way (Phase 5 gate 10).
- **Box faces are shaded** in the perspective path, still unlit: the top keeps today's
  speed colour, and the sides are fixed fractions of it, as vertex colours on the box mesh,
  so a tilted box reads as a solid. The fractions are iteration (§2.6). The orthographic
  path keeps its mesh.
- Roads, colours, MSAA and everything else in §2.3 and Phase 1 "Scene" are unchanged.

#### 2.11.3 Orbit and tilt in `view`

`view` starts at the default pose: Phase 3's launch fit, yaw 0, pitch 90. The state gains
`yaw_deg` and `pitch_deg`; `(cx, cy)` and `k` keep their Phase 3 meaning (§2.9.2), with
`height_m = H·k`.

**Right-drag orbits and tilts** (§2.11.8 a). A right press starts an **orbit**, with no
4 px threshold: nothing else uses the right button. Until release, each frame sets
- `yaw_deg ← yaw_press − 0.25·dx`, wrapped to [0, 360) (`rem_euclid(360)`, and a result
  of 360 from a tiny negative taken as 0; every wrap in Phase 5 is this one): dragging
  right turns the scene with the cursor, as if it were grabbed;
- `pitch_deg ← clamp(pitch_press + 0.25·dy, 25, 90)`: dragging up tilts toward the
  horizon, dragging down back toward straight down,

where `(dx, dy)` is the cursor from the press in logical pixels, read from `pointer`, else
`cursor`, else held, as a scrub reads it (§2.10.3). 0.25° per pixel is 320° across the
default window. The look-at point and `k` do not move: the camera orbits about the
look-at point. A right press in the slider's hit area starts nothing.

**Ctrl + left-drag is the same orbit** (§2.11.8 k), for a trackpad. A left press with
Control held is an **orbit press**, and from then on it is a right press in every respect:
- it orbits and tilts by the rules above until the left button is released; releasing
  Control mid-drag does not end it, and pressing Control during a left-drag that has
  already begun (a pan, or a press still under 4 px) does not turn it into an orbit. The
  mode is fixed at the press;
- it never pans, picks or scrubs, and it makes no Phase 3 `Press`. A press and release over
  a box picks nothing and leaves a follow as it was; an orbit does not stop a follow;
- in the slider's hit area it starts nothing, as a right press does there: no scrub, no
  orbit, `t` and the clock untouched. The bar owns a plain left press only (§2.10.3);
- while an orbit is on, a second press of either kind starts nothing.

Control is read as held (`KeyCode::ControlLeft` or `ControlRight`) in the press's frame.
On macOS a Control-click arrives as **Left + Ctrl**: winit 0.30.13 and bevy_winit 0.19.1
pass the button through untranslated, and winit's view has no `menuForEvent:` (the source
walk is in `specs/reviews/vis-001.md`, "Phase 5 draft — the user's calls"). The one step
those sources cannot show is AppKit's; were it re-sent as `Right`, right-drag already
orbits, so nothing mis-fires either way. Gate 13 confirms it on the user's trackpad.

**Keys, in fixed steps.** Matched by position, like `WASD`, so they sit next to `WASD` on
any layout; one step per press, held keys do not repeat:
- `Q` and `E` turn the yaw by −15° and +15°, wrapped;
- `R` tilts 5° toward the horizon, `F` 5° toward straight down, clamped to [25, 90]. From
  90, 13 presses of `R` reach 25 exactly.

They are ignored during an orbit, a left drag (a pan, or a press still under 4 px) and a
scrub, so a pose never changes under a pan's anchor; for the same reason a right press
during a left drag or a scrub starts nothing. `Q`, `E`, `R` and `F` are unbound today
(§2.9.1, §2.9.2, §2.9.3, §2.9.5).

**What stays, generalised to the pose** (§2.11.8 a). Each is Phase 3's rule with the
cursor's point taken from `ray_to_plane`, and each is Phase 3's formula to rounding at
the default pose (§2.11.1):
- **The cursor's ground point** is `ray_to_plane(pose, W, H, cursor, 0)`, and
  `ViewState::world` returns it. For a fixed yaw and pitch it is the look-at point plus
  `k` times a function of the pixel, so pan and zoom keep their Phase 3 shape.
- **Pan** (left-drag): the ground point under the cursor at the press stays under it:
  `c ← c_press + g(press) − g(cursor)`, `g` the cursor's ground point relative to the
  look-at point. `W`/`S` pan along the image's up direction on the ground,
  `(s(yaw), c(yaw))`, and `A`/`D` along its right, `(c(yaw), −s(yaw))`, at Phase 3's
  `0.5·W·k` m/s.
- **Zoom** (scroll) sets `k` as before and keeps the cursor's ground point fixed with
  Phase 3's `c ← p + (c − p)·k′/k`. This is exact at any pose, since `g` scales with `k`
  (in arithmetic; in floating point to rounding).
  The limits `k_min`, `k_max` and the centre bound are unchanged.
- **Pick** (§2.9.3) casts the cursor's ray to the plane at the boxes' mid-height,
  `z` = 0.80 m (`BOX_HEIGHT/2` + the 0.05 m base lift), and calls Phase 3's `pick` on
  that point, radius `8·k`. At a tilt, a click on a box's roof then lands inside its
  footprint where the ground point would miss it (Phase 5 gate 11). Straight down it
  moves the click point toward the image centre by `0.8·r/d` for a point `r` metres out:
  0.68 m at most at the launch fit (the image corner, `d` = 1 497 m), against a pick
  radius of `8·k_fit` = 13.8 m. For the shipped gates' clicks (Phase 5 gate 2): 0.23 m for
  Phase 3 gate 7's click on vehicle 1, 423 m from the fit's centre (300, 300); 0 for its
  centred click at `k_min`, and 7.36 m for its far click there, 160 m out, which must
  still pick nothing; and 0.31 m for Phase 4 gate 7's click at (640, 691) on the box at
  (0, −331), which lands at `y` = −330.69, inside its ±0.9 m footprint.
- **Follow** keeps the pose's yaw, pitch and `k` and moves the look-at point with the
  vehicle (§2.11.8 g). An orbit or a key step does not stop a follow; a drag, `WASD` and
  `Esc` still do.
- The slider, the clock and the readout do not change. The readout does not show the
  pose; `K` does (§2.11.4).

**Frame order** (§2.10.5, one step added): size and bar; bar press; clock; scrub; `Esc`;
**orbit and the `Q E R F` steps**; pan and zoom; click; follow; `K`. The pose is set
before pan and zoom, which read it.

`ViewInput` gains the right button's press and release, Control held, and the four keys;
it still derives `Default`, so every existing script leaves them off. In the frame order,
the bar press (step 2) is a left press without Control; an orbit press starts in the
orbit step.

#### 2.11.4 The keyframe line and the keyframe file

**`K` prints the full camera** (§2.11.8 f): `yaw_deg` and `pitch_deg` are always printed,
2 decimals, after `height_m`. A yaw that would print as `360.00` prints `0.00`, so a
printed yaw is always in [0, 360) and reads back to itself:

```
{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }
{ t = 200.000, follow = 103, height_m = 60.00, yaw_deg = 90.00, pitch_deg = 45.00 }
```

Everything else is §2.9.5: when `follow` is printed, the format of each number, the key
order. **Reading** is TOML's, so it is not Phase 3's exact-form parser:
- `yaw_deg` and `pitch_deg` are optional. Absent, the camera is straight down, north up
  (0 and 90), so **every Phase 3 line stays valid unchanged** and means what it meant.
- Key order and spacing are TOML's to decide, so any order is read. `K` still prints the
  one order.
- `Keyframe::parse(line)` reads one line as the value of one key (`k = <line>`) with the
  same reader as the file, and applies the checks below that need no run; so a trailing
  comma or an empty line is still an error. `format(parse(line)) == line` still holds for
  every line `K` prints.

**The file** is given to `render` as `--camera <file.toml>`: one key, `keyframes`, an
array of the lines above, a comma after each:

```toml
keyframes = [
  { t = 20.000, x = 0.00, y = 300.00, height_m = 1240.00 },
  { t = 64.100, follow = 1, height_m = 120.00, yaw_deg = 90.00, pitch_deg = 45.00 },
]
```

TOML's `[[keyframes]]` tables are the same data and are read too. `scene.toml` stays
roadmap item 6's (§2.7); Phase 5 designs no other key.

**Errors.** The file is read and checked after the run is loaded (a `follow` needs the
vehicles) and before the first frame. Each error is one `error: --camera <file>: …` line,
exit 1, no progress line and no output file (§2.4). A keyframe is named by its index
from 1 and its `t`. The errors:
- the file is missing or is not TOML;
- `keyframes` is missing or empty, or the file has another top-level key;
- a keyframe has an unknown key (`deny_unknown_fields`), lacks `t` or `height_m`, has both
  `x`/`y` and `follow`, has neither, or has `x` without `y` (or `y` without `x`);
- a value is not a number, or not finite (TOML allows `inf` and `nan`), or `follow` is
  not an integer ≥ 0;
- `height_m` ≤ 0, or `pitch_deg` outside [25, 90]. `yaw_deg` is any finite number,
  taken modulo 360;
- `t` is not strictly increasing: **out of order** and **duplicate `t`** are both errors,
  named as such. The file is never sorted for the user;
- a `follow` whose vehicle is not drawn at the keyframe's `t` (§2.8.5), including one that
  is not in the FCD at all. `K` never prints such a line (§2.9.5).

What is **not** an error:
- **One keyframe:** the camera holds that pose for the whole video (or follows).
- **Keyframes outside `[from, to]`:** the path is built from every keyframe, and the window
  only picks the frames, as §2.8 does with FCD rows. So one file serves a full render and a
  close-up of part of it. Before the first keyframe the camera holds the first pose, and
  after the last it holds the last (following, if that keyframe follows).
- `--camera` does not change the window, the frame count or any other flag's default.
  Whether the window should default to the keyframes' span is OQ-10.

#### 2.11.5 The flight through the keyframes

The camera passes through every keyframe at its `t`, on a smooth curve with no corner at a
keyframe, with eased speed (§2.11.8 d), and it holds exactly through a hold (§2.11.8 e).
A **hold** is two consecutive keyframes with the same camera. There is no hold field.

Each channel is interpolated in time on its own, from every keyframe:
- **Scalars:** `ln(height_m)`, `yaw_deg` and `pitch_deg` each get a **monotone cubic
  Hermite** in time (PCHIP). At an interior keyframe `j`, with interval lengths
  `h₀ = t_j − t_{j−1}`, `h₁ = t_{j+1} − t_j` and secants `δ₀`, `δ₁`, the slope is the
  weighted harmonic mean `(w₁ + w₂)/(w₁/δ₀ + w₂/δ₁)`, `w₁ = 2h₁ + h₀`, `w₂ = h₁ + 2h₀`
  (Fritsch–Butland, as SciPy's `PchipInterpolator`); 0 where the secants differ in sign
  or either is 0; 0 at the first and last keyframe. Height is interpolated in log
  space, so a zoom runs at a steady rate.
  Yaw is unwrapped first: each keyframe's yaw is the previous one plus the **short way
  round**, `Δ = 180 − ((180 − (y₁ − y₀)) mod 360)` in (−180°, 180°], so exactly 180° apart
  turns clockwise. The result is taken modulo 360.
- **Position** (the look-at point, keyframes that do not `follow`): a **centripetal
  Catmull–Rom** curve (α = 0.5, Barry–Goldman form) through each **run** of consecutive
  keyframes whose positions differ, with knots `τ_{j+1} = τ_j + |P_{j+1} − P_j|^½`. At the
  ends of a run the missing neighbour is the reflection, `2·P₀ − P₁`, with the same knot
  step. Time maps to `τ` by the same PCHIP, through `(t_j, τ_j)`, with slope 0 at the run's
  ends. Centripetal Catmull–Rom has no cusp or loop within a segment; `τ` never decreases,
  so the camera never backs up along the curve.

What that gives, and why:
- **No corner.** Inside a run the curve is tangent-continuous and `τ` has a positive slope
  at every interior keyframe, so the velocity is continuous and not zero there. A scalar
  channel is continuous in value and slope.
- **Eased speed.** Every channel starts and ends at rest: at the first and last keyframe, at
  the ends of each run, and at each hold. A move between two keyframes alone is the
  smoothstep, `3u² − 2u³`.
- **No overshoot, no drift in a hold.** PCHIP keeps each scalar between its two keyframes'
  values on every segment. A segment whose two keyframes are equal in a channel is
  **constant, and returns the keyframe's own value**, not an evaluated cubic (whose basis
  functions do not sum to exactly 1 in floating point) and not `exp(ln(h))`. At a
  keyframe's own `t` every channel is that keyframe's value. So a hold is bit-exact, and a
  partial hold (same place, new height) holds that channel only.
- **The curve is not the polyline.** Rounding a keyframe means leaving the straight lines
  between keyframes: on Phase 5 gate 6's 100 m square it swings 7.4 m outside them. It
  stays tangent-continuous and does not pass the end of a run.

**Follow** (§2.11.8 g). A `follow` keyframe's position is its vehicle's placed point at
`t`, the point `view` centres on (§2.9.3), which moves:
- between two keyframes that follow the same vehicle, the look-at point is that vehicle's
  placed point, exactly. The drawn interval is one interval (§2.8.5) and both ends are
  drawn (§2.11.4), so the vehicle is drawn throughout;
- on a segment where either end follows (and not both the same vehicle), the look-at point
  is `(1 − w)·P_a(t) + w·P_b(t)`, `w` the smoothstep of `(t − t_a)/(t_b − t_a)`, with
  `P` the keyframe's fixed point or its vehicle's placed point at `t`. At each end the
  velocity is that end's own (the vehicle's, or 0), so there is no corner. A fixed
  keyframe next to such a segment ends its Catmull–Rom run, at rest;
- **a followed vehicle that is not drawn** during a blend (it arrives after, or leaves
  before, the other end), or before a first or after a last keyframe that follows it,
  contributes the placed point of the nearest end of its drawn interval: it is held where
  it was last seen, as `view` holds (§2.9.3). Mid-blend that stops its motion abruptly,
  so the look-at point's velocity jumps by `(1 − w)` times the vehicle's (in
  `tests/flight.toml`, at 147.1 s, `w` ≈ 0.29). It is not at a keyframe, so §2.11.8 d
  holds; gate 13 is where it would show;
- height, yaw and pitch come from their own channels, as for any keyframe.

The path is built once, before the first frame (`src/keyframes.rs`), and `pose_at(t)`
answers any frame's pose from it and the loaded run's placed points.

#### 2.11.6 Build cost

- **Bevy: no feature added.** `PerspectiveProjection` is in `bevy_camera` and vertex colours
  in `bevy_pbr`, both already built.
- **Reading TOML** adds the `toml` crate (1.x, default features off, with `std`, `parse` and
  `serde`) and `serde`'s `derive`. `Cargo.lock` gains **2 packages**, `toml` and
  `serde_spanned`; `toml_parser`, `toml_datetime`, `winnow`, `serde_core`, `serde` and
  `serde_derive` are already in it (at `67c73e1`). Some of those are built today only as
  host dependencies of `proc-macro-crate`, so the release build may compile a few twice:
  **362–365 crates** against Phase 3's 360.
- Rejected: a hand parser for `K`'s own format, which adds no crate but reads only that
  format. The file is edited by hand (a comma after each line, a changed number), so it
  should read as TOML does.

#### 2.11.7 Not in Phase 5

- A horizon, sky or lower shots than 25° (OQ-12); lighting and shadows; any change to
  §2.6's look beyond the face shading.
- A chase camera (§2.11.8 g), roll, a per-keyframe field of view, and easing or duration
  fields in the file.
- Previewing a keyframe file in `view` (OQ-13); the pose in the readout; a compass.
- `scene.toml`, `check` and harness-facing documentation of the file (roadmap item 6).

#### 2.11.8 The user's calls (decision, recorded)

Decided by the user, 2026-09-30, before drafting:
- (a) **`view`:** right-drag orbits (sideways, yaw) and tilts (up and down, pitch); keys do
  the same in fixed steps. Left-drag pan, scroll zoom, the slider, pick and follow, and
  `K` all stay.
- (b) **Perspective always**, with no orthographic toggle, in `view` and in keyframed
  renders.
- (c) **`render` with no keyframe file stays byte-identical:** 8700 of 8700 against
  `scratch/ref-8eb9052.framemd5`. Today's orthographic top-down path is kept as the default
  for that case only.
- (d) **`render` with a keyframe file** flies along a smooth curve through all keyframes,
  with no corner at a keyframe, and with eased speed.
- (e) **A hold is two keyframes with the same camera** at different times; there is no hold
  field. The curve must not overshoot or drift during a hold, and that is gated.
- (f) **The keyframe file** is TOML of the same inline tables `K` prints, with new optional
  `yaw_deg` and `pitch_deg`. When they are absent the camera is straight down, north up, so
  every Phase 3 line stays valid unchanged. `K` prints the full camera.
- (g) **Following vehicle N** keeps the keyframe's yaw, pitch and height and moves with the
  vehicle. No chase camera.

The keys (`Q E R F`), the rates (0.25°/px, 15°, 5°), `φ` = 45°, the 25° floor, the curve,
the blends, the pick plane and the face shading were the draft's proposals.

Decided by the user, 2026-09-30, on the Phase 5 draft, before review round 1:
- (h) **The numbers are accepted as drafted:** `Q`/`E` ±15°, `R`/`F` 5°, right-drag
  0.25°/px, `φ` = 45° vertical, the 25° pitch floor. Tuning them after gate 13 is iteration
  (§2.6), not a spec change.
- (i) **Box-side shading stays in Phase 5** (§2.11.2): it is needed to judge tilted views.
- (j) **Keyframes are read with the `toml` crate** (+2 packages, §2.11.6), as drafted.
- (k) **OQ-11 answered: a second orbit binding.** The user orbits on a MacBook trackpad, so
  Ctrl + left-drag orbits and tilts exactly as right-drag does (§2.11.3). The sources show
  it arrives as Left + Ctrl, so that is the binding; Option + left-drag was the fallback
  and is not used.

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
  evidence: roadmap Phase 6)* *Note (2026-09-29):* Phase 2 does not change this. Its
  fixture runs the engine CLI with FCD on, as Phase 1's does, and the `view` command
  (§2.7 item 3) needs FCD in the same way.
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
- ~~**OQ-7** — Can the engine make its turn-path walk public?
  `crates/geometry/src/frame_collect.rs:interpolate_turn_path` is a private `fn` at
  `df8aec0`, so Phase 2 copies it, about 50 lines (§2.8.3). A `pub` on that function, or a
  public method on `LaneTurnPath` that does the same, would let the copy go. This is an
  engine request (asm-001 §10 owns that side), not done here. It does not block Phase 2.
  When the pin moves past the change, the copy is deleted. *(needs-input: engine;
  non-blocking)* *Note (2026-09-29, review round 1):* the position half is already
  public as `crates/core/src/spatial_conflict.rs:interpolate_pos`, and the heading half
  as `interpolate_heading` in another convention (§2.8.3 note). Whether the copy is
  still wanted is the user's call.~~ **WITHDRAWN 2026-09-29 (user): not needed.** The
  walk is already public in `spatial_conflict`, and Phase 2 reuses it (§2.8.3). No
  request goes to the engine.
- ~~**OQ-8** — Is the gap between `G` and `I` (§2.8.2) the frozen `position`'s shortfall
  from the link end, a constant to take out before scaling? In the fixture, `G − I`
  minus `(L_a − p_f)` is −1.2 to +1.4 m, median +0.5 m, so the half-length explains most
  of the gap but not all of it. Taking it out would bring `r` nearer 1, but it needs to
  know whether the engine's `JunctionTransit::s_on_path` measures the centre or the front.
  Phase 2 scales and records `r`, which is correct either way; this question only
  refines it. *(answerable-from-code: engine
  `crates/core/src/systems/junction_transit.rs`; non-blocking, a Phase 2 review item)*~~
  **ANSWERED 2026-09-29 (review round 1, engine `df8aec0`): the centre, and the
  shortfall is the engine's own jump.** Kinematics clamps an approaching centre at
  `L_a − length/2`; `run_junction_transit_execution` enters at `≥ L_a − length/2 − 0.1`
  and starts `s_on_path` at `speed·dt` from the path's start. `s_on_path` is a centre:
  the rear is `s_on_path − half_length`, the GUI draws the centre there, and on exit
  `Position.s = excess`. So `L_a − p_f` is never driven. The box still covers it and
  `r = G/I` is unchanged, but the band test moves to `r′` (§2.8.2), since otherwise it
  measures vehicle length. The `r` predictions do not change; `r′` was not measured,
  and gate 8 bounds it from recorded numbers.
- ~~**OQ-9** — Does `view` need a time slider (a bar to drag through the run, for example
  with `bevy_egui`)? Phase 3 has keyboard time only: play, speed, and steps by frame or
  by sample (§2.9.1). A slider is another dependency and UI code, and whether it is
  worth that depends on how the keyboard feels on real runs. *(design call; deferred by
  evidence to Phase 3 gate 11, the user's hands-on check. It blocks nothing in Phase 3;
  if the answer is yes, it is a phase of its own, before or with Phase 4.)*~~
  **ANSWERED 2026-09-29 (user, at gate 11): yes, `view` needs a time slider.** It will be
  a phase of its own; nothing about it is designed here.
  **CLOSED 2026-09-30 (user): answered by Phase 4.** The slider is designed in §2.10 and
  built by Phase 4, on `bevy_ui`, which the readout already needs; `bevy_egui` is not
  added, so the premise's "another dependency" does not hold (§2.10.6).
- **OQ-10** — With `--camera`, should `render`'s window default to the keyframes' span
  (first to last `t`) instead of the whole FCD? A shot is usually a stretch of the run, and
  today the user must repeat the first and last `t` as `--from` and `--to`. Phase 5 keeps
  §2.4's defaults, so a flag's meaning does not depend on another flag. *(design call;
  non-blocking; recommendation: keep §2.4's defaults in Phase 5 and decide from Phase 5
  gate 13, where the user renders a file.)* *Note (2026-09-30, Phase 5 gate 13):* the user
  rendered a file and left this open. The orchestrator's recommendation: keep the full-run
  default, and let `--from`/`--to` cut it.
- ~~**OQ-11** — Does orbiting need a second binding for a trackpad, for example Ctrl +
  left-drag? Right-drag on a Mac trackpad is a two-finger press and drag, which may be
  awkward to hold, and the `Q E R F` keys only step. *(design call; deferred by evidence to
  Phase 5 gate 13, the user's hands-on check; blocks nothing.)*~~ **ANSWERED 2026-09-30
  (user): yes.** Ctrl + left-drag orbits and tilts exactly as right-drag does (§2.11.3,
  §2.11.8 k; Phase 5 gates 11 and 13).
- **OQ-12** — Should the pitch floor go below 25° for low, near-horizon shots? The floor
  keeps the horizon out of every frame (§2.11.1), because today there is nothing to show
  above it and pan, zoom and pick need a ground point under the cursor. With buildings and
  a sky (roadmap item 4) a lower floor may be worth its cost. *(design call; deferred by
  evidence to the city phase; blocks nothing in Phase 5.)*
- **OQ-13** — Should `view` preview a keyframe file (`view --camera <file>`, playing the
  flight in the window)? Phase 5 writes paths blind: `K` in `view`, then a `render` to see
  the flight. Whether that loop is too slow is for the user to say. *(design call; deferred
  by evidence to Phase 5 gate 13; if yes, a phase of its own.)* *Note (2026-09-30, Phase 5
  gate 13):* the user left this open. The orchestrator's recommendation: a later phase; a
  2-minute render is enough for now.

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

### Phase 2 — Correct motion: smooth video from 1 Hz FCD
*Produces the observable: yes. It is the same MP4 as Phase 1's, and now the boxes glide
between samples at the video's frame rate. They brake and accelerate as the run did,
follow the engine's turn path through each junction instead of holding and jumping
(OQ-2), and slide between lanes.*

Drafted 2026-09-29; the design and its decisions are §2.8. Phase 2 is strictly after
Phase 1: it changes what Phase 1 draws, and nothing else.

- **Scope:**
  - **Motion (`src/motion.rs`, new).** Before the first frame it builds each vehicle's
    track from its rows in the whole file: the along-link Hermite intervals (§2.8.1),
    the junction spans with movement, path, `G`, `I`, `r` and `r′` (§2.8.2), the lane
    slides (§2.8.4) and the drawn interval (§2.8.5). It answers "where is vehicle `v` at
    time `t`", or "not drawn". It keeps a **motion report**: along-link intervals and
    clamped ones, `Δs < 0` intervals, spans by how their path was found (§2.8.2 steps
    1–3, transparent, no movement), every span's `r` and `r′`, spans out of band, and
    lane slides.
  - **Placement (`src/place.rs`).** It gains what motion needs from the engine: a
    link's `to_node`, `resolve_route_pair`, `lane_turn_path`, `resolve_departure_lane`,
    `turn_path`, `lane_offset`, `interpolate_with_lateral` (already there), and a
    **turn-path point**: `spatial_conflict::interpolate_pos` and `interpolate_heading`
    with §2.8.3's heading conversion. Its unit test checks:
    - the conversion: two-point paths heading north, east, south and west give 0°, 90°,
      180° and 270° to 1e-9°, by wrapped difference;
    - the wiring, on every `lane_turn_path` of the fixture (each movement's
      `from_lanes` × `to_lanes`, skipping pairs that return `None`) and every
      `turn_path`: at `s` = 0, each vertex's accumulated length, and `length`, the point
      is that vertex to 1e-9 m. It needs the fixture, so it is `#[ignore]`d like
      `tests/gates.rs`;
    - and it records how many first and last segments are shorter than 1e-9 m
      (§2.8.3), with no prediction.

    Every row
    in the file, not only the window's, and every span's movement lookup is done before
    the first frame. Nothing new can fail: a missing movement is drawn and counted.
  - **Reading (`src/fcd.rs`).** It keeps every row of each vehicle, grouped by
    `vehicle_id` in `time` order, as well as the window's snapshots. The snapshots are
    still used for `--from`'s default and by the determinism check.
  - **The frame (`src/lib.rs`, `src/render.rs`).** `Job::boxes_at(t)` returns one entry
    per vehicle drawn at `t` (§2.8.5), in `vehicle_id` order: `vehicle_id`; the placed
    point (x, y, heading); its length; its FCD speed, interpolated linearly, for the
    colour; and its **track position**. That is the piece it is on (a link, or a turn
    path keyed by approach link, departure link, entry lane and the path's own
    departure lane, or a chord), the distance along that piece, the lateral offset, and `odo`, the distance
    along the drawn path since `t_first` with lateral motion excluded.
    `Job::motion_report()` exposes the report. Drawing, the depth lift and the draw
    order of §2.3 are unchanged.
  - **Unchanged:** the CLI and its defaults (§2.4), including `--fps` 30; stderr, which
    carries only progress lines, so the report is not printed; the clock, scene, camera
    and output.
- **Exit gate.** Every gate runs on Phase 1's fixture (engine `df8aec0`, urban_grid,
  baseline, seed 42), on the development machine, at the defaults unless a gate says
  otherwise. At the defaults, `h = speedup/fps = 1/30` s of sim time per frame.
  - **The predictions** were measured on 2026-09-29, before building, by a probe
    against the engine's `NetworkData` at `df8aec0` (movements, paths, `G`, join gaps)
    and an arc-length model of §2.8 at 30 fps (steps, speeds, overlaps, drawn counts).
    The method is in `specs/reviews/vis-001.md`. urban_grid's 57 link geometries are
    all two-point straight lines, so arc length and world distance agree along a link.
    The model reproduces Phase 1's overlap count exactly (gate 9).
  - **Gates 6–10 measure through the library,** with `Job::boxes_at(t)` and
    `Job::motion_report()` at every frame time, not through pixels. "The rows around"
    a frame pair at `t`, `t + h` are the vehicle's rows from the last at or before `t`
    to the first at or after `t + h`.
  - **Phase 1's gates that still apply:**
  1. **Video contract** (Phase 1 gate 1). Defaults: `1920,1080,30/1,8700` and 8700
     progress lines, then `done`, with N from §2.4 and D = 290 s from the FCD. Explicit
     run: `1280,720,24/1,720`. The default render's wall time is recorded, with no
     prediction (Phase 1: 140 s).
  2. **Determinism** (Phase 1 gate 2). Two default renders give equal `framemd5`
     hashes, and `tests/gates.rs:determinism_overlap_frames` still gives 0 of 3510
     frames differing. Its frames were chosen for Phase 1's overlaps, which Phase 2
     predicts away (gate 9), so it now guards only against a regression.
  3. **Placement** (Phase 1 gate 3, with its junction case replaced). Single-vehicle
     subsets, `--from` at the row's time, 3840×2160 with `k ≤ 0.6`, within 2 px:
     - **lane 0 and lane 1**: Phase 1's two mid-link rows. The Hermite curve passes
       through its samples, so the prediction is Phase 1's 0.050 and 0.016 px.
     - **in a junction**: vehicle 1 at t = 64.1 s, in its span from row 60.1 s on
       `L_W0_J00` to row 66.1 s on `L_J00_J01`, lanes 0 and 0, `G` = 31.210 m,
       `I` = 28.428 m, `r` = 1.098. Prediction: on the per-lane turn path, 11.60 m along
       its 24.50 m, 13.87 m (about 24 px) past where Phase 1 drew it. The test computes
       the point from the FCD rows and `NetworkData` with the §2.8.2 formula and
       `spatial_conflict::interpolate_pos` called directly, not through `src/motion.rs`.
  4. **Input errors** (Phase 1 gate 4, unchanged): missing FCD, unknown `link_id`,
     ffmpeg not on `PATH`.
  5. **Build size** (Phase 1 gate 5): recorded next to Phase 1's.
  - **Motion:**
  6. **No jumps.** For each vehicle drawn in two consecutive frames, the distance
     between its placed centres is at most `v_ref·h + λ + ε`:
     - `v_ref` is the largest FCD speed among the rows around the pair. Where the pair
       is inside a span, that span's rows count with their speed × `r`.
     - `λ = 1.5·|Δoffset|·h/Δt` where the rows around the pair differ in lane, or a
       span's path lane differs from its entry or departure row's (steps 2–3), else 0. `Δoffset` comes from the rows'
       lanes, not the implementation's slide state, so a snapped change is not excused.
     - `ε = 0.05 m`, about three times the fixture's largest Hermite rise above the
       larger end speed (0.018 m). It is sized to this fixture: with both end speeds 0
       and `Δs > 0` the rise has no general bound.
     - `odo` never decreases by more than 1e-9 m, except at a counted `Δs < 0` interval.
       The tolerance is float noise only: without the tangent clamp, 312 of the 317
       clamped intervals step backwards, by at most 5.5 mm.

     This fails a 1 Hz step (13.9 m in one frame at 13.89 m/s), a junction hold and
     jump (15–40 m), and a snapped lane change (3.5 m against a bound of about 0.69 m).
     A linear sideways slide passes; only gate 12 sees it.

     Predictions: largest step 0.519 m, in a span; largest longitudinal excess over
     `v_ref·h` 0.018 m along a link and 0 in spans; largest lateral step 0.175 m;
     `odo` decreases 0 times; `Δs < 0` intervals 0.
  7. **Braking and acceleration show.** At each sample of an unclamped along-link
     interval, the one-frame speed next to the sample, measured on `odo`, is compared
     with FCD's `speed` there: `|(odo(t_i + h) − odo(t_i))/h − v_i|` after the start
     sample, and the same with `t_{i+1} − h` and `v_{i+1}` before the end sample. Both
     must be at most 0.25 m/s. On placed centres instead of `odo`, the slide alone would
     push 4 lane-change intervals near a stop over (worst 0.342 m/s).
     - Why 0.25 m/s: a one-frame difference is off by about `|a|·h/2`, and the tolerance
       is that error at `|a|` = 15 m/s², 2.5 times the fixture's largest FCD
       `|acceleration|` (6.0 m/s²). It is tied to `h`: at `--fps 60` it would halve.
     - Predictions: 19 508 along-link intervals, 317 (1.6 %) clamped and excluded,
       19 191 measured; largest error 0.136 m/s after the start sample and 0.130 m/s
       before the end sample.
     - A linear blend fails at 4 759 intervals on the start side (5 098 at either end),
       with errors up to 3.0 m/s. This gate, not gate 6, is what tells the Hermite curve
       from linear motion.
  8. **Junctions.** The motion report shows:
     - 422 spans, all through a `JunctionConfig`;
     - 421 by §2.8.2 step 1, 1 by step 2 (vehicle 103, `L_W0_J00 → L_J00_J01`, entry
       lane 0; FCD's departure lane is 1 and the engine's is 0), 0 by step 3, 0
       transparent, 0 with no matching movement;
     - `r` of 1.024–1.196, median 1.094;
     - **0 spans out of band.** `r′` was not measured. The prediction is a bound from
       recorded numbers: `r′ − 1 = e/I`, `|e| ≤ 1.39` m, and `I ≥ G − 3.73 ≥ 10.6 − 3.73 =
       6.87` m (the shortest turn path is 10.6 m), so `r′` lies in 0.82–1.20. The build
       records the measured range.

     At each span's two ends the placed centre is continuous: the position at `t_A` and
     at `t_D` equals that row's placement to within 1 mm.
  9. **Overlaps.** At each frame, two drawn vehicles overlap when their track positions
     are on the same piece (a link with lateral offset rounded to the same lane, or a
     turn path) and their distances along it differ by less than half the sum of their
     lengths; pairs across a join are not counted. Phase 1 on this metric: 354 pairs in
     117 of 291 snapshots, reproduced by the model. Prediction: **0** pairs in 8700
     frames.
  10. **Who is drawn.** At every frame the drawn set is the vehicles with
      `t_first − 1e-6 ≤ t ≤ t_last + 1e-6` (§2.8.5). Prediction: 640 861 vehicle-frames.
  11. **Box length** (Phase 1 gate 7, unchanged), measured at a row's time. Prediction:
      Phase 1's 0.281 / 0.090 / 0.102 / 0.244 px.
  - **The user's check:**
  12. **The user watches** the default MP4 at full frame, and a close-up of the centre
      junction for t = 150–190 s, and confirms that the motion is visibly smooth at
      30 fps: no once-a-second stepping; boxes slow into queues and pull away; boxes
      follow a curve through each junction with no hold and no jump; lane changes slide;
      no box moves backwards. This is the user's bar, and §2.8.6's fallback hangs on it.
- **Not predicted, and so not gated:** render time (gate 1 records it), the measured
  `r′` range (gate 8 bounds it), and the outcome of gate 12.
- **Close-out (standing plan steps, §3 of the methodology):**
  - **Commit plan:** one branch and one push, with commits for motion and placement,
    for the gates, and for the close-out.
  - **Reconciliation:**
    - a new `rules/motion.md` for §2.8 as built (`rules/render.md` is at its 60-line
      cap);
    - `rules/inputs.md`: "Window and snapshots" gains the per-vehicle rows, and
      "Placement" says every row in the file is placed up front;
    - `rules/render.md`, "Vehicles", points at `motion.md`;
    - `src/clock.rs` and `--help` stop saying 8970 frames;
    - the README gains a line on what the motion shows;
    - no `CLAUDE.md` stanza change.
  - Record the gate results in `specs/reviews/vis-001.md`, with any missed prediction
    and its cause, as Phase 1 did.
  - Write this phase's `shipped` date.
  - *Gates 1–11 passed (2026-09-29, build on branch `phase-2`).* Every prediction held as
    written, and none was missed. The results are in `specs/reviews/vis-001.md`.
    - The measured `r′` is 0.970–1.050, median 1.018, inside gate 8's bound.
    - 0 first or last turn-path segments are shorter than 1e-9 m (§2.8.3).
    - The default render took 144 s.
    - The reconciliation's `--help` item needed no change: only `src/clock.rs`'s unit
      test said 8970.
    - Gate 12, the user's check, is open, so `shipped` is not yet written.
  - *Gate 12 passed (2026-09-29): a human check, by the user.* The user watched
    `scratch/out/default.mp4` and the centre-junction close-up for t = 150–190 s
    (`scratch/out/closeup_centre_150-190.mp4`), and confirmed that the motion is visibly
    smooth at 30 fps. Every exit gate has passed: **Phase 2 `shipped: 2026-09-29`.**

### Phase 3 — View: a window over a finished run
*Produces the observable: no. It is argued for here. `view` opens a window and writes
keyframe lines; no video comes out of it, and `render`'s video does not change (gate 1).
It is built before the scripted camera (Phase 4) because it is how a person finds the
moments and framings worth scripting: Phase 4's keyframes are its lines (§2.9.5), and
the user's hands-on check (gate 11) decides whether a time slider is needed (OQ-9)
before Phase 4 relies on it. It is also a check on Phases 1–2 that a video cannot give:
pausing on any frame and following one vehicle through a junction.*

Drafted 2026-09-29; the design is §2.9. Phase 3 is strictly after Phase 2, whose motion
it reuses unchanged.

- **Scope:**
  - **Loading (`src/lib.rs`, or a new `src/run.rs`).** The part of `Job::prepare` from
    the project to the built tracks becomes one function, called by `Job::prepare` and by
    `view`, with the checks in `rules/inputs.md`'s order. It returns the window
    `[from, to]`, the FCD, the placed rows, the placement, the motion and the strips.
    `tests/gates.rs` is not edited (§2.9.6). What it returns has a `boxes_at(t)` that
    needs no `Renderer`; `Job::boxes_at` calls it, and gates 4–9 use it.
  - **Drawing (`src/draw.rs`, new).** Moved, not rewritten, from `src/render.rs`: the road
    mesh, the box pool, a box's transform (translation, yaw, scale and the rank lift),
    its speed material, and the camera's fixed parts (straight down, north up, MSAA ×4, no
    tonemapping or dither, the clear colour). `Renderer` calls them.
  - **`render` stays headless (`src/render.rs`).** `Renderer::new` disables
    `WinitPlugin` and sets `WindowPlugin { primary_window: None, exit_condition:
    DontExit, close_when_requested: false, ..default() }` (§2.9.6). With `primary_window:
    None` and the default `OnAllClosed`, Bevy would write `AppExit` on every update.
  - **View state (`src/view/state.rs`, new; no Bevy types).**
    - `ViewInput`: for one frame, the keys pressed and the keys held (space, `+`, `−`,
      `←`, `→`, Shift, `W A S D`, `Esc`, `K`), the cursor in logical pixels, left-button
      press and release, scroll in lines or pixels, the window's logical size, and the
      real `Δ`.
    - `ViewState::new(from, to, snapshot times, fit)`, where `fit` is the launch fit
      (its centre, `k_fit` and the fitted rectangle) and the snapshot times are the
      FCD's, including one before `from`; its state is §2.9.1–§2.9.2's start.
    - `fn frame(&mut self, input, boxes_at) -> Option<String>` applies one frame of input
      in this order: clock; `Esc`; pan and zoom; click; follow; then `K`, whose line
      (§2.9.5) is returned and describes the state after the frame.
    - `ViewState::readout()`, the text of §2.9.4, so the window only displays it.
    - `pick(boxes, world point, k)` (§2.9.3), and the keyframe line's `format` and
      `parse` (§2.9.5).
  - **The window (`src/view/mod.rs`, new).** A Bevy app with `DefaultPlugins` and a
    primary window of `--width`×`--height` logical pixels, titled `assimilator-video view`.
    Each frame it builds a `ViewInput` from Bevy's input, calls `ViewState::frame`, prints
    a returned line to stdout and flushes, sets the camera's transform and projection,
    fills the box pool from `boxes_at(t)` through `src/draw.rs`, and sets the readout
    (§2.9.4). Present mode is Bevy's default, vsync on.
    - A hidden `--bench <s>` flag, to record the frame rate at close-out (§2.9.8): it
      starts at `t` = 140 s playing at 2×, measures real frame times for `s` seconds after 3 s of warm-up, prints one JSON
      object on stderr (`frames`, `mean_fps`, `median_ms`, `p99_ms`, `worst_ms`) and
      exits 0. It is not in `--help`.
  - **The CLI (`src/main.rs`).** A `view` subcommand (§2.9). It does not look for ffmpeg.
    The event loop runs on the main thread.
  - **`Cargo.toml`.** Bevy gains `bevy_winit`, `bevy_text`, `default_font`, `bevy_ui` and
    `bevy_ui_render` (§2.9.7). No other dependency is added.
  - **Tests (`tests/view.rs`, new).** The gates below that need the fixture are
    `#[ignore]`d, like `tests/gates.rs`. `pick` on constructed boxes and the keyframe line
    need no fixture and run in plain `cargo test`.
- **Exit gate.** On Phase 1's fixture (engine `df8aec0`, urban_grid, baseline, seed 42),
  on the development machine (Apple M3, macOS, Retina display). Gates 4–9 drive
  `ViewState::frame` with scripted input and measure its state: no window, no GPU. Unless a
  gate says otherwise they use a 1280×720 logical window, whose fit gives `k_fit` =
  1240/720 = 1.7222 m per logical pixel (Phase 1's bounding box, 1200 m plus the margin).
  Distances are compared to 1e-9 m and times to 1e-9 s: the state is set by formulas, so
  anything beyond float noise is a bug.
  - **`render` is unchanged (Phase 1–2 gates):**
  1. **Same frames.** The default render built on the Phase 3 branch and the default
     render built at `8eb9052` give equal `framemd5` hashes, run on this machine the same
     day. Prediction: 8700 of 8700 equal.
     - **The reference:** `git archive 8eb9052` into `scratch/ref-8eb9052/`, built there
       release-only with its own `CARGO_TARGET_DIR` (about 1.2 GB, §2.9.7), run with
       `render`'s defaults on the fixture, hashed with `ffmpeg -f framemd5` into
       `scratch/ref-8eb9052.framemd5`; the build is then deleted. `scripts/gates.sh`
       rewrites `scratch/out/default.framemd5` on every run, so the existing file is not
       the reference.
     - `scripts/gates.sh` (gates 1, 2, 4) and `cargo test --release --test gates --
       --ignored --test-threads=1` (all of it: Phase 2 gates 3 and 6–11, Phase 1 gate 7,
       the determinism check) pass with Phase 2's recorded numbers, and `tests/gates.rs`
       is not edited. Those tests build a `Job` on libtest's threads, so with
       `bevy_winit` on they also show that `render` creates no event loop.
     - It runs twice: on the loading-and-drawing commit, and again after the `Cargo.toml`
       feature change, the only run that exercises `WinitPlugin`.
  2. **Input errors** (Phase 1 gate 4) as before for `render`. For `view`: a missing FCD
     and an unknown `link_id` each exit 1 with one stderr line and nothing on stdout. No
     window is possible: the checks run before any `App` is built (§2.9), which the code
     shows. With ffmpeg off `PATH`, `view`'s missing-FCD error is still the FCD's.
  3. **Build cost** (Phase 1 gate 5), clean builds with sccache off, recorded with the
     load average, with no pass or fail. Predictions, from §2.9.7: 360 crates in the
     release build; `target/` about 1.6 GB after it and about 6.3 GB with the debug build
     added. Wall time is recorded, not predicted (other sessions' load, §2.9.7).
  - **The view state, headless:**
  4. **Pan.** From the fit, a drag of (+100, −40) logical pixels moves the centre by
     (−100·`k_fit`, −40·`k_fit`) = (−172.22, −68.89) m, and the world point under the
     cursor at release is the one under it at press. Holding `D` for frames of real
     `Δ` summing to 0.5 s moves `cx` by `0.5 · 0.5 · 1280 · k_fit` = 551.11 m, before the
     centre bound. A drag that would pass the fitted rectangle stops at its edge.
  5. **Zoom.** With the cursor at (800, 300), over world (575.56, 403.33), 5 lines up give
     `k = k_fit · 1.1^−5` and leave the world point under the cursor where it was. From
     the fit, 100 lines up give exactly `k_min` = 0.02 (`k_fit · 1.1^−100` ≈ 1.25e-4); from
     there, 100 lines down give exactly `k_max` = 2·`k_fit` = 3.4444, with the same fixed
     point and the centre at (24.44, 196.67), inside the fitted rectangle. 40 trackpad
     pixels equal 2 lines.
  6. **Clock.**
     - Playing at 1× with `Δ` = 1/60 s for 600 frames moves `t` by 10 s. One frame with
       `Δ` = 0.5 s moves it by 0.1 s (the cap).
     - `+` from 1× reaches 64× in 6 presses and stays; `−` from 1× reaches 1/8× in 3 and
       stays. At 4× with `Δ` = 1/60, one frame moves `t` by 1/15 s.
     - `→` and `←` move `t` by exactly 1/30 s and pause.
     - With `t` at the snapshot time 60.100000000000584, `Shift+→` lands on the next,
       61.1000000000006, and `Shift+←` on the previous, 59.10000000000057. At the last
       snapshot `Shift+→` leaves `t` unchanged.
     - Playing into `to` pauses at `to` exactly; space then restarts from `from`.
  7. **Pick.** On constructed boxes: a click inside a box picks it; a click 7.9 logical
     pixels from its footprint's edge picks it and 8.1 does not; a click inside two
     overlapping boxes picks the higher `vehicle_id`; two boxes at equal distance, the
     higher `vehicle_id`. On the fixture at `t` = 64.1 s, a click on vehicle 1's placed
     point (Phase 2 gate 3's junction case) picks vehicle 1 at `k_fit` and at `k_min`.
  8. **Follow.** Pick vehicle 1 at `t` = 64.1 s, then play at 1× with `Δ` = 1/60 s,
     zooming in 10 lines along the way, until `t` is past vehicle 1's last row (147.1 s).
     In every frame where it is drawn, the centre equals its placed point from the loaded
     run's `boxes_at(t)` (which `Job::boxes_at` calls, so no `Renderer` is built). In every frame after, the centre is its last drawn point and
     `readout()` says "(not drawn)".
     Stepping back into its interval re-centres on it. A drag, a `WASD` pan and `Esc` each
     stop following. The centre bound never applies while following (every placed point is
     inside the fitted rectangle).
  9. **Keyframe line.** The state at `t` = 64.1, centre (512.3, −133.2), `k` = 1/3 in a
     720-pixel window prints exactly
     `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }`, and following vehicle
     103 at `t` = 200 and `k` = 1/12 prints `{ t = 200.000, follow = 103, height_m = 60.00 }`.
     In gate 8's hold (vehicle 1 not drawn), `K` prints the `x`, `y` form with the held
     centre, not `follow`. For the state at every frame of gates 4–8, its line satisfies
     `format(parse(line)) == line`, and `parse` gives back `t`, the centre or the vehicle,
     and `H·k` to within half the last decimal (the rounding is the only loss). `parse`
     rejects a line with a missing key, an extra key, keys out of order, or both `x` and
     `follow`.
  - **Frame rate:**
  10. *Recorded, not gated (§2.9.8 decision 2);* see "Not predicted" below. The number
      is kept so that gate 11 does not move.
  - **The user's check:**
  11. **The user uses `view`** on the fixture and confirms it is fit to find and frame
      shots. What to try:
      - open it: the whole network shows, paused at the first sample;
      - space to play; `+` to 64× and `−` to 1/8×; watch the readout;
      - pause and step with `←`/`→`, then `Shift+←`/`Shift+→` sample to sample;
      - drag and `WASD` to pan; scroll in on the centre junction to the zoom limit and out
        to the other; the point under the cursor stays put;
      - click a box near a junction and follow it through a turn at 1× and at 1/8×; zoom
        while following; play past its last row (the camera holds, "(not drawn)"), then
        step back; `Esc` to stop;
      - press `K` a few times and read the lines on stdout;
      - resize the window.

      And say whether a time slider is needed (OQ-9), and whether any number here (pan
      speed, zoom step, limits, pick radius, speed ladder) should change. Tuning those is
      iteration, not a spec change (§2.6), unless it changes the keyframe line.

      *Gate 11, first check (2026-09-29, the user): not passed.* There were two findings,
      and everything else worked. `+` did not work on the user's keyboard (§2.9.1 note),
      and the readout did not say how to stop following (§2.9.4 note). Both are fixed. The
      user answered OQ-9: yes, a time slider, as its own phase. Gate 11 is re-checked by
      the user.

      *Gate 11 passed (2026-09-29, the user's re-check after `5c479fc`).* `+`/`-` and the
      Esc hint work; everything else was fine at the first check. No number changes. With
      gates 1–9 passed, **Phase 3 `shipped: 2026-09-29`.**
- **Not predicted, and so not gated:**
  - **The frame rate** (§2.9.8, decision 2). At close-out, `view --bench 20` runs on the
    fixture at the default window, full zoom-out (up to 92 boxes drawn), and its JSON is
    recorded with the load average.
    - For comparison, not a prediction: the drafting prototype ran at the display's
      60 Hz (median 16.67 ms, even with vsync off), with one unexplained 2.9 s stall in
      four runs, and `boxes_at` at 0.26 ms at most (`specs/reviews/vis-001.md`).
  - frame rates on other machines or displays;
  - the feel of the controls (gate 11);
  - whether macOS itself writes to stderr (the OS, not `view`, would be the writer).
- **Close-out (standing plan steps, §3 of the methodology):**
  - **Commit plan:** one branch and one push, with commits for the shared loading and
    drawing (gate 1's first run), for the `Cargo.toml` features, `view` and its state
    (`cargo clean` once right after the feature change, §2.9.7; then gate 1's second
    run), for the tests and gates, and for the close-out.
  - **Reconciliation:**
    - a new `rules/view.md` for §2.9 as built: the CLI, the clock, the camera, picking and
      following, the keyframe line (`rules/render.md` is at its 60-line cap);
    - `rules/render.md`, "Pipeline", says `WinitPlugin` is disabled and no window is
      created, and "Scene" points at `src/draw.rs`; its `sources` gain `src/draw.rs`;
    - `rules/inputs.md`: "Checks before the first frame" says `view` runs the same
      checks without ffmpeg; its `sources` gain the loading module if it is new;
    - `rules/render.md` (60/60) and `rules/inputs.md` (50/50) are at their caps, so these
      edits reword to fit, moving detail into `rules/view.md`; `max_lines` is not
      raised;
    - the README gains the `view` command and its keys;
    - no `CLAUDE.md` stanza change.
  - Record the gate results in `specs/reviews/vis-001.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 4 — Time slider: click and drag through the run in `view`
*Produces the observable: no. It is argued for here. The slider is part of `view`, a tool
(§2.7 item 3); no video comes out of it, and `render`'s video does not change (gate 1).
It is built before the scripted camera (Phase 5) because the user asked for it at Phase
3's hands-on check (OQ-9): on a real run, finding a moment by play, speed and steps alone
is slow, and finding moments is what Phase 5's keyframes need. It is small: one module,
no dependency, and Phase 3's gates stand unchanged.*

Drafted 2026-09-30; the design is §2.10. Phase 4 is strictly after Phase 3, whose `view`
it extends.

- **Scope:**
  - **The slider (`src/view/slider.rs`, new; no Bevy types).** The geometry of §2.10.1
    from a window size, `None` when `W < 64` or `H < 64`; `x(t)` and `t(x)` exactly as §2.10.2 writes
    them; the hit test; and the ticks of §2.10.4 as the period `P` and the tick times.
    The numbers of §2.10.1 and the 8 px tick spacing are named constants.
  - **The state (`src/view/state.rs`).**
    - `ViewInput` gains `pointer: Option<(f64, f64)>`, the last `CursorMoved` position of
      the frame in logical pixels, not bounded to the window (§2.10.3).
    - `ViewState` gains the scrub: `Option` of whether the clock was playing at the press;
      and `bar()`, the geometry of §2.10.1 for its current `size` (`None` when too small),
      which the window draws from.
    - `ViewState::frame` applies §2.10.5's order. Steps 5–9 are Phase 3's code, changed
      only so that a bar press makes no `Press` and scroll is ignored per §2.10.3.
    - The existing public API keeps its signatures, so `tests/view.rs` compiles and runs
      unedited: `ViewInput` derives `Default`, which every `ViewInput` there comes from
      (through `idle()`), so `pointer` is `None`; `ViewState` is built with `new`.
  - **The window (`src/view/mod.rs`).** `input` fills `pointer` from Bevy's
    `CursorMoved` messages for the primary window. At start-up it spawns the track, the
    handle and a pool of tick nodes, grown when a frame needs more ticks, as absolutely positioned `bevy_ui` `Node`s with a
    `BackgroundColor`; each frame it sets their positions and sizes from the state's
    geometry, and hides unused ticks and, when there is no bar, the whole bar (`Display::None`).
    The readout's node moves to `left: 16 px, bottom: 30 px` (§2.10.1). Colours are
    iteration, not specification (§2.6).
  - **Unchanged:** `Cargo.toml` and `Cargo.lock` (§2.10.6), the CLI and `--help`, every
    key, the keyframe line, `--bench`, and everything `render` runs.
  - **Tests (`tests/slider.rs`, new).** Gates 4–8 run on constructed windows in plain
    `cargo test`, except gate 5's round trip over the fixture's snapshot times. That part
    and gates 9 and 10 need the fixture and are `#[ignore]`d, like `tests/view.rs`.
- **Exit gate.** On Phase 1's fixture (engine `df8aec0`, urban_grid, baseline, seed 42),
  on the development machine (Apple M3, macOS, Retina display). Gates 4–10 drive
  `ViewState::frame` and the slider functions headless, as Phase 3's gates 4–9 do, at a
  1280×720 logical window unless a gate says otherwise. Distances in pixels or metres are
  compared to 1e-9 and times to 1e-9 s; "exactly" means equal as `f64`. The fixture's
  window is `from` = 9.099999999999984, `to` = 299.0999999999995 (Phase 3's record). The
  constructed window is `[0, 300]` with Phase 3's `plain_state` fit (centre (0, 0), `k`
  = 1).
  - **What must not change:**
  1. **`render`'s frames.** `scripts/gates.sh` passes, and its Phase 3 gate 1 comparison
     gives the default render's `framemd5` equal to `scratch/ref-8eb9052.framemd5`.
     Prediction: 8700 of 8700. `cargo test --release --test gates -- --ignored
     --test-threads=1` passes 5 of 5 with Phase 2's printed numbers, and `tests/gates.rs`
     is not edited.
  2. **`view`'s Phase 3 gates.** `tests/view.rs` is not edited, and `cargo test --release
     --test view -- --include-ignored --test-threads=1` (Phase 3's command, in
     `scripts/gates.sh`) passes 10 of 10 (6 ignored, 4 plain: Phase 3 gates 2 and 4–9 and
     the gate 11 fixes) with the numbers Phase 3 printed. None of its presses or scrolls is
     in the hit area: all are in a 1280×720 window at `y` ≤ 371 (gate 8's re-follow after
     `W`, at 370.67; the rest at ≤ 360), above its top at 692.
  3. **Build cost.** `git diff` of `Cargo.toml` and `Cargo.lock` against the Phase 3
     merge is empty. Prediction: 0 crates added (360 in the release build). The
     incremental release build's wall time is recorded, not predicted.
  - **The slider, headless:**
  4. **Geometry.** At 1280×720: `x0` = 16, `x1` = 1264, `y_bar` = 706, hit area
     `[8, 1272] × [692, 720]`; (640, 692) and (8, 720) hit, (640, 691.9) and (7.9, 706) do
     not. At 64×64 the bar exists with `L` = 32; at 63×720 and at 1280×63 there is none.
     With `t` = 92.3076923076923 on the constructed window, a frame at 1600×900 leaves `t`
     unchanged and gives `x1` = 1584, `y_bar` = 886 and the handle at
     `x(t)` = 498.4615384615385.
  5. **x ↔ t.** On the fixture's window at 1280: `t(16)` is exactly `from` and `t(1264)`
     exactly `to`; `x(from)` is exactly 16 and `x(to)` exactly 1264; `t(640)` =
     154.09999999999974. `t(x(t))` = `t` for each of the fixture's 291 snapshot times in
     the window and the constructed times 0, 150 and 300 on `[0, 300]`. Mid on the
     constructed window: `t(640)` = 150.
  6. **Clamping and the pointer.** On the constructed window, press at (640, 706), then
     move: `pointer` at `x` = 0 gives `t` = 0 exactly; 1280 gives 300 exactly; −50 gives
     0 and 2000 gives 300 (off the window). With `pointer` `None` and `cursor` at
     (400, 706), `t` = 92.3076923076923; with both `None`, `t` holds.
  7. **The bar owns its input.** On the constructed window, with one box (4.5 m, heading
     90°) at world (0, −346), under pixel (640, 706):
     - press and release at (640, 706): nothing is picked, `t` = 150, `press` stays
       `None`;
     - press at (640, 706) and move 100 px right, then 100 px up off the bar: the centre
       does not move (0 m, exactly), and `t` follows `x` only (`t(740)` = 174.03846153846155);
     - with the box at (0, −331), under (640, 691), a click at (640, 691) picks it, and a
       drag from (640, 691) by (+100, 0) pans the centre to (−100, 0) m, and keeps panning
       when the cursor then moves onto the bar: at (740, 706) the centre is (−100, 15) m;
     - 3 scroll lines at (640, 706), and 3 during a scrub at (640, 300), leave `k` = 1;
       3 lines at (640, 691) zoom to `k` = 1.1^−3.
  8. **Playback across a drag.** On the constructed window, `Δ` = 1/60 s:
     - playing at 1×, press at (400, 706) and hold still for 30 frames: `t` =
       92.3076923076923 in every frame, the clock paused, the readout "paused"; release:
       playing, and the next frame gives `t` = 92.32435897435897;
     - paused before the press: paused after the release;
     - playing, press at (640, 706), move to (1264, 706) and release there: `t` = 300
       exactly, and paused;
     - playing, a click at (640, 706): `t` = 150, still playing;
     - during a scrub, space, `←` and `→` change neither `t` nor the resume; one `+` makes
       the speed 2×;
     - playing, press at (400, 706), then a frame at 63×720 (no bar): the scrub ends, `t`
       stays 92.3076923076923 in that frame, and playing resumes.
  9. **A follow survives a scrub** (fixture). Pick vehicle 1 at `t` = 64.1 s as Phase 3
     gate 7 does, then press on the bar at (400, 706) and drag, one frame per position:
     - at `x` = 400 (`t` = 98.33076923076906): following, drawn, the centre equal to
       vehicle 1's placed point from `boxes_at(t)`;
     - at `x` = 900 (`t` = 214.51666666666634, after vehicle 1's last row at 147.1 s):
       still following, "(not drawn)" in the readout, the centre unchanged from the frame
       before;
     - back at `x` = 400: re-centred on the placed point; released there, still following
       vehicle 1;
     - every frame's keyframe line passes Phase 3 gate 9's check (round trip, `follow`
       only while drawn).
  10. **Ticks.** The fixture's window at 1280: `P` = 60 s, ticks at 60, 120, 180, 240 s,
      `x` = 235.04551724137974, 493.25241379310427, 751.4593103448287 and
      1009.6662068965533. Constructed at 1280: `[0, 3600]` gives `P` = 60 s and 61 ticks,
      `[0, 10800]` 300 s and 37, `[0, 86400]` 600 s and 145. The fixture's window at 64
      wide: `P` = 300 s and no tick.
  - **The user's check:**
  11. **The user uses the slider** in `view` on the fixture and confirms it is fit for
      finding moments. What to try:
      - open it: the bar along the bottom, the handle at the left end, four ticks, the
        readout just above;
      - click near the right end, the middle and a tick; the readout's `t` jumps there;
      - drag the handle slowly and fast both ways; drag past each end and off the window
        (the handle stops at the end, never beyond);
      - while playing at 4×, drag and let go: it pauses while held and plays after;
        drag to the right end and let go: paused at `to`;
      - follow a vehicle, then scrub back and forth across its life: the camera stays on
        it while it is drawn and holds with "(not drawn)" when not; `Esc` still stops;
      - press on the bar and drag up onto the map: no pan; press just above the bar and
        drag down across it: a pan; scroll over the bar: no zoom;
      - resize the window, wide and narrow: the bar follows; very small: it goes;
      - press `K` after a scrub, and read the line.

      And say whether any number (inset, thickness, handle, hit area, the 8 px tick
      spacing) should change. Tuning them is iteration, not a spec change (§2.6).
- **Not predicted, and so not gated:**
  - the frame rate with the bar: `view --bench 20` is recorded at close-out, as Phase 3
    gate 10;
  - the incremental build time (gate 3);
  - how `bevy_ui` rounds the nodes to physical pixels: the drawn handle may sit up to
    half a physical pixel from `x(t)`; the state's `x(t)` is what gates 4–10 measure;
  - whether a `CursorMoved` arrives outside the window on another OS (§2.10.3 reads it
    on macOS only; Linux is roadmap item 6);
  - the feel of the slider (gate 11).
- **Close-out (standing plan steps, §3 of the methodology):**
  - **Commit plan:** one branch (`phase-4`) and one push, with commits for the slider and
    the state (with gates 4–8), for the window's drawing, for gates 9–10 and the gate run,
    and for the close-out.
  - **Reconciliation:**
    - a new `rules/slider.md` for §2.10 as built: the geometry, x ↔ t, the input rules and
      the ticks, with `sources` `src/view/slider.rs`, `src/view/state.rs` and
      `src/view/mod.rs`. `rules/view.md` is at its 60-line cap, so it gains only the new
      frame order, the readout's new place, and a pointer to `slider.md`, reworded to fit;
      its `max_lines` is not raised;
    - the README's `view` section gains the slider;
    - no `CLAUDE.md` stanza change.
  - Record the gate results in `specs/reviews/vis-001.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 5 — 3D camera: orbit and tilt in `view`, keyframed flights in `render`
*Produces the observable: yes. `render --camera <file.toml>` writes the run's video
through a perspective camera that flies through the keyframes `view` printed: tilted,
turned, holding, and following a vehicle. Without `--camera`, `render`'s video is
byte-identical to Phase 4's (gate 1).*

Drafted 2026-09-30; the design is §2.11, and the user's calls on it are §2.11.8. Phase 5 is
strictly after Phase 4: it extends `view`'s camera and reads `view`'s keyframe lines.

- **Scope:**
  - **The pose (`src/camera.rs`, new; no Bevy types).** `Pose { cx, cy, height_m, yaw_deg,
    pitch_deg }` and its default (yaw 0, pitch 90); the named constants `FOV_DEG` = 45,
    `PITCH_MIN` = 25, `PITCH_MAX` = 90, `FAR_PER_D` = 20; the degree sine and cosine, exact
    at multiples of 90° (§2.11.1); `distance(height_m)`; `project(pose, W, H, [x, y, z])`
    and `ray_to_plane(pose, W, H, (px, py), z)`; the short way round (§2.11.5).
  - **Keyframes and the flight (`src/keyframes.rs`, new; no Bevy types).**
    - `Keyframe` and `At` move here from `src/view/state.rs`, which re-exports both so the
      tests' paths stay. `Keyframe` gains `yaw_deg` and `pitch_deg` (defaults applied on
      reading). `format` prints §2.11.4's lines, and `parse(line)` reads one line with the
      file's reader.
    - `read(path)`: the file's TOML (`serde`, `deny_unknown_fields`) and every check of
      §2.11.4 that needs no run, in the order listed there, each error naming the
      keyframe; then `check_follows(&keyframes, &Motion, &Fcd, &Placement)` for the last
      one. Both `Run` and `Job` hold those three (`src/run.rs:boxes_at` takes them).
    - `Flight::new(keyframes)` builds §2.11.5's channels once;
      `Flight::pose_at(t, &Motion, &Fcd, &Placement)` gives the pose at any `t`, a
      vehicle's placed point from `run::boxes_at`.
  - **Drawing (`src/draw.rs`, `src/scene.rs`).** A perspective camera from a pose: its
    `Transform` in the baked frame and its `PerspectiveProjection` (`φ`, `far = 20·d`).
    `box_transform` takes the lift per rank as an argument; the orthographic callers pass
    today's `RANK_LIFT` (0.01 m), and the perspective ones `RANK_LIFT_3D` (0.001 m). A
    second box mesh with per-face vertex colours for the perspective path (§2.11.2).
  - **`render` (`src/render.rs`, `src/lib.rs`, `src/main.rs`).**
    - `render` gains `--camera <file.toml>`.
    - `RenderOptions` does not change (`tests/gates.rs` builds it field by field).
      `Job::prepare(&RenderOptions)` is Phase 4's orthographic job, unchanged.
      `Job::prepare_with_camera(&RenderOptions, &Path)` loads the run the same way, then
      reads the file (§2.11.4), builds the `Flight` and a perspective `Renderer` whose
      camera is set from `pose_at(t)` on every frame.
    - `Job::pose_at(t)` exposes the pose (`None` for the orthographic job).
      `Job::camera()` and `Job::k()` keep answering with the fit, the scene's bake origin.
    - The frame count, the progress lines, the ffmpeg output and every error and check of
      §2.4 are unchanged. The ffmpeg check still runs first; the file's checks run inside
      the prepare, before the encoder starts.
  - **`view` (`src/view/state.rs`, `src/view/mod.rs`).**
    - `ViewInput` gains `right_press`, `right_release`, `Held::ctrl` and
      `Pressed::{q, e, r, f}`. `ViewState` gains `yaw_deg`, `pitch_deg` and an `orbit`
      (the press's cursor, yaw and pitch, and which button ends it: right, or left for
      Ctrl + left, §2.11.3).
    - `ViewState::frame` applies §2.11.3's order. `world` casts the cursor's ray to the
      ground, the click casts it to the 0.80 m plane, and pan, `WASD` and zoom follow
      §2.11.3. `keyframe()` carries the pose.
    - The window sets the camera's `Transform` and `PerspectiveProjection` from the state's
      pose every frame, reads the right button, Control (either side) and `Q E R F` by
      `KeyCode`, and draws the
      boxes with the shaded mesh and `RANK_LIFT_3D`. The readout and the slider do not
      change.
    - `Fit`, `ViewState::new`, `pick` and every other public signature the tests use stay
      as they are.
  - **`Cargo.toml`.** `toml` 1 (`default-features = false`, `features = ["std", "parse",
    "serde"]`) and `serde` 1 with `derive` (§2.11.6). No Bevy feature changes.
  - **Tests.**
    - `tests/camera.rs` (new): gates 4–7 and 11–12 headless. Those that need the fixture
      (7, and part of 5) are `#[ignore]`d, like `tests/view.rs`. Gates 8 and 9 render
      through the GPU, and are `#[ignore]`d too.
    - `tests/flight.toml` (new): the keyframe file of gates 7 and 10, below.
    - `scripts/gates.sh` gains gate 10.
    - `tests/gates.rs` and `tests/slider.rs` are not edited. `tests/view.rs` is edited in
      `gate9_keyframe_line` only (gate 2 says why).
- **Exit gate.** On Phase 1's fixture (engine `df8aec0`, urban_grid, baseline, seed 42), on
  the development machine (Apple M3, macOS, Retina display). Headless gates measure the
  state and the pose, with no window and no GPU. Distances are compared to 1e-9 m, angles
  to 1e-9°, pixels to 1e-9 px and times to 1e-9 s unless a gate says otherwise: the state is
  set by formulas, so anything beyond float noise is a bug. "Exactly" means equal as `f64`.
  **The predictions** come from a Python model of §2.11's formulas in IEEE doubles, with the
  operations in §2.11's order (`specs/reviews/vis-001.md`). Nothing was built. The plain
  state is Phase 3's (`[0, 300]`, centre (0, 0), `k` = 1, 1280×720, so `height_m` = 720).

  **The flight file** of gates 7 and 10, `tests/flight.toml`. It holds a Phase 3 line
  unchanged, two holds, a tilt, a turn the short way and a follow with a zoom:

  ```toml
  keyframes = [
    { t = 20.000, x = 0.00, y = 300.00, height_m = 1240.00 },
    { t = 50.000, x = 0.00, y = 300.00, height_m = 1240.00 },
    { t = 64.100, follow = 1, height_m = 120.00, yaw_deg = 90.00, pitch_deg = 45.00 },
    { t = 140.000, follow = 1, height_m = 60.00, yaw_deg = 90.00, pitch_deg = 45.00 },
    { t = 160.000, x = 600.00, y = 300.00, height_m = 400.00, yaw_deg = 0.00, pitch_deg = 60.00 },
    { t = 220.000, x = 300.00, y = 600.00, height_m = 700.00, yaw_deg = 315.00, pitch_deg = 35.00 },
    { t = 250.000, x = 300.00, y = 600.00, height_m = 700.00, yaw_deg = 315.00, pitch_deg = 35.00 },
  ]
  ```

  Vehicle 1 is drawn at 64.1 s (Phase 2 gate 3) and until its last row at 147.1 s (Phase 3
  gate 8), so both `follow` lines are valid and the blend to 160 s outlives it.
  - **What must not change:**
  1. **`render` without `--camera`.** `scripts/gates.sh` passes, and its Phase 3 gate 1
     comparison gives the default render's `framemd5` equal to
     `scratch/ref-8eb9052.framemd5`. Prediction: **8700 of 8700** (§2.11.8 c).
     `cargo test --release --test gates -- --ignored --test-threads=1` passes 5 of 5 with
     Phase 2's printed numbers, and `tests/gates.rs` is not edited.
  2. **`view`'s Phase 3 and 4 gates.** Both run with `--include-ignored --test-threads=1`,
     with every printed number equal to Phase 3's and Phase 4's records to 1e-9.
     - `tests/slider.rs` is not edited and passes **8 of 8**.
     - `tests/view.rs` passes **10 of 10**. It is edited in `gate9_keyframe_line` only,
       because §2.11.8 f changes what that test asserts. Its three exact lines (the hold's
       included) gain `yaw_deg = 0.00, pitch_deg = 90.00`; the `-0.00` keyframe it builds
       field by field gains the two fields; Phase 3's two lines are checked to parse (to
       yaw 0, pitch 90) rather than to re-format to themselves; and its rejection list
       drops the three cases TOML reads (its two "keys out of order" lines and the one with
       no spaces inside the braces). The missing keys, the extra keys, `x` with `follow`, `inf`, a negative
       `follow`, the trailing comma and the empty line are still rejected.

     Why the rest stands: every other gate runs at the default pose, where the cursor's
     ground point, pan, `WASD` and zoom are Phase 3's formulas to rounding, and those
     gates compare to 1e-9 (§2.11.1). The pick plane's shifts are listed in §2.11.3; none
     changes a gate's outcome. Every line `check_line` reads now has six or five keys, and
     still round-trips.
  3. **Build cost** (§2.11.6). `Cargo.lock` gains exactly 2 packages, and no Bevy feature
     changes. The crate count of a clean release build is taken from its
     `Compiling` lines, in a throwaway `CARGO_TARGET_DIR` under `scratch/` (about 1.2 GB,
     deleted afterwards). Prediction: **362–365** (360 at Phase 3). Wall time and the
     working `target/` growth are recorded, not predicted.
  - **The pose, headless:**
  4. **Projection.** `distance(h)` = 1.2071067811865475·`h`.
     - At the default pose, `project` of a ground point is Phase 1's `world_to_pixel` to
       1e-9 px. At the render fit's scale (1920×1080, `height_m` = 1240, look-at (0, 300)),
       the point (100, 350, 0) is at (1047.0967741935483, 496.4516129032258) both ways.
     - Yaw 90, pitch 90, same scale: (100, 300, 0), 100 m east of the look-at, is at
       (960, 452.9032258064516), straight above the centre.
     - Pitch 30, yaw 0, `height_m` = 240, look-at (0, 0), 1920×1080: (0, 100, 0) is at
       (960, 366.7808893062154).
     - `ray_to_plane(project(X))` gives back `X` for each of these.
     - On the plain state, the cursor (740, 300) is over the ground point (100, 60), which
       is Phase 3's `world`; at yaw 30, pitch 40 it is over (145.222180146317,
       33.60236249663478).
  5. **The keyframe file.** Each error of §2.11.4 has a case. Each exits 1 with one stderr
     line starting `error: --camera`, no progress line and no file at `--out`; those that
     need the run use the fixture through the CLI. The cases:
     - no file; not TOML; no `keyframes`; `keyframes = []`; a second top-level key;
     - a keyframe with `zoom = 2.0`; no `t`; no `height_m`; `x` and `follow`; neither;
       `x` without `y`;
     - `t = inf`; `height_m = nan`; `follow = -3`; `follow = 1.5`;
     - `height_m = 0`; `pitch_deg = 24.99`; `pitch_deg = 90.01`;
     - `t` out of order (named "out of order"); a duplicate `t` (named "duplicate");
     - `follow = 1` at `t = 200` (after its last row); `follow` of an id not in the FCD.

     Accepted, each with its expected pose: both Phase 3 lines unchanged (yaw 0, pitch
     90); `yaw_deg = 370` (read as 10); `yaw_deg = -90` (270); the file as `[[keyframes]]`
     tables, which gives the same keyframes as the inline array; a single keyframe; a
     keyframe at `t` = 5, before `from`.
  6. **The flight, constructed.** Five keyframes, all fixed: `K1` `t` = 10, (0, 0), `h` =
     200, yaw 0, pitch 90; `K2` 20, (100, 0), 200, 0, 90; `K3` 30, (100, 100), 100, 90, 45;
     `K4` = `K3` at 40 (a hold); `K5` 50, (0, 100), 300, 350, 60. `pose_at(t)` gives
     (x, y, `height_m`, yaw, pitch):

     | `t` | pose |
     |---|---|
     | 5, 10 | `K1` exactly (before the first keyframe, and at it) |
     | 15 | (41.89453125, −4.39453125, 200, 0, 90) |
     | 20 | `K2` exactly |
     | 25 | (104.39453125, 58.10546875, 141.42135623730945, 45, 67.5) |
     | 30, 35, 40 | `K3` exactly: the hold |
     | 45 | (50, 100, 173.20508075688775, 40, 52.5) |
     | 50, 55 | `K5` exactly |

     - **The hold:** at all 301 frame times of [30, 40] at 30 fps, the pose equals `K3`
       exactly. **0 drift.**
     - **No overshoot:** at every frame time of each segment, each scalar lies between its
       two keyframes' values. **0 violations.** `τ` never decreases, and on `K2 → K3` it
       never passes `K3`'s knot.
     - **No corner at `K2`:** the velocity there is (5, 5) m/s from both sides (central
       and one-sided differences at ±1e-4 s agree to 1e-3 m/s). It is 0 at 10, 30, 40 and
       50.
     - **The curve leaves the lines:** over the frame times `t` = 10 + n/30, the largest
       distance from the straight line between two keyframes is 7.4073228602604 m, at
       17.2333 s on `K1 → K2` and 22.7667 s on `K2 → K3` (the continuous maximum is 200/27
       = 7.4074 m, which these samples miss by 8.5e-5). The largest speed, from the curve's
       derivative at the same times, is 15.006 m/s to 1e-3.
     - **Yaw the short way:** `K4 → K5` (90° to 350°) runs 90, 87.2, 79.6, 68.4, 54.8, 40,
       25.2, 11.6, 0.4, 352.8, 350 at whole seconds, through 0°, never through 180°. A tie
       turns clockwise: 0° to 180° over [0, 10] is 90° at 5 s, and 180° to 0° is 270°.
  7. **The flight on the fixture** (`tests/flight.toml`), at every frame time of the
     default render (`from` = 9.099999999999984, `h` = 1/30 s):
     - `t` ≤ 50: the pose is the first keyframe exactly (held before 20, then the 20–50
       hold). `t` ≥ 220: the pose is the sixth keyframe exactly (the 220–250 hold, then
       held after the last keyframe to `to`).
     - [64.1, 140]: the look-at point equals vehicle 1's placed point from `boxes_at(t)`;
       yaw is 90 and pitch 45 exactly; `height_m` decreases strictly from 120 to 60.
     - (147.1, 160): vehicle 1 is not drawn, and the look-at point lies on the segment
       from its last placed point to (600, 300), to 1e-9 m.
     - At each keyframe in the window, the look-at point's one-frame steps just before and
       just after differ by at most 0.1 m. Predicted largest: about 0.02 m. A blend's
       acceleration at its ends is `6·L/T²` (9.0 m/s² for 50 → 64.1 s, about 298 m in
       14.1 s), plus the vehicle's own, at most 6 m/s² (Phase 2 gate 7), so the steps
       differ by about 15·`h²` = 0.017 m. Each scalar stays between its segment's
       keyframes: 0 violations.
  - **The pose, through the GPU:**
  8. **Placement.** Phase 2 gate 3's single-vehicle subset for vehicle 1, rendered at
     `t` = 64.1 at 3840×2160 through `prepare_with_camera` with one keyframe at the look-at
     (−150, 500), `height_m` = 400. The weighted centroid of the vehicle's pixels (Phase 1
     gate 3's method, against the same pose's empty frame) is within **6 px** of `project`
     of its placed point at `z` = 0.80. Predictions, from the placed point
     (−0.649, 598.000) (±0.1 px for its rounding):
     - yaw 0, pitch 90: (2727.83, 549.92);
     - yaw 90, pitch 90: (1389.92, 272.17);
     - yaw 30, pitch 40: (2266.51, 635.05).

     Why 6 px: the silhouette's centroid is not the projected centre of a box seen from an
     angle, and at 5.4 px per metre a 0.5 m bias is 3 px. Each error this gate exists to
     catch is larger: the yaw's sign (727 and 1 932 px), 5° of pitch (18–53 px), a 60°
     field of view (41 px at the tilt).
  9. **Old lines frame the same ground.** The roads with no vehicles, at 1920×1080: the
     orthographic job's `render_empty()`, and a perspective job with one keyframe at the
     fit's centre (to 2 decimals, as a line carries it), `height_m` = 1080·`k_fit`, yaw 0,
     pitch 90. Every pixel that differs is on a road edge: its 3×3 neighbourhood in the
     orthographic frame holds both road and background. Prediction: at most 2 % of the
     frame, which is about every edge pixel; the count is recorded.
  10. **A keyframed render.** `render --camera tests/flight.toml` with the other defaults:
      `ffprobe` gives `1920,1080,30/1,8700`, and stderr is 8700 progress lines and `done`.
      Rendering it twice gives equal `framemd5`: **8700 of 8700**, with the 0.001 m lift
      (§2.11.2). The wall time is recorded, not predicted (Phase 4: 144 s).
  - **The view, headless:**
  11. **Orbit, tilt and the generalised camera,** on the plain state:
      - a right press at (640, 360), moved to (740, 300): yaw 335 and pitch 75 exactly;
        the centre (0, 0) and `k` = 1 exactly unchanged. A right drag down from pitch 90
        stays at 90;
      - from the default pose: `E` gives 15; from 350, `E` gives 5 exactly; `Q` undoes
        either. `R` 13 times gives 25 exactly, and a 14th leaves it; `F` 13 times gives
        90. During an orbit, `Q E R F` change nothing;
      - a right press in the slider's hit area, at (640, 706), then moved: yaw and pitch
        unchanged. A right press and release over a box picks nothing;
      - following a picked box, an orbit and each key step leave the follow on;
      - **Ctrl + left-drag** (Control held in the press's frame): a press at (640, 360),
        moved to (740, 300), gives yaw 335 and pitch 75 exactly, the centre (0, 0) and
        `k` = 1 exactly, and no `Press`; releasing Control in the next frame while moving
        on to (740, 300) changes nothing: the orbit goes on to the release. A plain left
        press at (640, 360) dragged to (740, 300), with Control pressed only after the
        press, pans: the centre is (−100, −60) and yaw and pitch stay 0 and 90;
      - Ctrl + left over a box: on the plain state a box (4.5 m, heading 0°) at (0, 0)
        under (640, 360); a Ctrl press and release there picks nothing (`follow` stays
        `None`); following it (picked by a plain click), a Ctrl + left orbit leaves the
        follow on and the centre on the box;
      - Ctrl + left on the slider: a press at (640, 706), moved to (900, 706), then
        released: `t` stays 0, no scrub starts, the clock stays paused, and yaw and pitch
        stay 0 and 90; the same while playing leaves it playing;
      - during a right-button orbit a Ctrl + left press starts nothing, and the orbit ends
        on the right release;
      - at yaw 30, pitch 40: a left drag from (640, 360) to (740, 300) moves the centre to
        (−145.222180146317, −33.60236249663467), and the ground point under the cursor is
        (0, 0), the one under the press. 5 scroll lines at (800, 300) give `k` =
        1.1^−5, the centre (76.5140026126445, 0.3460562657681663), and leave the cursor
        over (201.84201134738174, 0.9128877112287341);
      - at yaw 90, holding `W` for 30 frames of 1/60 s moves the centre to (320, 0), `cy`
        exactly 0;
      - **pick at a tilt:** yaw 30, pitch 45, `k` = 0.05 (`height_m` = 36), one box (4.5 m,
        heading 120°) at the look-at (0, 0). A click at (640, 337.5125250710996), where its
        roof centre projects, picks it: the click's point at 0.80 m is (0.395, 0.684),
        0.79 m from the centre across the box, inside its 0.9 m half-width. The ground point
        (0.816, 1.414) would be 0.73 m outside the footprint, beyond the 0.4 m radius, so a
        ground-plane pick fails this case.
  12. **The keyframe line.** After gate 11's orbit (`t` = 0):
      `{ t = 0.000, x = 0.00, y = 0.00, height_m = 720.00, yaw_deg = 335.00, pitch_deg = 75.00 }`.
      Phase 3 gate 9's states print
      `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }`
      and `{ t = 200.000, follow = 103, height_m = 60.00, yaw_deg = 0.00, pitch_deg = 90.00 }`.
      Every line of gates 11–12 satisfies `format(parse(line)) == line` and gives back the
      pose to within half its last decimal. Wrapped in `keyframes = [ … ]`, gate 11's lines
      read as a file with no syntax error.
  - **The user's check:**
  13. **The user flies the camera.** On the fixture:
      - in `view`: right-drag and Ctrl + left-drag on the trackpad to orbit and tilt,
        slowly and fast (Ctrl + left-drag must orbit, never open a menu or pan); a Ctrl +
        click on a box picks nothing, and on the slider does nothing; `Q`/`E`, `R`/`F`; pan,
        zoom and `WASD` at a tilt, with the point under the cursor staying put; click a box
        at a tilt and follow it through a turn while orbiting; press `K` and read the line;
      - write a keyframe file from `K` lines with at least one hold, one tilt and one
        follow (or start from `tests/flight.toml`), render it, and watch the MP4: the path
        is smooth, with no corner at a keyframe, eases in and out, holds dead still, and
        follows the vehicle;
      - watch Phase 4's default render once more, if wanted: it is the same file, hash for
        hash (gate 1).

      And say whether any number of §2.11.8 (h) or the face shading should change
      (iteration, §2.6), and answer OQ-10 and OQ-13.
- **Not predicted, and so not gated:**
  - the frame rate in perspective: `view --bench 20` at the default window, recorded at
    close-out (Phase 4: 60.00 fps, median 16.67 ms);
  - `render --camera`'s wall time (gate 10), and the clean build's (gate 3);
  - the silhouette bias of gate 8 beyond its bound, and the edge-pixel count of gate 9;
  - the feel of the orbit, the look of the shading and of the flights (gate 13).
- **Close-out (standing plan steps, §3 of the methodology):**
  - **Commit plan:** one branch (`phase-5`) and one push, with commits for the pose and the
    keyframes (gates 4–6 and 12's parsing), for `render --camera` (gates 1, 5, 7–10), for
    `view`'s orbit and tilt (gates 2, 11, 12), for the gate run and its record, and for the
    close-out.
  - **Reconciliation:**
    - a new `rules/camera.md` for §2.11 as built: the pose and projection, the keyframe
      line and file with its errors, the flight, and what each path draws; `sources`
      `src/camera.rs`, `src/keyframes.rs`, `src/draw.rs`;
    - `rules/render.md` (60/60) and `rules/view.md` (60/60) are at their caps, so they
      gain only `--camera`, the perspective path, the orbit keys, the new frame order and a
      pointer to `rules/camera.md`, reworded to fit; `rules/inputs.md` (50/50) gains the
      file's checks as one pointer. No `max_lines` is raised;
    - the README gains `--camera`, the file's format, and `view`'s orbit and keys;
    - no `CLAUDE.md` stanza change.
  - Record the gate results in `specs/reviews/vis-001.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.
