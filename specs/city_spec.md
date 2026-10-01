---
id: vis-002
title: city
note: >
  Real buildings around a georeferenced network, in render and view. Overture footprints are
  fetched once by a script into a cached GeoJSON, then projected with the engine's formula,
  extruded and lit. Phase 1 draws opaque grey blocks on the Midtown fixture.
status: draft
last_updated: 2026-10-01

phases:
  - name: "Phase 1 — Buildings: real blocks around the network, in render and view"
    reviewed: null
    shipped: null
    cut: null
    by: null

extends: null
supersedes: null
superseded_by: null
related: [vis-001]
reference: >
  Overture Maps, buildings theme, type building, on the public S3 bucket
  overturemaps-us-west-2 under release/<release>/theme=buildings/type=building/
  (GeoParquet, ODbL). Engine: ~/dev/main/assimilator at the pin of vis-001 §2.2.2 (df8aec0),
  read-only; its import projection and metadata.map_origin are what this spec relies on.
  The user's Midtown project is /Users/ivapo/assimilator/projects/midtown-section at
  e2de274, read-only. Out of scope from Overture: building_part, the base theme (land use,
  water), transportation (roads come from the network) and places.
---

# City

## 1. Goal

Put the real city around a run. A video of a Manhattan network with no buildings shows grey
strips on a dark plane. With the blocks that stand there, a viewer recognises the place, and
a flight between towers reads as the street it is.

**The observable is vis-001's: a video file rendered from one simulation run.** Here it is
the run's video with the real buildings around its roads, from `render --buildings`, with
or without a keyframed flight (vis-001 §2.11).

```
# Once, with network access. DuckDB CLI with httpfs and spatial. The project needs only its network.
scripts/fetch-buildings.sh --project <project> --out <dir>/buildings.geojson
{"release": "2026-09-23.1", "bbox": [-73.9938413, 40.7544211, -73.9643086, 40.7740495],
 "buildings": 4336, "height": 4277, "num_floors": 8, "default": 51, "bytes": 2140989, ...}

# Offline from here on.
assimilator-video view   --project <project> --scenario baseline --seed 42 \
                         --buildings <dir>/buildings.geojson          # B hides and shows them
assimilator-video render --project <project> --scenario baseline --seed 42 \
                         --buildings <dir>/buildings.geojson --camera flight.toml --out city.mp4
```

(The numbers are the Midtown fixture's, §2.9, measured while drafting, §2.12. The report
is one line; it is wrapped here.)

Rejected candidates for the observable:
- *`buildings.geojson`.* It is an input to a video, as vis-001 §1 says of the scene bundle.
- *The `view` window with buildings.* A tool for writing flights, as in vis-001 §2.9.

### 1.1 Non-goals

- **No generated buildings, ever** (§2.2 a). A synthetic network (urban_grid) gets none: it
  has no `metadata.map_origin`, so `--buildings` on it is an error (§2.4.3).
- **No other source** than Overture's `building` type: no OSM directly, NYC Open Data or
  photogrammetry (roadmap, §2.13).
- **Not in Phase 1:**
  - building parts (towers on podiums), raised bases (`min_height`), roof shapes and
    textures;
  - shadows; see-through or fading buildings;
  - land use, water, terrain, sky and fog.
- **No network call in `render` or `view`.** No Overture reading in Rust: no parquet or
  arrow for it, and no new crate for fetching.
- **No collision.** The camera may pass into a building. Buildings do not block a pick or a
  pan.
- **No change to the pitch floor.** It stays 25° (vis-001 §2.11.1); vis-001 OQ-12 stays open
  (§2.2 g).
- **Nothing derived from OSM or Overture is committed** to this repo (§2.10).
- **Not `prepare`, chunked meshes or the scene bundle** of vis-001 §2.7 item 4, which this
  spec narrows (§2.1).

## 2. Design

### 2.1 A spec of its own (decision, recorded)

Decided by the user, 2026-09-30, and checked against the methodology's §6.1:
- **Step 0:** decisions change: a new input from outside the run, a fetch step, a projection
  and a licence.
- **Step 1:** nothing shipped is removed. vis-001's roadmap item 4 was never a phase.
- **Step 2:** vis-001 owns *rendering one run's files* (vis-001 §2.1: "Inputs are a run's
  files, read in place"). Buildings are not a run file. They come from a third-party dataset
  under its own licence, through a fetch with network access, a new external tool (DuckDB),
  a cache and a georeference. The work also spans five places at once: a fetch script,
  inputs, scene, `render` and `view`. So it is a new spec, `extends: null`, `related:
  [vis-001]`.

vis-001 §2.7 item 4 ("`prepare`: Overture buildings and land use for the network area,
chunked meshes, a cached scene bundle") is **narrowed** here to buildings via a fetch
script. It gets one dated note pointing here (2026-10-01). `prepare`, land use, chunked
meshes and the bundle are either later (§2.13) or dropped: §2.6 shows Midtown needs no
chunks.

### 2.2 The user's decisions (decision, recorded)

Decided by the user, 2026-10-01, before drafting.
- (a) **Real buildings only, from Overture only.** Overture's buildings layer already
  conflates OSM and ML-derived footprints. Never generated buildings; synthetic networks get
  none.
- (b) **Fetch:** `scripts/fetch-buildings.sh` queries Overture's public S3 release with the
  DuckDB CLI (1.5.1 installed) and a bbox filter, and writes `buildings.geojson`. The release
  is pinned. No new Rust crate for fetching; no parquet or arrow. Prefer the release the roads
  came from, if it is still on S3. It is, so the pin is `2026-09-23.1` (§2.3.3).
- (c) **Cache:** `buildings.geojson` sits beside the frozen fixture, in raw lng/lat, with
  Overture's `id`, `height` and `num_floors` kept. `render` and `view` read it with
  `--buildings <file>`. The visualizer projects it with the engine's formula
  (equirectangular about `metadata.map_origin`, §2.5), read with `serde_json`. The formula
  is copied, not taken from the engine's crate (j).
- (d) **Height:** `height`, else `num_floors` × 3.5 m, else a fixed 10 m. The fetch script
  reports how many buildings take each rule.
- (e) **Extent:** the network's bbox plus a margin, a flag defaulting to 250 m. Every
  fetched building is drawn. The basis of the margin is §2.3.1.
- (f) **Look:**
  - opaque grey blocks, roofs lighter than walls, one fixed sun, basic lighting;
  - `B` toggles buildings in `view`; `render` draws them only with `--buildings`;
  - `render` without `--buildings` stays byte-identical: 8700 of 8700 against
    `scratch/ref-8eb9052.framemd5`, as `scripts/gates.sh` checks;
  - see-through or fading buildings, land use and textures are later phases.
- (g) **Pitch:** the 25° floor stays. vis-001 OQ-12 stays open. It belongs to a later phase
  here that brings sky, fog and a wider extent (§2.13).
- (h) **Fixture:** the Midtown 2850 copy (demand rates halved), built by a fixture script
  into gitignored `scratch/`:
  - from the user's project at `e2de274`, by read-only `git archive`;
  - run at the pin with `scratch/engine`;
  - then the buildings fetched once. After that, gates run offline.

  Nothing derived from OSM or Overture (ODbL) is committed to this MIT/Apache repo.
- (i) **vis-001 roadmap item 4** is narrowed by vis-002 to buildings via a fetch script, with
  one dated note there (§2.1).

Decided by the user, 2026-10-01, on the draft, before review round 1 (the answers to OQ-1,
OQ-2, OQ-3 and OQ-6; OQ-4 and OQ-5 stay open):
- (j) **The projection is copied:** the engine's three lines, in `src/buildings.rs`, pinned
  bit for bit by gate 6. No dependency on `assimilator-import` (§2.5.1).
- (k) **Roofs are triangulated with the `earcut` crate**, +1 package (§2.6, §2.11).
- (l) **The ODbL credit goes in both places, split** (§2.10):
  - Phase 1's close-out adds it to the README;
  - a credit line in the video is a small phase of its own (§2.13), which must ship before
    any Midtown video is shown outside. Phase 1's scope does not change.
- (m) **No copy of the cache is kept elsewhere** (§2.3.3). If `2026-09-23.1` is gone from
  S3 and `scratch/` is lost, a newer release is pinned and the predictions are made again,
  with the change recorded and nothing rewritten.

The rest of this section is the draft's proposal: the margin basis, the report, the mesh,
the sun, the checks, the fixture steps and the gates.

### 2.3 Where buildings come from: the fetch

```
scripts/fetch-buildings.sh --project <dir> --out <file.geojson>
                           [--scenario baseline] [--margin 250] [--release 2026-09-23.1]
```

It is the only step that uses the network. It needs the DuckDB CLI (≥ 1.5.1; `INSTALL`s
`httpfs` and `spatial` on first use) and a build of this repo, for the extent. It never
writes into `--project`.

#### 2.3.1 The extent: the network's own, not the import's

The margin is applied to **the network's own extent**: every node point and every link
`geometry` point of the scenario's resolved network (vis-001 §2.1, overrides applied), in
metres. The import bbox is not used. Why:
- **A network is often a cut, then an edit, of an import.** Midtown's import (1,369 links)
  was cut to 255 links. The user's edits moved its west endpoint, N940, to x = −1126.5 m:
  78 m *outside* the import bbox (x ±1048.5 m). The import bbox plus 250 m would leave
  172 m on that side and 174–185 m too much on the others.
- **Not every network has an import report.** One drawn by hand has only `map_origin`.
- **The network is what the video shows.** The resolved network is also what `render`
  draws, overrides applied.

For Midtown the node extent and the extent with every geometry point are the same: x
−1126.5…863.5 m, y −896.5…788.5 m (1990 × 1685 m). With 250 m that is 2490 × 2185 m.

The extent is turned into lng/lat with the inverse of the projection (§2.5), and rounded
**outward** to 7 decimals so the margin is never short:
- west and south take the floor of value × 10⁷, east and north the ceiling;
- Midtown at 250 m: `-73.9938413 40.7544211 -73.9643086 40.7740495` (W S E N).

A small binary of this repo, `network-extent` (`src/bin/network-extent.rs`), prints that
line:
- arguments: `--project`, `--scenario` (default `baseline`), `--margin` (default 250);
- it uses `src/inputs.rs:load_network`, as `render` does, and adds no crate;
- an error is one line and exit 1. A network with no `map_origin` is one.

#### 2.3.2 The query

```sql
INSTALL httpfs; INSTALL spatial; LOAD httpfs; LOAD spatial;
SET s3_region = 'us-west-2';
SET geometry_always_xy = true;   -- [lng, lat]; silences DuckDB's axis-order warning
COPY (
  SELECT id, height, num_floors, geometry
  FROM read_parquet('s3://overturemaps-us-west-2/release/<release>/theme=buildings/type=building/*.parquet')
  WHERE bbox.xmin <= <E> AND bbox.xmax >= <W> AND bbox.ymin <= <N> AND bbox.ymax >= <S>
  ORDER BY id
) TO '<tmp>/buildings.geojson' WITH (FORMAT GDAL, DRIVER 'GeoJSON');
```

- **Intersects, not contains.** A building that straddles the margin is fetched whole, never
  clipped.
- **`ORDER BY id`** fixes the order, so the same release and bbox give the same bytes (§2.12:
  three fetches, one SHA-256).
- **The file name is fixed.** GDAL writes the output file's stem as the collection's `name`.
  So the script writes `buildings.geojson` in a temporary directory beside `--out`, then
  moves it. The bytes do not depend on `--out`, and a failed fetch leaves nothing at `--out`.
- The output is a GeoJSON `FeatureCollection` in CRS84 (`[lng, lat]`), 7 decimals, with
  properties `id`, `height` and `num_floors`.

#### 2.3.3 The release

The default `--release` is **`2026-09-23.1`**, the release Midtown's roads were imported
from (`import_report.json`, `source.release`). On 2026-10-01 S3 lists three releases:
`2026-08-19.0`, `2026-09-23.0` and `2026-09-23.1`. The buildings layer is 512 parquet files.

So Overture keeps only recent releases, and **the cache will outlive its release**:
- once `2026-09-23.1` is gone, a lost cache cannot be fetched again;
- a new release changes every count in Phase 1's gates;
- the fixture therefore never deletes `buildings.geojson` unless asked (§2.9). No other copy
  is kept (§2.2 m). If the release is gone and `scratch/` is lost, a newer release is
  pinned and the predictions are made again, recorded as a change and never rewritten;
- a release that is not on S3 is an error naming it, exit non-zero, no file at `--out`.

#### 2.3.4 The report

When the file is written, the script prints one JSON line on stdout. The counts come from
DuckDB over the written file, with the height rule of §2.4.2:

```json
{"release": "2026-09-23.1", "bbox": [-73.9938413, 40.7544211, -73.9643086, 40.7740495],
 "buildings": 4336, "height": 4277, "num_floors": 8, "default": 51,
 "bytes": 2140989, "seconds": 122, "out": "<file>"}
```

- `height`: the buildings with a `height`.
- `num_floors`: the buildings with no `height` but `num_floors`.
- `default`: the buildings with neither.

An error is one line on stderr and a non-zero exit, as in vis-001 §2.4. DuckDB's own output
is kept off both streams unless it fails.

### 2.4 The cache file and how it is read

#### 2.4.1 Format

The file is GeoJSON, read with `serde_json`, which is already a dependency.
- **The top level** is a `FeatureCollection` with a `features` array. Other members (`name`,
  `crs`) are ignored. Coordinates are taken as `[lng, lat]` in degrees.
- **Each feature** has:
  - `properties.id`, a string;
  - `properties.height`, a number or null or absent;
  - `properties.num_floors`, an integer or null or absent;
  - a `geometry` that is a `Polygon` or a `MultiPolygon`.

  Other properties are ignored. A position may carry a third number, which is ignored.
- **A building** is one feature. A `MultiPolygon` is one building of several polygons, each
  with its own walls and roof.

Every feature is drawn: none is skipped, clipped or merged. Overlapping footprints are drawn
as they are.

#### 2.4.2 Height

- `height`, if present and not null;
- else `num_floors` × 3.5 m (`FLOOR_HEIGHT_M`);
- else 10 m (`DEFAULT_HEIGHT_M`).

`height` wins over `num_floors` when both are present. For Midtown: 4,277 by `height`, 8 by
`num_floors` (1, 1, 1, 1, 3, 4, 6 and 10 floors), and 51 at 10 m. Heights run from 1.0 m to
472.0 m, median 18.5 m, p99 186.2 m.

#### 2.4.3 Errors

The file is read after the run is loaded: it needs the network's `map_origin` and extent.
It is read after `--camera`'s file (vis-001 §2.11.4) and before the first frame or the
window.

Each error is one line, `error: --buildings <file>: …`, with exit 1. There is no progress
line, no output file and no window (vis-001 §2.4). A feature is named by its `id`, or by its
index from 1 when it has none. The errors:
- the file is missing, or is not JSON (with serde's line and column);
- it is not a `FeatureCollection` with a `features` array;
- a feature has no string `properties.id`;
- a feature's geometry is missing, or is not a `Polygon` or `MultiPolygon`;
- a ring has fewer than 4 positions, is not closed (first ≠ last), or has a position that is
  not two or more numbers;
- a longitude is outside [−180, 180] or a latitude outside [−90, 90]. This also catches a
  file written in metres;
- a `height` that is used is not a positive finite number, or a `num_floors` that is used is
  not an integer ≥ 1;
- the network has no `metadata.map_origin`: "buildings need a georeferenced network"
  (urban_grid);
- no feature's bounding box meets the network's extent (§2.3.1, no margin). This catches a
  file fetched for another project, and an empty `features`.

### 2.5 Projection: the engine's formula

The engine's import lays every node and link out with
`crates/import/src/common/geometry.rs:wgs84_to_metric(lat, lng, origin_lat, origin_lng)`:

```
x = (lng − origin_lng) · 111320 · cos(origin_lat · π / 180)
y = (lat − origin_lat) · 111320
```

It writes the origin into the network as `metadata.map_origin = [origin_lng, origin_lat]`
(`crates/import/src/common/network_build.rs:build_network`). The origin is the import bbox's
centre (`crates/import/src/common/mod.rs:run_pipeline`).
- **Checked at the pin (`df8aec0`) on 2026-10-01.** The function is as above. Its file has
  one commit before the pin, `564500f1` (2026-04-08), and the function is the same on engine
  main (`be50b773`, 2026-10-01).
- **Midtown's `map_origin` is the centre of its `import_report.json` bbox** to the last
  printed digit: `[-73.9775152177763, 40.76472024499192]`.

The visualizer projects every position with exactly this formula, **in this operation
order**, so a result is bit-identical to the engine's. The order matters:
`origin_lat * PI / 180.0` is not `to_radians()`, which multiplies by a rounded `π/180`.
Note the argument order, `(lat, lng)` against `map_origin`'s `[lng, lat]`. A swap is the
likeliest bug. Gate 6 catches it, and so would the reader's extent check (§2.4.3), since a
swap puts every building thousands of kilometres away.

The inverse (for the extent, §2.3.1) is `lng = origin_lng + x / (111320 · cos(origin_lat ·
π / 180))` and `lat = origin_lat + y / 111320`.

**Measured (§2.12): the formula lines buildings up with the network.**
- 2.239 % of the 255 links' centreline length (33,530 m) falls inside a footprint.
- Shifting the buildings by up to about ±6 m keeps the share at 1.9–2.3 %. That floor is
  roads that really pass under footprints, not an offset.
- At 10 m the share depends on the direction, because Midtown's avenues are wide and its
  grid is turned about 29°:
  - it rises fast north (7.5 %), south (14.5 %), north-east and south-west (38–39 %);
  - it rises less north-west and south-east (3.5–8.4 %), and hardly east or west (about 2 %).

  So this measure catches gross errors only. Gate 6 pins the formula's values, and gate 11
  checks the drawing to 2 px.
- Without the `cos(origin_lat)` factor the share is 48.0 %.

#### 2.5.1 The formula is copied, not a dependency (decision, recorded — OQ-1)

The engine function is in `assimilator-import`, a crate this repo does not depend on. It
uses `assimilator-config`, `-core` and `-geometry` (vis-001 §2.2.1). The two options the
draft weighed:

| | Depend on `assimilator-import` | Copy the three lines |
|---|---|---|
| `Cargo.lock` | +1 package (its other dependencies — config, serde, serde_json, anyhow, thiserror 2, log — are already in it) | 0 |
| Build | compiles the importer, 10,806 lines at the pin (OSM, Overture and HERE parsers, topology), for one 3-line function | 3 lines in `src/buildings.rs` |
| Coupling | a crate under active work: 9 commits to `crates/import` between the pin and engine main on 2026-10-01; a pin move may break the build there | none; gate 6 pins the values bit for bit |
| An engine change to the projection | followed at the next pin move | not followed |

**Decided by the user, 2026-10-01: copy, with gate 6** (§2.2 j). It is the draft's
recommendation. vis-001 §2.2.1 reused the placement because it is about 1,200 lines that a
port would drift from. These are 3 lines, unchanged since 2026-04-08.

"Followed at the next pin move" is worth less than it looks. What must match is the formula
that made **the project's** network when it was imported, not the formula at the pin.
Midtown was imported by the user's own build of the engine, not at the pin. If the engine
ever changed its projection:
- old projects would need the old formula and new ones the new;
- neither option knows which, because a network records its origin but not its projection;
- recording the projection in `metadata` would settle it. That is a request for the engine
  if it ever happens, not now.

### 2.6 The mesh

Built once, before the first frame or the window, from the projected footprints in metres:
- **Rings are oriented first:** the exterior counter-clockwise and the holes clockwise, seen
  from above (`x` east, `y` north). Midtown's exteriors are all counter-clockwise already,
  but the reader does not rely on it.
- **Walls:** one quad per ring edge, from `z` = 0 to the height, holes' rings included, so a
  courtyard has walls. Each quad has its own four vertices and one normal: the right of the
  ring's direction, which with that orientation points out of the solid.
- **Roof:** each polygon's rings are triangulated with the `earcut` crate (§2.2 k) at `z` =
  height. The normal is straight up.
- **No floor.** At pitch ≥ 25° it is never seen.

All buildings are one Bevy mesh, baked like the roads relative to the fit's centre `(fx,
fy)`. World `(x, y, z)` becomes Bevy `(x − fx, z, −(y − fy))`, a proper rotation, so
counter-clockwise from above stays counter-clockwise (vis-001 §2.11.1). Back faces are
culled, so the winding must be right; gates 9, 11 and 12 check it.

Midtown has 4,336 buildings, 4,373 rings (37 holes in 33 buildings) and 42,776 edges, none
of zero length. That gives:
- 42,776 wall quads (85,552 triangles, 171,104 vertices);
- at most 34,178 roof triangles: Σ (n + 2h − 2) over polygons of n vertices and h holes.
  The triangulator may drop collinear points.

About 120,000 triangles in all. No chunks: one mesh, always submitted, and frustum-culled as
a whole.

The camera fit does not change: `src/scene.rs:Camera` is still fitted to the road strips
alone, in `render` and in `view`'s launch. So every keyframe line frames what it framed
before, and buildings in the margin may lie outside the default top-down frame. "Every
fetched building is drawn" means it is in the scene, not in every frame.

### 2.7 Look and light

- **One material, lit:** neutral grey (R = G = B), matte (roughness 1, reflectance 0, not
  metallic), opaque.
- **One fixed sun:** a directional light from azimuth `SUN_AZIMUTH_DEG` = 210° (where the
  light comes from, clockwise from north) at elevation `SUN_ELEVATION_DEG` = 60°. It has no
  shadows. The ambient light is weaker than the sun.
- **Roofs are lighter than every wall by construction.** A roof gets the sun's `sin 60°` =
  0.866. A wall gets at most `cos 60°` = 0.5, and only when it faces the sun. Any elevation
  above 45° keeps every roof lighter. With 210°, a south wall gets 0.433 and a west wall
  0.25 (gate 12).
- **A building's pixel can never equal a road or background pixel.** Light and material are
  neutral, so its pixels are neutral grey. The road `#5c6068` and the background `#12161e`
  are not neutral. Gate 11 rests on this.
- **Roads and boxes stay unlit** (vis-001 §2.11.2). The light does not touch their pixels.
- **Without `--buildings`, nothing new is spawned:** no mesh and no light. That is what keeps
  `render` byte-identical (§2.2 f).

The base grey, the ambient level, the sun's illuminance and the exact angles are iteration
(vis-001 §2.6). The constraints above are not: neutral colour, elevation above 45°, and
nothing spawned without `--buildings`.

### 2.8 `render` and `view`

Both gain `--buildings <file.geojson>`. Without it, nothing changes.

`render`:
- **With or without `--camera`.** Without it the camera is vis-001's orthographic top-down
  fit, and only roofs show.
- **The orthographic camera rises above the tallest roof.** Today it sits 500 m up with a
  depth range of 1,000 m (`src/draw.rs:look_down`, `src/draw.rs:projection`), and a roof
  above 500 m would be behind it. With buildings, both grow by the tallest height (472 m for
  Midtown). Without buildings they are unchanged.
- **The perspective far plane covers the buildings.** It is `20·d` (vis-001 §2.11.1). That
  can cut off a distant tower in a low, zoomed shot, because towers rise into view above the
  ground's far edge. With buildings, `far` is the larger of `20·d` and the distance from the
  eye to the farthest corner of the buildings' 3D bounding box. Bevy's projection is
  reverse-Z with an infinite far plane, so `far` only culls. Without buildings it is
  unchanged.
- **The checks:** after `run::load` and `--camera`, before the encoder starts (§2.4.3).

`view`:
- **`B`** (by position, `KeyCode::KeyB`; unbound today) flips the buildings' visibility. They
  start shown. It works at any moment, including during an orbit, a drag or a scrub, and
  changes nothing else: not `t`, the clock, the pose, a follow or a drag.
- **Without `--buildings`, `B`** flips a flag that nothing reads.
- **Unchanged:** the readout, the keyframe line and the slider. `B`'s state is not in a `K`
  line: `render` takes `--buildings` as a flag.
- **The far plane** follows `render`'s rule, every frame.
- **Pick, pan and zoom** keep vis-001 §2.11.3's planes. A click on a roof may pick a box
  hidden under it, and the point under the cursor is the ground's, behind a building. `B`
  clears the view when that matters.

**The API.** `RenderOptions` and `LoadOptions` do not change: `tests/gates.rs`,
`tests/view.rs`, `tests/slider.rs` and `tests/camera.rs` build them field by field.
- `src/lib.rs:Job` gains `prepare_with(o, camera, buildings)`, both optional.
  `Job::prepare(o)` is `prepare_with(o, None, None)`, and `Job::prepare_with_camera(o, c)` is
  `prepare_with(o, Some(c), None)`.
- `ViewOptions` gains `buildings`; no test builds it.

### 2.9 The Midtown fixture

The user's project `midtown-section` at `e2de274`: an Overture import of release
`2026-09-23.1`, cut and edited by the user. It has 148 nodes, 255 links and 106 junctions
(83 signalised, with plans), and 65 boundary-to-boundary Poisson flows (5,650 veh/h), with
1,200 s simulated and 180 s of warm-up. The user chose its 2850 copy (§2.2 h).

**`scripts/fixture.sh midtown`.** Today's script, with no argument, still builds
urban_grid. With `midtown` it:
1. reads the pin and builds the engine CLI into `scratch/engine`, as today (vis-001 Phase 1);
2. exports the project with `git -C "$MIDTOWN_PROJECT" archive e2de274` into
   `scratch/midtown/`. `MIDTOWN_PROJECT` defaults to
   `$HOME/assimilator/projects/midtown-section`, and `git archive` only reads it;
3. **halves the demand.** Each `rate: r` line of `demand.yaml` becomes `r/2` rounded to a
   multiple of 10, with ties to even: 30→20, 50→20, 60→30, 80→40, 100→50, 160→80, 200→100,
   230→120, 290→140. No other line changes. The 65 rates sum to 5,650 and then to 2,850, and
   the script checks that total. This is yesterday's `scratch/midtown-2850/demand.yaml`, byte
   for byte (§2.12);
4. runs `scratch/engine/bin/assimilator run -c scratch/midtown -s baseline -o
   scratch/midtown/results.db --set simulation.output.fcd.enabled=true`, about 8 minutes;
5. fetches `scratch/midtown/buildings.geojson` with `scripts/fetch-buildings.sh` (defaults),
   **only if it is absent or `REFETCH=1`**. Its report line goes to
   `scratch/midtown/fetch.log`.

`FORCE=1` redoes steps 1–4 and **keeps `buildings.geojson`**, because the release may be
gone (§2.3.3). After step 5 every gate runs offline.

**The traffic gridlocks** at the pin, at both demand levels: an engine fault, asked of the
engine and not yet answered (OQ-4). Buildings need only the network's geometry and
`map_origin`, so vis-002 does not wait. When the engine is fixed:
- the user moves the pin (vis-001 §2.2.2);
- every gate of vis-001 and vis-002 is re-run;
- the fixture's traffic is frozen again with `FORCE=1 scripts/fixture.sh midtown`, which
  keeps the buildings.

**Who can run these gates:** only this machine. They need the private engine (vis-001
§2.2.1) and the user's own project.

### 2.10 Licence: nothing derived from OSM or Overture is committed

Overture's buildings are ODbL, and so are Midtown's roads (Overture transportation). This
repo is MIT OR Apache-2.0. So:
- the fixture's network, demand, run and buildings stay in gitignored `scratch/`;
- a user's own cache lives wherever they put it, never in the repo;
- the tests' own GeoJSON (`tests/shapes.geojson`) is six hand-written shapes, not data from
  any map (Phase 1);
- the spec and its review record quote counts, a few Overture ids and measured numbers, not
  data.

The credit goes in both places (§2.2 l). Phase 1's close-out adds it to the README. A
credit line in the video is a small phase of its own (§2.13), which must ship before any
Midtown video is shown outside. The wording is the user's to set.

### 2.11 Build cost

- **No crate for fetching.** DuckDB is an external CLI, needed only by the fetch script.
- **Reading:** `serde_json` and `serde`'s `derive`, both already dependencies; 0 packages.
- **Triangulation:** `earcut` 0.4.11 (georust; MIT OR Apache-2.0; a port of Mapbox's
  earcut; holes supported). Its one dependency, `num-traits` 0.2, is already compiled
  (through `approx` and arrow). So it is **+1 package and +1 compiled crate** (§2.2 k).
  `bevy_mesh` 0.19.1 meshes a `ConvexPolygon` but has no triangulator for concave polygons
  or holes.
- **The projection:** copied, so 0 packages (§2.2 j). Depending on `assimilator-import`
  would have added 1 (§2.5.1).
- **Bevy: no feature added.** `DirectionalLight` and `GlobalAmbientLight` are in
  `bevy_light`, already built through `bevy_pbr`.
- **`network-extent`** is a binary of this crate: no package.

So `Cargo.lock` gains **1 package** and a clean release build compiles **363 crates** (362
at vis-001 Phase 5).

### 2.12 Measured while drafting

Every number above that is not cited to source was measured on 2026-10-01. That covers the
counts, coverage, file size, timings, alignment, extents and hashes. The record is
`specs/reviews/vis-002.md`, "Draft". It covers the method and the queries, all run
read-only against the pin and the Midtown project, with output only under `scratch/`.
Nothing was built.

### 2.13 Roadmap (not yet phases)

Each item becomes a phase appended here when it is drafted and reviewed. Each ends with a
video, unless it says why not. The order is the user's to set, except for the first item's
constraint.
- **An ODbL credit line in the video** (§2.2 l). It is a small phase of its own, and it
  **must ship before any Midtown video is shown outside**. This machine's ffmpeg has no
  `drawtext`, so it is Bevy text in the headless render (OQ-3), and `render` without
  `--buildings` stays byte-identical.
- **Nicer buildings:**
  - Overture `building_part` (towers on podiums; OQ-5) and raised bases;
  - see-through or fading buildings near the camera or around a followed vehicle (§2.2 f);
  - later sources: NYC Open Data footprints (measured roof heights, NYC only), and Google
    Photorealistic 3D Tiles (paid; its terms for video to be checked).
- **Ground:** land use and water from Overture's `base` theme.
- **Sky, fog and a wider extent**, where vis-001 OQ-12 (a pitch floor below 25°) is decided
  (§2.2 g).
- **Re-frozen traffic** after the engine fix (OQ-4). This is a pin move with a re-gate
  (vis-001 §2.2.2), not a phase.

## 3. Open questions

- **OQ-1** — Depend on `assimilator-import` for the projection, or copy its three lines?
  **RESOLVED.**
  - *Costs* are in §2.5.1: +1 package, a 10.8k-line crate compiled, and coupling to the
    importer, against a copy pinned by a bit-exact test.
  - *Recommendation:* copy.
  - ~~*(design call: the user; blocks Phase 1's `Cargo.toml` and gate 6's form. Either way
    gate 6's values hold.)*~~
  - *(answered 2026-10-01, user)* **Copy the three lines.** Gate 6's bit-exact test pins
    the values, and there is no dependency on `assimilator-import`. Recorded as §2.2 j and
    §2.5.1.
- **OQ-2** — Triangulate roofs with the `earcut` crate, or by hand? **RESOLVED.**
  - *Earcut* adds 1 package; its only dependency is already compiled.
  - *By hand:* 0 packages, but ear clipping with hole bridging is about 150–250 lines, and
    hole bridging is where hand-written triangulators fail. Bevy has none for concave
    polygons (§2.11). 33 of Midtown's footprints have holes, and many are concave.
  - *Recommendation:* earcut.
  - ~~*(design call: the user; blocks Phase 1's `Cargo.toml`.)*~~
  - *(answered 2026-10-01, user)* **The `earcut` crate**, +1 package. Recorded as §2.2 k,
    §2.6 and §2.11.
- **OQ-3** — Where does the ODbL credit go: the README, the video, or both? **RESOLVED.**
  - *The licence:* Overture's buildings are ODbL, and so are Midtown's roads, so this
    concerns any video of an imported network, with or without buildings. ODbL 1.0 §4.3
    asks that a publicly used Produced Work carry a notice that makes its viewers aware of
    the source. Overture publishes attribution guidance per theme. The wording is the
    user's to set.
  - *README only:* no code, but the notice does not travel with the video.
  - *In the video:* a small credit line burned into every frame rendered with
    `--buildings`. This machine's ffmpeg 9.0.2 has no `drawtext` filter (no libfreetype),
    so it would be Bevy text in the headless path. `bevy_text` and `bevy_ui` are already
    built for `view`'s readout, but `render` has never drawn UI. Its gate would also keep
    `render` without `--buildings` byte-identical. About one small phase.
  - *Recommendation:* both. The README text goes in Phase 1's close-out, and the in-video
    line comes before any Midtown video is shown outside: as a phase of its own (§2.13), or
    added to Phase 1 if the user prefers.
  - ~~*(needs-input: the user; blocks Phase 1's close-out, and Phase 1's scope if it goes
    there.)*~~
  - *(answered 2026-10-01, user)* **Both, split.**
    - Phase 1's close-out adds the ODbL credit to the README.
    - The in-video credit line is its own small phase, listed in §2.13, and it must ship
      before any Midtown video is shown outside.
    - Phase 1's scope does not change.

    Recorded as §2.2 l, §2.10, §2.13 and Phase 1's close-out.
- **OQ-4** — Midtown gridlocks at the pin, at both demand levels. This looks like an engine
  fault.
  - *The symptom:* vehicles freeze at a link end, often just after a late lane change,
    while their FCD keeps reporting driving speed. Followers brake and queue. There are 21
    such cases in the 2850 run, 36 at 5650, and 0 in urban_grid.
  - *Where:* the worst sites are simple unsignalised diverges with no conflict pairs
    (N442, N115, N717). Engine `231ec605` fixes a similar symptom only for trips that end
    at a junction, and all of Midtown's destinations are boundary nodes.
  - *Status:* the user sent the question to the engine's orchestrator on 2026-09-30. It is
    saved there and, on 2026-10-01, queued and not started. The text is in
    `~/dev/ivapo/Orchtr-assimilator-visualizer/engine-question-midtown.md`.
  - *The plan* (§2.9): vis-002 does not wait. When the engine is fixed, the user moves the
    pin, every gate is re-run, and the fixture's traffic is frozen again.
  - *(needs-input: engine. It blocks nothing in Phase 1, whose gates measure geometry and
    the user's check expects the gridlock. It does block a presentable Midtown video.)*
- **OQ-5** — Should Overture's building parts come sooner?
  - *The data:* 487 of Midtown's 4,336 buildings (11 %) have `building_part`s. The
    `building` footprint carries the whole building's height, so a tower on a podium is
    drawn as one slab as tall as the tower. Two buildings have a raised base
    (`min_height` > 0), and are drawn from the ground.
  - *Whether that is acceptable* is for the user to see.
  - *(design call; deferred by evidence to Phase 1's gate 16; blocks nothing in Phase 1. If
    yes, it is the next phase.)*
- **OQ-6** — Where should a copy of the cache live once its release leaves S3?
  **RESOLVED.**
  - *Why it matters:* Overture keeps only recent releases (§2.3.3). If
    `scratch/midtown/buildings.geojson` is lost after `2026-09-23.1` is gone, Phase 1's
    predictions cannot be reproduced. A re-fetch from a newer release moves every count.
  - *Option (a):* accept it, and re-pin and re-predict when it happens.
  - *Option (b):* keep a copy outside this repo, which ODbL allows. One place is the user's
    own project or another private place; that is the user's to write.
  - *Recommendation:* (b) if there is such a place, else (a).
  - ~~*(design call: the user; non-blocking.)*~~
  - *(answered 2026-10-01, user)* **Option (a): keep no copy.** If `2026-09-23.1` is gone
    from S3 and `scratch/` is lost, re-pin a newer release and re-predict, recording the
    change and never rewriting. Recorded as §2.2 m and §2.3.3.

## 4. Implementation phases

Strictly sequential; each is one plan-mode pass. Each says whether it produces the
observable.

### Phase 1 — Buildings: real blocks around the network, in render and view
*Produces the observable: yes. `render --buildings` writes the run's video with the real
buildings around its roads. With `--camera`, the flight passes among them. Without
`--buildings`, `render`'s video is byte-identical to vis-001 Phase 5's (gate 1).*

Drafted 2026-10-01; the design is §2, and the user's decisions are §2.2. It builds on vis-001
Phase 5 (the perspective camera, the keyframe file) and changes none of its behaviour without
`--buildings`. The projection is copied (§2.2 j) and roofs are triangulated with `earcut`
(§2.2 k). Phase 1 adds the ODbL credit to the README only; the in-video credit is its own
phase (§2.2 l, §2.13).

- **Scope:**
  - **The fetch.**
    - `scripts/fetch-buildings.sh` (new), §2.3: flags, the query, the fixed collection
      name, the report, errors.
    - `src/bin/network-extent.rs` (new), §2.3.1.
  - **Reading and geometry (`src/buildings.rs`, new; no Bevy types).**
    - `lnglat_to_xy(origin, lng, lat)` and `xy_to_lnglat(origin, x, y)` (§2.5), with
      `origin` as `map_origin` (`[lng, lat]`);
    - `network_extent(&NetworkConfig)` and `fetch_bbox(&NetworkConfig, margin)` (§2.3.1);
    - `read(path, &NetworkConfig) -> Result<Buildings>`, with every check of §2.4.3 in the
      order listed. `Buildings` holds each building's `id`, its projected and oriented
      polygons, its height and the rule that gave it (§2.4.2), the counts per rule, the
      tallest height and the 3D bounding box;
    - `mesh_data(&Buildings)`: positions, normals and indices in world metres (§2.6), with
      the count of wall quads and roof triangles; roofs through `earcut`.
  - **Drawing (`src/draw.rs`).**
    - The Bevy mesh from that data, baked at `(fx, fy)`;
    - the lit material and the sun (§2.7), with `SUN_AZIMUTH_DEG`, `SUN_ELEVATION_DEG` and
      the ambient level as named constants;
    - `src/draw.rs:look_down` and `src/draw.rs:projection` take the extra height (§2.8),
      and the perspective `far` takes the buildings' box, both only when buildings are
      given.
  - **`render` (`src/render.rs`, `src/lib.rs`, `src/main.rs`).**
    - `--buildings <file>` on `render`;
    - `src/render.rs:Renderer` takes an optional building mesh and spawns it and the sun
      only then;
    - `Job::prepare_with` as in §2.8, `Job::buildings()` for the gates, and `prepare` and
      `prepare_with_camera` delegating, with their signatures unchanged;
    - the frame count, progress lines, ffmpeg output and every vis-001 check are unchanged.
  - **`view` (`src/view/state.rs`, `src/view/mod.rs`).**
    - `--buildings <file>` on `view`. `src/view/state.rs:Pressed` gains `b`, and
      `src/view/state.rs:ViewState` a `buildings_shown` flag, true at `new`, flipped by `b`;
    - the window reads `KeyCode::KeyB`, spawns the mesh and the sun when buildings are
      given, sets their visibility from the flag every frame, and applies §2.8's far plane;
    - nothing else in the frame order changes.
  - **The fixture.** `scripts/fixture.sh midtown`, §2.9.
  - **`Cargo.toml`.** `earcut` 0.4 (§2.11). No Bevy feature changes.
  - **Tests.**
    - `tests/buildings.rs` (new): gates 6–12 and 14. The headless ones (6, 7's library
      cases, 8 on the shapes, 14) need no fixture. Those that need the Midtown fixture or
      the GPU are `#[ignore]`d, like `tests/camera.rs`.
    - `tests/shapes.geojson` (new): six hand-written shapes, given in metres about
      Midtown's origin. They are written as lng/lat to 7 decimals through
      `xy_to_lnglat`:

      | `id` | shape (metres) | properties | height |
      |---|---|---|---|
      | `shape-cube` | square x −115…−85, y −15…15 | `height` 30 | 30 |
      | `shape-courtyard` | square x −20…20, y −20…20, hole x −8…8, y −8…8 | `height` 20 | 20 |
      | `shape-l` | square x 60…100, y −20…20 without x 80…100, y 0…20 | `num_floors` 4 | 14 |
      | `shape-default` | square x 120…130, y −5…5 | none | 10 |
      | `shape-both` | square x 140…150, y −5…5 | `height` 12, `num_floors` 10 | 12 |
      | `shape-multi` | `MultiPolygon`: x 160…170 and 180…190, y −5…5 | `height` 8 | 8 |

    - `tests/city-flight.toml` (new): gate 13's keyframes, in Midtown's metres:
      ```toml
      keyframes = [
        { t = 300.000, x = 0.00, y = 0.00, height_m = 1800.00 },
        { t = 330.000, x = -200.00, y = 300.00, height_m = 500.00, yaw_deg = 30.00, pitch_deg = 40.00 },
        { t = 360.000, x = 100.00, y = 500.00, height_m = 300.00, yaw_deg = 330.00, pitch_deg = 30.00 },
      ]
      ```
    - `scripts/gates-city.sh` (new): gates 3 and 4, 5's re-fetch (only with `REFETCH=1`,
      since it uses the network), 7's CLI cases and 13. `scripts/gates.sh` is not edited.
      Gate 15 is run by hand, as vis-001's `--bench` was.
    - `tests/gates.rs`, `tests/view.rs`, `tests/slider.rs` and `tests/camera.rs` are not
      edited.
- **Exit gate.** On the development machine (Apple M3, macOS), on two fixtures:
  - vis-001's urban_grid (engine `df8aec0`, baseline, seed 42), for what must not change;
  - Midtown (`scripts/fixture.sh midtown`, §2.9) for the rest.

  Gates 3–5 use the network: they are the fixture's fetch. Every other gate runs offline,
  after it. The predictions are §2.12's drafting measurements; nothing was built. Distances
  are compared to 1e-9 m, unless a gate says otherwise.
  - **What must not change:**
  1. **`render` without `--buildings`.**
     - `scripts/gates.sh` passes, and its reference comparison gives **8700 of 8700**
       against `scratch/ref-8eb9052.framemd5` (§2.2 f).
     - With `--include-ignored --test-threads=1`, `tests/gates.rs` (5), `tests/view.rs`
       (10), `tests/slider.rs` (8) and `tests/camera.rs` (19) pass, with none of those
       files edited.
  2. **Build cost** (§2.11).
     - `Cargo.lock` gains exactly **1 package** (`earcut`), and no Bevy feature changes.
     - `cargo tree -e normal` lists **no HTTP client** (reqwest, hyper, ureq, curl, isahc,
       attohttpc).
     - A clean release build, counted from its `Compiling` lines in a throwaway
       `CARGO_TARGET_DIR` under `scratch/` (vis-001 Phase 5 gate 3's method), compiles
       **363** crates.
     - The working `target/` growth is recorded. Prediction: under 0.2 GB.
  - **The fixture and the fetch (network):**
  3. **The fixture** (`scripts/fixture.sh midtown`).
     - `scratch/midtown/`'s `project.yaml`, `network.yaml` and `import_report.json` are
       byte-identical to `git show e2de274:<file>` of the user's project.
     - Its `demand.yaml` differs only on the 65 `rate:` lines, which sum to 2,850, and its
       SHA-256 is `66f54820d3981aa7e6deb8569b59558d137230bdbad09a4a6f827e2068d85d9a`
       (yesterday's copy).
     - The run's FCD has **280,872 rows** and **985 vehicles**, with `time` 1.1…1199.1 s.
       When `scratch/midtown-2850` exists, DuckDB `EXCEPT ALL` between the two FCDs gives 0
       rows both ways.
     - The engine run's wall time is recorded (468 s on 2026-09-30).
  4. **The extent.** `network-extent --project scratch/midtown` prints
     `-73.9938413 40.7544211 -73.9643086 40.7740495`, and with `--margin 0`
     `-73.9908762 40.7566669 -73.9672737 40.7718037`. On `scratch/urban_grid` it prints one
     error line naming `metadata.map_origin` and exits 1.
  5. **The fetch.**
     - The fixture's report line gives release `2026-09-23.1`, gate 4's bbox, **4,336**
       buildings: **4,277** by `height`, **8** by `num_floors`, **51** at the default. The
       file is **2,140,989 bytes**, SHA-256
       `561a615d6fa77bbef482b99f9f771b8b12f5891a46e661015e034b833d2bce47`.
     - A second fetch to another `--out` gives the same SHA-256.
     - `--release 2020-01-01.0` prints one error line naming it, exits non-zero and leaves
       no file at `--out`.
     - The fetch time is recorded. Drafting measured 122–145 s for the same query.
  - **Reading, projection and mesh — headless, offline:**
  6. **Projection** (§2.5). With Midtown's origin, `lnglat_to_xy` gives exactly (`==` as
     `f64`):
     - the origin → (0, 0);
     - the import bbox's south-west corner (lat 40.75608090194475, lng −73.9899513167745) →
       (−1048.5305648910369, −961.7316680111448);
     - its north-east corner (40.773359588039085, −73.96507911877812) →
       (1048.5305648898386, 961.7316680103539);
     - (lng −73.9740258, lat 40.7625626) → (294.2048943683543, −240.18904050031864).

     These are the engine formula's values in IEEE doubles, with its operation order (Python
     on this machine, which uses the same libm `cos` as Rust). `xy_to_lnglat` of each gives
     back the lng/lat to 1e-12°.
  7. **Reading errors** (§2.4.3).
     - Each case exits 1 with one stderr line beginning `error: --buildings`, with no
       progress line, no file at `--out` and no window. Each runs through `render`; the
       first, the map-origin case and the far-file case also run through `view`.
     - The cases:
       - a missing file; not JSON; a JSON array;
       - a `Feature` at the top level; a feature with no `id`; one with a `Point`
         geometry;
       - a ring of 3 positions; an unclosed ring; a position `["a", 40.7]`;
       - a latitude of 91; a file in metres (positions like `[294.2, -240.2]`);
       - `height` 0; `height` −5; a used `num_floors` of 0 and of 2.5;
       - `features: []`; one building at lng 0, lat 0 (far from the network);
       - `--buildings tests/shapes.geojson` on urban_grid (no `map_origin`).
     - Accepted:
       - `tests/shapes.geojson`;
       - a `[lng, lat, 0]` position;
       - extra properties and a `crs` member;
       - a `height` with an invalid `num_floors` beside it (not used).
  8. **Heights** (§2.4.2).
     - `tests/shapes.geojson` gives the table's heights. Its counts are 4 by `height`, 1 by
       `num_floors` and 1 at the default. `shape-multi` is one building of two polygons.
     - Midtown gives 4,277, 8 and 51, equal to the fetch report's.
  9. **The mesh, on Midtown's 4,336 buildings** (§2.6):
     - all **4,336** are meshed;
     - for each building, the roof triangles' total area equals its footprint's area
       (exterior minus holes, by the shoelace formula) to 1e-9 relative: **0 failures**;
     - wall quads: **42,776**, one per ring edge;
     - every wall's normal points out of the solid: a point 1 mm along it from the edge's
       midpoint is outside the footprint (outside the exterior, or inside a hole). **0
       inward**;
     - every roof vertex is at its building's height;
     - roof triangles at most 34,178, recorded.
  10. **Alignment** (§2.5). Of the 255 links' centreline length (raw `geometry`, 33,530.0 m),
      the share inside a projected footprint is **at most 3 %**.
      - The footprints are taken as a union: each point counts once, holes are outside.
      - Prediction: **2.239 %** (750.7 m; 23 links with more than 1 m inside).
      - For scale: a missing `cos(origin_lat)` gives 48.0 %, and buildings 10 m north give
        7.5 %. A lat/lng swap never gets here: the reader rejects it (§2.4.3).
  - **Through the GPU, offline:**
  11. **Top-down coverage.** Midtown at 3840×2160, the orthographic job (no `--camera`),
      with and without `--buildings`.
      - `Job::camera()` is equal for both, and `render_empty()` is taken of each.
      - An *edge pixel* has its centre within 2 px of a projected footprint boundary.
      - Every pixel that differs between the two frames is inside a footprint or is an edge
        pixel. Every pixel inside a footprint that is not an edge pixel differs.
      - Prediction: **0** violations of each. The building pixel count is recorded.
      - It checks the drawing against the library's footprints, not the projection (gate 6
        does that). It catches an offset of more than 2 px (about 1.6 m at this fit's
        scale), a flip, a filled courtyard, a missing roof triangle, and roofs culled by a
        wrong winding.
  12. **Shapes in perspective.** Midtown with `--buildings tests/shapes.geojson`, through a
      one-keyframe `--camera` file, `render_empty()` at 1920×1080. "Ground" means the pixel
      equals the same frame without `--buildings`.
      - **Roof over walls:** look-at (−100, 0), `height_m` 120, yaw 45, pitch 45. Take the
        3×3 mean of the red channel at `project` of the roof's centre (−100, 0, 30), of the
        south wall's centre (−100, −15, 15) and of the west wall's (−115, 0, 15).
        Prediction: **roof > south wall > west wall** (0.866, 0.433 and 0.25 of the sun,
        §2.7).
      - **Courtyard:** look-at (0, 0), pitch 90, `height_m` 60. The image centre is ground;
        `project` of (0, 14, 20) on the roof is not.
      - **The L's notch:** look-at (90, 10), pitch 90, `height_m` 60. The image centre is
        ground; `project` of (70, −10, 14) on the roof is not.
  13. **Deterministic and offline.**
      - `render --buildings scratch/midtown/buildings.geojson --camera tests/city-flight.toml
        --from 300 --to 360 --speedup 1`, twice: `ffprobe` gives `1920,1080,30/1,1800`, and
        `framemd5` is equal, **1800 of 1800**.
      - A third run under `sandbox-exec -p '(version 1)(allow default)(deny network*)'`
        completes, equal to both.
      - The same window rendered twice without `--camera` (orthographic, roofs only) is
        equal too.
  14. **`B`, headless.**
      - On the plain state of vis-001 Phase 3, `b` flips `buildings_shown` from true to
        false, and a second `b` flips it back.
      - `t`, the clock, the pose, `k` and `follow` stay exactly as they were.
      - During a right-button orbit, a left drag and a scrub, `b` flips the flag and the
        gesture goes on unchanged.
  15. **Frame rate.** `view --bench 20` on Midtown at the default window. With
      `--buildings`, the mean is **at least 30 fps**, vis-001's bar. Prediction: 60.00 fps
      with and without (vsync; urban_grid gave 60.00 at vis-001 Phases 4 and 5).
  - **The user's check:**
  16. **The user looks at Midtown with buildings:**
      - in `view` with `--buildings`, check that buildings line up with the roads and the
        junctions; orbit and tilt among them, high and at the 25° floor; `B` hides and shows
        them; roofs read lighter than walls;
      - write a flight from `K` lines (or start from `tests/city-flight.toml`), render it
        with `--buildings`, and watch the MP4: blocks are opaque, with no flicker, no
        missing roofs or walls, and courtyards open;
      - the gridlocked traffic is expected (OQ-4).

      And say whether the grey, the sun or the ambient should change (iteration, §2.7), and
      answer OQ-5 (building parts).
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `render` without `--buildings`, urban_grid | 8700 of 8700 frames equal to the reference | 1 |
  | `Cargo.lock` packages added; HTTP clients; release crates | 1 (`earcut`); 0; 363 | 2 |
  | `target/` growth | under 0.2 GB (recorded) | 2 |
  | Fixture FCD | 280,872 rows, 985 vehicles, 1.1…1199.1 s | 3 |
  | Fetch bbox (W S E N) | −73.9938413 40.7544211 −73.9643086 40.7740495 | 4, 5 |
  | Buildings: by height / by num_floors / at 10 m | 4,336: 4,277 / 8 / 51 | 5, 8 |
  | Cache file | 2,140,989 bytes, one SHA-256 on every fetch | 5 |
  | Fetch time | 122–145 s (recorded) | 5 |
  | Projection | 4 exact values | 6 |
  | Roof-area failures; inward walls; wall quads | 0; 0; 42,776 | 9 |
  | Centreline inside footprints | 2.239 % (bar 3 %) | 10 |
  | Top-down coverage violations | 0 and 0 | 11 |
  | Shape luminance | roof > south wall > west wall | 12 |
  | Keyframed render with buildings, twice and sandboxed | 1800 of 1800 | 13 |
  | `view` frame rate with buildings | 60.00 fps (bar 30) | 15 |
  | Midtown default render (9,000 frames), with / without buildings | at most 1.10× (recorded) | — |

- **Not predicted, and so not gated:**
  - the look: the grey, the ambient level, the sun's illuminance and the shading of the
    shapes beyond gate 12's order;
  - wall times: the fetch (gate 5), the engine run (gate 3), the clean build (gate 2) and
    the Midtown render with and without buildings, of which only the ratio is predicted.
    Other sessions load this machine, so times are recorded (vis-001);
  - the building pixel count (gate 11) and the roof triangle count (gate 9).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-1`), one push. The commits:
    - the fetch, `network-extent` and the Midtown fixture (gates 3–5);
    - reading, projection and mesh data (gates 6–10);
    - drawing in `render` and `view`, and `B` (gates 1, 2, 11–15);
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - a new `rules/buildings.md` for §2.3–§2.8 as built: the fetch and its report, the
      cache format and its checks, the projection, the height rule, the mesh, the light and
      `B`. Its `sources` are `src/buildings.rs`, `src/bin/network-extent.rs`,
      `scripts/fetch-buildings.sh` and `src/draw.rs`;
    - `rules/render.md` (60/60), `rules/view.md` (60/60), `rules/camera.md` (60/60) and
      `rules/inputs.md` (50/50) are at their caps. They gain only `--buildings`, `B`, the
      check's place in the order, the far plane and the orthographic height, as pointers
      to `rules/buildings.md`, reworded to fit. No `max_lines` is raised;
    - the README gains the fetch script with its prerequisites (DuckDB ≥ 1.5.1 with
      `httpfs` and `spatial`; the network, for the fetch only), `--buildings`, `B`, the
      Midtown fixture (it needs the user's project), and the ODbL credit (§2.2 l). The
      credit covers Overture's buildings and the roads of an imported network, in wording
      the user sets. The README also says that a credit line in the video is a later
      phase, and that it must ship before any Midtown video is shown outside;
    - `CLAUDE.md` gains one line beside the licence: nothing derived from OSM or Overture
      (ODbL) is committed; fetched buildings and imported networks stay out of the repo.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and its
    cause.
  - Write this phase's `shipped` date.
