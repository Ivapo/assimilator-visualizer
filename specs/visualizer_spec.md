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
  - name: "Phase 2 — Correct motion: smooth video from 1 Hz FCD"
    reviewed: 2026-09-29
    shipped: 2026-09-29
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

2. **Correct motion** — now Phase 2 (§2.8, §4). *Revised 2026-09-29 (user):* the bar
   is at least 30 fps and visibly smooth, not 60 fps; `--fps 60` stays available.
   Signal states moved to item 5.
3. **Camera** — two tools that share the reading, placement, interpolation (§2.8) and
   drawing code:
   - scripted camera moves for `render`, in `scene.toml`: keyframes, orbit, follow a
     vehicle, zoom to a junction, pan;
   - *(added 2026-09-29)* an interactive `view` command. It opens a Bevy window over a
     **finished** run, with a free camera and time scrubbing, and can save camera paths
     for `render`. Like the preview window of §1, it is a tool and not the observable.

   Watching a run live is out of scope: it needs the engine to stream (§1.1).
4. **City** — `prepare`: Overture buildings and land use for the network area, chunked
   meshes, a cached scene bundle; `render` makes no network calls.
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
