---
id: vis-002
title: city
note: >
  Real buildings around a georeferenced network, in render and view. Overture footprints are
  fetched once by a script into a cached GeoJSON, then projected with the engine's formula,
  extruded and lit. Phase 1 draws opaque grey blocks on the Midtown fixture. Phase 2 draws
  the data's credit line (OpenStreetMap, Overture, building sources) on every frame of an
  imported network's render. Phase 3 cuts the buildings between the camera and the point it
  looks at down to stubs, by default, in a keyframed render and in view;
  `--no-see-through` turns it off. Phase 4 draws a building that has Overture
  building_parts from its parts, each from its own base, so towers stand on podiums and
  raised bases stand above the street; a cache without parts draws as before. Phase 5
  draws the streets by default: junction surfaces in the road's grey, white lane lines,
  white stop lines at signals and a double yellow centre line on two-way streets, from
  the engine's own network data; `--no-streets` turns them off. Phase 6 draws the markings
  as the engine's dashboard does, in place of Phase 5's US set: its lane dashes, centre
  line, 0.4 m stop lines and lane arrows, straight from the engine's `NetworkJson`, with
  the dashboard's colours and arrow glyphs copied from its front end. Amended at its gate 14
  (2026-10-09): a two-way street's median gap shows dark, as on the dashboard, its junction
  ends rounded by the engine's median noses in the road's grey, and no median fill is drawn.
status: accepted
last_updated: 2026-10-09

phases:
  - name: "Phase 1 — Buildings: real blocks around the network, in render and view"
    reviewed: 2026-10-01
    shipped: 2026-10-02
    cut: null
    by: null
  - name: "Phase 2 — Credit: a line built from the data, on every frame of an imported network's render"
    reviewed: 2026-10-02
    shipped: 2026-10-03
    cut: null
    by: null
  - name: "Phase 3 — See-through: buildings in the way cut to stubs, in a keyframed render and in view"
    reviewed: 2026-10-03
    shipped: 2026-10-03
    cut: null
    by: null
  - name: "Phase 4 — Building parts: towers on podiums and raised bases, from Overture's parts"
    reviewed: 2026-10-04
    shipped: 2026-10-04
    cut: null
    by: null
  - name: "Phase 5 — Streets: junction surfaces, lane lines, stop lines and the yellow centre line, by default"
    reviewed: 2026-10-05
    shipped: 2026-10-08
    cut: null
    by: null
  - name: "Phase 6 — Engine look and lane arrows: the dashboard's markings, from NetworkJson, by default"
    reviewed: 2026-10-08
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
  (GeoParquet, ODbL). Engine: ~/dev/main/assimilator at the pin of vis-001 §2.2.2 (90b39292),
  read-only; its import projection and metadata.map_origin are what this spec relies on.
  The user's Midtown project is /Users/ivapo/assimilator/projects/midtown-section at
  e2de274, read-only. Out of scope from Overture: the base theme (land use, water),
  transportation (roads come from the network) and places. From Phase 4, type
  building_part too (theme=buildings/type=building_part/), whose release-2026-09-23.1
  rows for Midtown are saved in gitignored scratch/overture-2026-09-23.1/ (§2.16.10).
  Phase 2's credit follows Overture's attribution page
  (https://docs.overturemaps.org/attribution/) and OSMF's attribution guidelines
  (https://osmfoundation.org/wiki/Licence/Attribution_Guidelines), both read 2026-10-02,
  and draws with Fira Sans Medium (OFL 1.1, google/fonts). Phase 5's markings follow the
  MUTCD 2009 edition, part 3 (https://mutcd.fhwa.dot.gov/htm/2009/part3/part3a.htm and
  part3b.htm), and NYSDOT standard sheet 685-01, pavement marking details
  (https://www.dot.ny.gov/main/business-center/engineering/cadd-info/drawings/standard-sheets-us-repository/685-01_082718.pdf),
  both read 2026-10-05. Phase 6's markings are the engine's `NetworkJson`
  (crates/geometry/src/network_json.rs), and its colours and arrow glyphs are copied from
  the engine's web front end, web/src/canvas/colors.ts and
  web/src/canvas/networkRenderer.ts, read at 90b39292 on 2026-10-08.
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
is one line; it is wrapped here.) *(2026-10-02)* From Phase 2, the report also has
`sources` and `no_sources`, the file is 2,284,830 bytes, and the video carries the credit
line (§2.14). *(2026-10-03)* From Phase 3, the `render … --buildings … --camera` above cuts
the buildings between the camera and the point it looks at down to stubs, by default;
`--no-see-through` gives the video as before (§2.15). *(2026-10-04)* From Phase 4, the
fetch also writes each building's `building_part`s into the same file, and `render` and
`view` draw a building that has them from its parts; the report gains `raised` and `parts`,
and `--no-parts` writes the file as before (§2.16). *(2026-10-05)* From Phase 5, `render`
and `view` also draw the streets by default: junction surfaces, lane lines, stop lines at
signals and the yellow centre line; `--no-streets` gives the video as before (§2.17).
*(2026-10-08)* From Phase 6, the markings are the engine dashboard's instead: its lane
dashes, centre line, 0.4 m stop lines and lane arrows (§2.18).

Rejected candidates for the observable:
- *`buildings.geojson`.* It is an input to a video, as vis-001 §1 says of the scene bundle.
- *The `view` window with buildings.* A tool for writing flights, as in vis-001 §2.9.

### 1.1 Non-goals

- **No generated buildings, ever** (§2.2 a). A synthetic network (urban_grid) gets none: it
  has no `metadata.map_origin`, so `--buildings` on it is an error (§2.4.3).
- **No other source** than Overture's `building` type: no OSM directly, NYC Open Data or
  photogrammetry (roadmap, §2.13). *(2026-10-04)* From Phase 4, its `building_part` type
  too (§2.16).
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
script. It gets one dated note pointing here (2026-10-01). Land use is a roadmap item
(§2.13). `prepare`, chunked meshes and the scene bundle are dropped: the fetch script does
`prepare`'s job, and §2.6 shows Midtown needs no chunks.

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
    *(2026-10-02)* from Phase 2, that holds for urban_grid only: every render of a network
    with `map_origin` carries the credit line (§2.14.1 decision 2);
    *(2026-10-05)* from Phase 5, only with `--no-streets`: streets are drawn by default
    on every network (§2.17.12);
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
  - Phase 1's close-out adds it to the README, in the exact wording the user set in OQ-7
    (answered 2026-10-01);
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
`httpfs` and `spatial` on first use) and a build of this repo, for the extent. It only reads
`--project`, and writes nothing but `--out`, through a temporary directory beside it that it
removes. (The fixture's `--out` is inside its own project folder, §2.9.)

#### 2.3.1 The extent: the network's own, not the import's

The margin is applied to **the network's own extent**: every node point and every link
`geometry` point of the scenario's resolved network (vis-001 §2.1, overrides applied), in
metres. The import bbox is not used. Why:
- **A network is often a cut, then an edit, of an import.** Midtown's import (1,369 links)
  was cut to 255 links. The user's edits moved its west endpoint, N940, to x = −1126.5 m:
  78 m *outside* the import bbox (x ±1048.5 m). The import bbox plus 250 m would leave
  172 m on that side and 65–185 m too much on the others (south 65 m, north 173 m, east
  185 m).
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
  three fetches gave one SHA-256 once the collection's `name` was fixed).
- The drafting fetches ran without the `SET geometry_always_xy` line. Review round 1
  re-exported the cache through DuckDB 1.5.1 with and without it, and the bytes are the same.
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

*(2026-10-04)* From Phase 4, a feature with a string `building_id` is a part of that
building, a building that has parts is drawn from them and not from its own polygons, and
every volume has a base (§2.16.5). The checks of §2.4.3, the walls of §2.6 and `tallest` of
§2.8 change with it as §2.16.5–§2.16.6 say; a file without parts reads and draws as above.

#### 2.4.2 Height

- `height`, if present and not null;
- else `num_floors` × 3.5 m (`FLOOR_HEIGHT_M`);
- else 10 m (`DEFAULT_HEIGHT_M`).

`height` wins over `num_floors` when both are present. For Midtown: 4,277 by `height`, 8 by
`num_floors` (1, 1, 1, 1, 3, 4, 6 and 10 floors), and 51 at 10 m. The heights drawn run from
1.0 m to 472.0 m, median 18.4 m, p99 185.5 m (review round 1).

#### 2.4.3 Errors

The file is read after the run is loaded: it needs the network's `map_origin` and extent.
It is read after `--camera`'s file (vis-001 §2.11.4) and before the first frame or the
window.

Each error is one line, `error: --buildings <file>: …`, with exit 1. There is no progress
line, no output file and no window (vis-001 §2.4). A feature is named by its `id`, or by its
index from 1 when it has none. The checks run in the order below and stop at the first error:
the file-level ones, then each feature in file order with its own checks in order, then
`map_origin`, then the extent. The errors:
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
`origin_lat * PI / 180.0` is not `to_radians()`, which multiplies by a rounded `π/180`. At
Midtown's origin the two happen to agree, so gate 6 adds a made-up origin where they do not.
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
| `Cargo.lock` | +1 package, plus every optional dependency it declares that is not locked already: `reqwest` 0.12 with its HTTP stack, and `parquet` and `arrow-*` 54 beside the 60 already locked. A lock file lists optional dependencies whatever the features (vis-001 Phase 5's `toml_writer`). Corrected in review round 1; the draft said +1 | 0 |
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
  height. The normal is straight up. `Earcut::<f64>::earcut` takes the vertices without each
  ring's closing duplicate and the start index of each hole, counted in vertices. It emits
  counter-clockwise triangles (`y` up): 0 clockwise of Midtown's 34,178 (review round 1).
- **No floor.** At pitch ≥ 25° it is never seen.

All buildings are one Bevy mesh, baked like the roads relative to the fit's centre `(fx,
fy)`. World `(x, y, z)` becomes Bevy `(x − fx, z, −(y − fy))`, a proper rotation, so
counter-clockwise from above stays counter-clockwise (vis-001 §2.11.1). Back faces are
culled, so the winding must be right; gates 9, 11 and 12 check it.

Midtown has 4,336 buildings, 4,373 rings (37 holes in 33 buildings) and 42,776 edges, none
of zero length. That gives:
- 42,776 wall quads (85,552 triangles, 171,104 vertices);
- **34,178 roof triangles**: Σ (n + 2h − 2) over polygons of n distinct vertices (closing
  duplicates not counted) and h holes. earcut 0.4.11 on `f64` gives exactly that on Midtown,
  with no roof-area failure (worst 9.8e-12 relative), measured in review round 1.

About 120,000 triangles in all. No chunks: one mesh, always submitted. It carries Bevy's
`NoFrustumCulling`, so it is never culled (§2.8). *(2026-10-03)* From Phase 3, with
see-through on (the default in a keyframed render and in `view`), the mesh is rebuilt
whenever a building's cut height changes; it stays one mesh and one draw (§2.15.5).

The mesh data is in `f64` world metres. The Bevy mesh converts to `f32` only at the bake, as
`src/draw.rs:road_mesh` does.

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
- **The sun's direction.** For azimuth `a` and elevation `e`, the light travels along world
  `(−cos e·sin a, −cos e·cos a, −sin e)`, which is Bevy `(−cos e·sin a, −sin e,
  cos e·cos a)`. At 210° and 60° that is Bevy `(1/4, −√3/2, −√3/4)` ≈ `(0.25, −0.866,
  −0.433)`: north-east and down (gate 12).
- **Roofs are lighter than every wall.** By Lambert's law:
  - a roof gets the sun's `sin 60°` = 0.866;
  - a wall gets at most `cos 60°` = 0.5, and only when it faces the sun, so any elevation
    above 45° keeps every roof lighter;
  - with 210°, a south wall gets 0.433, a west wall 0.25 and an east wall nothing.

  Bevy's diffuse term is Burley's, which depends a little on the view. At gate 12's pose it
  gives about 0.88, 0.50 and 0.36, in the same order (review round 1). Gate 12 measures the
  order.
- **A building's pixel can never equal a road or background pixel.** Light and material are
  neutral, so its pixels are neutral grey. The road `#5c6068` and the background `#12161e`
  are not neutral. Gate 11 rests on this.
- **Roads and boxes stay unlit** (vis-001 §2.11.2). The light does not touch their pixels.
- **Without `--buildings`, nothing new is spawned:** no mesh and no light. That is what keeps
  `render` byte-identical (§2.2 f). *(2026-10-02)* From Phase 2, a network with `map_origin`
  also gets the credit's UI tree, with or without `--buildings` (§2.14.5). *(2026-10-05)*
  From Phase 5, every render also gets the streets' two meshes unless `--no-streets` is
  given (§2.17.11).

The base grey, the ambient level, the sun's illuminance and the exact angles are iteration
(vis-001 §2.6). These constraints are not:
- a neutral colour;
- an elevation above 45°;
- a roof below full white. The camera has `Tonemapping::None`, so a channel clips at 255,
  and a clipped roof and wall would compare equal in gate 12. Bevy's default sun of
  10,000 lux likely clips;
- nothing spawned without `--buildings`.

### 2.8 `render` and `view`

Both gain `--buildings <file.geojson>`. Without it, nothing changes.
*(2026-10-02)* From Phase 2, `render` of an imported network draws the credit line with or
without it, and `render --buildings` needs a cache fetched in Phase 2's form (§2.14).
*(2026-10-05)* From Phase 5, both draw the streets unless `--no-streets` is given (§2.17.12).

`render`:
- **With or without `--camera`.** Without it the camera is vis-001's orthographic top-down
  fit, and only roofs show.
- **The orthographic camera rises only for a roof that would reach it.** Today it sits
  500 m up with a depth range of 1,000 m (`src/draw.rs:look_down`,
  `src/draw.rs:projection`), and a roof above 500 m would be behind it.
  - With buildings, the eye is at `max(500, tallest + 10)` m and the depth range is the
    eye's height + 500 m. Without buildings, `tallest` is 0, so they are unchanged.
  - Midtown's tallest is 472 m, so its camera does not move. That matters: moving the eye
    moves every ground vertex by a few 1e-5 px, through `f32` rounding in `looking_at`
    (`m11` comes out 5.96e-8, not 0; review round 1). Along road edges that flips MSAA
    samples, and gate 11's two frames would differ where no building is.
- **The perspective camera is unchanged** (`src/draw.rs:perspective`, `far = 20·d`, vis-001
  §2.11.1). Bevy's projection is reverse-Z with an infinite far plane, so `far` only culls
  whole meshes. The building mesh carries `NoFrustumCulling` (§2.6), so no tower drops out
  of a low, zoomed shot.
- **The checks:** after `run::load` and `--camera`, before the encoder starts (§2.4.3).

`view`:
- **`B`** (by position, `KeyCode::KeyB`; unbound today) flips the buildings' visibility. They
  start shown. It works at any moment, including during an orbit, a drag or a scrub, and
  changes nothing else: not `t`, the clock, the pose, a follow or a drag.
- **Without `--buildings`, `B`** flips a flag that nothing reads.
- **Unchanged:** the readout, the keyframe line and the slider. `B`'s state is not in a `K`
  line: `render` takes `--buildings` as a flag.
- **The camera is unchanged.** `view` draws through `src/draw.rs:perspective` only, and the
  mesh carries `NoFrustumCulling`, as in `render`.
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

*(2026-10-04)* From Phase 4, step 5 passes `--no-parts` (and `--source` when the saved
mirror exists), a step 6 writes `buildings-parts.geojson`, and `FORCE=1` keeps both caches
and both logs (§2.16.3).

*(Note, 2026-10-01, Phase 1's gate run.)* The fixture's traffic cannot be reproduced at
the pin: four runs of the same Midtown inputs at `df8aec0` gave four FCDs (OQ-4). So the
run in `scratch/midtown` is the fixture, and `FORCE=1` gives different traffic. Gate 3's
FCD check is a recorded miss: `specs/reviews/vis-002.md`, "Phase 1 gate run".

**The traffic gridlocks** at the pin, at both demand levels: an engine fault, asked of the
engine and not yet answered (OQ-4). Buildings need only the network's geometry and
`map_origin`, so vis-002 does not wait. When the engine is fixed:
- the user moves the pin (vis-001 §2.2.2);
- every gate of vis-001 and vis-002 is re-run;
- the fixture's traffic is frozen again with `FORCE=1 scripts/fixture.sh midtown`, which
  keeps the buildings.

*(2026-10-04)* The engine has fixed it at `90331591` (asm-016 Phase 4, route repair, with
the approach index visited in a fixed order). The plan above is drafted as vis-001 Phase 7
(`specs/visualizer_spec.md` §2.13), which the user scheduled before Streets: the pin moves
`df8aec0` → `90b39292` (engine main after the hash-order audit, which contains route repair;
vis-001 §2.13.1 a′), both fixtures are re-frozen, and every gate of both specs is re-run
with its new numbers predicted. Midtown 2850 at the new pin gives the engine's FCD,
`01becc86…`, on every run. OQ-4 stays open until that phase ships (vis-001 §2.13.8).

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
Midtown video is shown outside. The README's wording is the user's, set in OQ-7 (answered
2026-10-01) and used exactly as written there.

*(2026-10-02)* The line in the video is drafted as Phase 2 (§2.14). It brings the first
file under another licence: a font under the OFL (§2.14.6, OQ-12).

### 2.11 Build cost

- **No crate for fetching.** DuckDB is an external CLI, needed only by the fetch script.
- **Reading:** `serde_json` and `serde`'s `derive`, both already dependencies; 0 packages.
- **Triangulation:** `earcut` 0.4.11 (georust; MIT OR Apache-2.0; a port of Mapbox's
  earcut; holes supported). Its one dependency, `num-traits` 0.2, is already compiled
  (through `approx` and arrow). So it is **+1 package and +1 compiled crate** (§2.2 k).
  `bevy_mesh` 0.19.1 meshes a `ConvexPolygon` but has no triangulator for concave polygons
  or holes.
- **The projection:** copied, so 0 packages (§2.2 j). Depending on `assimilator-import`
  would have added it and its optional dependencies (§2.5.1).
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

Review round 1 (same file) re-measured some of them and added others; the spec marks each
where it is used ("review round 1"). Among them are gate 6's new cases, the drawn heights'
median and p99, and an earcut 0.4.11 run on Midtown's footprints (gate 9). That run was a
throwaway crate built outside the repo. OQ-4's counts are the engine
question's, which gives their method.

### 2.13 Roadmap (not yet phases)

Each item becomes a phase appended here when it is drafted and reviewed. Each ends with a
video, unless it says why not. The order is the user's to set, except for the first item's
constraint.
- **An ODbL credit line in the video** (§2.2 l). It is a small phase of its own, and it
  **must ship before any Midtown video is shown outside**. This machine's ffmpeg has no
  `drawtext`, so it is Bevy text in the headless render (OQ-3), and `render` without
  `--buildings` stays byte-identical. One question is that phase's to settle. Midtown's
  roads are ODbL too (OQ-3), so a roads-only Midtown video needs the credit as well. The
  line may then have to key on an imported network, not on `--buildings`.
  *(2026-10-02)* Drafted as Phase 2 (§2.14). It keys on `metadata.map_origin`, with or
  without `--buildings` (§2.14.1, decision 2), so it is `render` on urban_grid that stays
  byte-identical.
- **A deterministic `--camera` render on Midtown** (OQ-8): a small phase in vis-001
  §2.11's draw path. The user scheduled it right after the credit (2026-10-02).
  *(2026-10-03)* Drafted as vis-001 Phase 6 (§2.12 there).
- **Nicer buildings:**
  - Overture `building_part` (towers on podiums; OQ-5) and raised bases;
    *(2026-10-03)* This is **Phase 4**, next after Phase 3, and its content is decided
    (§2.15.1, decision 4):
    - a building with parts is drawn from its parts, each from its own `min_height`;
    - raised bases are honoured;
    - flat roofs and the neutral grey stay;
    - no roof shapes, and no Overture colours.

    **Its deadline:** the parts must come from release `2026-09-23.1`, the cache's and
    the roads' release. It leaves S3 about 2026-11-22 (§2.14.3), so Phase 4 starts by
    early November 2026. Otherwise a newer release is pinned and every count is
    predicted again (§2.2 m);
    *(2026-10-03)* Drafted as **Phase 4** (§2.16). Its deadline is met: Midtown's
    buildings and parts of `2026-09-23.1` were saved as parquet before drafting
    (§2.16.10), and the fetch gains `--source` to read them offline (§2.16.3);
  - see-through or fading buildings near the camera or around a followed vehicle (§2.2 f);
    *(2026-10-03)* See-through, opaque, is drafted as **Phase 3** (§2.15). The user
    chose to cut to stubs, not to fade (§2.15.1, decision 2);
  - later sources: NYC Open Data footprints (measured roof heights, NYC only), and Google
    Photorealistic 3D Tiles (paid; its terms for video to be checked).
- **Ground:** land use and water from Overture's `base` theme.
- **Sky, fog and a wider extent**, where vis-001 OQ-12 (a pitch floor below 25°) is decided
  (§2.2 g).
  *(2026-10-03)* A floor below 22.5° (half the field of view) also lets a view ray rise,
  so that phase draws the undersides of raised volumes, which Phase 4 leaves out
  (§2.16.6).
- **Streets** *(added 2026-10-03, agreed by the user)*: junction surfaces, lane lines and
  stop lines. It comes before vis-001's data item (vis-001 §2.7 item 5), because signal
  states need stop lines to be drawn at.
  *(2026-10-05)* Drafted as **Phase 5** (§2.17), after vis-001 Phase 7. The user's
  decisions are §2.17.1: no crosswalks, curbs or sidewalks; US markings; on by default.
- **Engine look + lane arrows** *(user, 2026-10-08)*. Draw the road markings as the
  engine's own UI does: `NetworkJson` (`crates/geometry/src/network_json.rs`) for shapes
  and places, its web front end (`web/src/canvas/networkRenderer.ts`) for colours and the
  arrow glyphs. That means one 0.15 m yellow centre line, its dashes (2.5 m every 6.5 m),
  0.4 m stop lines, so the first stopped box shows the engine's 0.2 m gap
  (`STOP_LINE_STANDOFF` 0.6 m), stop lines also at the unsignalised approaches it marks,
  and its lane arrows (`lane_arrows`). Kept from Phase 5: the junction surfaces, the
  median fills, the fade, the default, `--no-streets` and `M`. Phase 5's US set stands
  until then. Next item, before data.
  *(2026-10-08)* Drafted as **Phase 6** (§2.18). The user's decisions are §2.18.1.
- **Vehicle shapes** *(added 2026-10-03, agreed by the user)*: a shape per FCD
  `vehicle_class`, after vis-001's data item. Midtown's 985 vehicles are all `car`
  (counted 2026-10-03).
- **Re-frozen traffic** after the engine fix (OQ-4). This is a pin move with a re-gate
  (vis-001 §2.2.2), not a phase.
  *(2026-10-04, user)* A phase after all: vis-001 Phase 7 (§2.13 there), before Streets.

### 2.14 The credit line in the video (Phase 2)

Drafted 2026-10-02, the first item of §2.13. It must ship before any Midtown video is shown
outside (§2.2 l). The observable is the video itself: every frame of a render of an
imported network carries the line.

#### 2.14.1 The user's decisions (decision, recorded)

Decided by the user, 2026-10-02, before drafting:
1. **The text is built from the data.**
   - The OpenStreetMap credit (ODbL) is always present.
   - Overture Maps Foundation and its release are named when the data came from Overture.
   - CC BY building sources (Esri Community Maps, Google Open Buildings, …) are named only
     when buildings from them are present.
   - The user agreed that a video shows the data of its fetch, so the release in the line
     is the fetch's.
2. **When:** any render of an imported network, which is one with `metadata.map_origin`,
   with or without `--buildings`. A synthetic network (urban_grid) gets none, so vis-001's
   renders stay byte-identical: 8700 of 8700 against `scratch/ref-8eb9052.framemd5`, as
   `scripts/gates.sh` checks.
3. **Placement:** one small line, bottom-right, on every frame. Light text with a dark
   outline or shadow, readable on any background, sized relative to the output height.
4. **`render` only.** `view` does not change.
5. **No switch** to turn it off in Phase 2. It is revisited at the harness contract (v1).

Decided by the user, 2026-10-02, on the draft (the answers to OQ-9 to OQ-13):
6. **Every non-OSM building source present is named**, with §2.14.2's wording (OQ-9).
7. **TomTom is not named until the data shows it** (OQ-10). Midtown's roads were checked
   once, against `2026-09-23.1` (§2.14.4). The engine request in OQ-10 stays the user's to
   carry.
8. **A HERE import, or any source type other than `Overture` and `Osm`, is an error** until
   the user gives HERE's wording (OQ-11).
9. **The font is committed under the OFL**, and `Cargo.toml` says so: `license = "(MIT OR
   Apache-2.0) AND OFL-1.1"`. The README and `CLAUDE.md` name the exception (OQ-12).
10. **Publishing:** the README tells whoever publishes a video to put the full credit, OQ-7's
    text with `openstreetmap.org/copyright`, in its description. A closing card waits for
    the harness contract (OQ-13).

The rest of this section is the plan. Where it weighed choices, it keeps them with their
costs and says which was taken:
- §2.14.3, where the buildings' sources come from;
- §2.14.4, the roads;
- §2.14.5, how the text is drawn;
- §2.14.6, the font;
- §2.14.8, how Phase 1's gates are re-run.

#### 2.14.2 What the line says

Three inputs, all read before the first frame:
- the resolved network's `metadata.map_origin` (vis-001 §2.1);
- `<project>/import_report.json`, if the file exists. It is the engine's
  `crates/import/src/types.rs:ImportReport`, of which only `source` is read: its `type`
  and, for `Overture`, its `release` (`crates/import/src/types.rs:ImportSource`);
- the `--buildings` file, if given: its release and each building's datasets (§2.14.3).

The rule, item by item:
0. **No `map_origin`: no line**, and nothing new is spawned. urban_grid's frames do not
   change (decision 2). Nothing else is read.
1. **Always:** `© OpenStreetMap contributors (ODbL)`.
2. **Overture**, when there are buildings or the report's `source.type` is `Overture`:
   `Overture Maps Foundation, release <r>`. `r` is the buildings file's release when there
   is one (decision 1: the fetch's), else the report's `source.release`. When the two
   differ, only the buildings' is shown.
3. **Each building source other than OpenStreetMap** that at least one building of the
   file names, once. The table's rows come first, in its order. Datasets the table does not
   know follow, verbatim, in byte order.

Items are joined by ` · ` (a middle dot between two spaces). The line has no full stop.

| `sources.dataset` | Licence, per Overture's page | In the line |
|---|---|---|
| `OpenStreetMap` | ODbL | item 1 (always) |
| `Esri Community Maps` | CC BY 4.0 | `Esri Community Maps contributors (CC BY 4.0)` |
| `Google Open Buildings` | CC BY 4.0 | `Google Open Buildings (CC BY 4.0)` |
| `Microsoft ML Buildings` | ODbL, licensed by Microsoft | `Microsoft ML Buildings (ODbL)` |
| `USGS Lidar` | none named; the page lists the "USGS 3D Elevation Program" | `USGS Lidar` |
| any other | its own, on the page | the dataset name, verbatim |

- The texts follow Overture's attribution page (docs v2.0.0, "Last updated on May 15,
  2026", read 2026-10-02, §2.14.10). The dataset names are those of Overture's buildings
  guide legend, and `USGS Lidar` is the name the probe found.
- **Midtown's buildings name three datasets** (the probe, §2.14.10):
  - OpenStreetMap on 4,327 of the 4,336;
  - USGS Lidar on 316, each beside OpenStreetMap;
  - Microsoft ML Buildings on 29, alone on 9 and beside OpenStreetMap on 20.

  **No CC BY source is among them.**
- **The Microsoft and USGS rows go beyond decision 1**, which names CC BY sources. Any
  non-OSM source that is present is named, because Overture's page lists each one
  (decision 6, OQ-9). For Midtown that is the difference between the roads' line (40.23
  em) and one half as long again (60.37 em).
- **An unknown dataset is named as it is, never dropped.** A source added in a later
  release then shows in the line, unpolished, instead of going uncredited.
- **The wording is the user's** (decision 6, OQ-9), as the README's was in OQ-7. Item 1
  names OpenStreetMap and the ODbL, as OSMF's guidelines ask (§2.14.10), and every item
  comes from the inputs above.

**Midtown** (the fixture's `import_report.json` is the user's, byte for byte, Phase 1's
gate 3):
- without `--buildings`: `© OpenStreetMap contributors (ODbL) · Overture Maps Foundation,
  release 2026-09-23.1`;
- with `--buildings scratch/midtown/buildings.geojson`, fetched again in Phase 2's form:
  `© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1 ·
  Microsoft ML Buildings (ODbL) · USGS Lidar`.

#### 2.14.3 Where the buildings' sources and release come from

Phase 1's cache keeps only `id`, `height` and `num_floors` (§2.2 c), and Phase 1's gate 5
pins its bytes. Nothing in it says which release it came from, or which source each
building came from. The options weighed:

| | (a) The cache carries them | (b) A sidecar file | (c) A fixed list | (d) A second flag |
|---|---|---|---|---|
| What | each feature gains `sources`, its datasets; the collection gains the release | the fetch also writes `<out>.sources.json`, which `render` finds by name | name every source on Overture's page | `render --buildings-sources <file>` |
| Decision 1 | kept | kept | **broken**: it names absent sources | kept |
| Travels with the cache | yes, one file | only if both files are copied | — | only if both are passed |
| Phase 1's gate 5 | bytes and SHA-256 change: recorded as a change in Phase 2, Phase 1 not rewritten | unchanged | unchanged | unchanged |
| Needs `2026-09-23.1` on S3 | one re-fetch of Midtown | one sources-only query for Midtown | no | as (b) |
| Code | one column and one layer option in the query; two members read | a second output, its own format, and a check that it belongs to its cache (a hash) | none | as (b), and a flag the harness must pass |

**The plan: (a).** One self-describing file, whose release is the fetch's by
construction (decision 1), and two members a reader can check. (b) costs as much and can be
parted from its cache. (c) breaks decision 1. (d) adds a flag to the harness contract.

**Its cost is a deadline.** The fixture's cache must be fetched again, in the new form, from
`2026-09-23.1`.
- Overture keeps a release "for a maximum of 60 days (two monthly releases)". Its next
  releases are 2026-10-21 and 2026-11-18 (release calendar, read 2026-10-02). So
  `2026-09-23.1` leaves S3 around **2026-11-22**, and after that this file cannot be made.
- Drafting's probe wrote that file into gitignored `scratch/` (§2.14.10), which buys
  time. If Phase 2 is built after the release is gone and the probe's file is lost too, a
  newer release is pinned and every count is predicted again (§2.2 m), recorded and never
  rewritten.

**The fetch's query** (§2.3.2) changes in two places, and gains two settings:

```sql
SET http_timeout = 120;
SET http_retries = 8;
COPY (
  SELECT id, height, num_floors,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources,
         geometry
  FROM read_parquet('s3://overturemaps-us-west-2/release/<release>/theme=buildings/type=building/*.parquet')
  WHERE bbox.xmin <= <E> AND bbox.xmax >= <W> AND bbox.ymin <= <N> AND bbox.ymax >= <S>
  ORDER BY id
) TO '<tmp>/buildings.geojson' WITH (FORMAT GDAL, DRIVER 'GeoJSON',
      LAYER_CREATION_OPTIONS 'DESCRIPTION=Overture Maps buildings, release <release>');
```

- **`sources`** is the building's datasets over every entry of Overture's `sources` list:
  the footprint's and any property's. They are distinct and sorted. A building with no
  `sources` gets `null`, and one whose entries name no dataset gets `[]` (tried on made-up
  rows, §2.14.10).
- **The release** goes in the collection's `"description"`: `"Overture Maps buildings,
  release 2026-09-23.1"`, written after `"name"`. GDAL's `FOREIGN_MEMBERS_COLLECTION`
  option, which could write a `release` member of its own, is ignored by DuckDB 1.5.1's
  GDAL (tried, §2.14.10).
- **The two settings** change no byte. Drafting's first probe failed on DuckDB's defaults
  (30 s, 3 retries; §2.14.10).
- **Nothing of Phase 1's changes:** the same rows in the same order, with the same
  properties and coordinates. Gate 5 checks this against Phase 1's file, byte for byte once
  the two new members are taken out.
- **The report** (§2.3.4) gains two keys:
  - `"sources"`: an object, dataset → the buildings that name it, keys sorted;
  - `"no_sources"`: the buildings with `null` or `[]`.

  A building counts once under each of its datasets, so the counts add up to more than the
  buildings when one names two. Midtown's, from the probe:
  - `{"Microsoft ML Buildings": 29, "OpenStreetMap": 4327, "USGS Lidar": 316}`, which is
    4,672;
  - `no_sources` 0;
  - 4,000 buildings with one dataset and 336 with two, which is 4,336.

**Reading it, in `render` only.** `src/buildings.rs:read` does not change. It ignores other
members and properties (§2.4.1), so `view` and every Phase 1 check read either form of the
file exactly as before (decision 4). The credit reads the file a second time, in
`src/credit.rs` (new; no Bevy types), after Phase 1's checks have passed. It reads two
things:
- `description` must be the string `Overture Maps buildings, release <r>`, with `r`
  non-empty and without spaces. Otherwise: `error: --buildings <file>: no release: fetched
  before vis-002 Phase 2; fetch it again with scripts/fetch-buildings.sh`.
- Each feature's `properties.sources` is absent, `null`, or an array of strings. Anything
  else is an error naming the feature, by its `id` as in §2.4.3.

A second parse of Midtown's 2 MB costs milliseconds, and it keeps the provenance out of the
reader that `view` shares. `tests/shapes.geojson` was never fetched and has no release, so
`render --buildings tests/shapes.geojson` errors from Phase 2 on. Phase 1's gate 12 uses
that file. §2.14.8 says how gate 12 runs.

#### 2.14.4 The roads, and what the report can say

`source.type` in `import_report.json` (`crates/import/src/types.rs:ImportSource`, a tagged
enum of three types at the pin and on engine main):
- **`Overture`:** item 2, with `source.release`. A missing or non-string `release` is an
  error.
- **`Osm`:** nothing beyond item 1.
- **No report, or `"source": null`:** the roads' origin is unknown. Item 1 still appears
  (decision 1), and nothing else is added for the roads. A network drawn by hand on a map has
  only `map_origin` (§2.3.1).
- **`Here`, or any other type:** an error, `error: <project>/import_report.json: source
  type "Here" has no credit (vis-002 OQ-11)`. HERE's roads are not OSM data, and their
  credit is HERE's to set. Failing closed keeps a new source type from being rendered
  uncredited. It also stops `render` on a HERE project, which works today. Decided by the
  user (decision 8, OQ-11): the error stays until the user gives HERE's wording.

**TomTom is not named.** Overture's transportation theme lists "Data from TomTom." beside
OpenStreetMap. Nothing in a project says which of its roads came from TomTom:
- the engine reads each segment's `sources`
  (`crates/import/src/overture/types.rs:OvertureSource`), but writes none of it into the
  network or the report;
- Midtown's report maps 236 of the network's 255 links to 222 Overture segments
  (`overture_link_ids`). The other 19 links have ids the report does not hold; the network
  was cut and edited after the import (§2.3.1). A query of those segments against the
  transportation theme could say, but only while `2026-09-23.1` is on S3.

So TomTom is not named until the data shows it (decisions 1 and 7, OQ-10).

**Midtown's roads were checked once** (2026-10-02, §2.14.10). The query was one DuckDB read
of the 222 segments against `theme=transportation/type=segment` of `2026-09-23.1`. **All 222
name OpenStreetMap and nothing else; none names TomTom.** So Midtown's line is complete
without TomTom (gate 7). The 19 links the report does not map cannot be traced to a
segment, and nothing in the project records a source for them.

The engine request in OQ-10 is the user's to carry. Until the engine records datasets, a
project other than Midtown gets no TomTom check.

The report is read only when the network has `map_origin`. It is the project's, not the
scenario's: a project whose scenario points at another network file still takes its credit
from the one report it has.

#### 2.14.5 Drawing: Bevy text in the headless render

**How.** `render`'s app already has Bevy's `TextPlugin`, `UiPlugin` and `UiRenderPlugin`:
`DefaultPlugins` adds them because `bevy_text`, `bevy_ui` and `bevy_ui_render` are on for
`view` (`Cargo.toml`). It has never spawned a node. With a line, `src/render.rs:Renderer`
spawns one UI tree:
- **a root `Node`**, absolutely positioned at `right: m`, `bottom: m`, with
  `UiTargetCamera(<the image camera>)`. That component must be there: with no window,
  Bevy has no default UI camera ("the highest order camera targeting the primary window",
  `bevy_ui` 0.19.1);
- **in it, the line as a `Text`** in the light fill colour, in the root's flow. It sets the
  root's size;
- **beside it, eight copies of the line** in the dark outline colour: children of the root
  and siblings of the fill, never the fill's children. Each is absolutely positioned at an
  offset `(dx, dy)`, with `dx, dy ∈ {−o, 0, o}` and not both 0. Each has `ZIndex(-1)`, so
  it draws below the fill. `ZIndex` orders siblings only, so a child of the fill would draw
  above it (`bevy_ui` 0.19.1, `src/stack.rs:update_uistack_recursive`). Bevy 0.19 has a
  `TextShadow` (one offset) but no outline. A one-sided shadow leaves the top and left
  edges of light glyphs unbordered over a light roof;
- **all nine have `TextLayout` with `LineBreak::NoWrap`.** Bevy wraps at word boundaries by
  default, and a wrapped fill would report a width that skips the fit.

`draw::spawn_credit(world, line, camera: Entity, font: Handle<Font>, size, margin,
offset) -> CreditNodes` spawns this tree. `CreditNodes` holds the root, the fill and the
eight copies. `draw::set_credit_size(world, &CreditNodes, size)` sets the font size of all
nine nodes for the fit.

Without a line, nothing is spawned, as §2.7 does for buildings.

**The ways weighed:**

| | Bevy UI text (the draft) | A CPU raster on the readback | ffmpeg's `drawtext` |
|---|---|---|---|
| How | nine `Text` nodes in `render`'s app | `swash` rasterises the line once, and it is blended into each frame's RGBA before ffmpeg | a filter in the encoder's command |
| Packages | 0 | 0: `swash` 0.2.10 is locked through `bevy_text`; `Cargo.toml` gains a direct dependency on it | — |
| Code | about 60 lines; Bevy shapes and lays out the text | about 120 lines: shaping, glyph raster, the outline by dilation, blending | — |
| Deterministic | through Bevy's UI pass; gates 10 and 14 check it | by construction | — |
| On this machine | built, for `view` | built | **absent**: ffmpeg 9.0.2 has no `drawtext` (§2.14.10) |

**The plan: Bevy UI text**, as §2.13 planned. It is less code, and it uses the text
stack `view` already draws its readout with. The CPU raster is the fallback if gate 10 finds
a pixel changed outside the box, or gate 14 finds two renders that differ. Taking it is a
scope change, not part of Phase 2 as reviewed: it adds `swash` to `Cargo.toml`, which gate
2 forbids. So it clears Phase 2's `reviewed` (the methodology's §7), and the phase goes
back to review.

**The numbers**, in output pixels (an image target's scale factor is 1):
- **font size** `S = round(min(H, 9W/16) / 54)`: 13 px at 1280×720, 20 at 1920×1080, 40 at
  3840×2160. The `9W/16` term keeps a portrait frame's line inside its width;
- **margin** `m = S`, from the right and bottom edges to the fill text's box;
- **outline offset** `o = max(1, round(S/12))`: 1, 2 and 3 px at those sizes;
- **fill** sRGB (240, 240, 240) and **outline** sRGB (0, 0, 0), both opaque. Their WCAG
  contrast ratio is 18.4:1.

`S`, `m`, `o`, the ratio 1/54 and the two colours are iteration (vis-001 §2.6), settled at
the user's check (gate 16). These constraints are not:
- one line in the bottom-right corner, on every frame;
- a light fill with an opaque dark outline;
- a size in proportion to the output, smaller only to fit (below);
- no pixel outside the credit box changes.

**The fit.** The line never wraps and is never clipped. It shrinks to fit instead.
- After the settle frames, the fill text's computed width `w` is read. If `w > W − 2m`,
  the font size becomes `S' = floor(S · (W − 2m) / w)`. Here `S` is the current size and
  `w` its width.
- `set_credit_size` then sets `S'` on all nine nodes, and `Renderer` runs the three settle
  renders again (`render(&[])`, as `build` does). Then `w` is read again, until it fits.
- `m` and `o` keep their values for the first `S`. `credit::fit_size(size, width_px,
  avail_px)` takes `avail_px = W − 2·margin(S)` and computes one pass.
- A final size under 10 px is an error (§2.14.7): a credit too small to read is no credit.
  That includes an `S` under 10 before any shrink, which a frame under about 513 px tall
  gives at 16:9 (640×360 gives 7).

Estimated from Fira Sans Medium's advance widths, without kerning (§2.14.10), the share of
`W − 2m` is the same at every 16:9 size:

| Line | Width | 1280×720 | Share | Font size |
|---|---|---|---|---|
| Midtown without buildings | 40.23 em | 523 px of 1,254 | 42 % | not shrunk |
| Midtown with buildings | 60.37 em | 785 px of 1,254 | 63 % | not shrunk |
| Naming Esri, Google, Microsoft and USGS too | 98.06 em | — | 102–104 % | 12, 19 and 38 px |
| Gate 11's made-up *six sources* line | 105.39 em | — | 109–112 % | 11, 17 and 35 px |
| Gate 9's made-up *long* line | 241.28 em | — | about 250–257 % | 5, 7 and 15 px: an error at 720p and 1080p. At 1080p, 7 is 2.6 % above 8, inside the kerning allowance, so 7 or 8 |

The font sizes are at 1280×720, 1920×1080 and 3840×2160.

**The credit box** is the fill text's computed rectangle grown by `o` on every side,
rounded outward to whole pixels. `Job::credit_box()` returns it for the gates as
`Option<[u32; 4]>`, `[x0, y0, x1, y1]`, with `x1` and `y1` exclusive: the box's pixels are
`x0 ≤ x < x1`, `y0 ≤ y < y1`. Its height is the fill's line box, `1.2·S` (Bevy's default
`LineHeight`; Fira Sans's ascent and descent are 935 and −265), plus `2o`. At 1920×1080
that is 24 + 4 = 28 px, so `y0` is 1034.

**Its height.** The tallest glyphs of both Midtown lines are `(` and `)`, and the lowest
are `p` and `g`: 1.070 em from top to bottom. With the outline that is 1.070·S + 2o: 15.9 px at
1280×720 and 48.8 px at 3840×2160. Capitals are 0.691·S: 9.0 px and 27.6 px.

#### 2.14.6 The font: Fira Sans Medium, OFL 1.1

Bevy's built-in font cannot draw the line. `default_font` embeds `FiraMono-subset.ttf`
(Fira Mono Medium 3.206, OFL 1.1, in `bevy_text` 0.19.1), and it maps only ASCII
0x20–0x7E: 95 code points, with no `©` and no `·` (measured, §2.14.10). The options:

| | Bevy's `FiraMono-subset` | Fira Sans Medium, committed | A font crate |
|---|---|---|---|
| Glyphs | ASCII only | 1,686 code points; `©`, `·`, `é`, `ñ` | depends |
| Licence | OFL 1.1, inside `bevy_text` | OFL 1.1, `OFL.txt` beside it | the crate's |
| Cost | 0 | `assets/fonts/FiraSans-Medium.ttf` (457,248 bytes) and `assets/fonts/OFL.txt` (4,370 bytes); `include_bytes!` into the binary; 0 packages | +1 package at least |
| The line | `(c) OpenStreetMap …` or no symbol, and a monospace line 25 % wider (50.40 em against 40.23) | as §2.14.2 | as §2.14.2 |

**The plan: Fira Sans Medium 4.203**, from `google/fonts` (`ofl/firasans/`). It is the
proportional sibling of `view`'s Fira Mono, from the same foundry under the same licence.
- **The pin is the SHA-256**, since no `google/fonts` commit was recorded:
  - `FiraSans-Medium.ttf`: 457,248 bytes,
    `cbc1842cbed8c1d1146ba7c9db97d8f28c9bedfd25f41c5b0e1259ca48622328`;
  - `OFL.txt`: 4,370 bytes,
    `8f24842e9174beda18a556c2ae7d54f5dc444340c19a3a9ef77e23bca366adbd`.
- **Where to get them:** drafting's copies are kept in gitignored
  `scratch/vis002p2-font/`. Failing that, download from
  `https://raw.githubusercontent.com/google/fonts/main/ofl/firasans/`, and the hashes must
  match. If they do not, the font has moved on. §2.14.5's widths are then measured again
  before building, and the change is recorded.
- Copying them into `assets/fonts/` is a build step, not a gate. The exit gate's
  "only gate 4 uses the network" does not cover a download made there.
- `Font::from_bytes` (`bevy_text` 0.19.1) takes the embedded bytes, so `render` reads no
  font file at run time.
- The OFL's condition 2: the font "may be bundled, redistributed and/or sold with any
  software, provided that each copy contains the above copyright notice and this
  license". `OFL.txt` carries both, and so does the font's own `name` table.
- Its condition 5: the font "must be distributed entirely under this license", and that
  requirement "does not apply to any document created using the Font Software", which is
  the video.
- **It is the first file in this repo under a licence other than MIT OR Apache-2.0.**
  Condition 5 bears on `Cargo.toml`'s `license`. `bevy_text` declares `MIT OR Apache-2.0`
  while it embeds its OFL subset. The more exact expression is `(MIT OR Apache-2.0) AND
  OFL-1.1`. Decided by the user (decision 9, OQ-12): `Cargo.toml` takes it, and the README
  and `CLAUDE.md` name the exception, which amends `CLAUDE.md`'s rule that every crate
  sets `MIT OR Apache-2.0`.

#### 2.14.7 The order of checks, and the API

`render`'s checks (vis-001 §2.4) gain three:
- **Two after `--buildings`' file** (§2.4.3), before the scene is built:
  1. `<project>/import_report.json`, only when the network has `map_origin`. The errors:
     the file cannot be read (an absent file is not an error), or is not JSON; `source` is
     neither `null` nor an object with a string `type`; `Overture` has no string `release`;
     the type is neither `Overture` nor `Osm`. Each is `error: <project>/import_report.json:
     …`. A report with no `source` key reads as `"source": null`, as serde reads the
     engine's `Option`. The engine always writes the key, so only a hand-made report lacks
     it;
  2. the buildings file's two members (§2.14.3).
- **One after the scene's settle frames**, before the encoder starts: the fit (§2.14.5),
  `error: the credit line would be <S'> px in a <W>×<H> frame, under 10 px; render a
  larger frame`. `S'` is the final size: the shrunk size, or `S` itself when `S` is
  already under 10.

Each is one line with exit 1, no progress line and no output file (vis-001 §2.4).

**The API.**
- `RenderOptions` does not change.
- `Job::prepare_with(o, camera, buildings)` builds the line as the CLI does (decision 5).
- `Job::prepare_without_credit(o, camera, buildings)` is the same job with no line. It is
  for the gates; no flag reaches it.
- `Job::credit() -> Option<&str>` returns the line, if any. `Job::credit_size() ->
  Option<u32>` returns its font size after the fit. `Job::credit_box()` returns its box in
  pixels (§2.14.5).
- `src/render.rs:Renderer`'s two constructors take the line as an `Option`.
- `src/buildings.rs` and every file under `src/view/` are not edited.

#### 2.14.8 Phase 1's gates that read Midtown frames

The line is in the corner of every Midtown frame from Phase 2 on, so some of Phase 1's
gates see it. Phase 1's text is not edited. Phase 2's exit gate re-runs them as follows:
- **Gate 11 (top-down coverage)** runs with the line drawn, as the CLI draws it, and
  **excludes the credit box**. Pixels inside either frame's box (the union of the two) are
  left out of both of its checks. The predictions stay 0 and 0. Its test in
  `tests/buildings.rs` is edited to do so, and to report the excluded count.
- **Gate 12 (shapes in perspective)** runs through `Job::prepare_without_credit`, because
  `tests/shapes.geojson` has no release (§2.14.3). Its frames are then Phase 1's, and so are
  its predictions.
- **Gate 13 (deterministic)**: the orthographic pair is Phase 2's own gate 14, with the
  line. The flight pair stays OQ-8's recorded miss and is not a gate here.
- **Gate 5 (the cache)** changes, recorded in Phase 2's gate 4. `scripts/gates-city.sh`
  takes the new bytes and SHA-256.
- **Gates 8–10** read the new cache. They give Phase 1's numbers, since its content is
  Phase 1's (Phase 2's gate 5).

Phase 2's gate 10 ties these together. Outside the box, a frame with the line equals the
same frame without it, byte for byte. So what gates 11 and 12 measure is what the video
shows.

Phase 2's own frame comparisons are orthographic where they can be. The `--camera` render
is not deterministic on Midtown (OQ-8), whose cause is stacked boxes. So a perspective frame
is compared only at `render_empty()`, with no boxes, as Phase 1's gate 12 was.

#### 2.14.9 Build cost

- **0 packages.** `Cargo.lock` does not change, and `Cargo.toml` changes only in its
  `license` (OQ-12). No Bevy feature is added:
  `bevy_text`, `bevy_ui`, `bevy_ui_render` and `default_font` are already on. A clean
  release build compiles the same **363** crates as Phase 1.
- **Files:** the font (457,248 bytes) and `OFL.txt` (4,370 bytes). The binary grows by
  about the font.
- **No network** in `render`, and no font file read at run time.

#### 2.14.10 Measured while drafting (2026-10-02)

Every number in §2.14 that is not cited to source was measured on 2026-10-02, read-only
against the engine at the pin, the Midtown project at `e2de274` and the web, with output
only under `scratch/` and the session's scratchpad. The record, with the method, is
`specs/reviews/vis-002.md`, "Phase 2 draft". In short:
- **Overture's attribution page** (docs v2.0.0, last updated 2026-05-15):
  - buildings, transportation and base are ODbL, each with "© OpenStreetMap contributors.
    Available under the Open Database License.";
  - buildings also list Esri Community Maps contributors (CC BY 4.0), Global ML Building
    Footprints (ODbL, by Microsoft), Google Open Buildings (CC BY 4.0), the USGS 3D
    Elevation Program, a Qian Shi et al. dataset (CC BY 4.0) and BTN 2024 ign.es (CC BY
    4.0);
  - transportation lists "Data from TomTom.", with no licence text.
- **OSMF's attribution guidelines:**
  - attribution must be to "OpenStreetMap" and make clear that the data is under the ODbL;
  - "© OpenStreetMap contributors" is acceptable;
  - the text must be legible "taking into consideration the font, size, colour, contrast,
    positioning and amount of time that it is visible";
  - for video where the map is a major component, the credit "should typically appear in
    a corner of the map, in addition to attribution in the end credits or description",
    and those must include the URL `openstreetmap.org/copyright` (OQ-13).
- **Overture's release calendar:** releases are kept "for a maximum of 60 days (two
  monthly releases)"; the next are 2026-10-21 and 2026-11-18. S3 lists `2026-08-19.0`,
  `2026-09-23.0` and `2026-09-23.1`.
- **The engine:** `ImportSource` is `Osm`, `Here` or `Overture` (tagged by `type`) at the
  pin and on main. Segment `sources` are read and not kept. Midtown's report:
  `{"type": "Overture", …, "release": "2026-09-23.1"}`, with 236 of the 255 links in
  `overture_link_ids`.
- **Fonts:**
  - Bevy's subset maps 95 code points;
  - Fira Sans Medium 4.203 maps 1,686, with a cap height of 691/1000;
  - the widths and heights of §2.14.5 come from its `hmtx` and `glyf` tables, read by a
    script.
- **Bevy 0.19.1:**
  - `DefaultPlugins` adds `TextPlugin`, `UiPlugin` and `UiRenderPlugin` under the three
    features;
  - `TextShadow` is one offset, and there is no text outline;
  - `UiTargetCamera` overrides the default UI camera, which is only ever a window's.
- **DuckDB 1.5.1's GDAL**, on made-up rows into `scratch/`:
  - a `VARCHAR[]` column becomes a JSON string array: `null` for NULL, `[ ]` for empty;
  - `DESCRIPTION=` writes a top-level `"description"`, commas included;
  - `FOREIGN_MEMBERS_COLLECTION` is ignored.
- **The probe.** The brief allowed one against `2026-09-23.1`.
  - The first attempt timed out on one parquet file after 643 s and wrote nothing.
  - The user allowed one retry, run as §2.14.3's query exactly, into
    `scratch/vis002p2-probe/fetch/buildings.geojson`. It took **1,253 s** wall (45 s
    user), against Phase 1's 122–185 s for the same box: the link was slow that day.
  - It wrote **4,336** buildings, the release in `"description"`, and the same `crs`.
  - Height rules: 4,277 by `height`, 8 by `num_floors` and 51 at the default.
  - Datasets: OpenStreetMap 4,327, USGS Lidar 316 and Microsoft ML Buildings 29, and 0
    with none.
  - By building: 3,991 OpenStreetMap only, 316 OpenStreetMap and USGS Lidar, 20
    OpenStreetMap and Microsoft, and 9 Microsoft only.
  - The file is **2,284,830 bytes**, SHA-256
    `f241ccbd9a2fba2e3c80fa28ec7522ca2a33b84628c18954952ad6bc1d4a1dfd`.
  - **Without its `"description"` line and its `, "sources": […]` members, it is
    byte-identical to Phase 1's cache** (2,140,989 bytes, `561a615d…`), with the same
    ids in the same order.
- **The TomTom check** (OQ-10's (c), allowed by the user on 2026-10-02 as evidence only):
  - Its input: the 222 segment UUIDs of the 236 mapped links, taken from
    `overture_link_ids` (the second field of `L_<uuid>_<i>_<j>[_r]`) and written to
    `scratch/vis002p2-tomtom/segments.csv`.
  - The query, with §2.14.3's HTTP settings:

    ```sql
    SELECT s.id, list_sort(list_distinct([x.dataset FOR x IN s.sources])) AS datasets
    FROM read_parquet('s3://overturemaps-us-west-2/release/2026-09-23.1/theme=transportation/type=segment/*.parquet') s
    WHERE <Phase 1's gate 4 bbox on s.bbox> AND s.id IN (<the 222>)
    ORDER BY s.id
    ```

    It wrote `scratch/vis002p2-tomtom/segments-datasets.csv`.
  - **70.3 s** wall (user 3.5 s), first attempt, exit 0.
  - **222 of 222 segments found, each with `[OpenStreetMap]` only.** No other dataset
    appears, TomTom included.

### 2.15 See-through (Phase 3)

Drafted 2026-10-03. At vis-001 Phase 6's gate 12 the user found that tall buildings hide
much of the traffic in Midtown's flight. This phase cuts the buildings that stand between
the camera and the point it looks at down to stubs, by default wherever buildings are
drawn in perspective (decision 3, as changed 2026-10-03). It works in the
perspective draw path that vis-001 Phase 6 made deterministic (`specs/visualizer_spec.md`
§2.12), and changes only the buildings' mesh, never the boxes' draw. The numbers below
come from a probe run while drafting (§2.15.9).

#### 2.15.1 The user's decisions (decision, recorded)

Decided by the user, 2026-10-03, before drafting:
1. **Order:** see-through first, as this phase. Building parts are the next phase, Phase 4.
2. **Look:** a building that stands between the camera and the point the camera looks at
   is cut down to a stub a few metres tall. The street and its traffic then show, and the
   block's footprint stays. It is opaque: no transparency.
3. **Opt-in:** `render` gets a flag (or a setting in the flight file; the draft proposes
   one), and `view` gets a key. Without it, every render stays byte-identical.

   *(Changed by the user, 2026-10-03, on the draft, before review; this also answers
   OQ-14.)* **See-through is on by default** wherever buildings are drawn in
   perspective: `render --buildings --camera`, and `view --buildings`.
   - `--no-see-through` turns it off, in `render` and in `view`. `X` in `view` still
     toggles it.
   - With `--no-see-through`, every frame stays byte-identical to today's.
   - The orthographic render is unchanged either way.

   So from Phase 3 on, a flight rendered with `--buildings` is cut unless the flag says
   otherwise. §2.15.6 and Phase 3's gates follow this.
4. **Phase 4, recorded now:** a building with `building_part`s is drawn from its parts,
   each standing from its own `min_height`. Raised bases are honoured. Flat roofs and the
   neutral grey stay. No roof shapes, and no Overture colours.

Agreed by the user the same day, for the roadmap (§2.13): two new items, "Streets" and
"Vehicle shapes", and Phase 4's deadline.

The rest of this section is the draft's proposal. It settles:
- which buildings are in the way (§2.15.2);
- how far down they go, and how they get there (§2.15.3);
- which renders it applies to (§2.15.4);
- how the cut is drawn, and its cost (§2.15.5);
- the flag, the key, and where the flag has no effect (§2.15.6).

§2.15.7 says what it does not fix.

#### 2.15.2 Which buildings are in the way: a wedge from the camera to the look-at point

The point the camera looks at is already defined:
- in `render --camera` it is the pose's `(cx, cy)` at each frame. That is a keyframe's
  centre, the followed vehicle's placed point, or the flight between them (vis-001
  §2.11.5);
- in `view` it is the state's centre, which the camera orbits and a follow moves
  (`rules/view.md`).

From the pose (`src/camera.rs:Pose`):
- `L = (cx, cy)` is the look-at point. `G` is the eye's ground point, the `x` and `y` of
  `src/camera.rs:Pose::eye`. `G` lies `d·cos(pitch)` from `L` toward the camera, where
  `d = 1.2071·height_m`;
- **the half-width** is `r = 0.15 · height_m · cos(pitch)` metres (`see_through::WIDTH` =
  0.15). `cos` is `src/camera.rs:cos_deg`, so `r` is exactly 0 at pitch 90;
- **the wedge** `W` is the convex hull of `G` and the disc of radius `r` about `L`. That
  is the triangle from `G` to the two points where lines from `G` touch the circle,
  together with the disc. When `G` lies inside the disc or on its edge (`|GL| ≤ r`, as
  straight down, where both are 0), `W` is the disc. Seen from
  above, the wedge opens from the camera's foot at a fixed half-angle,
  `asin(0.15/1.2071)` = 7.14°, whatever the pose, and ends round the look-at point;
- **a building's distance** `δ` is the ground distance from its footprint to `W`. The
  footprint is all of the building's polygons, with holes as outside. `δ` is 0 when they
  meet. Because `W` is a triangle and a disc, `δ` is the smaller of the footprint's
  distance to the triangle and its distance to `L` less `r` (at least 0);
- **in the way** means `δ < e`, where `e = max(r, 10 m)` is the ease band
  (`see_through::EASE_MIN_M` = 10, §2.15.3).

**Why a wedge, and why it narrows to nothing straight down.**
- A building hides what lies behind it, seen from the eye. The ground line from the
  camera's foot to the look-at point is where a building hides the look-at point itself.
  The wedge widens toward `L` so that the ground around `L` shows too.
- A building near the camera's foot stands under a sight line that is high up there, and
  hides little of the middle. A capsule of even width cuts more such buildings for the
  same gain. In the drafting probe, with a hard edge, a capsule of half-width
  `0.1·height_m` cut 44.7 buildings a frame on the city flight, and a wedge of the same
  `r` cut 28.3, with the same share of centre boxes hidden (12.9 %) (§2.15.9).
- **Straight down, a building hides only the ground under it and a strip beside it**,
  through perspective. A disc of even radius would then cut a crater round the look-at
  point for little gain. At `view`'s launch fit on Midtown (`height_m` about 1,725 m),
  a radius of `0.15·height_m` would be 259 m. `cos(pitch)` makes `r` 0 at pitch 90, so
  only a building within 10 m of `L` is lowered there.
- **The width is measured, and it is iteration** (vis-001 §2.6). On the probe's poses:

  | Width `WIDTH` | Orbit flight: centre boxes hidden | Orbit: buildings cut a frame (in frame) | City flight: centre boxes hidden | City: buildings cut a frame (in frame) |
  |---|---|---|---|---|
  | none | 59.4 % | 0 | 22.7 % | 0 |
  | 0.10 | 3.8 % | 24.1 (14.8) | 18.3 % | 23.8 (17.3) |
  | **0.15** | **0.0 %** | **33.4 (20.8)** | **16.0 %** | **33.3 (23.9)** |
  | 0.20 | 0.0 % | 44.9 (28.7) | 12.0 % | 43.9 (30.5) |

  "Centre" is the middle sixteenth of the frame, a quarter of its width by a quarter of
  its height. A box counts as hidden when the sight line to its top centre (1.55 m) meets
  a building. "In frame" counts buildings whose footprint centroid projects into the
  frame. The flights are §2.15.9's: the orbit circles a busy crossing at `height_m` 250
  and pitch 35, and the city flight is `tests/city-flight.toml`. Of about 300 buildings in
  the orbit's frame, 0.15 cuts about 21 (7 %). Of about 946 in the city flight's frame, it
  cuts about 24 (2.5 %). 0.15 is the narrowest width that clears the orbit's centre in
  every frame. The traffic-free pose grid (24 look-at points on roads × 8 yaws, at
  `height_m` 80, 200 and 500 and pitch 25–60) agrees: road points hidden in the centre
  fall from 51–93 % to 0.4–22 % at 0.15, and the worst cases are at `height_m` 500 and
  pitch 25. Straight down they barely move (11.3 % to 11.2 % at most), and 0.7
  buildings a pose are lowered.
- **The look-at point must be on the traffic.** From 327 s on, the city flight looks
  into Central Park, at the network's north edge, and its centre holds no box at all. So
  the wedge helps it less (22.7 % to 16.0 %). §2.15.7 and OQ-15 say what that leaves.

#### 2.15.3 How far down: a 3 m stub, eased, never popped

A building in the way is drawn with height

```
h′ = min(h, s + (h − s) · smoothstep(δ / e))     for δ < e
h′ = h                                           for δ ≥ e  (untouched, bit for bit)
```

with `s` = 3 m (`see_through::STUB_M`) and `smoothstep(x) = x·x·(3 − 2x)`, written in
that order as the probe did (`3x² − 2x³` is the same function, but differs in the last
bits). A building
touching the wedge is a 3 m stub. One `e` away it is at full height. One already under
3 m keeps its own height. Only the height changes: footprint, courtyards, walls and roof
shape stay, so a stub is the block's footprint as a low slab with its roof lighter than
its walls (§2.7).

- **The stub's height barely matters** to what shows. A 6 m stub instead of 3 m left
  16.2 % of the city flight's centre boxes hidden instead of 16.0 %, and 0.0 % on the
  orbit, in an earlier probe run that drew the disc as a 32-gon (the probe's `geom3.txt`
  and `geom-orbit.txt`, `s=6`). Over 1–8 m, another
  variant moved by 0.2 points (§2.15.9). 3 m is one storey, so the block still reads as a
  block. Near the middle of the frame at the 25° floor, a 3 m wall hides the ground for
  6.4 m behind it, about a sidewalk.
- **Easing rather than a hard edge.** With a hard edge (`h′ = s` inside `W`, else `h`), a
  building pops from full height to a stub between two frames as the wedge sweeps over
  it: up to 469 m in one frame on the city flight, and 163 times a building in frame
  changed by more than 10 m from one frame to the next. Eased, the largest change of a
  building in frame between two consecutive frames is **9.39 m** on the city flight and
  **3.16 m** on the orbit, and **0** changes exceed 10 m. A 200 m tower takes at least
  21 frames (0.7 s) to sink, so it sinks and rises rather than pops.
- **`h′` is continuous in the pose:** `G`, `r`, `e` and `δ` are, and so is `smoothstep`.
  A flight's pose is continuous in time (vis-001 §2.11.5), so a flight never pops.
  `view`'s fixed steps (`Q E R F`, 15° and 5°) move the pose in one frame, and the heights
  move with the image.
- **The 10 m floor on `e`** keeps it continuous straight down, where `r` is 0. There, as
  the look-at point is panned onto a building, it sinks over 10 m of travel.

#### 2.15.4 Which renders: the perspective ones

- **`render --camera` and `view`** draw in perspective, at every pose (vis-001 §2.11.2).
  See-through applies to both.
- **The orthographic render (`render` without `--camera`) is unchanged.** Straight down
  through an orthographic camera, a building hides exactly the road under its footprint:
  2.239 % of Midtown's centreline length (Phase 1 gate 10). A stub would still cover
  that, and a roof's lit colour does not depend on its height, so the cut would change
  nothing a viewer could see. It draws no cut, with or without `--no-see-through`
  (§2.15.6).

#### 2.15.5 How the cut is drawn: the building mesh's heights, rewritten on the CPU

The buildings stay **one mesh, one draw, opaque** (§2.6). Phase 6 showed that Bevy's
binned and sorted passes do not keep draw order stable (vis-001 §2.12.1), so nothing
here adds an entity per building or a transparent pass. The candidates:

| | (a) Rewrite the mesh's heights on the CPU | (b) A vertex shader clamps the heights | (c) An entity per building, scaled in height |
|---|---|---|---|
| How | Each vertex of building `b` gets `z′ = min(z, h′_b)`, and the mesh asset is replaced when any `h′` changed | `ExtendedMaterial` on the lit material. Each vertex carries its building's index, and a buffer of 4,336 heights is uploaded each frame | 4,336 entities; `Transform` scale `h′/h` |
| Per frame, Midtown (measured, §2.15.9) | **+3.1 to +4.1 ms** at the median: 0.5 ms to compute the heights and build and insert the mesh, and the rest Bevy re-uploading 213,880 vertices and 359,190 indices | not built; 17 KB a frame | not built |
| Draw order | one draw, as today | one draw | 4,336 draws through the binned pass, **not stable**; and overlapping footprints of equal height (§2.4.1) have tied roofs |
| Off | the shipped mesh, untouched | the shipped material, or the extension with its output unproven equal | — |
| Code | about 60 lines in Rust; no shader | a WGSL vertex stage, a new vertex attribute, a material plugin | — |
| Packages | 0 | 0 | 0 |

**The plan: (a).** It is exact by construction:
- `min(z, h′)` with `h′ = h` returns every vertex as it was. So a frame where no building
  is in the way is the shipped frame, byte for byte;
- with `h′` = `c`, the vertices are exactly those `src/buildings.rs:building_mesh` gives
  for a building of height `c`. Its roof triangulation does not depend on the height.
  So a cut frame equals the frame of a buildings file in which the cut buildings have
  those heights, byte for byte (gate 6; 0 pixels apart in the probe).

The heights are plain `f64` arithmetic on the pose and the file, in building order, and
the vertices go to `f32` as `src/draw.rs:buildings_mesh` converts them today. So the
same pose gives the same mesh. In the probe, two runs of each flight gave the same hash
over all 1800 frames. (b) is the fallback if `view`'s bench (gate 13) falls below 30 fps.
Taking it is a scope change that sends the phase back to review.

**Normals and indices do not change**, so a later optimisation could upload positions
only. Bevy 0.19 re-uploads a replaced mesh whole, and the measured cost does not call for
it.

#### 2.15.6 The flag, the key, and where the flag has no effect

*(Rewritten 2026-10-03, before review, for decision 3 as changed and OQ-14's answer. The
draft's first proposal, an opt-in `--see-through` with two errors, is in OQ-14.)*

**On by default, where it can act:**
- **`render`** cuts when it has both `--buildings` and `--camera`. Each frame's heights
  follow that frame's pose. Without either, nothing is cut.
- **`view`** starts with see-through on when it has `--buildings`.
- **`--no-see-through`**, on `render` and on `view`, turns it off. With it, `render`'s
  scene is built and drawn exactly as today: no `BuildingsCut`, no mesh replaced, and so
  every frame byte-identical to `2b25d5e`'s (gate 2). `view` keeps its `Buildings` so
  that `X` can turn the cut on later, and builds its `BuildingsCut` the first time it is
  on; until then no mesh is replaced.
- **`X`** in `view` (by position, `KeyCode::KeyX`; unbound today) flips see-through at any
  moment, like `B`, and changes nothing else. `view --no-see-through` starts it off, and
  `X` can still turn it on.
  - With the buildings hidden (`B`), nothing changes on screen until `B` shows them, cut
    or not as `X` says.
  - Without `--buildings`, `X` flips a flag that nothing reads.
  - The keyframe line does not carry it, since `render` takes it as a flag.
- **`--bench` with buildings and see-through on**, now the default, rebuilds the mesh on
  every frame, changed or not. The bench's camera stands still at the launch fit, where
  nothing would be rebuilt, so this measures the worst case. `--no-see-through` gives
  Phase 1's bench.

**`--no-see-through` where there is nothing to cut is accepted, with no effect.** That
covers `render` without `--buildings` or without `--camera`, and `view` without
`--buildings`. The draft proposes no error there:
- the flag asks for today's frames, and those renders already give them;
- a harness can then pass `--no-see-through` on every call to keep today's output,
  whatever else it passes;
- an error would make the flag depend on two others for no gain.

**No new error.** The two errors drafted for `--see-through` go with it. Every check runs
in its own order as today. A flight on urban_grid with `--buildings` still stops at
`--buildings`' "needs a georeferenced network" (§2.4.3), with or without the flag.
`--see-through` is not a flag, so clap rejects it as a usage error (exit 2).

**Existing scripts that render with `--buildings --camera`** now render cut:
`scripts/gates-ties.sh` gate 9 and `scripts/gates-city.sh` gate 13. Both compare their
runs only with each other, so they still hold, and neither is edited (Phase 3 gate 3).

**The flight-file setting** was the alternative (OQ-14). It is not taken.

#### 2.15.7 What it does not do

- **Buildings outside the wedge still hide traffic.** Over the frame, boxes hidden fall
  from 79.2 % to 54.8 % on the orbit flight, and from 47.5 % to 46.3 % on the city
  flight. That is the user's look (decision 2): the street at the look-at point shows,
  and the rest of the city stands. Wider cuts are measured above (0.20), and anything
  beyond the wedge is OQ-15.
- **A stub still covers a road that passes under its footprint.** For example, at 330 s
  six queued vehicles near (−337, 400) stand inside the footprint of a 10 m building
  (`196bfb75-…`). Midtown's roads under footprints are 2.239 % of the
  centreline (Phase 1 gate 10).
- **Pick, pan and zoom** keep their planes (§2.8). A click on a stub picks as a click on
  the building did.
- **Building parts** (Phase 4, decision 4). When parts come, a part is cut with its
  building: the wedge test is on the building's footprint, and each part's top is
  clamped to `h′`. Phase 4 settles a part whose `min_height` is above `h′`.
- **Not in Phase 3:** fading or transparency (decision 2); see-through in the
  orthographic render (§2.15.4); per-keyframe see-through; shadows.

#### 2.15.8 Build cost

0 packages, no Bevy feature, no `Cargo.toml` change. One new module, `src/see_through.rs`,
with no Bevy types. At run time, with see-through on, the building mesh's `f64` positions
(Midtown: 213,880 vertices) are kept beside the Bevy mesh, about 5 MB, with a copy of
the `Buildings`. `render` with `--no-see-through`, and every render without buildings or
without `--camera`, keeps or spawns nothing new beyond the buildings' mesh entity.
`view --no-see-through` keeps the `Buildings` it already reads, and builds the rest only
when `X` first turns the cut on.

#### 2.15.9 Measured while drafting (2026-10-03)

A throwaway probe crate in gitignored `scratch/vis002p3-probe/` (release build, Apple M3,
sharing this repo's `target/`). It used the library's public API at `2b25d5e`
(`run::load`, `keyframes::Flight`, `buildings::read`, `draw::*`) and a copy of
`src/render.rs:Renderer`'s perspective path whose building mesh can be replaced. Nothing
in `src/` or `tests/` was touched. The record is `specs/reviews/vis-002.md`, "Phase 3
draft".
- **The copy is the shipped renderer:** frames 0, 450, 900, 1350 and 1799 of the city
  flight with buildings, through the copy and through `Job::prepare_without_credit`, are
  byte-identical.
- **The flights:** `tests/city-flight.toml`, and an orbit of (−675, 375) (the probe's
  `orbit-flight.toml`, proposed as `tests/see-through-flight.toml`): four keyframes at
  300, 320, 340 and 360 s, `height_m` 250, pitch 35, yaw 0, 120, 240 and 0, so one full
  turn clockwise. (−675, 375) is the second-busiest 50 m cell of the window (428
  box-seconds over 300–360 s at a mean 9.9 m/s). The busiest (−325, 375) is OQ-4's queue
  at 0.2 m/s.
- **Occlusion, geometric:** sight lines from the eye to each box's top centre, and to
  road points about every 10 m (3,178 of them), tested against every footprint prism at
  its drawn height, over all 1800 frames. The tables are in §2.15.2 and §2.15.3.
- **Occlusion, in pixels:** for every 15th frame (120 frames), a box pixel is one that
  differs between the frame with boxes and the same frame without (`render(&[])`).
  - Orbit flight, off → on: centre 59,506 → 159,502, middle quarter 91,305 → 258,570, and
    whole frame 168,189 → 387,849.
  - City flight, off → on: 3,928 → 4,719, 42,502 → 45,322, and 90,710 → 99,748.
- **Cost**, interleaved per frame on the same pose, so the machine's load (other
  sessions) cancels. Each flight ran 900 pairs at a load average of 1.5–1.9:
  - orbit: render median 18.74 ms without the rebuild and 22.86 ms with it (p99 20.3 and
    26.6);
  - city: 14.99 and 18.05 ms;
  - the heights and the mesh build and insert are 0.48–0.55 ms of that.
- **Deterministic:** two runs of each flight with the cut gave the same hash over all
  1800 frames: `b1d3d025994688bf` (city) and `95f90640178fa91e` (orbit).
- **The synthetic case** (gate 6's scene): the box's pixels are 0 without the cut and
  1,564 with it, which is also the count without buildings. The cut frame equals the
  frame built from a file with the in-the-way block at 3 m: 0 pixels apart. Every pixel
  that changes lies inside the in-the-way block's image rectangle.

### 2.16 Building parts (Phase 4)

Drafted 2026-10-03, the phase §2.15.1 decision 4 recorded. Overture's `building` footprint
carries the whole building's height, so a tower on a podium is drawn today as one slab as
tall as the tower, and a building on stilts stands on the ground (OQ-5). Overture's
`building_part` type describes each volume of such a building with its own base and top.
This phase draws a building that has parts from its parts, and honours raised bases. The
numbers below come from the raw data saved and the probe run while drafting (§2.16.10).

#### 2.16.1 The user's decisions (decision, recorded)

Decided by the user, 2026-10-03, at §2.15.1 decision 4, and not reopened here:
- **a building with `building_part`s is drawn from its parts**, each standing from its own
  `min_height`;
- **raised bases are honoured**;
- **flat roofs and the neutral grey stay**: no roof shapes, and no Overture colours.

Given by the user with the brief, 2026-10-03:
- **The raw data is saved first.** Release `2026-09-23.1` leaves S3 about 2026-11-22
  (§2.14.3), and the cache keeps only `id`, `height`, `num_floors` and `sources`, so it has
  neither the parts nor the buildings' `min_height`. Before anything else, every column of
  `type=building` and of `type=building_part` for Midtown's box (§2.3.1, 250 m) was saved
  as parquet in gitignored `scratch/overture-2026-09-23.1/`, read as
  `scripts/fetch-buildings.sh` reads, `ORDER BY id` (§2.16.10).
  `scratch/midtown/buildings.geojson` was not touched.
- **The gates the draft must carry** (Phase 4's exit gate): today's cache renders
  byte-identical; with parts, two Midtown flights agree and the counts and the credit line
  are predicted from the saved data; a synthetic test fails without parts; `view`'s bench
  stops the build below 30 fps; 0 crates, or the reason; and the user's visual gate,
  written as what shows on screen.

Decided by the user, 2026-10-04, on the draft, before review (the answers to OQ-16 and
OQ-17, and the draft's two additions to the brief):
1. **A building with parts is drawn from its parts only** (OQ-16's (a)). The part of its
   footprint no part covers is not drawn.
2. **A part, or a raised base, wholly above its building's cut height lies as a lid at
   that height** (OQ-17's (a)): §2.15.7's clamp, as built.
3. **The third raw file is kept**:
   `scratch/overture-2026-09-23.1/building_part-of-buildings.parquet`, every part of the
   487 buildings with parts, selected by `building_id` (§2.16.2, §2.16.10).
4. **`scripts/fetch-buildings.sh` gains `--source <dir>`**, which reads a directory laid
   out like the release's `theme=buildings/` instead of S3 (§2.16.3).

The draft's third addition, `tests/see_through.rs` gaining `base: 0.0,` and `parts:
vec![],` in its two `Building` literals, was accepted the same day; Phase 4's gate 3
checks that nothing else in it changes.

Decided by the user, 2026-10-04, at review round 1's blocker (the fetch could no longer
write today's form, so `REFETCH=1` would have overwritten the cache every Phase 1–3 gate
reads):
5. **`scripts/fetch-buildings.sh` gains `--no-parts`**, which writes today's form: Phase
   2's query, byte for byte. The parts form stays the default. `scripts/fixture.sh
   midtown`'s step 5 and the `REFETCH=1` legs of `scripts/gates-city.sh` and
   `scripts/gates-credit.sh` pass it (§2.16.3).

The rest of this section is the draft's proposal, with OQ-16 and OQ-17 now decided. It
settles, each with one recommendation:
- the fetch and the cache (§2.16.3);
- what is drawn for a building with parts (§2.16.4, OQ-16);
- heights and bases (§2.16.5);
- raised volumes (§2.16.6);
- see-through with parts (§2.16.7, OQ-17);
- the credit line (§2.16.8);
- old caches (§2.16.9);
- the mesh, its cost and the build cost (§2.16.11).

#### 2.16.2 What Overture has for Midtown

From the saved data (§2.16.10). Midtown's box is §2.3.1's.
- **4,336 buildings**, as the cache. 487 have `has_parts`, and every one of the 487 has at
  least one part; of the 4,087 parts that meet the box, none belongs to a building without
  `has_parts`. (Parts beyond the box were saved only for the 487, §2.16.3's `--source`.)
- **4,209 parts** belong to those 487 buildings: 1 to 69 a building, median 5.
  - **A part need not meet the box.** The box gives 4,087 parts. The other 122 belong to
    buildings that straddle the box's edge, and lie wholly outside it. A query by
    `building_id`, over the fetched buildings' own extent, gives all 4,209. The same query
    over that extent widened by 0.01° on every side gives the same 4,209, byte for byte.
  - Every part is a `POLYGON` (no `MULTIPOLYGON`), valid (`ST_IsValid`), 2 with holes
    (2 holes). *(Corrected at review round 1: the draft's 28 and 32 were over all 8,058
    volumes, §2.16.11.)*
  - **Heights:** `height` on 4,152, `num_floors` only on 17, neither on 40. 1.0 to 472.0 m;
    the median drawn top (§2.16.5) is 66.0 m, and the median `height` 67.0 m.
  - **Bases:** `min_height` on 695, from 3.0 to 245.0 m (10th, 50th and 90th percentiles
    10, 70 and 148 m), 694 of them above 3 m. `min_floor` is never the only one given.
    No part has its base at or above its top.
  - `is_underground` on none. `roof_shape` on 3,659 and a colour on 3,431, both unused
    (§2.16.1).
  - **Datasets:** every part names `OpenStreetMap` only.
- **The buildings' own bases:** three have one. `2bd09890-…` (`min_height` 7.5, `height` 8,
  371 m²), `c92d28b5-…` (`min_floor` 2, so 7 m; `num_floors` 10, so 35 m; 1,140 m²) and
  `de4ad6d6-…` (`min_height` 3.7, `height` 4; it has parts).
- **A building's `height` is not its parts' top.** Of the 487, the tallest part is above
  the building's own height in 226 (by up to 405.8 m), below it in 222 (by up to 266.8 m),
  and equal in 39. So this phase lowers slabs and also raises towers that are missing
  today:
  - `62ebe85a-…`, 8,018 m² at 366 m today, keeps a 366 m part, and its other parts are
    lower; it loses the most volume of any building, about 1.9 million m³ (an estimate
    that ignores overlaps between parts);
  - `8dceada5-…` has no height and is a 10 m block today (9,076 m²); its parts reach
    260 m.
- **Three of the 487 are underground** (`is_underground`, no height, 10 m today):
  `8f543471-…` (155,767 m²), `ef5effa2-…` (30,090 m²) and `196bfb75-…` (13,260 m²), the
  footprint §2.15.7 found over a queue at (−337, 400). Their parts are what stands above
  ground.

#### 2.16.3 The fetch and the cache: one file, the script extended

| | (a) The script extended, one file | (b) A second script and file | (c) The script extended, two files |
|---|---|---|---|
| What | `fetch-buildings.sh` writes the parts into `buildings.geojson`, after the buildings | `fetch-parts.sh` writes `parts.geojson`; `render --building-parts <file>` | `fetch-buildings.sh --out` writes `<out>` and `<out>.parts.geojson` |
| Flags | none new for `render` and `view` | one more, which the harness must pass | none, but `render` finds a file by name |
| Travels with the cache | yes | only if both are passed | only if both are copied |
| The credit (§2.14.3) | reads every feature's `sources`: parts included, no code | a second provenance to read and check against the first | as (b) |
| Belongs to its cache | by construction | needs a check (a hash) | as (b) |

**The plan: (a).** It is §2.14.3's choice again: one self-describing file, which `render`
and `view` take with the same `--buildings`.

**The query.** The buildings' query is §2.14.3's, with two columns more. The parts are
selected by `building_id`, not by the box (§2.16.2), over a box that is the fetched
buildings' own extent (`min(bbox.xmin)`, … of the buildings selected). The script runs
DuckDB twice, so that the extent reaches the parts' filter as literals, which DuckDB pushes
down into the parquet scan:
1. the first run writes the selected buildings, every column, to `$TMP/b.parquet`, and
   prints their extent, `<W2> <S2> <E2> <N2>`;
2. the second reads them back as `b` and writes the file.

The buildings are read from `<source>` once. (A DuckDB database file in `$TMP` cannot carry
them: DuckDB 1.5.1 refuses a geometry column with a CRS in its default storage version,
measured at review round 1.) The extent is passed as DuckDB prints it, shortest round-trip
doubles (`-73.9946517944336`, …), unrounded. When run 1 selects no building, its extent is
`NULL`, and run 2 leaves out the parts' `SELECT`: the file holds no feature, and
`--buildings` on it fails as §2.4.3 says of an empty `features`.

```sql
-- run 1
COPY (SELECT * FROM read_parquet('<source>/type=building/*.parquet')
      WHERE bbox.xmin <= <E> AND bbox.xmax >= <W> AND bbox.ymin <= <N> AND bbox.ymax >= <S>)
  TO '<tmp>/b.parquet';
SELECT min(bbox.xmin), min(bbox.ymin), max(bbox.xmax), max(bbox.ymax)
  FROM read_parquet('<tmp>/b.parquet');
-- run 2, with <W2> <S2> <E2> <N2> from run 1
CREATE TEMP TABLE b AS SELECT * FROM read_parquet('<tmp>/b.parquet');
COPY (
  SELECT id, NULL::VARCHAR AS building_id, height, num_floors, min_height, min_floor,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources, geometry
  FROM b
  UNION ALL
  SELECT id, building_id, height, num_floors, min_height, min_floor,
         list_sort(list_distinct([s.dataset FOR s IN sources])) AS sources, geometry
  FROM read_parquet('<source>/type=building_part/*.parquet')
  WHERE bbox.xmin <= <E2> AND bbox.xmax >= <W2> AND bbox.ymin <= <N2> AND bbox.ymax >= <S2>
    AND building_id IN (SELECT id FROM b)
  ORDER BY building_id NULLS FIRST, id
) TO '<tmp>/buildings.geojson' WITH (FORMAT GDAL, DRIVER 'GeoJSON',
      LAYER_CREATION_OPTIONS 'DESCRIPTION=Overture Maps buildings, release <release>');
```

- **The fields kept:** a part's `id`, `building_id`, `height`, `num_floors`, `min_height`,
  `min_floor`, `sources` (datasets) and `geometry`; a building gains `min_height` and
  `min_floor`, and a `building_id` of `null`. Nothing else is drawn (§2.16.1), so nothing
  else is kept.
- **The order:** the buildings first, by `id`, then the parts, by `building_id` and then
  `id`. The buildings are today's rows in today's order. Taken out of the file, the three
  new members (`"building_id": null, ` and `, "min_height": …, "min_floor": …`) leave each
  building's line equal to today's cache's, byte for byte (§2.16.10). The last building's
  line then also ends in a comma, since parts follow it.
- **The bytes are deterministic.** The same release and box give the same file: Midtown's,
  written twice, is 4,944,032 bytes both times, SHA-256 `714e2f3a…` (§2.16.10). It holds
  8,545 features.
- **The description** is unchanged, `Overture Maps buildings, release <r>`, so §2.14.3's
  check reads either form.

**`--source <dir>`** (new, optional). It replaces the default
`s3://overturemaps-us-west-2/release/<release>/theme=buildings` with a local directory laid
out the same way: `<dir>/type=building/*.parquet` and `<dir>/type=building_part/*.parquet`.
Given, its value must be a directory (`error: --source: no directory <dir>`, and nothing at
`--out`); an `s3://` value is not accepted. Not given, the script reads S3 as today.
- It makes the fixture's cache reproducible after `2026-09-23.1` leaves S3, from the data
  saved while drafting, and lets the Phase 4 gates fetch offline.
- `--release` still names the release in `description`. With `--source`, the script cannot
  check that the files are that release; the README says so.
- Midtown through `--source` and through S3 is predicted to give the same rows: the saved
  parquet was read from S3 with the same filters (§2.16.10). The one difference S3 could
  show is a part, beyond the box, of a building without `has_parts`, which the saved third
  file would not hold (§2.16.2); none meets the box. Through the mirror, the saved
  `building.parquet` rebuilds today's cache with `--no-parts` byte for byte (`f241ccbd…`),
  and the parts form is `714e2f3a…` twice (§2.16.10).

**`--no-parts`** (new, optional; §2.16.1 decision 5). It writes today's form: Phase 2's
query, one DuckDB run, no parts and no `building_id`, `min_height` or `min_floor`, and the
report of today, without `raised` or `parts`. It combines with `--source`. It exists so
that the fixture's `buildings.geojson`, which every gate of Phases 1–3 reads, can still be
fetched, and so that those phases' network gates keep their meaning.

**The report** (§2.3.4) keeps every key and its meaning, and gains two. The existing keys
(`buildings`, `height`, `num_floors`, `default`, `sources`, `no_sources`) count the
features without a `building_id` only, as today; the parts are counted apart:
- `"raised"`: the buildings with a base above 0 (§2.16.5), 3 for Midtown;
- `"parts"`: an object, `{"count": 4209, "buildings": 487, "height": 4152, "num_floors":
  17, "default": 40, "raised": 695, "sources": {"OpenStreetMap": 4209}, "no_sources": 0}`
  for Midtown, counted over the written file. `buildings` is the buildings with at least
  one part.

The fixture (§2.9) keeps `scratch/midtown/buildings.geojson`, today's form, which every
gate of Phases 1–3 reads. `scripts/fixture.sh midtown` changes in three places:
- **step 5** passes `--no-parts`, so that `REFETCH=1` writes today's form again, and
  `--source scratch/overture-2026-09-23.1/theme=buildings` when that directory exists;
- **step 6** (new) writes `scratch/midtown/buildings-parts.geojson`, the parts form, only
  if absent or `REFETCH=1`, with the same `--source` rule; its report line goes to
  `scratch/midtown/fetch-parts.log`;
- **`FORCE=1`** keeps both caches and both logs: step 2's `find … ! -name buildings.geojson
  ! -name fetch.log` gains `! -name buildings-parts.geojson ! -name fetch-parts.log`.

The `REFETCH=1` legs of `scripts/gates-city.sh` (gate 5) and `scripts/gates-credit.sh` (gate
4) each pass `--no-parts` to their second fetch, and nothing else in them changes. Gate 5's
missing-release case keeps its meaning with or without it.

#### 2.16.4 What is drawn for a building with parts: its parts only (OQ-16)

Decision 1 says a building with parts is drawn from its parts. What it leaves open is the
part of the footprint no part covers. Measured over the 487 (§2.16.10):
- **the parts cover the footprint to within 1 m² in 414**: 65 have more than 1 % of it
  uncovered, and 50 more than 10 %;
- **196,624 m² of footprint is uncovered**, of 1,222,541 m² (16 %). 160,808 m² of it is
  the three underground buildings' (§2.16.2); the other 35,817 m² is spread over 70
  buildings;
- parts reach outside their building's footprint by 101.9 m² in all (13 buildings over
  1 m²), which is drawn as it is.

| | (a) The parts only | (b) The parts, and the rest of the footprint at the building's height | (c) The parts, and the rest at a fixed low height |
|---|---|---|---|
| The three underground buildings | their parts only | 160,808 m² drawn at 10 m, as today | 160,808 m² drawn low |
| Towers whose footprint the parts do not cover | the parts | the rest as tall as the building: up to 472 m, the slab this phase removes | a low slab under them |
| Geometry | none | a polygon difference per building: a crate (+1 package at least) or one more `ST_Difference` column in the fetch | as (b) |
| Decision 1 | as written | adds a volume the data does not describe | as (b) |

**The recommendation: (a).** It is decision 1 as written. The uncovered rest has no height
of its own, and the largest part of it is underground. (b) puts back the slab this phase
removes, and (c) invents a height. The user judges what (a) leaves at Phase 4's gate 15,
which describes the largest of these changes on screen.

*(2026-10-04)* **Decided by the user: (a), the parts only** (§2.16.1, decision 1).

#### 2.16.5 Heights and bases

Each feature drawn is a **volume** with a base and a top:
- **the top** is §2.4.2's rule, unchanged: `height`, else `num_floors` × 3.5 m, else 10 m.
  A part with neither is 10 m, like a building: 40 of Midtown's parts.
  - The building's height is not used for its parts (§2.16.2: it is above the tallest part
    in 222 and below it in 226). Taking it as a part's default would make a podium as tall
    as the building, the slab this phase removes.
- **the base** follows the same pattern: `min_height`, else `min_floor` × 3.5 m, else 0.
  - `min_height` wins over `min_floor`, as `height` wins over `num_floors`.
  - The same 3.5 m a storey as `num_floors`, so `min_floor` 2 is 7 m (`c92d28b5-…`).

A building with parts is drawn from its parts: its own base and top are not drawn, though
its `height` and `num_floors` are still checked as today (§2.4.3). A building without parts
is one volume: its own polygons, from its base to its top. With base 0 that is today's
building, vertex for vertex.

**A building's drawn top** is the highest top of its volumes: its tallest part's, or its
own height. `tallest` (§2.8, the orthographic eye) and see-through (§2.16.7) take it. For a
building without parts it is its `height`, the same `f64`. Midtown's tallest stays 472.0 m,
so its orthographic eye does not move.

**The checks** (§2.4.3) gain these, each `error: --buildings <file>: feature <id>: …`, in
each feature's own order after today's:
- a `building_id` that is neither absent, `null` nor a string: `building_id 7 is not a
  string`;
- a `min_height` that is used and is not a finite number ≥ 0: `min_height -1 is not a
  finite number of at least 0`; or a `min_floor` that is used and is not an integer ≥ 0:
  `min_floor 1.5 is not an integer of at least 0`. "Used" is the base rule's: `min_height`
  when it is not null, else `min_floor` when it is not null. A building with parts has its
  base read and checked like any other, though it is not drawn;
- a volume whose base is not below its top: `base 30 m is not below its top 20 m`. It is
  checked on a building only when the building has no parts, since otherwise it is not
  drawn.

Whether a building has parts is known before any feature is checked: `read` already parses
the whole file first (`src/buildings.rs:read`), and the set of buildings with parts is every
string `building_id` of the file. A `building_id` that is not a string adds nothing to it,
and fails at its own feature's check.

Then, after every feature and before `map_origin`, in file order: a part whose
`building_id` names no building of the file, `part of <bid>, which is not a building of
the file`. A part of a part names a part, so it is caught here too. Midtown has none of
these: no orphan part, and no base at or above its top.

#### 2.16.6 Raised volumes: walls from the base, and no underside

A volume with a base above 0 has its walls from the base to its top, and its roof at its
top. 697 of Midtown's 8,058 volumes are raised (695 parts and 2 buildings).

**The underside is not drawn**, and no frame can show it:
- it would face down, so with back faces culled it shows only to an eye below it;
- every view ray in this renderer descends: the pitch floor is 25°, above half the 45°
  field of view (`src/camera.rs:PITCH_MIN`, `src/camera.rs:FOV_DEG`; the floor "keeps the
  horizon out of every frame"), and the orthographic camera looks straight down. So no
  visible point lies above the eye, and an underside above the eye is never in sight;
- the open bottom of a raised volume is seen only from below, for the same reason.

So an underside would cost 4,093 triangles and 5,487 vertices on Midtown and change no
pixel. The synthetic probe agrees: with and without it, its frames are 0 pixels apart
(§2.16.10). If vis-001 OQ-12 ever lowers the pitch floor below 22.5°, this changes, and
that phase draws the underside (§2.13).

A raised volume's walls are today's quads with their lower edge at the base. The
see-through rule needs nothing more (§2.16.7).

#### 2.16.7 See-through with parts: cut with the building (OQ-17)

§2.15.7 already says how parts are cut: the wedge test is on the building's footprint,
and each part's top is clamped to `h′`. So:
- **`δ` is unchanged.** It is measured from the building's own polygons, so every building
  is in the way exactly when it is today.
- **`h′` takes the drawn top** (§2.16.5) for `h`: `min(top, s + (top − s)·smoothstep(δ/e))`.
  With `δ ≥ e` it is `top`, and every vertex keeps its `z`. For a building without parts,
  `top` is `height`, so its `h′` is today's, bit for bit.
- **The drawing is unchanged:** `draw::BuildingsCut` gives every vertex of building `b`
  `z′ = min(z, h′_b)`, its parts' vertices included, since `building_mesh` appends a
  building's volumes in order.

What §2.15.7 left to this phase is a volume whose base is at or above `h′`. The clamp
puts its base and top both at `h′`: its walls have no height, and its roof lies at `h′`,
a lid. Where a lower part of the same building lies under it, the lid is in the plane of
that part's roof and changes no pixel: the synthetic tower on its podium, cut, is 0 pixels
apart from a 3 m slab of the podium (§2.16.10). Where nothing lies under it, an overhang,
the lid covers the street below it at `h′`.

| | (a) The lid at `h′` (the clamp) | (b) The volume removed once `h′` falls to its base |
|---|---|---|
| An overhang in the way | a lid at `h′`, down to 3 m, over the street, as Phase 3 draws that footprint today (the whole building at `h′`) | gone: the street under it shows |
| As the wedge reaches it | continuous: the lid sinks with `h′` | its roof, at its base, vanishes in one frame: a pop (§2.15.3 has none) |
| Code | none: the clamp as built | each vertex's volume and its base kept beside its building; a removed volume's vertices collapsed to one point |
| Exactness (Phase 3 gate 6's form) | the cut frame equals the frame of a file with each part's base and top at `min(·, h′)` | the same, with removed parts left out |

Measured on the two flights, with see-through on (§2.16.10): on the city flight, 1,157
frames have at least one part at or above its `h′`, and 726 have one with more than 10 m²
over no ground part of its building (a part with a base of 3 m or less), up to 2,567 m² of
such lids in one frame. On the orbit, 248 frames have such a part, and none over nothing.
These counts are over every frame's whole wedge, not only what is in frame. Over all 694 raised parts, 19,841 m² of their area lies over
no ground part of their building, in 103 parts of 41 buildings: 11 % of the raised parts'
180,766 m².

**The recommendation: (a), the lid.** It is §2.15.7's rule as written, it never pops, and
where it covers a street, Phase 3 covers it too today. (b) shows a little more street on
the city flight, at the cost of a pop where §2.15.3 promised none.

*(2026-10-04)* **Decided by the user: (a), the lid** (§2.16.1, decision 2).

#### 2.16.8 The credit line

§2.14.2's item 3 names each dataset that at least one feature of the file names, parts
included: `src/credit.rs` reads every feature's `sources` (§2.14.3) and needs no change.
Every Midtown part names OpenStreetMap only (§2.16.2), so the line with the new cache is
the line today:

`© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1 ·
Microsoft ML Buildings (ODbL) · USGS Lidar`

Phase 4 adds no item and no wording. A part naming a dataset no building names is credited
as §2.14.2 already says.

#### 2.16.9 Old caches: read, and drawn as today

A cache written before Phase 4 has no `building_id`, `min_height` or `min_floor`. Read by
Phase 4, every feature is a building with base 0 and no parts, so it is drawn vertex for
vertex as today, cut as today and credited as today. Every frame of a render with it is
byte-identical to today's (Phase 4 gate 2).
- **No error and no warning.** Phase 2 made an old cache an error because the credit
  needed its release. Parts change no credit (§2.16.8), and a cache without them is still
  a correct, if plainer, picture. The README says to fetch again for parts.
- `tests/shapes.geojson`, hand-written and without parts, reads as before.

#### 2.16.10 Measured while drafting (2026-10-03)

The record, with every query, is `specs/reviews/vis-002.md`, "Phase 4 draft". Read-only
against S3 and the Midtown fixture; output only under gitignored `scratch/`; nothing in
`src/` or `tests/`.
- **The raw data**, saved first (§2.16.1), in `scratch/overture-2026-09-23.1/`, every
  column, through DuckDB 1.5.1 as the fetch script reads, `ORDER BY id`:

  | File | What | Rows | Bytes | SHA-256 |
  |---|---|---|---|---|
  | `building.parquet` | `type=building`, the box | 4,336 | 1,023,962 | `2d41d5ed…` |
  | `building_part.parquet` | `type=building_part`, the box | 4,087 | 838,028 | `3af90053…` |
  | `building_part-of-buildings.parquet` | `type=building_part` of the 487, over their own extent (§2.16.3) | 4,209 | 861,070 | `92f19431…` |

  The first two took 19 min 39 s together; the third 23 s.
- **The saved buildings are the cache's:** Phase 2's query over `building.parquet` gives
  today's `scratch/midtown/buildings.geojson` byte for byte (2,284,830 bytes,
  `f241ccbd…`).
- **The new cache**, §2.16.3's query over the saved parquet: 4,944,032 bytes, `714e2f3a…`,
  twice.
- **The probe** (`scratch/vis002p4-probe/`, a crate with a path dependency on this repo at
  `fb53d7f`, built in release in this repo's `target/`). It copies
  `src/render.rs:Renderer` without the credit, with a building mesh given as data (so that
  parts, bases and undersides can be drawn and cut), and draws §2.16.5–§2.16.7's rule.
  - **The copy is the shipped renderer:** frames 0, 450, 900, 1350 and 1799 of the city
    flight with today's cache and see-through on are byte-identical through the copy and
    through `Job::prepare_without_credit` with `set_see_through(true)`.
  - Its mesh for today's cache equals `buildings::mesh_data`'s, position for position.
- The counts, the flights, the synthetic scenes and the costs are in §2.16.11 and in
  Phase 4's gates.
- *(2026-10-04, review round 1)* **Through a mirror** laid out as `--source` reads it (the
  saved `building.parquet` under `theme=buildings/type=building/` and
  `building_part-of-buildings.parquet` under `theme=buildings/type=building_part/`, in the
  session's scratch, DuckDB 1.5.1), so with hive partitions detected:
  - Phase 2's query (`--no-parts`) gives `f241ccbd…`, today's cache;
  - §2.16.3's two runs, with `b` handed over as `$TMP/b.parquet`, give 4,944,032 bytes,
    `714e2f3a…`, twice. The extent run 1 prints is `-73.9946517944336 40.75189208984375
    -73.96367645263672 40.774688720703125`, the probe's literals.

#### 2.16.11 The mesh, its cost, and the build cost

Midtown's building mesh, today and with parts (the probe, §2.16.10):

| | Volumes | Raised | Wall quads | Roof triangles | Triangles | Vertices | Indices |
|---|---|---|---|---|---|---|---|
| Today's cache | 4,336 | 0 | 42,776 | 34,178 | 119,730 | 213,880 | 359,190 |
| With parts | 8,058 | 697 | 67,367 | 51,315 | 186,049 | 336,835 | 558,147 |
| With parts and undersides (not drawn, §2.16.6) | 8,058 | 697 | 67,367 | 51,315 + 4,093 | 190,142 | 342,322 | 570,426 |

- 8,058 volumes = 4,336 − 487 + 4,209. The roofs follow §2.6's Σ (n + 2h − 2): 67,367
  ring vertices, 32 holes (30 in the buildings without parts, 2 in parts) and 8,058
  polygons give 51,315, and `earcut` 0.4.11 gives
  exactly that.
- **The mesh grows by 57 %.** It is still one mesh, one draw, opaque (§2.15.5).
- **Cost.** With see-through on, the mesh is rebuilt on most frames (§2.15.5): 1,653 of
  the city flight's 1,800 and all 1,800 of the orbit's, today and with parts alike.
  Interleaved on the same pose, 900 pairs a flight at a load average of 5–12, a frame with
  the mesh rebuilt and rendered takes, at the median:
  - city flight: 22.25 ms today and 25.02 ms with parts (p99 31.14 and 34.00);
  - orbit: 20.13 and 22.70 ms (p99 25.27 and 27.90).

  So the parts add 2.6–2.8 ms a frame. Computing the heights and building the mesh is
  0.51 → 0.70 ms (city) and 0.46 → 0.65 ms (orbit) of it; the rest is Bevy uploading 57 %
  more vertices. Phase 3's `view --bench`, rebuilding every frame, gave 59.43 fps (median
  16.65 ms), so the bench is predicted to stay above 30 fps (Phase 4 gate 14).
- **0 packages.** No crate is added: the parts are GeoJSON read with `serde_json`, the
  roofs use `earcut`, and no polygon clipping is needed (OQ-16's (a)). `Cargo.toml` and
  `Cargo.lock` do not change; no Bevy feature is added.
- **No network** in `render` or `view`; the fetch alone uses it, and with `--source` not
  even that.

### 2.17 Streets (Phase 5)

Drafted 2026-10-05, the roadmap's "Streets" item (§2.13). Today a road is a grey strip per
link and nothing else: every junction is a dark hole between the strips, so a vehicle
crossing it is drawn over the background, and a two-way street shows a dark seam down its
middle. This phase draws the junctions' surfaces and the US markings: lane lines, stop
lines and the yellow centre line. It changes no box, building, camera or credit line. The
numbers below come from a probe run while drafting (§2.17.15).

#### 2.17.1 The user's decisions (decision, recorded)

Decided by the user, 2026-10-05, before drafting. The user said they can be polished later
if they do not work.
1. **Streets is vis-002 Phase 5,** not a new spec.
2. **Scope:** junction surfaces, lane lines and stop lines.
   - **No crosswalks:** neither fixture defines a crossing (`crossings` is empty in both,
     `NetworkData::crossing_configs` is 0), and none is invented.
   - **No curbs or sidewalks:** they come later, with the ground (§2.13).
3. **Look: US markings.**
   - White dashed lines between lanes;
   - white stop lines;
   - a yellow centre line where a two-way pair meets;
   - junction surfaces flat, opaque, the road's grey.

   *(2026-10-08)* The markings in this decision are replaced by the engine dashboard's
   when Phase 6 ships (§2.18.1, decision 2). The junction surfaces stay.
4. **On by default** in `render` and `view`, orthographic and perspective.
   - An off flag gives today's frames, byte-identical;
   - a key toggles it in `view`, as see-through's `X` does (§2.15.6).

By the methodology's §6.1: step 0, a decision changes (what a road is drawn as); step 1,
nothing shipped is removed (the flag gives today's frames); step 2, vis-002 owns the scene
around the traffic and its roadmap names this item, so a phase is appended (decision 1).
vis-001 is not edited.

Decided by the user, 2026-10-05, on the draft, before review (the answers to OQ-18 and
OQ-19, and three calls on the draft):
5. **The median gap is a painted median** (OQ-18's (a)): filled with the road's grey,
   with one double yellow line in a gap under 1 m and a double yellow line inside each
   edge of a wider one (§2.17.5). *(2026-10-08)* The yellow lines are replaced when Phase
   6 ships: a pair with a gap shows its grey fill and no line, and the engine's one
   centre line is drawn only where the gap is 0 (§2.18.1, decision 5; §2.18.3). The fill
   stays. *(2026-10-09, changed by the user at gate 14)* The fill is replaced too when
   Phase 6 ships: a pair's gap shows the background, as with `--no-streets`, its junction
   ends rounded by `NetworkJson::median_noses` in the road's grey (§2.18.1, decision 5 as
   changed; §2.18.16).
6. **Markings fade where they are under a pixel** (OQ-19's (a), §2.17.10).
7. **`M` is the key** that shows and hides streets in `view` (§2.17.12).
8. **Stop lines stay 0.60 m deep,** though the engine stops a vehicle's front 0.6 m
   behind the line's junction-facing edge, a standoff written for a 0.4 m line
   (§2.17.7). *(2026-10-08)* Replaced when Phase 6 ships: its stop lines are the
   engine's, 0.4 m deep (§2.18.1, decision 2; §2.18.7).
9. **The engine's own street geometry for its dashboard was considered** and the draft
   draws its own; §2.17.2 says why and compares the two. *(2026-10-08)* Reversed when
   Phase 6 ships: it draws the dashboard's markings from that geometry (§2.18.2).

The rest of this section is the draft's proposal. It settles:
- what the engine gives, and that no engine request and no crate is needed (§2.17.2);
- the strips, kept (§2.17.3), and the junction surfaces (§2.17.4);
- the median gap (§2.17.5, OQ-18, answered (a));
- the markings' widths, patterns and colours (§2.17.6), the stop lines (§2.17.7) and the
  yellow line (§2.17.8);
- lifts and draw order (§2.17.9);
- what a viewer can see of a 0.15 m line, and the fade (§2.17.10, OQ-19, answered (a));
- the meshes and their cost (§2.17.11);
- the flag, the key and the scripts (§2.17.12).

#### 2.17.2 What the engine gives: everything, so 0 crates and no engine request

`src/place.rs:Placement::new` already builds the engine's
`crates/core/src/network_data.rs:NetworkData::from_config` and keeps it as
`Placement::data`, at the pin (`90b39292`). Phase 5 reads, read-only:
- **`NetworkData::junction_polygons`**: one polygon per `junction` or `waypoint` node, in
  metres. Neither fixture gives explicit vertices (`JunctionConfig::geometry` has only a
  `setback`), so each is the engine's own `generate_junction_polygon` result. Its last
  vertex is a copy of its first (the last corner arc ends on the first arm's tip): 44
  vertices at a four-arm junction, 33 at a three-arm one;
- **`NetworkData::lane_stop_line_offset(link, lane)`**: where the engine stops a vehicle,
  as a distance back from the link's trimmed end. It is 0 for a link trimmed at a junction
  polygon, plus the crossing pushback (none here) and the per-lane delta from
  `JunctionConfig::stop_line_offsets`;
- `assimilator_config::network::LinkConfig`'s `lanes`, `total_width()`,
  `lane_center_offsets()` and `median_gap`, and `JunctionConfig::control`.

**The engine's own street drawing was considered** (decision 9). For its dashboard the
engine builds street geometry in
`crates/geometry/src/network_json.rs:NetworkJson::from_config_with_network_data`, public,
in `assimilator-geometry`, which this crate already builds. It differs from §2.17.6–§2.17.8:
- **dashes 2.5 m long every 6.5 m** (`DASH_LENGTH` 2.5, `DASH_GAP` 4.0), against the
  draft's 3 m every 12 m: 3,921 in Midtown and 2,044 in urban_grid against 2,208 and
  1,105;
- **the centre line is one 0.15 m line,** and only on a pair with a zero gap (3 in
  Midtown, none in urban_grid); a pair with a gap shows the gap, with "median noses" at
  its ends. The draft draws a double yellow line on every pair and fills the gap;
- **stop lines 0.4 m deep,** one rectangle across the whole approach unless a lane has
  its own delta, and **also at unsignalised approaches** whose lanes have conflicting
  paths (`NetworkData::is_transparent_lane_path`): 39 rectangles on 38 of Midtown's
  approaches that the draft leaves unmarked (§2.17.7);
- it carries no colour, height or triangulation: its fills and lines are 2D rings for a
  dashboard to paint.

**Where they share data, they agree exactly** (`streets-engine`, §2.17.15):
- its `junction_fills` are `NetworkData::junction_polygon`'s rings: 106 and 9. Each ring
  ends with two copies of its first vertex, the polygon's own and the one `NetworkJson`
  pushes to close it (45 vertices at a four-arm junction); the draft's surface drops the
  polygon's one (43). With every trailing copy of the first vertex stripped, each ring
  equals its surface's polygon vertex for vertex (worst distance 0.000000 m);
- its `stop_lines` at signals put their junction-facing edge where the draft's stop lines
  end: every corner of the draft's 402 and 71 lines lies on the engine's edge for that
  link and lane (804 and 142 corners, worst 0.000000 m).

**Why the draft draws its own:** the user decided the US look (decision 3), which differs
from the dashboard's in the dashes, the centre line and the stop lines' places; the data
item needs a stop line per `(link, lane)` (§2.17.7), where the dashboard draws one per
approach; and the draft needs heights, colours and triangles, which the dashboard's rings
do not have. Building the whole `NetworkJson` would also build its turn paths, conflict
points, arrows and detectors for nothing. Since the shared geometry agrees to 0 m, drawing
our own puts nothing anywhere the engine would not.

The strips' geometry is unchanged (§2.17.3): the trimmed, offset polyline through
`Placement::place_lateral`. The other fields the engine exposes (`junction_curb_arcs`,
`arm_widths`, `trimmed_link_geometry`, `link_total_widths`, `lane_offsets`) are not
needed: the curb arcs are already the polygons' rounded corners, and the rest reach
Phase 5 through the placement and the link config. **No engine request** (confirmed
against the pin's sources, 2026-10-05).

The fixtures, as the engine builds them (`streets-probe`, §2.17.15):

| | Midtown | urban_grid |
|---|---|---|
| Links | 255 | 48 |
| Lanes per link (1 / 2 / 3 / 4) | 63 / 119 / 60 / 13 | 1 / 47 / 0 / 0 |
| Two-way pairs (links) | 29 (58) | 24 (48) |
| Pairs by median gap | 0 m: 3; 0.5 m: 11; 3.5 m: 4; 4 m: 11 | 0.5 m: 24 |
| Junctions (signal / unsignalised, rule) | 106 (83 / 23, every one with no `rule`) | 9 (9 / 0) |
| Junction polygons, all generated: vertices as given (after the closing copy is dropped) | 106, 33–44 (32–43) each, 18,859 m² | 9, 44 (43) each, 4,628 m² |
| Setback | 3 m at 103, 5 m at 2, 6 m at 1 | 5 m at 9 |
| Waypoints; crossings; lanes with `gap_after` | 0; 0; 0 | 0; 0; 0 |
| Per-lane stop-line deltas | none | `L_J12_J11` lane 0: 5 m |
| A node pair with two links | none | none |

Midtown's `import_report.json` counts the raw import (327 junctions); `network.yaml` is
what is drawn.

#### 2.17.3 The strips: kept as they are, since they already end at the junction edge

The brief asked whether to keep today's strips or end them at the junction edge. **They
already end there.** `src/scene.rs:strips` samples `Placement::place_lateral`, which is
the engine's `crates/geometry/src/geo_util.rs:LinkGeometryIndex::interpolate_with_lateral`
on the polyline `LinkGeometryIndex::from_network_config` builds from
`NetworkData::trimmed_geometry`: the link cut where its centreline enters the junction
polygon, then offset right by `total_width/2 + median_gap/2` on a two-way pair. Measured:
- **every strip-end corner lies on its junction polygon's boundary:** 922 corners in
  Midtown and 144 in urban_grid, the largest distance 0.000 m (under 0.5 mm), none
  outside by more than 5 cm;
- **so the dark holes are exactly the polygons**, and filling them meets the strips with
  no gap and no overlap.

**Placed vehicles sit on the drawn road.** Every vehicle's placed point at every whole
second of each run, tested against the strips alone and against the strips with the
junction surfaces:

| | Box centres | On strips (today) | On strips and junction surfaces | In a junction span: on strips / on drawn road | Box corners on drawn road (today) |
|---|---|---|---|---|---|
| Midtown | 201,256 | 91.202 % | **100.000 %** | 0.000 % / **100.000 %** (17,706) | 99.957 % (91.106 %) |
| urban_grid | 21,358 | 94.236 % | **100.000 %** | 0.000 % / **100.000 %** (1,231) | 99.963 % (94.183 %) |

Today every vehicle inside a junction is drawn over the background; with the surfaces
every one is drawn over road. The corners not on the drawn road (0.04 %) are boxes on
turn paths whose 1.8 m width overhangs a polygon's rounded corner.

**The proposal: keep the strips, bit for bit.** The road mesh, its order and its material
do not change, on or off. That keeps `--no-streets` trivially today's, and makes the
junction surfaces the only new road.

#### 2.17.4 Junction surfaces

- **What:** every polygon in `NetworkData::junction_polygons`, in the network's node
  order: 106 in Midtown, 9 in urban_grid. A closing vertex equal to the first is dropped.
  A polygon of fewer than 3 vertices is skipped (none here). A waypoint's polygon is drawn
  too: the engine makes it for drawing, and over strips that run through it, it is the
  same grey (no fixture has one).
- **How:** triangulated with `earcut` (already a dependency, §2.11), flat at height 0,
  opaque, with the road's material, `scene::ROAD`. The polygons are concave at their
  rounded corners, which `earcut` handles; the engine's polygons repeat some vertices
  (a corner arc's ends), which `earcut` drops: 3,478 triangles in Midtown (4,510 vertices
  as given, 4,404 after each closing copy is dropped) and 306 in urban_grid (396, 387).
- **The look:** a junction is the road's grey, joined to its strips with no seam, and the
  boxes crossing it stand on it. It carries no marking (no crosswalk, decision 2).

#### 2.17.5 The median gap: filled grey (OQ-18)

On a two-way pair, the engine offsets each link right by `total_width/2 + median_gap/2`,
so the two strips stand `median_gap` apart, and the gap shows the background: a dark seam
at 0.5 m, a dark band at 3.5–4 m. The proposal:
- **The gap is filled with the road's grey,** at height 0 in the junction surfaces' mesh:
  a ribbon along the pair's first link `A`, between laterals `−w_A/2 − g` and `−w_A/2`
  from `A`'s strip centre, where `g = (g_A + g_B)/2` is the pair's gap. Midtown has 26 such
  fills (the 3 pairs with a 0 m gap need none), urban_grid 24. The fill's far edge lies on
  the twin's left edge to within 7.7 mm in Midtown and 0 in urban_grid.
- **What is painted in it** depends on its width (§2.17.8): under 1 m, one double yellow
  line in its middle; 1 m or more, a *flush median*, a double yellow line inside each of
  its edges (MUTCD 2009 §3B.24: a flush median island between opposing traffic is marked
  by two sets of solid double yellow lines).

Whether a 3.5–4 m gap is a painted median or a raised one is not in the data. A raised one
has curbs, which come with the ground (decision 2), so this draft paints it. *(2026-10-05,
user)* OQ-18 is answered (a), as drafted (decision 5).

*(2026-10-09, changed by the user at gate 14)* Replaced when Phase 6 ships: no fill is
drawn. The gap shows the background, as with `--no-streets`, rounded at its junction ends
by the engine's median noses in the road's grey (§2.18.16).

#### 2.17.6 The markings: widths, patterns and colours

| Marking | Proposed | US practice (cited) |
|---|---|---|
| Lane line (white) | **0.15 m** wide, **broken: 3.0 m line, 9.0 m gap** | MUTCD 2009 §3A.06: a normal line is 4–6 in (0.10–0.15 m) wide; a broken line "should consist of 10-foot line segments and 30-foot gaps". NYSDOT standard sheet 685-01 (pavement marking details, sheet 1): "normal broken lane line" 10 ft / 30 ft, lines 4 in, or 6 in at 45 mph and over |
| Centre line (yellow) | **double solid**: two 0.15 m lines, **0.10 m** apart (0.40 m in all) | MUTCD §3A.06: a double line is "two parallel lines separated by a discernible space"; §3B.01: two normal solid yellow lines where passing is prohibited both ways. NYSDOT 685-01: "normal solid double line", space 3 in minimum |
| Stop line (white) | **0.60 m** deep, solid, across each approach lane | MUTCD §3B.16: stop lines "should be 12 to 24 inches wide"; NYC DOT's standard drawings mark "24" WHITE" stop lines (attachment A of NYC DOT's markings specifications) |

- **0.15 m, not 4 in.** NYC and NYSDOT paint 4 in (0.10 m) on city streets. 6 in is the top
  of MUTCD's normal line, and it gives a line half again as much coverage in a frame where
  it is under a pixel (§2.17.10). The probe measured both.
- **Dashes start at the link's start**, `s = 0`, the junction's edge on the departure side,
  so each block begins with a full 3 m line. They end where the stop lines begin
  (§2.17.7), or at the link's end on a link with none. A dash cut by that end is drawn
  short.
- **Where:** a lane line runs between lanes `k` and `k + 1` of one link, at lateral
  `lane_center_offsets()[k] + width_k/2` (no fixture has a `gap_after`; with one, the line
  sits in the middle of the gap). A one-lane link has none. No edge lines, no turn arrows,
  no lines through junctions.
- **Colours** (sRGB): white **(235, 235, 235)** and yellow **(230, 170, 20)**. The yellow
  is a darker amber than the 5–9 m/s box colour (247, 201, 72), and the white is not the
  buildings' lit grey. Both are unlit, like the road.
- Every marking is a ribbon sampled along its link at most 1 m apart, as the strips are
  (`scene::STRIP_STEP`), so it follows a curved link. A ribbon from `s0` to `s1` takes
  `n = max(1, ceil((s1 − s0) / STRIP_STEP))` equal steps, so `n + 1` samples, each placed
  with `Placement::place_lateral` and offset along its right normal to the ribbon's two
  laterals: 2 vertices a sample, 2 triangles a step, no vertex shared between ribbons.
  The median fills are ribbons by the same rule. Gate 6's mesh counts follow from it.

#### 2.17.7 Stop lines: at signals, one per lane, where the engine stops

- **Which approaches:** every link whose end node is a junction with `control: signal`.
  Midtown: 83 junctions, **188 approach links, 402 stop lines**. urban_grid: 9, **36 and
  71**. A link ending at an endpoint has none.
- **Not at the other junctions.** Midtown's 23 unsignalised junctions all have no `rule`:
  the engine resolves them by gap acceptance against every conflicting movement, and the
  data names no stop or yield sign. NYSDOT 685-01's note S3: "On an uncontrolled approach
  (no stop sign, yield sign, or traffic signal) omit stop line." An all-way stop or a
  priority junction's minor approaches would take stop lines, but no fixture has one, and
  none is drawn in Phase 5 (§2.17.13).
- **One per `(link, lane)`**, across that lane's width
  (`lane_center_offsets()[k] ± width_k/2`). Its downstream edge is at
  `s = L − lane_stop_line_offset(link, k)`, where `L` is `Placement::link_length` (the
  trimmed length), and it runs 0.60 m upstream from there. That is the engine's own stop
  position, so a queue stops with its front at the line. On every approach in both
  fixtures the base offset is 0 (each is trimmed), so the line touches the junction's
  edge. urban_grid's `L_J12_J11` lane 0 has a 5 m delta, so its line stands 5 m back from
  lane 1's.
- **Where the first vehicle stops: at the line's back edge.** The engine stops a
  vehicle's front `STOP_LINE_STANDOFF` = 0.6 m behind the stop line's junction-facing edge
  (`crates/core/src/systems/conflict.rs:STOP_LINE_STANDOFF`: `stop_pos = link_length −
  stop_offset − half_length − STOP_LINE_STANDOFF`, `stop_pos` being the centre, as FCD's
  `position` is). Its comment: the standoff "must exceed the stop line depth (0.4m) to
  produce a visible gap behind the line". Here the line is 0.60 m deep, so **the
  engine's code predicts a gap of 0 between a stopped first box's front and the line's
  back edge**: the box touches the line and no grey shows between them. The box is drawn
  centred on its placed point (`src/draw.rs:box_transform`), so its drawn front is that
  front.
  - **Measured** on both whole runs (§2.17.15, `streets-engine`): an FCD row with speed
    under 0.1 m/s on a link into a signal, ahead of every other row on its `(link,
    lane)` in that snapshot, with its front within 5 m of the line's back edge; an
    *episode* starts at such a row when the vehicle had none a second before. The gap is
    the back edge's `s` less the front's:

    | | Episodes (rows) | Median | Within 0.10 m | 1.35–1.55 m | Front at the junction-facing edge (−0.600 m) | Other |
    |---|---|---|---|---|---|---|
    | Midtown | 2,491 (48,899) | **+0.042 m** | 2,441 (98.0 %) | 27 | 10 | 13 |
    | urban_grid | 129 (2,555) | **+0.042 m** | 123 (95.3 %) | 6 | 0 | 0 |

    So a stopped first box stops about 4 cm short of the line's back edge, a gap under a
    pixel in every frame here (§2.17.10), and the box reads as standing at the line. The
    27 and 6 at about 1.4 m stop early, and Midtown's 10 stand over the line, their front
    at the junction's edge; both are the engine's, recorded, not changed.
  - **Kept at 0.60 m anyway** (decision 8). The data item will colour stop lines by
    signal state (§2.17.13), so the line must read on its own; and in Midtown's
    orthographic frame a 0.60 m line shows at about 77 % of its colour (`w_px` 0.37,
    §2.17.10), a 0.40 m one at about 32 % (`w_px` 0.25). A 0.40 m line would leave the
    engine's 0.2 m of grey behind it, which is under a pixel in every frame here anyway.
- **The lane lines of an approach end** at the upstream edge of its furthest-back stop
  line.
- **Kept per `(link, lane)`**, in the mesh's order, so a later phase can recolour each by
  its signal's state (vis-001 §2.7 item 5). That phase is not designed here, and no gate
  of this phase reads the order.

#### 2.17.8 The yellow line: double, between a two-way pair's strips

- **Twins:** link `B` is `A`'s twin when `B.from_node = A.to_node` and `B.to_node =
  A.from_node`. That is the engine's own test for a bidirectional pair, the one that offsets
  both strips (`LinkGeometryIndex::from_network_config`'s `link_pairs`). No node pair has
  two links in either fixture. If one ever did, `A`'s twin is the first such link in link
  order. Midtown: **29 pairs**; urban_grid: **24**. A one-way link has no twin and gets no
  yellow, as decision 3 says.
- **Drawn once per pair,** along `A`, the pair's link that comes first in link order, from
  `s = 0` to `A`'s trimmed length. The pair's two ends meet the same junction polygons.
- **Where:** in the gap, whose middle is at lateral `c = −w_A/2 − g/2` from `A`'s strip
  centre (`g` as §2.17.5), which is `A`'s raw centreline when both gaps are equal, as they
  are in every pair here.
  - gap under 1 m: one double yellow line, its two lines at `c ± (0.05 … 0.20)`;
  - gap 0 (3 pairs in Midtown): the same, over the seam between the two strips;
  - gap 1 m or more: a flush median (§2.17.5), a double yellow line inside each edge,
    its outer line flush with the edge: with the gap between laterals `g0 = −w_A/2 − g`
    and `g1 = −w_A/2`, the four lines are at `[g0, g0 + 0.15]`, `[g0 + 0.25, g0 + 0.40]`,
    `[g1 − 0.40, g1 − 0.25]` and `[g1 − 0.15, g1]`.
- **Double, not single.** Every pair here has at least two lanes in all, and US city
  streets mark a two-way centre line double solid (§2.17.6). Midtown has 15 flush medians
  (4 lines each) and 14 centred lines (2 each): **88 yellow lines**; urban_grid **48**.
- **It runs the whole length** of the pair, to each junction's edge. Where a stop line
  meets it (a gap of 0), the stop line is drawn over it (§2.17.9), as US practice ends the
  centre line at the stop line.

#### 2.17.9 Lifts and draw order: no depth tie between different colours

Phase 6's lesson (vis-001 §2.12.1): coplanar faces tie in depth, and Bevy's binned
opaque pass keeps no draw order between entities, so a tie may resolve differently on
each run. So no two faces of different colours share a height:

| What | Height | Material |
|---|---|---|
| Road strips (unchanged) | 0 | the road's, `scene::ROAD` |
| Junction surfaces, median fills | **0** | **the same material** |
| Yellow lines | **0.01 m** | one unlit white material; the colour is per vertex |
| White lines: lane lines and stop lines | **0.02 m** | the same |
| Every box's base | 0.05 m + rank × lift | `src/draw.rs:box_transform` |

- **The surfaces tie with the strips, harmlessly.** Both are unlit with the same
  material, so whichever wins a sample gives the same colour, bit for bit. They meet at
  the polygon's edge (§2.17.3), and the median fill only touches its strips' edges.
- **Markings over road, white over yellow.** Lane lines and yellow lines never overlap
  (one is between lanes, the other in the gap). A stop line meets a yellow line only at a
  0 m gap, and is 0.01 m above it.
- **Every marking is under every box.** `draw::box_transform` puts a box's base 0.05 m
  above the road (and higher by rank), so 0.02 m is below every box, orthographic or
  perspective; a marking under a box is inside it and never shows.
- **Depth precision.** Bevy's perspective projection is reverse-Z with an infinite far
  plane and a 32-bit float depth, so a depth step at distance `z` is about `z · 1.2·10⁻⁷`:
  0.5 mm at 4.2 km, the eye's farthest from the look-at point at the view's widest zoom
  over Midtown (`height_m` 3.46 km). A 0.01 m lift is 20 steps there, and more at a
  slant, where the lift's length along the ray is `lift / sin θ`. The orthographic depth is linear over at most 1 km (`draw::ortho_eye`):
  a 0.01 m lift is about 160 steps.
- **Measured** (§2.17.15). The orbit's frames 0 and 900 and the city flight's 900 and
  1799, rendered with the markings at 0 m (a tie with the road), at 0.01/0.02 m and at
  three times that, each in two processes:
  - **at 0 m the markings lose over half their coverage** to the road: the orbit's frame
    0 has 1,403 pixels at least half a marking, against 3,285 at 0.01/0.02 m. Here the
    tie went the same way in both processes, but Phase 6 showed it need not;
  - **0.01/0.02 m against 0.03/0.06 m:** 3,285 against 3,316 such pixels (0.9 %), and
    the summed coverage 2,293 against 2,299. The rest is the larger lift's parallax at
    the lines' edges. So 0.01 m already clears the depth test;
  - **every case gave the same bytes in both processes.**

#### 2.17.10 What can be seen of a 0.15 m line, and the fade (OQ-19)

**Metres per pixel** (orthographic: `scene::Camera::fit`'s `k`; perspective, at the look-at
point: `height_m / H`, since the vertical field of view at the look-at distance is
`height_m`):

| Frame | m/px | A 0.15 m line | The 0.60 m stop line | A 3 m dash |
|---|---|---|---|---|
| Midtown ortho, 1920×1080 | 1.601 | 0.09 px | 0.37 px | 1.9 px |
| Midtown, `view`'s launch fit, 1280×720 | 2.401 | 0.06 px | 0.25 px | 1.2 px |
| urban_grid ortho, 1920×1080 | 1.148 | 0.13 px | 0.52 px | 2.6 px |
| City flight, `height_m` 1800 → 500 → 300, 1080p | 1.67 → 0.46 → 0.28 | 0.09 → 0.32 → 0.54 px | 0.36 → 1.3 → 2.2 px | — |
| The orbit, `height_m` 250: 1080p / 720p | 0.231 / 0.347 | 0.65 / 0.43 px | 2.6 / 1.7 px | — |
| urban_grid's follow, `height_m` 60, 1080p | 0.056 | 2.7 px | 11 px | — |

**So a lane line is under a pixel in every Midtown frame but the close orbit, and in every
orthographic frame.** MSAA ×4 then decides: a line a tenth of a pixel wide covers one of
a pixel's four samples in some pixels and none in others. In the probe's Midtown ortho
frame with true colours, the lines came out as a scatter of separate white and amber dots
along each street: 5,747 changed pixels, 2,341 of them at least a quarter a marking and
1,581 at least half, where the paint's area is 1,221 pixels' worth. An amber dot a pixel
wide reads as a slow vehicle, since the 2–5 and 5–9 m/s boxes are orange and yellow. The
city flight's first seconds, at 1,800 m, look the same. urban_grid's streets run along
the pixel grid, so there a line falls on one row of samples and shows as a thin
continuous line, white or amber (4,052 pixels at least half a marking).

**The proposal: fade each marking by its width on screen.**
- For each marking vertex, `w_px = w / mpp`, where `w` is its line's width (0.15 m, or
  0.60 m for a stop line) and `mpp` the metres per pixel there: `k` in the orthographic
  render; in perspective, `|p − eye| · 2·tan(φ/2) / H`, the pixel's size across the ray
  at the vertex, with `eye` = `Pose::eye`, `φ` = `camera::FOV_DEG` and `H` the frame's
  height in pixels (`streets::mpp_at`). Straight down at the look-at point it is
  `height_m / H`.
- `α = smoothstep((w_px − 0.1) / (0.5 − 0.1))`, clamped to [0, 1]: 0 under a tenth of a
  pixel, 1 from half a pixel up.
- The vertex's colour is the road's grey blended toward the marking's colour by `α`, in
  linear light, per channel in `f32` as `(1 − α)·road + α·marking`, which is exactly the
  road at `α` 0 and exactly the marking at 1. The three linear colours are computed once.
  It stays opaque: no transparency, no sorted pass.
- The orthographic render computes it once. In perspective it follows the pose: the
  markings' mesh (positions and indices unchanged) gets new colours at every `set_pose`,
  as see-through's mesh gets new heights (§2.15.5), and `view` does it each frame while
  streets are shown.
- It is continuous in the pose, so a flight never pops a line on or off; it is plain
  `f64` arithmetic per vertex, so the same pose gives the same mesh.

What it gives, in the probe's frames (§2.17.15):
- **Midtown ortho:** the junction surfaces and median fills, and each stop line at 77 %
  (`w_px` 0.37). No lane line or yellow line shows (`w_px` 0.09, `α` 0). Pixels at
  least half a marking: 1,581 → 50; at least a quarter: 2,341 → 568.
- **urban_grid ortho:** stop lines whole (`w_px` 0.52), lines all but gone (`α` 0.02):
  4,052 → 145 pixels at least half a marking.
- **The city flight:** at 0:00 as the ortho. From 0:30 (`height_m` 500) to the end (300),
  dashes, stop lines and the yellow lines read near the middle of the frame and fade
  toward its top, where the ground is farther.
- **The orbit and closer:** every marking at its own colour, as without the fade.
- In `view`, `H` is the window's height in logical pixels, as its camera's `height_m =
  H·k` is.

The alternatives were OQ-19's: true colours everywhere, or a minimum width on screen.
*(2026-10-05, user)* OQ-19 is answered (a), the fade (decision 6).

#### 2.17.11 The meshes and their cost

Two new entities, both baked relative to the fit's centre like the roads:
- **the surface mesh:** the junction surfaces, then the median fills, with the road's
  material. Built once; never changes.
- **the markings mesh:** the yellow lines, then the lane lines, then the stop lines, with
  per-vertex colours on one unlit white material that does not cull. `NoFrustumCulling`,
  as the buildings' mesh has. Its colours are rebuilt as §2.17.10 says.

| | Midtown | urban_grid |
|---|---|---|
| Today's road mesh | 61,038 vertices | 27,168 vertices |
| Surface mesh | 10,510 vertices, 9,532 triangles | 13,971 vertices, 13,842 triangles |
| Markings mesh | 37,594 vertices, 32,198 triangles | 36,292 vertices, 33,844 triangles |
| Ribbons: lane-line dashes / stop lines / yellow lines / median fills | 2,208 / 402 / 88 / 26 | 1,105 / 71 / 48 / 24 |
| Building the streets, then both meshes' data | 1.37 ms, then 0.75 ms | 0.76 ms, then 0.50 ms |

- **The markings mesh is a sixth of the buildings'** (213,880 vertices, §2.16.11), which
  see-through rebuilds most frames for 3–4 ms. Measured in perspective with the fade,
  interleaved on one job (the rebuild on and off frame by frame, 600 pairs, buildings and
  see-through on, load 2.4–3.1): **+5.2 ms** a frame at the median on the city flight
  (23.96 against 18.71 ms; p99 27.15 and 21.58) and **+5.1 ms** on the orbit (23.83
  against 18.77). Of that, the colours take 0.17 ms and building and inserting the mesh
  0.24 ms; the rest is Bevy uploading it. About 9 s over an 1,800-frame flight. A probe
  that recomputed sRGB to linear per vertex paid 7–8 ms; the colours are blended from
  three linear colours computed once.
- **Two meshes, not one or many:** the surface needs the road's material, so that its tie
  with the strips is harmless (§2.17.9); the markings need vertex colours. One entity per
  marking would put thousands of entities through the binned pass, whose order is not
  stable, for nothing.
- **`view --bench 20`** on Midtown with buildings (see-through on), the probe's window:
  **60.00 fps** today, with streets and the fade (twice), and with streets and no fade;
  p99 18.50–18.77 ms, worst 19.22–21.83 ms, at a load average of 1.8–2.4. So the rebuild
  fits in vsync's frame.

#### 2.17.12 The flag, the key, and the scripts that read references

- **On by default** (decision 4). `render` and `view` draw streets on every network.
- **`--no-streets`**, on `render` and `view`, turns them off. With it, `render` builds its
  scene exactly as today: no streets are built and nothing is spawned, so every frame is
  today's, byte for byte.
- **`M`** (accepted by the user, decision 7) in `view` (by position, `KeyCode::KeyM`;
  unbound today) shows and hides them at
  any moment, as `B` does the buildings. `view --no-streets` starts with them hidden, and
  `M` shows them. It changes nothing else. The keyframe line does not carry it.
- **No new error.** `--no-streets` is accepted everywhere. A network can give an empty
  mesh (one-lane one-way links and no junction give no surface and no marking); a mesh
  with no triangle is not spawned, so such a network draws as with `--no-streets`.
  `--streets` is not a flag, so clap rejects it (exit 2), as it rejects `--see-through`.
- **The library keeps today's default.** `Job::prepare*` build every job with streets
  off, so every test that builds a job draws as today. `src/main.rs` turns them on with
  `Job::set_streets(true)` unless `--no-streets` is given, as it does see-through.
  `RenderOptions` and every `Job::prepare*` signature are unchanged.
- **A harness** that needs today's video passes `--no-streets`, as it can pass
  `--no-see-through` (§2.15.6). This is the first phase that changes urban_grid's default
  render.

**The scripts whose renders are compared with a `ref-pin90b39292` file**, directly, by hand
(gate 1's `camera`) or through a recorded line, give them `--no-streets`, so that those
comparisons still hold. These 29 CLI renders gain the flag and nothing else:
- `scripts/gates.sh`: `default`, `default2`, `camera`, `camera2` (4);
- `scripts/gates-ties.sh` gate 2: `ortho-city`, `ortho-roads` (2);
- `scripts/gates-see-through.sh` gate 2's seven and gate 7's five (12): gate 7's recorded
  line compares its renders with gate 2's;
- `scripts/gates-parts.sh` gate 2's five and gate 9's six (11): gate 9's recorded line
  compares its renders with the references.

The other renders in scripts compare only with each other: `gates.sh`'s `explicit`
(none), `gates-ties.sh` gate 9, `gates-city.sh` gate 13 and `gates-credit.sh` gate 14.
They are not edited, now draw streets, and must still give equal pairs (Phase 5 gate 3).
The error cases in every script stop before a frame, so streets do not reach them.
`gates-see-through.sh` gate 10's `view --no-see-through --bench 1` on urban_grid now opens
with streets; it checks only its exit and its JSON line, so it still passes.

#### 2.17.13 What it does not do

- **No crosswalks, curbs, sidewalks or edge lines** (decision 2), and no turn arrows,
  lines through junctions, or yellow left edge on a one-way street.
- **No stop lines at unsignalised junctions,** nor at all-way stops or a priority
  junction's minor approaches: no fixture has one (§2.17.7).
- **No colour by signal state.** The stop lines are kept per `(link, lane)` for that
  phase (vis-001 §2.7 item 5).
- **Pick, pan and zoom** are unchanged: they use the ground plane, and the markings are
  2 cm above it.

#### 2.17.14 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change. `earcut` triangulates the
polygons (a dependency since Phase 1), the engine's data is already in `Placement`, and
vertex colours are what the boxes' mesh uses (vis-001 §2.12.2). One new module,
`src/streets.rs`, with no Bevy types.

#### 2.17.15 Measured while drafting (2026-10-05)

A throwaway probe in gitignored `scratch/vis002p5-probe/`: a `git archive` of `origin/main`
(`2f4ec59`) in `repo/`, with a draft `src/streets.rs` and the drawing behind an
environment variable, built in its own `CARGO_TARGET_DIR`. Nothing in `src/`, `tests/` or
`scripts/` was touched. The record is `specs/reviews/vis-002.md`, "Phase 5 draft". It
holds:
- `streets-probe`: the counts of §2.17.2, the strips' ends, the boxes on the drawn road,
  the mesh sizes and build times, and the metres per pixel;
- `streets-frames`: frames of Midtown with streets off and on, the changed pixels
  classed as fill or marking, each marking pixel's coverage, with and without the fade
  and at 0.10 and 0.15 m;
- `streets-synth`: the synthetic crossing of gate 8, its pixels;
- the CLI renders (`run.sh`, `runs/`), through the probe's `assimilator-video`:
  - streets off: urban_grid's default render **8700 of 8700** against
    `ref-pin90b39292-default`; Midtown's ortho **1800 of 1800** against `ortho-roads`,
    and its city flight with buildings **1800 of 1800** against `city-on` (see-through
    on, as `render` defaults);
  - streets on, twice each, every pair equal: urban_grid default and `--camera
    tests/flight.toml` **8700 of 8700**; Midtown ortho with and without buildings, the
    city flight with and without, and the orbit with buildings, **1800 of 1800**. Each
    differs from its reference in every frame. Wall times: urban_grid 138–167 s, Midtown
    30–51 s, at load 2.3–4.0;
- `streets-cost`: the rebuild's cost (§2.17.11);
- `streets-engine` (2026-10-05, for decisions 8 and 9): the stopped first boxes' gaps
  (§2.17.7) and the engine's `NetworkJson` against the draft (§2.17.2);
- the lift test (§2.17.9), and `view --bench` (§2.17.11).

**Where the probe differs from the scope** (noted in review round 1, 2026-10-05):
- it spawned the streets inside `Renderer::build`, before the boxes, the buildings and the
  credit and before the first settle, and `streets-synth` built two renderers, one off and
  one on. The scope spawns them through `Renderer::set_streets` after the build and
  settles again, and gate 8 turns them on part-way on one renderer. No two faces of
  different colours tie in depth (§2.17.9), so the same pixels are expected, but gate 8's
  counts were measured through the probe's path;
- it blended colours as `road + (marking − road)·α`, not the scope's `(1 − α)·road +
  α·marking`. The two agree to an `f32` ulp; gate 8's lines are whole (`α` 1), where the
  scope's form is exact;
- its comparison with `NetworkJson` dropped one trailing copy of the first vertex, not two,
  and compared the first 43 vertices; that is the comparison gate 6 now states;
- it draws stop lines at all-way stops too, and takes the last twin of a node pair, not
  the first. No fixture has either; the scope follows §2.17.7 and §2.17.8.

### 2.18 Engine look and lane arrows (Phase 6)

Drafted 2026-10-08, the roadmap's "Engine look + lane arrows" item (§2.13). Phase 5 draws
the streets with a US marking set of its own (§2.17.6–§2.17.8). The engine's dashboard
draws the same streets differently: its own dashes, centre line, stop lines and lane
arrows. This phase draws the markings as the dashboard does, from the engine's own
`NetworkJson`, with the dashboard's colours and arrow glyphs copied from its front end.
It changes no box, building, camera, credit line, road strip, junction surface or median
fill. The numbers below come from a probe run while drafting (§2.18.15).

#### 2.18.1 The user's decisions (decision, recorded)

Decided by the user, 2026-10-08, before drafting:
1. **vis-002 Phase 6,** next, before vis-001's data item (vis-001 §2.7 item 5).
2. **Draw the road markings as the engine's own UI (its dashboard) does,** in place of
   Phase 5's US set: its centre (median) line, its lane dashes, its stop lines (their
   width, and the approaches it marks), its solid lane lines and its lane arrows (turn
   arrows before the stop line, direction arrows along links). In the user's words: "we
   can use what the simulator engine UI shows already instead of creating new things,
   which would include the road arrows". The reasons:
   - one look in both tools;
   - the engine's stop position was made for its 0.4 m line: the first stopped car shows
     a 0.2 m gap behind it, where Phase 5's 0.60 m line shows none (§2.17.7).
3. **Kept from Phase 5:** the junction surfaces, the median fills, the fade, streets on by
   default, `--no-streets` (today's frames, byte-identical) and `M` (no new key).
   *(2026-10-09, changed by the user at gate 14)* The median fills are no longer kept
   (decision 5 as changed). The junction surfaces, the fade, the default, `--no-streets`
   and `M` are.

**By the methodology's §6.1.** Step 0: a decision changes (Phase 5's decision 3, the
look). Step 1 matches in part: the change contradicts part of a shipped phase, Phase 5's
US markings (§2.17.6, §2.17.8, the stop lines' depth in §2.17.7). That phase is not cut,
since its surfaces, fills, fade, meshes, lifts, flag and key stay in use and are what this
phase draws on, so it gets no `cut` and no `## 0.` note. The contradicted statements get
dated notes in place instead (step 1's third bullet): §2.17.1's decisions 3, 5, 8 and 9,
and OQ-18. Step 2: vis-002 owns the scene around the traffic and its roadmap names the
item, so the new work is a phase appended here (decision 1). vis-001 is not edited.

Decided by the user, 2026-10-08, on the draft, before review (the answer to OQ-20, and a
call on the draft):
4. **The colours are the dark palette at the dashboard's opacities, composited over our
   road** (OQ-20's (a), §2.18.4).
5. **A median gap stays as drafted** (§2.18.3). The user saw the two other ways, the gap
   as the dashboard shows it (dark, with rounded noses) and the grey fill with one engine
   yellow line down it, and keeps the draft: the grey fill stays (decision 3), a pair
   with a gap shows no line between its directions, and the centre line is drawn only
   where the gap is 0, as the engine draws it.
   *(2026-10-09, changed by the user at gate 14)* **Changed.** On the grey fill with no
   line: "there is no gap.. it looks like more road". A pair with a gap now shows it as
   the engine's dashboard does:
   - it is not road: it shows our background, as with `--no-streets`;
   - its junction ends are rounded by `NetworkJson::median_noses`, in the road's grey;
   - no median fill is drawn;
   - the centre line stays only on 0 m pairs.

   The reason: the gap reads as a gap, as on the dashboard. The design is §2.18.16.

*(2026-10-09, changed by the user at gate 14)* By §6.1 again: step 0, decision 5 changes.
Phase 6 has not shipped, so it is amended in place, on its branch, before it ships; no
phase is added. Phase 5's fill gets dated notes (§2.17.1 decision 5, §2.17.5, OQ-18). The
first build's gate record stands as written; the amended build's gate run gets its own.

The rest of this section is the draft's proposal, with OQ-20 now decided. It settles:
- drawing `NetworkJson`'s shapes directly, not rebuilding them, and its cost and order
  (§2.18.2);
- what is drawn and what is not (§2.18.3);
- the colours (§2.18.4, OQ-20, answered (a));
- the arrow glyphs (§2.18.5);
- the stop lines, kept per `(link, lane)` (§2.18.6), and where the first vehicle now stops
  (§2.18.7);
- lifts and order (§2.18.8), and the fade for each kind (§2.18.9);
- the meshes and their cost (§2.18.10);
- what becomes of Phase 5's code, tests and gates (§2.18.11);
- what a pin move does (§2.18.12).

#### 2.18.2 Drawn from `NetworkJson` directly, not rebuilt

`crates/geometry/src/network_json.rs:NetworkJson::from_config_with_network_data(config,
&NetworkData)` is public, in `assimilator-geometry`, already a dependency. Phase 5's gate 6
calls it in `tests/streets.rs`. It returns the dashboard's whole street geometry as 2D
rings in metres. Phase 6 calls it once when the streets are built, with
`(&placement.network, &placement.data)`, and draws five of its fields:
`median_lines`, `lane_marking_dashes`, `solid_lane_lines`, `stop_lines` and `lane_arrows`
(§2.18.3).

**The proposal: draw its rings and arrows directly.** Each ring, with its closing copy of
the first vertex stripped, is triangulated with `earcut` and placed at its kind's lift
(§2.18.8). Each arrow is a glyph copied from the front end (§2.18.5), placed at its `(x, y,
heading)`. Nothing of the engine's placement logic is rebuilt here.

**The alternative: keep a builder of our own** (Phase 5's ribbons) with the engine's
numbers, plus a gate comparing it with `NetworkJson`. That copies the engine's rules into
this repo, where they would drift from the engine's own code:
- the dash pattern, centred within each boundary's usable length
  (`generate_lane_dashes`), and its exclusions at junctions and crossings;
- which approaches get a stop line: every approach at a signal, the minor ones at a
  priority junction, and at an uncontrolled junction only those with a lane path that is
  not transparent (`NetworkData::is_transparent_lane_path`);
- full-width against per-lane stop lines, and the connectors between staggered ones;
- each lane's arrow type from its movements, the separate u-turn arrow, dead_end, and
  the direction arrows near a link's start.

That is about 400 lines of the engine's code. At a pin move, the builder would keep its
old rules until the comparison gate failed. **Drawn directly, the markings follow the
engine at a pin move by themselves**, and the gates show what changed (§2.18.12). The
data item also gains: the engine's own stop line is what a signal colour will paint.

**Its cost and order**, measured (§2.18.15):

| | Midtown | urban_grid |
|---|---|---|
| `NetworkJson::from_config_with_network_data`, median of 9 in one process (load 70–130) | **4.9–6.1 ms** (fastest 3.6) | **0.6–0.7 ms** |
| Phase 5's `Streets::build`, the same runs | 3.6–4.0 ms | 1.3–1.4 ms |
| Two processes: the drawn fields as JSON; the whole `NetworkJson`; the triangulated markings mesh | **the same bytes** each (1,247,553; 3,198,014; 1,014,096) | **the same bytes** each (255,265; 602,516; 368,952) |

- **It is built once,** when a render or `view` builds its streets, so its 5 ms is paid
  once per run. It also builds turn paths, conflict points and the rest, which are not
  drawn; that is included in the 5 ms.
- **Its drawn fields have a fixed order.** Each walks `config.links` (`median_lines`,
  `lane_marking_dashes`, `solid_lane_lines`, `stop_lines`, `lane_arrows`) or
  `config.nodes` (`junction_fills`) in file order, lane by lane. The `HashMap`s and
  `HashSet`s in it (`link_pairs`, `bidi_opposite`, `junction_excl`,
  `stop_line_approaches`, `lane_needs_stop`, `lane_movement_types`, the `seen_*` sets)
  are only looked up, never walked, for those fields. The fields walked from a map
  (`conflict_pairs` and the like, from `NetworkData::conflict_pairs_map`) are debug items,
  not drawn (§2.18.3). Phase 5's lesson holds: no order here comes from walking a
  `HashMap`.

**Where it sits on our road.** Our strips sample `Placement::place_lateral` along the
trimmed polyline (§2.17.3); `NetworkJson` offsets the same trimmed polyline with
`offset_polyline`, which mitres its corners. Measured, every marking vertex against our
drawn road (the strips, the junction surfaces and the median fills):
- every dash and every arrow vertex lies on it (15,684 and 15,020 in Midtown; 8,176 and
  3,482 in urban_grid);
- the stop lines' vertices lie on it or within **6.3 mm** of it (Midtown: 316 of 1,884
  just outside, at the ends of links that bend near a junction; urban_grid: all on it);
- the centre lines' 1,074 vertices lie on it (2 exactly on its edge).

So a marking never shows over the background, and nothing sits off the road by more than a
centimetre.

*(2026-10-09, changed by the user at gate 14)* The fills are gone, so the drawn road is
now the strips, the junction surfaces and the median noses. Measured again against it, the
figures stand:
- dashes and arrows: all on it;
- centre lines: 2 on its edge;
- stop lines: within 6.3 mm, with 325 of Midtown's 1,884 vertices just off. Nine more than
  316, since those nine stood on a fill's edge (§2.18.16).

#### 2.18.3 What is drawn, and what is not

`drawStaticNetwork` (`web/src/canvas/networkRenderer.ts`, l. 573 at `90b39292`) draws, in
this order: road polygons, median noses and junction fills in one fill (l. 584); lane
dashes and solid lane lines (l. 587–588); median lines (l. 591); the generic road arrows
and the lane arrows (l. 594–595); stop lines (l. 598); crossing stripes and detector
stripes (l. 602, 605); nodes. Phase 6:

| `NetworkJson` field | Midtown | urban_grid | Drawn? |
|---|---|---|---|
| `median_lines`: one 0.15 m line along a pair with a 0 m gap, at its seam | 3 | 0 | **yes** |
| `lane_marking_dashes`: 2.5 m long, 0.15 m wide, every 6.5 m, centred in each boundary | 3,921 | 2,044 | **yes** |
| `solid_lane_lines`: a lane boundary with `no_change_left`/`right` | 0 | 0 | **yes** (no lane in either fixture has the flags) |
| `stop_lines`: full-width / per lane / connector | 223 / 4 / 0 | 35 / 2 / 1 | **yes**, cut per lane (§2.18.6) |
| `lane_arrows` | 1,050 | 190 | **yes** (§2.18.5) |
| `junction_fills` | 106 | 9 | no: Phase 5's junction surfaces are the same polygons (§2.17.2, 0.000000 m), kept |
| `road_polygons` | 255 | 48 | no: our strips, kept bit for bit (§2.17.3) |
| `median_noses`: two crescents at each end of a gap, in the road's fill | 94 | 72 | no: every vertex lies inside our median fills, junction surfaces and strips, so in the road's grey they would change no pixel |
| `curb_arcs` | 410 | 36 | no: no curbs (§2.17.1, decision 2) |
| `crossing_stripes`, `detector_stripes`, `stops` | 0 | 0 | no: none in either fixture; crossings, detectors and PT stops are not this phase's |
| debug items: `turn_paths`, `lane_turn_paths`, `conflict_points`, `conflict_pairs`, `lane_conflict_pairs`, `nodes`, `detectors` | 520, 770, 435, 910, 951, 148, 0 | 108, 143, 257, 470, 501, 21, 0 | no: the dashboard shows them only on selection or as editor aids |

*(2026-10-09, changed by the user at gate 14)* `median_noses` **is drawn now**: 94 in
Midtown, 72 in urban_grid, in the road's grey in the surface mesh. The median fills are
not drawn, so the noses are the only road in a gap (§2.18.16). The table's "no" for it is
replaced.

**The generic road arrows are not drawn.** The front end also makes straight arrows of
its own (`generateRoadArrows`, l. 236) on every link that has no lane arrow. `NetworkJson`
gives every link at least 8 m long a lane arrow per lane, so that is only a link under
8 m. Neither fixture has one, so they would add nothing. Drawing them would copy a third
rule from TypeScript (their placement, not only their shape), so they are left out until
a network needs them (§2.18.13).

**What the centre line means now.** The engine draws one only on a pair with a 0 m gap
(3 of Midtown's 29 pairs, none of urban_grid's 24). On a pair with a gap, the dashboard
shows the gap itself, dark, with rounded noses. Here the gap stays filled with the road's
grey (decision 3, Phase 5's decision 5), so **a pair with a gap shows no line between its
two directions**. That is 26 of Midtown's pairs and all 24 of urban_grid's. Phase 5 drew a
double yellow line on each. OQ-18's yellow part is replaced (its dated note), and gate 14
shows it. *(2026-10-08, user)* Kept as drafted, after seeing the dashboard's dark gap with
rounded noses and the grey fill with one engine yellow line down it (§2.18.1, decision 5):
the fill stays, a pair with a gap shows no line, and the centre line is drawn only where
the gap is 0.

**What the centre line means now, as amended.** *(2026-10-09, changed by the user at gate
14)*
- The engine draws one only on a pair with a 0 m gap: 3 of Midtown's 29 pairs, none of
  urban_grid's 24.
- **A pair with a gap shows the gap itself**, as the dashboard does: our background
  between its two directions, rounded at each junction end by two grey noses, with no
  fill and no line. That is 26 of Midtown's pairs (11 at 0.5 m, 4 at 3.5 m, 11 at 4 m) and
  all 24 of urban_grid's (0.5 m).
- Phase 5's double yellow lines and its grey fill are both gone. The two directions of a
  two-way street are parted by a dark gap, or, on a 0 m pair, by one amber line.

**The movements behind the arrows:**

| | Midtown | urban_grid |
|---|---|---|
| Movements: through / right / left / u-turn | 520: 219 / 127 / 123 / 51 | 108: 36 / 36 / 36 / 0 |
| Arrows: arriving (one per lane of a link ≥ 8 m, 4 m before its stop line) / separate u-turn / departing (a lane of a link ≥ 25 m, 4 m after its start) | 533 / 0 / 517 | 95 / 0 / 95 |
| of the arriving, on a link that ends at an endpoint (always `through`) | 44 | 24 |
| By type: `through` / `through_left` / `through_right` / `left` / `right` / `left_right` / `through_left_right` / `u_turn` / `dead_end` | 788 / 111 / 119 / 13 / 12 / 2 / 5 / **0** / **0** | 121 / 29 / 30 / 2 / 2 / 1 / 5 / **0** / **0** |

- **No u-turn arrow in either fixture.** Midtown's 51 u-turn movements all have
  `from_lanes: []`, so no lane carries one; urban_grid has none.
- **No `dead_end` in either fixture:** every lane of every link into a junction has a
  movement. The glyph is drawn as the dashboard draws it, a white shaft with a red bar,
  and the synthetic test covers it (gates 5 and 8). Its red, composited (217, 72, 73), is
  near the slowest boxes' (230, 57, 70). It is 1.0 × 0.4 m, under a box's 1.8 m width,
  and no fixture has one.

#### 2.18.4 The colours: the dashboard's dark palette, at its opacities, over our road (OQ-20)

The dashboard's colours exist only in its front end, so they are copied
(`web/src/canvas/colors.ts` at `90b39292`). It has two palettes, `dark` (the default:
`web/src/utils/theme.ts:getTheme` returns `'dark'` unless set) and `light`. It paints
every marking with a canvas opacity over its road, which is `#555555` at 0.95 over
`#040810`, (81, 81, 82).

| Kind | Dark (l.) | Light (l.) | Opacity (l.) | Dark over **our** road (92, 96, 104) | Dark over the dashboard's road |
|---|---|---|---|---|---|
| Lane dashes, solid lines | `LANE_MARKING` `#ffffff` (75) | `#ffffff` (137) | `LANE_MARKING_OPACITY` 0.5 (198) | **(174, 176, 180)** | (168, 168, 169) |
| Centre line | `MEDIAN_LINE` `#f5c542` (76) | `#d97706` (138) | `MEDIAN_LINE_OPACITY` 0.7 (199) | **(199, 167, 77)** | (196, 162, 71) |
| Stop lines, connectors | `STOP_LINE` `#ffffff` (77) | `#ffffff` (139) | `STOP_LINE_OPACITY` 0.9 (200) | **(239, 239, 240)** | (238, 238, 238) |
| Arrows | `ROAD_ARROW` `#ffffff` (78) | `#ffffff` (140) | `ROAD_ARROW_OPACITY` 0.6 (201) | **(190, 191, 195)** | (185, 185, 186) |
| Dead-end bar | `DEAD_END_BAR` `#ef4444` (79) | `#dc2626` (141) | `DEAD_END_BAR_OPACITY` 0.85 (202) | **(217, 72, 73)** | (215, 70, 70) |

**The proposal: the dark palette, each colour composited at its opacity over our road
grey,** in sRGB as the canvas composites, rounded once: `c = round(a·marking + (1 −
a)·road)` per channel. The result is opaque, like Phase 5's markings, so there is no
transparency and no sorted pass, and the markings keep the dashboard's contrast with the
road under them:
- **dark, not light:** it is the dashboard's default, and its amber centre line is the
  one meant for a dark scene like ours. The two palettes differ only in the centre line
  and the dead-end bar; light's centre line, (180, 112, 35) over our road, is an orange
  near the 2–5 m/s boxes' (244, 132, 45);
- **composited over our road, not the dashboard's colours as it shows them:** our road
  is lighter than the dashboard's by 11–22 levels. Taking its on-screen colours as they
  are would put lane dashes 6–11 levels darker against our road than the dashboard shows
  them against its own;
- **not the raw colours:** white `#ffffff` lane dashes at full strength are what Phase 5
  drew (235, 235, 235) and brighter. The dashboard shows them as a half-grey dash, and
  that is the look decided (decision 2).

The road keeps its grey (§2.17.3: the strips do not change, and decision 3 keeps the
junction surfaces and fills in it). The alternatives are OQ-20's. *(2026-10-08, user)*
OQ-20 is answered (a), as drafted (§2.18.1, decision 4).

#### 2.18.5 The arrow glyphs

The glyphs exist only in `networkRenderer.ts`, so they are copied, value for value, from
`90b39292`: `drawLaneArrow` (l. 1324) dispatches on `arrow_type` to `drawStraightArrow`
(l. 1371), `drawCurvedTurnArrow` (l. 1429), `drawThroughTurn` (l. 1446),
`drawThroughLeftRight` (l. 1490), `drawLeftRight` (l. 1536), `drawUTurnArrow`
(l. 1603) and `drawDeadEndArrow` (l. 1393), with `fillCurvedArrow` (l. 1296),
`buildCurveEdges` (l. 1258) and `sampleQuadBezier` (l. 1274). Their constants (l.
1249–1254): shaft half-width `LA_SW` 0.15 m, head half-width `LA_HW` 0.40, head length
`LA_HL` 0.70, straight length `LA_LEN` 3.0, turn shaft `LA_TURN_SHAFT` 1.50, turn radius
`LA_TURN_R` 0.50; the u-turn's radius 0.30 (l. 1608) and its return leg to `−0.4 ×` the
shaft (l. 1624); the branches' fork 1.00 m after the tail (l. 1455, 1499); the dead-end
bar 1.00 m wide and 0.40 m deep (l. 1396–1397).
- **Size: in metres, as the dashboard draws them.** `drawLaneArrows` (l. 1631) scales
  every glyph by the map's pixels per metre, so a straight arrow is 3.0 m long, its shaft
  0.30 m and its head 0.80 m wide, at any zoom.
- **Placed:** local `+x` is forward and local `+y` right of travel, since the canvas
  rotates by `−heading` on a y-down screen. A glyph point `(u, v)` goes to
  `(x, y) + u·(cos h, sin h) + v·(sin h, −cos h)`, with `h` the arrow's `heading`, which
  `NetworkJson` gives as `atan2(dy, dx)` in degrees.
- **Curves are already polylines.** The canvas paths have no curve commands: each turn
  is `sampleQuadBezier(…, 12)`, 13 points, and the u-turn's arc 12 segments, joined with
  `lineTo`. So each glyph's outline is a polygon, copied vertex for vertex, and
  triangulated with `earcut` (`n − 2` triangles each).
- **Several outlines a glyph.** The canvas fills a glyph's subpaths together, non-zero.
  Each subpath is a simple polygon and is kept as one, overlapping its siblings in the
  same colour: `through_left`, `through_right` (a straight arrow and a branch) and
  `through_left_right` (a straight arrow and two branches). `drawLeftRight`'s one path
  crosses itself, tracing the shared shaft twice; it is drawn as its two halves (left
  turn, right turn), whose union is the same filled set.
- **The types `NetworkJson` emits at the pin** are these nine. The front end also draws
  `u_turn_left` (`drawUTurnLeft`) and `through_left_u_turn` (as `drawThroughTurn`'s left
  branch), which the pin never emits, so they are not copied. Here any type not in the
  table draws as a straight arrow, as `drawLaneArrow`'s `else` does for an unknown type;
  a pin move that starts emitting either is caught by §2.18.12's read of `arrow_type`.

| Glyph | Outlines | Vertices | Triangles | Area (m²) | Extent along × across (m) |
|---|---|---|---|---|---|
| `through` | 1 | 7 | 5 | 0.9700 | −1.50…1.50 × ±0.40 |
| `left`, `right` | 1 | 31 | 29 | 0.9728 | −1.50…0.90 × 0.15…−1.20 (mirrored) |
| `through_left`, `through_right` | 2 | 36 | 32 | 1.4928 (the overlap counted twice) | −1.50…1.50 × 0.40…−1.20 |
| `left_right` | 2 | 62 | 58 | 1.9456 | −1.50…0.90 × ±1.20 |
| `through_left_right` | 3 | 65 | 59 | 2.0156 | −1.50…1.50 × ±1.20 |
| `u_turn` | 1 | 33 | 31 | 1.1895 | −1.50…0.45 × 0.15…−1.00 |
| `dead_end` | 2 (white shaft, red bar) | 8 | 4 | 1.1800 | −1.50…1.50 × ±0.50 |

Each outline's triangles' area equals its shoelace area. **No glyph overlaps another
glyph or any other marking** in either fixture, and its widest extent, 1.20 m from the
lane's centre, stays inside a 3 m lane.

#### 2.18.6 Stop lines: where `NetworkJson` puts them, cut per lane

- **Which approaches:** the engine's rule (§2.18.2), so more than Phase 5's.

  | | Midtown | urban_grid |
  |---|---|---|
  | Approach links with a stop line: at a signal / elsewhere | **226**: 188 / **38** (at 19 of the 23 unsignalised junctions) | **36**: 36 / 0 |
  | Rings: full-width / per lane / connector | 223 / 4 / 0 | 35 / 2 / 1 |
  | `(link, lane)` stop lines after the cut: at a signal / elsewhere | **471**: 402 / 69 | **71**: 71 / 0 |
  | Phase 5 | 188 links, 402 | 36 links, 71 |

  At signals the approaches and `(link, lane)`s are Phase 5's, exactly. The 38 new
  approaches are those at an unsignalised junction with a lane path that crosses
  another (the engine marks them; Phase 5 did not, §2.17.7).
- **Depth 0.4 m** (`half_depth = 0.20`, `network_json.rs` l. 1643), its junction-facing
  edge where Phase 5's was: at `s = L − lane_stop_line_offset(link, lane)` (Phase 5's
  gate 6: 0.000000 m). So it stands 0.2 m nearer the junction at its back edge than
  Phase 5's.
- **Kept per `(link, lane)`, cut where the ring is full-width.** A full-width ring
  (`link_id` and `lane` both `None`) covers every lane of its approach. That is what the
  engine draws when every lane is marked at one offset with no gap between lanes. One
  signal colour per approach cannot show a protected left turn red while the through
  lanes are green, so for the data item (vis-001 §2.7 item 5) each full-width ring is cut
  at its link's lane boundaries into one quad per lane:
  - **its link** is the one whose trimmed end at lateral 0 (`Placement::place_lateral` at
    `link_length − stop_line_offset`) lies at the midpoint of the ring's junction-facing
    edge (`coords[1]`–`coords[2]`). Measured: every one of 223 and 35 within 0.000000 m,
    and none ambiguous. A ring matched to no link within 1 mm is drawn whole, with no
    link and no lane, never given a guessed one; gate 6 asserts there is none;
  - **the cut points** lie on the ring's own two long edges, at lateral `offs[k] ±
    w_k/2` from the link's `lane_center_offsets()`, as a fraction `(l + W/2)/W` of the edge
    from its left corner to its right. The ring's corners, as `network_json.rs` builds them
    (l. 1694–1717, with `p = (dir.y, −dir.x)` right of travel): `coords[0]` back right,
    `coords[1]` front right, `coords[2]` front left, `coords[3]` back left. So the
    junction-facing edge runs `coords[2]` → `coords[1]` and the back edge `coords[3]` →
    `coords[0]`, left to right, and lane 0, the leftmost (`lane_center_offsets()` is
    right-positive), is cut nearest `coords[2]`/`coords[3]`. The quads' union is the ring
    exactly, and each quad's corners are on the ring's edges;
  - **per-lane rings** (`lane: Some`) carry their link and lane already; **connectors**
    (`link_id` set, `lane: None`: a 0.10 m strip along a lane boundary between two
    staggered lines) stay whole, kept with their link.

  The data item will then colour quad by quad, as it would have Phase 5's; no gate of
  this phase reads the order.

#### 2.18.7 Where the first vehicle stops: the engine's 0.2 m

The engine stops a vehicle's front `STOP_LINE_STANDOFF` = 0.6 m behind the stop line's
junction-facing edge (`crates/core/src/systems/conflict.rs:STOP_LINE_STANDOFF`, §2.17.7).
Its comment: the standoff "must exceed the stop line depth (0.4m) to produce a visible gap
behind the line". With the 0.4 m line, **the engine's code predicts a 0.2 m gap** between
a stopped first box's front and the line's back edge.

Measured by Phase 5's rule (§2.17.7, gate 7 there), with the back edge 0.4 m from the
junction-facing edge, on both whole runs:

| | Episodes (rows) | Median | Within 0.10 m of +0.242 | 1.55–1.75 m | Front at the junction-facing edge (−0.4 m bin) | Other |
|---|---|---|---|---|---|---|
| Midtown, at signals | 2,490 (48,896) | **+0.242 m** | 2,441 | 27 | 10 | 12 |
| urban_grid, at signals | 129 (2,555) | **+0.242 m** | 123 | 6 | 0 | 0 |
| Midtown, the new unsignalised stop lines | 214 (445) | **+0.233 m** | 132 | 2 | 67 | 13 |

- **At signals it is Phase 5's measure moved by 0.2 m,** episode for episode (+0.042 m
  then), so a stopped first box stands about 24 cm short of the line: 0.52 px in the
  orbit's frame (0.231 m/px), 0.86 px at the city flight's lowest (0.28 m/px), and
  4.3 px in urban_grid's follow at `height_m` 60. One episode fewer and 3 rows fewer at Midtown's signals: the
  one whose gap was within 0.2 m of the rule's 5 m reach (Phase 5's histogram's +5.0 m
  bin) falls outside it now.
- **At the new unsignalised stop lines,** most first boxes stop as at signals, and 67 of
  214 stand with their front at the junction's edge, over the line. That is the engine's
  gap acceptance, recorded, not changed.

#### 2.18.8 Lifts and order: no depth tie between different colours

Two heights, as Phase 5 had (§2.17.9):

| What | Height | Order in the mesh |
|---|---|---|
| Road strips, junction surfaces, median fills (unchanged) | 0 | — |
| Centre lines, lane dashes, solid lane lines | **0.01 m** | first, in that order |
| Stop lines, connectors, arrows, dead-end bars | **0.02 m** | then, in that order |
| Every box's base | 0.05 m + rank × lift | `src/draw.rs:box_transform` |

- **What overlaps, measured:** lane dashes running under a stop line (the engine ends
  dashes at the junction-facing edge, under the line): **21** in Midtown, **22** in
  urban_grid; a centre line under a stop line: **3** (Midtown). Both have the stop line
  0.01 m above them, as the dashboard paints stop lines last (l. 598). Arrows overlap
  nothing. A dead-end bar touches its shaft's end and does not overlap it.
- **Overlaps at one lift tie, deterministically, inside one mesh.** A glyph's outlines
  overlap each other, in one colour (up to the fade's spread across 3 m, §2.18.9). A
  connector overlaps its two stop lines' corners: the same colour at full strength, but
  under the fade they differ (widths 0.10 and 0.40 m), and the overlap shows the one the
  depth test keeps by mesh order (one connector in urban_grid, sub-pixel where the fade
  parts them). All are drawn by the one markings entity in index order, and a
  GPU resolves equal depths in primitive order within one draw, so the result is the same
  every frame. vis-001 Phase 6's lesson (vis-001 §2.12.1, as §2.17.9 cites it) is about
  ties between entities in Bevy's binned pass, whose order is not kept, and there are
  none here.
- The lifts' depth precision is §2.17.9's, unchanged.
- *(2026-10-09, changed by the user at gate 14)* **The median noses are at height 0 in the
  surface mesh, after the junction surfaces, in the road's own material,** in place of the
  median fills.
  - A nose overlaps the strips and junction surfaces it meets: half of its vertices lie
    on them.
  - Where they tie in depth, all three are the same material at the same height with the
    same normal. So whichever the depth test keeps, the pixel is the same colour: no
    visible tie, as with Phase 5's fills (§2.17.9).
  - No marking lies over a gap, and the markings stay 0.01 and 0.02 m above (§2.18.16).

#### 2.18.9 The fade for each kind

Phase 5's fade (§2.17.10) is kept as it is: per vertex,
`α = smoothstep((w/mpp − 0.1)/(0.5 − 0.1))`, blending from the road's grey to the
marking's colour in linear light. What changes is each kind's `w`, its narrow dimension:

| Kind | `w` | Midtown ortho (1.601 m/px) | `view` launch (2.401) | urban_grid ortho (1.148) | City flight 1.67 → 0.46 → 0.28 | Orbit 1080p (0.231) |
|---|---|---|---|---|---|---|
| Dash, centre line, solid line | 0.15 m | 0.09 px, **α 0** | 0, **0** | 0.13 px, **0.017** | 0 → 0.60 → 1 | 1 |
| Stop line (Phase 5: 0.60 m, α 0.767) | **0.40 m** | 0.25 px, **0.316** | **0.074** | 0.35 px, **0.678** | 0.28 → 1 → 1 | 1 |
| Arrow: the shaft | **0.30 m** | 0.19 px, **0.122** | **0.011** | 0.26 px, **0.357** | 0.10 → 1 → 1 | 1 |
| Dead-end bar | 0.40 m | as the stop line | | | | |
| Connector | 0.10 m | 0 | 0 | 0 | 0 → 0.21 → 0.71 | 0.93 |

- **An arrow fades by its shaft's width,** 0.30 m, not its head's 0.80 m. By the head,
  every arrow would be at full colour in Midtown's overview: 1,050 two-pixel specks along
  the streets, the dots OQ-19 answered against. By the shaft, an arrow shows at 12 % there
  and whole from the city flight's 0:30 (`height_m` 500) down.
- **The stop line is fainter in the overview than Phase 5's** (0.316 against 0.767 in
  Midtown's orthographic frame; 0.074 against 0.316 at `view`'s launch), since it is
  thinner. That is decision 2's line. Close up it is whole either way.
- **Per vertex, for every kind, on every vertex.** A centre line is a long ring with few
  vertices: Midtown's three are 4, 4 and 178 vertices along 73, 131 and 296 m. So each
  centre line's and solid line's edges are split at most 1 m apart (`scene::STRIP_STEP`),
  as Phase 5's ribbons were sampled, and the fade follows the distance along it: 186 → 1,074
  vertices in Midtown. Dashes (2.5 m), stop lines and glyphs (3 m) are short, and keep
  `NetworkJson`'s vertices.

#### 2.18.10 The meshes and their cost

The surface mesh is Phase 5's, unchanged (junction surfaces, then median fills: 10,510 and
13,971 vertices). The markings mesh is replaced:

| | Midtown | urban_grid |
|---|---|---|
| Centre lines (pieces; vertices; triangles) | 3; 1,074; 1,068 | 0 |
| Lane dashes | 3,921; 15,684; 7,842 | 2,044; 8,176; 4,088 |
| Solid lane lines | 0 | 0 |
| Stop lines, per lane | 471; 1,884; 942 | 71; 284; 142 |
| Connectors | 0 | 1; 4; 2 |
| Arrows (glyphs → outlines; vertices; triangles) | 1,050 → 1,292; 15,020; 12,436 | 190 → 260; 3,482; 2,962 |
| Dead-end bars | 0 | 0 |
| **Markings mesh** | **33,662 vertices, 22,288 triangles** | **11,946 vertices, 7,194 triangles** |
| Phase 5's markings mesh | 37,594, 32,198 | 36,292, 33,844 |

**Smaller than Phase 5's in both fixtures,** so the per-pose recolouring (§2.17.11:
+5.1–5.2 ms a frame in perspective, mostly Bevy's upload) costs no more. It is per vertex,
over 10 % fewer vertices in Midtown and 67 % fewer in urban_grid. **Building the streets**
adds `NetworkJson`'s 5 ms once per run. **`view --bench 20`** on Midtown with buildings (see-through on), at a load of 48–58 from
other sessions, interleaved: Phase 6's look **51.18, 55.73 and 58.49 fps**, Phase 5's
58.17 and 57.22, `--no-streets` 57.24; the median frame **15.5–16.5 ms** in every run,
vsync's; p99 32.6–61.8 ms. On a machine this loaded the means move more between runs than
between looks. Phase 5's figure on a quiet machine was 60.00 (§2.17.11).

*(2026-10-09, changed by the user at gate 14)* The surface mesh is no longer Phase 5's: it
is the junction surfaces, then the median noses, with no fill. Midtown has 5,344 vertices
and 4,042 triangles, and urban_grid 1,107 and 738 (was 10,510 and 9,532; 13,971 and
13,842). The markings mesh is unchanged (§2.18.16).

#### 2.18.11 What becomes of Phase 5's code, tests and gates

- **`src/streets.rs`** keeps the junction surfaces, the pairs, the median fills, the
  ribbons that draw them, the lifts' values, the fade and `mpp_at`. The US markings are
  removed: their constants (`LINE_M`, `DOUBLE_SPACE_M`, `DASH_M`, `GAP_M`, `STOP_M`,
  `FLUSH_MIN_M`, `WHITE`, `YELLOW`) and the yellow, lane-line and stop-line code in
  `Streets::build`. `--no-streets` needs none of it, since it draws today's frames.
- **`src/draw.rs`, `src/render.rs`, `src/view/` and `src/main.rs` do not change,** but
  for `src/main.rs`'s `--no-streets` help text, which names the US markings. They
  take `Streets::surface()` and `Streets::markings()` (positions, a colour and a width per
  vertex, indices) and fade by those, as now.
- **Phase 5's tests** (`tests/streets.rs`), assertion by assertion:

  | Gate | Kept as it is | Removed from `tests/streets.rs`, and where it goes |
  |---|---|---|
  | 5 (synthetic crossing) | the surface (43 vertices, 236.69 m²); pairs; 2 fills | 4 yellow lines, 24 dashes, 6 stop lines: the US set. Phase 6's synthetic test is gate 5 below |
  | 6 (fixtures' counts) | surfaces 106 / 9, pairs 29 / 24, fills 26 / 24, surface mesh; `junction_fills` against the surfaces, 0.000000 m | yellow 88 / 48, stop lines 402 / 71, dashes 2,208 / 1,105, markings mesh; the stop-line edge comparison. They go to gate 6 below |
  | 7 (strips and boxes) | strip ends; box centres on the drawn road | the first stopped box against a 0.60 m line; it goes to gate 7 below with the 0.4 m line |
  | 8 (synthetic, drawn) | the junction (0, 0) and the fill (−20, 0), off → on | the yellow, white, stop-line and dash points; the counts 112,608 / 6,316 / 6,120 / 0. Phase 6's is gate 8 below |
  | 9 (the fade) | `fade`'s 0, 0.5, 1; urban_grid's 0.017; `mpp_at` | the 0.60 m stop line's 0.767 and `colour`'s two Phase 5 colours. They go to gate 9 below |
  | 12 (`M`) | all | — |

  **Phase 5's old numbers stay as recorded:** in its phase (§4, Phase 5) and its gate
  run's record (`specs/reviews/vis-002.md`), both append-only and untouched. Phase 6's
  gate run lists each assertion removed, and each kept line narrowed to drop a removed
  value (Phase 6's scope, "Tests", says which may be). The new tests are a file of their own,
  `tests/look.rs`.
- **Phase 5's scripts:** `scripts/gates-streets.sh` is not edited. Its gates 10 and 11
  still hold with Phase 6's markings (each render twice equal, each differing from its
  `--no-streets` reference). It writes into `scratch/out/streets/`, over Phase 5's videos,
  so Phase 6 copies those first (gate 10 compares with the copies' `framemd5`, and gate 14
  shows the copied videos). The 29 `--no-streets`
  lines in the four reference scripts stay as they are.

*(2026-10-09, changed by the user at gate 14)* Phase 5's median fills go too:
- from `src/streets.rs`: `Fill`, `Streets::fills`, the fill ribbon, and `Ribbon`, `ribbon`
  and `ribbon_mesh`, which only the fills used;
- from `tests/streets.rs`: Phase 5's fill checks, as the amendment's scope lists them
  (Phase 6, "Amended at gate 14").

Phase 5's gate 7 (strip ends, box centres) is untouched: it never counted the fills.

#### 2.18.12 A pin move

The look changes only when the user moves the pin (vis-001 §2.2.2: the `rev` moved by the
user in a commit of its own that re-runs the gates). Then:
- **What follows by itself:** everything from `NetworkJson` and `NetworkData`: where each
  dash, centre line, stop line, connector and arrow is, how many there are, and each
  arrow's type. Gate 6 computes its counts independently from the config and asserts
  the fixtures' numbers, and gate 7 its stopped-box figures, so a changed engine rule shows as a failed count with
  both numbers printed. Gate 10's renders show the changed frames against the old
  renders' `framemd5`.
- **What does not:** the values copied from the front end. **A pin-move phase runs
  `git diff <old pin> <new pin> -- web/src/canvas/colors.ts web/src/canvas/networkRenderer.ts
  crates/geometry/src/network_json.rs`** in the engine (read-only) and reads it for:
  - in `colors.ts`, the dark palette's `ROAD_FILL`, `LANE_MARKING`, `MEDIAN_LINE`,
    `STOP_LINE`, `ROAD_ARROW`, `DEAD_END_BAR` and their five `*_OPACITY` constants;
  - in `networkRenderer.ts`, the `LA_*` constants, the seven glyph functions and three
    helpers of §2.18.5, `drawLaneArrow`'s dispatch, `drawStaticNetwork`'s layers and
    their order, and `generateRoadArrows` (whether it now reaches a fixture);
  - in `network_json.rs`, `NetworkJson`'s fields (a new marking field is not drawn until
    a phase adds it) and the `arrow_type` strings `lane_arrows` can emit.

  Each change found is copied into `src/streets.rs`, with its new line number, and gates
  5, 6, 8 and 9 are predicted again. With no change in those, the copy stands, and the
  record says so. `src/streets.rs` names `90b39292` and the line of every copied value,
  so the diff has a place to land.

At the engine's HEAD today (`fb7b4f4`, read 2026-10-08), `colors.ts` and
`networkRenderer.ts` are unchanged since the pin, and `network_json.rs` gains 11 lines
that label lane conflict pairs, a debug item: nothing drawn would change.

#### 2.18.13 What it does not do

- **No crosswalks, curbs, sidewalks, detectors or PT stops,** as Phase 5 (§2.17.13); the
  engine's `crossing_stripes`, `curb_arcs`, `detector_stripes` and `stops` wait for their
  phases.
- **No generic road arrows** (the front end's own, on links under 8 m; none in either
  fixture), **no median noses** (inside the fill), and no debug item.
- **No change to the junction surfaces, the fills, the strips, the boxes, the fade's
  thresholds, `--no-streets` or `M`.** *(2026-10-09, changed by the user at gate 14)* But
  for the fills, which are removed (§2.18.16).
- **No colour by signal state:** the stop lines are kept per `(link, lane)` for it.

#### 2.18.14 Build cost

**0 packages,** no Bevy feature, no `Cargo.toml` change. `NetworkJson` is in
`assimilator-geometry`, a dependency since vis-001 and already called by Phase 5's gate 6;
`earcut` triangulates the rings and glyphs. No engine request: `NetworkJson` is public and
gives everything, and the colours and glyphs, which the engine keeps only in TypeScript,
are copied (§2.18.12 says how they follow). One changed module, `src/streets.rs`, and the `--no-streets` help text in `src/main.rs`.

#### 2.18.15 Measured while drafting (2026-10-08)

A throwaway probe in gitignored `scratch/vis002p6-probe/`: a `git archive` of `origin/main`
(`0479cd1`) in `repo/`, with a draft `src/streets6.rs` (Phase 6's markings from
`NetworkJson`, the copied glyphs and colours) swapped into `Streets::markings` behind an
environment variable (`P6=1`), built in its own `CARGO_TARGET_DIR`. Nothing in `src/`,
`tests/` or `scripts/` was touched. The engine was read at the pin with `git show
90b39292:<file>`, read-only. The record is `specs/reviews/vis-002.md`, "Phase 6 draft". It
holds:
- `p6-probe` on each fixture (`out/stats*-<fixture>.txt`): `NetworkJson`'s counts, build
  time and order (two processes), the movements and arrows, the stop lines and their
  links, every marking vertex against the drawn road, the overlaps, the mesh, the glyphs,
  and the stopped first boxes against a 0.60 and a 0.40 m line;
- `p6-synth` (`synth/network.yaml`, `out/synth.txt`, `out/synth-on.png`): gate 5's and 8's
  synthetic crossing;
- the CLI renders (`run.sh`, `runs/`), through the probe's `assimilator-video`:
  - streets off: Midtown's ortho with `--no-streets`, **1800 of 1800** against
    `ref-pin90b39292-ortho-roads`;
  - Phase 6's look, twice each: urban_grid's default render and `--camera
    tests/flight.toml`, **8700 of 8700** equal; Midtown's ortho with and without
    buildings, the city flight with and without, and the orbit with buildings, **1800 of
    1800** equal (`ffprobe` as gate 10). Each differs from Phase 5's render of the same case
    (`scratch/out/streets/*.framemd5`) in every frame, and from its reference in every
    frame (checked for `ortho-city`, `default` and `orbit-on`). Wall times: Midtown 48.8–87.5 s, urban_grid 232–451 s, at load 11–66;
- `view --bench 20` on Midtown with buildings (§2.18.10). (`bench.txt`).

**Where the probe differs from the scope:** it keeps Phase 5's markings code and adds
Phase 6's beside it; it composites the colours in `streets6::over` and looks them up per
vertex, where the scope keeps a constant table; its `Kind` names differ (`Median` for
`Centre`); its `glyph()` maps the two unemitted types (§2.18.5) its own way; and its
markings interleave stop lines and connectors in `NetworkJson`'s order, where the scope
groups each kind. The counts, vertices and triangles are the same either way; the mesh
data it hands the drawing are otherwise the scope's.

#### 2.18.16 Amended at gate 14: the median gaps as the dashboard shows them (2026-10-09)

*Changed by the user at gate 14, 2026-10-09.* Gate 14 passed on every point but one: the
space between a two-way street's directions. The user's words, on the grey fill with no
line: "there is no gap.. it looks like more road". Decision 5 changes (§2.18.1), and with
it decision 3's "median fills" are no longer kept:
- **A pair with a gap shows it as the engine's dashboard does.** It is not road: it shows
  our background, as with `--no-streets`. Its ends at a junction are rounded by
  `NetworkJson::median_noses`, drawn in the road's grey.
- **No median fill is drawn.**
- **The centre line stays only on 0 m pairs**, as before.

**The reason:** the gap reads as a gap, as on the dashboard. Phase 6 has not shipped, so
by §6.1 it is amended in place on its branch `vis-002-phase-6` before it ships, rather
than given a new phase. Phase 5's fill (§2.17.1 decision 5, §2.17.5, OQ-18) gets dated
notes. The first build's gate record (`specs/reviews/vis-002.md`, "Phase 6 gate run")
stays as written; the amended build's gate run gets a record of its own.

**What the dashboard draws** (`drawStaticNetwork`, `networkRenderer.ts` l. 584 at
`90b39292`): the road polygons, the median noses and the junction fills in one fill, in
`ROAD_FILL`, over its canvas background. The road polygons stand `median_gap` apart, as
our strips do (§2.17.5), so the gap shows the canvas. The noses round the gap's two
corners at each junction end.

**Its colour here is our background,** `scene::BACKGROUND` (18, 22, 30), not the
dashboard's `CANVAS_BG` `#040810`. It is what `--no-streets` shows there, and nothing is
drawn in the gap. The noses take the road's grey: they are in the surface mesh, with the
road's own material, as the junction surfaces.

**The median noses** (`network_json.rs` l. 1353–1471 at the pin):
- they walk `config.links` in file order, and their dedup set (`seen_nose`, one nose pair
  per two-way pair per junction) is only looked up, never walked. So their order is
  fixed, and none comes from walking a `HashMap`;
- they come only on a link with a twin and a `median_gap` of 0.01 m or more, at each end
  of that link that is a junction, drawn once per pair by the pair's link that comes
  first in link order. Their radius is that link's own `median_gap / 2`;
- each is a quarter-circle crescent between a gap corner and a fillet arc: corner, tangent
  point, 7 arc points, gap centre, and a closing copy. Its first arc point repeats the
  tangent point, and its last repeats the gap centre, so with the closing copy dropped it
  is 10 vertices, 8 of them distinct, and `earcut` gives 6 triangles. Their area equals
  the triangles' within 5.1e-9 relative.

| | Midtown | urban_grid | Synthetic (gate 5) |
|---|---|---|---|
| Pairs: 0 m / 0.5 m / 3.5 m / 4 m gap | 3 / 11 / 4 / 11 | 0 / 24 / 0 / 0 | `L_WC`, `L_NC` 0 m; `L_EC` 0.5 m; `L_SC` 4 m |
| Median noses (vertices; triangles) | **94** (940; 564) | **72** (720; 432) | **4** (40; 24) |
| Their area | 45.91 m² | 1.01 m² | 1.82 m² |
| Nose vertices off the strips and junction surfaces (over the gap) | 470 of 940, at most 0.586 m off | 360 of 720, at most 0.073 m | — |
| Surface mesh: junction surfaces + noses (vertices; triangles) | **5,344; 4,042** (4,404 + 940; 3,478 + 564) | **1,107; 738** (387 + 720; 306 + 432) | **83; 58** (43 + 40; 34 + 24) |
| Phase 5's surface mesh, with the fills | 10,510; 9,532 | 13,971; 13,842 | — |
| Markings mesh | 33,662; 22,288, unchanged | 11,946; 7,194, unchanged | 1,023; 781, unchanged |

Every nose vertex lies inside or on Phase 5's fill for its pair: 940 of 940 and 720 of 720.
So the noses change nothing outside the gap, and inside it they are the only road.

**The meshes and the draw order.**
- **The surface mesh** is the junction surfaces, then the noses, in `NetworkJson`'s order,
  at height 0, in the road's own material.
- **No visible tie:** a nose overlaps the strips and junction surfaces it meets (half of
  each nose's vertices lie on them). Where a nose, a strip and a junction surface tie in
  depth, all three are the same material at the same height with the same normal. So
  whichever the depth test keeps, the pixel is the same colour, as with Phase 5's fills
  (§2.17.9).
- **The markings stay 0.01 and 0.02 m above** (§2.18.8). No marking lies in a gap: every
  marking vertex lies on the strips, junction surfaces or noses within 6.3 mm (below).

**What is drawn on the drawn road** (§2.18.2, measured again without the fills), every
marking vertex against the strips, junction surfaces and noses:
- dashes and arrows: all on it;
- centre lines: 2 of 1,074 on its edge, at 0.0000 m;
- stop lines: within **6.3 mm**, as before. In Midtown 325 of 1,884 lie just off (316
  with the fills: nine stood on a fill's edge).
- urban_grid: every vertex on it, worst 0.0000 m.

**The boxes and the gaps.** Phase 5's measure (its gate 7) counts a box centre on the
drawn road if it lies on a strip or a junction surface, every whole second of each run.
It never counted the fills. Measured again with the noses and without the fills, on the
whole runs:

| | Midtown | urban_grid |
|---|---|---|
| Box centres | 201,256 | 21,358 |
| On strips or junction surfaces | 201,256 | 21,358 |
| On strips, junction surfaces or noses | 201,256 | 21,358 |
| Inside Phase 5's fills | **0** | **0** |
| On a nose alone | 0 | 0 |
| **In a gap** (on none of strips, surfaces, noses) | **0** | **0** |

No box centre is in a gap, so none is drawn over the background. The cause: the engine
places a vehicle at its lane's centre on its link's offset geometry, which our strips
draw, or on its turn path inside a junction surface (Phase 5's gate 7: every junction-span
centre on the surfaces). The gap lies between two strips, where no lane is. Phase 5's gate
7 prediction holds unchanged.

**What a viewer sees** (gate 14):
- a 0.5 m gap is 0.31 px in Midtown's orthographic frame (1.601 m/px) and 0.44 px in
  urban_grid's (1.148 m/px): a thin dark line along the street from high up;
- a 3.5–4 m gap is 2.2–2.5 px there: a dark band down the 15 wide two-way streets;
- close up, in the city flight's low part and the orbit, the gap is a dark strip between
  the two directions, ending at each crossing in two rounded grey corners: the noses,
  0.25 m in radius on a 0.5 m gap and 1.75–2 m on a wide one;
- the amber centre line shows only on Midtown's 3 gapless pairs.

**What changes, against the first build:**
- `src/streets.rs`:
  - removed: `Fill`, `Streets::fills`, the fill ribbon in `Streets::build`, and `Ribbon`,
    `ribbon` and `ribbon_mesh`, which only the fills used;
  - added: `Nose` (a polygon with no closing copy, and its `earcut` triangles),
    `Streets::noses` in `NetworkJson`'s order, and the junction surfaces, then the noses,
    in `Streets::surface`;
  - `Pair` and `Streets::pairs` stay. Nothing draws from them now, but Phase 5's gates 5
    and 6 count them, and they cost nothing.
- **Nothing else changes in the code:** the drawing, `render`, `view`, the CLI, the
  markings and their mesh. The `--no-streets` help text ("junctions filled, and the
  engine dashboard's markings") stays true.

**Measured while amending** (2026-10-09), with a throwaway probe:
- **The probe:** `scratch/vis002p6a-probe/`, a `git archive` of the branch's head
  `8db1742`, with `src/streets.rs` patched as above (the fills behind an environment
  variable, unused). Its `src/bin/p6a-probe.rs` measured the fixtures and the synthetic
  crossing, gate 8's frame included.
- **Its build:** its own `CARGO_TARGET_DIR`, sccache off; 7 m 56 s at a load of 40–55.
  The target folder was deleted afterwards.
- **The renders:** `run.sh`, gate 10's seven cases twice through the probe's
  `assimilator-video`, with `--no-streets` checked once and `view --bench 20` run.
- **Kept:** the probe's outputs (`out/`, `runs/*.framemd5`, `runs/run.log`).

## 3. Open questions

- **OQ-1** — Depend on `assimilator-import` for the projection, or copy its three lines?
  **RESOLVED.**
  - *Costs* are in §2.5.1: +1 package and its optional dependencies (the draft said +1;
    corrected in review round 1), a 10.8k-line crate compiled, and coupling to the importer,
    against a copy pinned by a bit-exact test.
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
  fault. **RESOLVED.**
  - *The symptom:* vehicles freeze at a link end, often just after a late lane change,
    while their FCD keeps reporting driving speed. Followers brake and queue. There are 21
    such cases in the 2850 run, 36 at 5650, and 0 in urban_grid ("position frozen ≥ 10 s
    while speed > 5 m/s", counted 2026-09-30; the engine question below has the method).
  - *Where:* the worst sites are simple unsignalised diverges with no conflict pairs
    (N442, N115, N717). Engine `231ec605` fixes a similar symptom only for trips that end
    at a junction, and all of Midtown's destinations are boundary nodes.
  - *Status:* the user sent the question to the engine's orchestrator on 2026-09-30. It is
    saved there and, on 2026-10-01, queued and not started. The text is in
    `~/dev/ivapo/Orchtr-assimilator-visualizer/engine-question-midtown.md`.
  - *The plan* (§2.9): vis-002 does not wait. When the engine is fixed, the user moves the
    pin, every gate is re-run, and the fixture's traffic is frozen again.
  - *(the engine's answer, 2026-10-01, relayed by the user)* Reproduced at `df8aec0`. A
    real engine fault, with three causes:
    1. a junction lock at unsignalised `conflict_area` junctions without a priority rule
       (N717, N332): new;
    2. a vehicle held at a link end keeps its speed: known, not fixed;
    3. a vehicle that misses its exit lane can circle a block, its route never repaired:
       new.

    It is not fixed on engine main (`aef76a5`). Main now removes stray vehicles and counts
    them as completed, so its numbers look better while traffic does not move better. The
    fix is two engine phases, and the pin stays at `df8aec0` until the engine names the
    commit. A stopgap, removing `conflict_area` from Midtown's 9 unsignalised junctions
    (tested only at `aef76a5`), was declined by the user. The engine side also saw two runs
    of one seed at the pin differ in 75 FCD rows: 2 vehicles, after 1,155 s, speeds only.
  - *(measured in Phase 1's gate run, 2026-10-01)* Far larger than those 75 rows. Four
    Midtown 2850 runs of the same inputs with the same engine binary gave four FCDs, of
    280,729 to 280,875 rows. They part from 502–721 s on, in 331–519 vehicles, and in
    `link_id` and `lane` too, not only in speeds:

    | runs | first difference | vehicles | rows apart (both ways) |
    |---|---|---|---|
    | 2026-09-30's (`scratch/midtown-2850`) vs the fixture's | 721.1 s | 331 | 85,777 |
    | two scratch runs, a vs b | 721.1 s | 366 | 108,907 |
    | a vs the fixture's | 502.1 s | 514 | 206,476 |
    | b vs the fixture's | 502.1 s | 519 | 207,259 |

    urban_grid, run twice the same way on copies in a scratch folder, gave equal FCDs:
    21,557 rows each, 0 rows apart either way. Gate 3's FCD check is a recorded miss, and
    the run in `scratch/midtown` is the fixture (§2.9).
  - *(the engine orchestrator's answer, 2026-10-02, relayed by the user)* The pin gives a
    different Midtown FCD on every run, and neither the seed nor the build is the cause.
    Engine main (`aef76a5`) gives byte-identical FCDs over 11 runs, some of them run at the
    same time. But main only masks the cause, which is likely the stray-vehicle path main
    removes. The commit the engine names must give a byte-identical FCD on repeated Midtown
    runs, checked on our inputs. At the pin, urban_grid is deterministic: two runs gave
    equal FCDs (Phase 1's gate run, above).
  - ~~*(needs-input: engine. It blocks nothing in Phase 1, whose gates measure geometry and
    the user's check expects the gridlock. It does block a presentable Midtown video.)*~~
  - *(resolved 2026-10-05, user)* **Fixed by the engine, taken in by vis-001 Phase 7**
    (`specs/visualizer_spec.md` §2.13.8), which shipped 2026-10-05 when the user passed its
    gate 10. The pin moved `df8aec0` → `90b39292`, and both fixtures were re-frozen. Midtown
    2850 gives one FCD, `01becc86…`, 202,241 rows, on every run (3 of 3 in vis-001 Phase 7
    gate 4), and `gates-city.sh`'s gate 3 compares it with `scratch/midtown-2850-90b39292`,
    `EXCEPT ALL` 0 / 0 (gate 7). Phase 1's recorded gate 3 miss stands as recorded.
- **OQ-5** — Should Overture's building parts come sooner? **RESOLVED.**
  - *The data:* 487 of Midtown's 4,336 buildings (11 %) have `building_part`s. The
    `building` footprint carries the whole building's height, so a tower on a podium is
    drawn as one slab as tall as the tower. Two buildings have a raised base
    (`min_height` > 0), and are drawn from the ground.
  - *Whether that is acceptable* is for the user to see.
  - ~~*(design call; deferred by evidence to Phase 1's gate 16; blocks nothing in Phase 1. If
    yes, it is the next phase.)*~~
  - *(answered 2026-10-02, user, at gate 16)* **Not sooner.** Building parts wait for the
    "nicer buildings" work on the roadmap (§2.13).
  - *(2026-10-03)* They are Phase 4 (§2.15.1 decision 4, §2.16).
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
- **OQ-7** — The README credit's wording (§2.2 l, §2.10). **RESOLVED.**
  - OQ-3 settled where the credit goes, not what it says.
  - ~~*Review round 1's proposal*, to be checked against Overture's published attribution for
    the buildings and transportation themes: "Buildings and roads © OpenStreetMap
    contributors and Overture Maps Foundation, under the Open Database License (ODbL 1.0)."~~
  - ~~*(needs-input: the user; blocks Phase 1's close-out, not its build or its gates.)*~~
  - *(answered 2026-10-01, user)* **The README credit reads exactly:**

    > Building footprints and imported road networks come from Overture Maps (Overture Maps
    > Foundation, overturemaps.org). © OpenStreetMap contributors. Available under the Open
    > Database License (ODbL). Overture's buildings also include data from other sources
    > under their own licences, such as Esri Community Maps contributors, Microsoft Global
    > ML Building Footprints and Google Open Buildings; see
    > https://docs.overturemaps.org/attribution/.

    The source is Overture's attribution page. The OSM credit is required for both themes,
    buildings and transportation, while crediting Overture is optional. The buildings
    theme also has CC BY 4.0 sources. Recorded as §2.2 l, §2.10 and Phase 1's close-out.
    No gate or prediction changes.
- **OQ-8** — The `--camera` render is not deterministic on Midtown (found in Phase 1's gate
  run, 2026-10-01; gate 13's flight comparison is a recorded miss). **RESOLVED.**
  - *The evidence:* `render --camera tests/city-flight.toml --from 300 --to 360 --speedup
    1` on the Midtown fixture, run again and again, gives 1663 or 1695 of 1800 equal
    frames. The ranges that differ are 363–499 or 395–499, with or without `--buildings`.
    The orthographic render is equal, 1800 of 1800, and vis-001's own flight on urban_grid
    is too. Decoded, the first differing frame differs in 16 pixels by 2 in blue or
    green, and every range ends before frame 500, the encoder's next full frame.
  - *Where (bounded check, 2026-10-02):* in raw frames rendered by two processes, without
    buildings, only 3 frames of 350–380 differ, each in one pixel: (390, 403) in 363 and
    364, and (680, 329) in 367, by up to 24 in a channel. Every such pixel is covered by
    vehicle boxes stacked at the same point. In frame 363, vehicles 72, 126, 130 and 180
    sit at (−687.006, 367.803), 0.000 m apart, all at heading 209.23°. Three are drawn at
    16.67 m/s and one at 12.97 m/s, and the two runs' colours are blends of those two
    speed colours. In frame 367 there are two pairs, 0.032 m and 0.000 m apart. These are
    OQ-4's frozen vehicles, several still at driving speed. The draw order itself was not
    observed.
  - *The hypothesis:* depth ties between overlapping boxes, resolved by draw order, which
    Bevy's binned opaque pass does not keep stable. vis-001 §2.11.2's rank lift (0.001 m)
    separates stacked boxes' tops. Their side faces still share planes, and the tilted
    camera shows them; straight down it does not, so the orthographic path is deterministic.
  - *Where the fix belongs:* the `--camera` draw path of vis-001 §2.11, as a small phase of
    its own (§2.13). Until then, any gate that compares two Midtown flight renders will
    miss the same way. vis-001 is not edited here.
  - *Testing the fix:* it must be tested with boxes made to overlap on purpose, not only on
    Midtown. Use two or more boxes at one point and heading, in different colours, seen
    through a tilted camera. A fixed engine may stop stacking frozen vehicles, and that
    would hide the bug on Midtown.
  - *(2026-10-02, user)* Stays open. Its fix phase is scheduled right after the in-video
    credit (§2.13).
  - *(2026-10-03, user, at Phase 2's gate 16)* Stays open. The credit has shipped, so its
    fix phase comes next, as §2.13 says.
  - *(2026-10-03)* Drafted as vis-001 Phase 6 (`specs/visualizer_spec.md` §2.12), not yet
    reviewed. A drafting probe confirmed the hypothesis on a synthetic stack, with neither
    Midtown nor the engine. The fix draws the perspective boxes as one mesh in
    `vehicle_id` order, and leaves out a box identical to a higher id's, so a stack shows
    as its highest id. Its gates include this spec's flight pair, with and without
    buildings. Stays open until vis-001 Phase 6 ships.
  - ~~*(design call: the user; non-blocking.)*~~
  - *(resolved 2026-10-03, user)* **Fixed by vis-001 Phase 6**
    (`specs/visualizer_spec.md` §2.12), which shipped 2026-10-03 when the user passed its
    gate 12. The perspective boxes are drawn as one mesh in `vehicle_id` order, and a
    box identical to a later box is not drawn, so a stack shows as its highest id. The
    flight pair is now 1800 of 1800, with and without buildings, and `gates-city.sh`'s
    gate 13 gives 1800 against the sandboxed run too (vis-001 Phase 6 gates 9 and 10).
    Phase 1's recorded gate 13 miss stands as recorded.
- **OQ-9** — The credit line's wording, and which non-CC-BY building sources it names
  (§2.14.2). **RESOLVED.**
  - *The proposal:* items joined by ` · `: `© OpenStreetMap contributors (ODbL)`, then
    `Overture Maps Foundation, release <r>`, then each building source as in §2.14.2's
    table. Midtown without buildings reads `© OpenStreetMap contributors (ODbL) · Overture
    Maps Foundation, release 2026-09-23.1`.
  - *What Midtown's buildings carry* (the probe): OpenStreetMap, USGS Lidar (316
    buildings) and Microsoft ML Buildings (29). They include no CC BY source.
  - *Beyond decision 1:* the table also names Microsoft's (ODbL, not CC BY) and USGS's (no
    licence named on Overture's page) when present, and any dataset it does not know,
    verbatim. The options, with Midtown's line with buildings and its width:
    - (a) every non-OSM source present, which is the draft: `… · Microsoft ML Buildings
      (ODbL) · USGS Lidar`, 60.37 em, 63 % of a 16:9 frame's allowed width;
    - (b) the CC BY and ODbL sources only: `… · Microsoft ML Buildings (ODbL)`, 54.64 em;
    - (c) decision 1 as written, CC BY only: the roads' line, 40.23 em.
  - *The trade-off:* (a) and (b) credit Microsoft's ODbL footprints, which Overture's page
    lists with their licence. (c) is the shortest and leaves them uncredited in the video.
    Only (a) never decides in code which listed source to leave out.
  - *Recommendation:* (a), with the wording as written.
  - ~~*(needs-input: the user; blocks gate 7's exact strings and gate 11's width, not the
    design.)*~~
  - *(answered 2026-10-02, user)* **(a): every non-OSM source present is named, with the
    wording as written in §2.14.2.** Gates 7 and 11 keep their values. Recorded as
    §2.14.1 decision 6 and §2.14.2.
- **OQ-10** — TomTom on imported Overture roads (§2.14.4). **RESOLVED.**
  - *The facts:* Overture's transportation theme lists "Data from TomTom." beside
    OpenStreetMap. The engine reads each segment's datasets and keeps none of them, so no
    project says whether its roads include TomTom's.
  - *The options:*
    - (a) never name TomTom until the data says so, which is the draft;
    - (b) name it on every Overture-imported network: always safe, but sometimes untrue;
    - (c) check Midtown once: one DuckDB query of its 222 segments against
      `theme=transportation/type=segment` of `2026-09-23.1`, before about 2026-11-22. It
      needs the user's leave, as drafting's probe did.
  - *An engine request*, written here and not sent: `import_report.json` could record the
    datasets of the segments it kept, as `source.datasets` with link counts. That would
    let the line name TomTom from the data, as decision 1 asks.
  - *Recommendation:* (a) with (c) before 2026-11-22. If Midtown has TomTom segments, the
    user decides between (b) and waiting for the engine.
  - ~~*(design call: the user; flags gate 7's claim that Midtown's line is complete, and
    blocks nothing else.)*~~
  - *(answered 2026-10-02, user)* **(a) with (c).** TomTom is not named until the data
    shows it. Midtown's check was run the same day: all 222 segments name OpenStreetMap
    only, so Midtown's line is complete for gate 7 (§2.14.4, §2.14.10). The engine request
    above stays as written, not sent; the user carries it. Recorded as §2.14.1 decision 7,
    §2.14.4 and gate 7.
- **OQ-11** — A HERE import, or any source type other than `Overture` and `Osm`
  (§2.14.4). **RESOLVED.**
  - *The draft:* an error, failing closed. HERE's roads are not OSM data, and their credit
    is HERE's to set.
  - *Its cost:* `render` on a HERE project, which works today, stops until a credit is
    written for it. No such project is known here; Midtown is Overture.
  - *The alternative:* item 1 only, as decision 1 has it for every imported network. That
    names OpenStreetMap on roads that did not come from it, and names nothing for HERE.
  - *Recommendation:* the error, until the user gives HERE's wording.
  - ~~*(design call: the user; blocks nothing for Midtown.)*~~
  - *(answered 2026-10-02, user)* **The error, until the user gives HERE's wording.**
    Recorded as §2.14.1 decision 8 and §2.14.4.
- **OQ-12** — A font under the OFL in an MIT OR Apache-2.0 repo (§2.14.6). **RESOLVED.**
  - *Why:* Bevy's built-in font has no `©` or `·`.
  - *The options:*
    - (a) commit Fira Sans Medium (457,248 bytes) with `OFL.txt` under `assets/fonts/`.
      `Cargo.toml` keeps `license = "MIT OR Apache-2.0"`, as `bevy_text` does for its own
      OFL subset, and the README and `CLAUDE.md` name the exception;
    - (b) as (a), with `license = "(MIT OR Apache-2.0) AND OFL-1.1"`. This states what
      the package holds, and the OFL's condition 5 asks the font to be "distributed
      entirely under this license". It changes `CLAUDE.md`'s rule that every crate sets
      `MIT OR Apache-2.0`;
    - (c) Bevy's subset and ASCII text: `(c)` in place of `©`, ` - ` in place of ` · `, and a
      monospace line 25 % wider.
  - *Recommendation:* (b). It is exact, and the crate is not published (`publish =
    false`), so the field has no other reader. (a) is the common practice, with the
    exception stated only in prose.
  - ~~*(design call: the user; blocks Phase 2's build. The draft's scope writes (b).)*~~
  - *(answered 2026-10-02, user)* **(b): `license = "(MIT OR Apache-2.0) AND OFL-1.1"`,
    and the README and the repo's `CLAUDE.md` name the font exception.** Recorded as
    §2.14.1 decision 9, §2.14.6, and Phase 2's scope and close-out.
- **OQ-13** — The full credit when a video is published (§2.14.10). **RESOLVED.**
  - *The facts:* OSMF's safe harbour for video where the map is a major component asks for
    the corner credit **and** a credit in the end credits or the description, with the URL
    `openstreetmap.org/copyright`. Phase 2 draws the corner line only.
  - *The options:*
    - (a) the README tells whoever publishes a video to put the full credit, OQ-7's text
      with that URL, in its description;
    - (b) a closing card in the video, a later phase.
  - *Recommendation:* (a) in Phase 2's close-out. (b) waits for the harness contract
    (decision 5).
  - ~~*(design call: the user; non-blocking.)*~~
  - *(answered 2026-10-02, user)* **(a).** Phase 2's close-out adds README guidance:
    whoever publishes a video puts the full credit, OQ-7's text with
    `openstreetmap.org/copyright`, in its description. A closing card waits for the
    harness contract. Recorded as §2.14.1 decision 10 and Phase 2's close-out.
- **OQ-14** — How see-through is asked for: a flag, or a setting in the flight file
  (§2.15.6). Decision 3 left it to the draft. **RESOLVED.**
  - *The draft:*
    - `render --see-through` needs `--buildings` and `--camera`;
    - `view --see-through` starts with it on and needs `--buildings`;
    - `X` in `view` flips it.

    Each missing companion flag is one error line, checked before any file is read.
  - *The alternative:* `see_through = true` beside `keyframes` in the flight file.
    - The flight would carry its own look.
    - The keyframe file would stop being the camera's alone: today another top-level
      key is an error.
    - A file with the key would fail, or be ignored, on a network without buildings.
    - `view` would still need a key and a flag, and `K` cannot print the setting.
  - *Recommendation:* the flag, with the names as drafted: `--see-through` on both
    commands and `X` in `view`.
  - ~~*(design call: the user; blocks Phase 3's CLI and gate 10's cases, not the rule, the
    drawing or any other gate.)*~~
  - *(answered 2026-10-03, user, before review)* **A flag, and see-through on by
    default:** `--no-see-through` on `render` and `view` turns it off, and `X` in `view`
    toggles it. On wherever buildings are drawn in perspective (`render --buildings
    --camera`, `view --buildings`). With the flag every frame is byte-identical to
    today's, and the orthographic render is unchanged either way. The opt-in
    `--see-through` and its two errors are dropped. The draft proposes that
    `--no-see-through` with nothing to cut is accepted with no effect. Recorded as
    §2.15.1 decision 3's dated note, §2.15.6 and Phase 3's scope and gates 2, 3, 7, 10,
    12, 13 and 14.
- **OQ-15** — Should more than the wedge be cut (§2.15.7)? **RESOLVED.**
  - *The facts:*
    - The wedge clears the look-at point's surroundings: centre boxes hidden fall from
      59.4 % to 0.0 % on the orbit flight. Over the whole frame, boxes hidden fall
      only from 79.2 % to 54.8 % there, and from 47.5 % to 46.3 % on the city flight.
    - From 327 s on, the city flight looks into Central Park, where no box is in its
      centre, so the wedge has little traffic to show there.
    - `WIDTH` is the dial. 0.20 instead of 0.15 clears the city flight's centre a little
      more (16.0 % to 12.0 % hidden), at a third more buildings cut a frame (33.3 to
      43.9).
  - *The options:*
    - (a) the wedge, as decided (decision 2), with flights written to look at the
      traffic;
    - (b) a later phase that also cuts buildings hiding roads elsewhere in the middle of
      the frame. It is not measured here, and it would change decision 2's look;
    - (c) a wider `WIDTH`, which is iteration (§2.15.2).
  - *Recommendation:* (a) for Phase 3. The user judges at gate 14, on the orbit, whether
    (b) or (c) is wanted.
  - ~~*(design call: the user, at gate 14; non-blocking.)*~~
  - *(answered 2026-10-03, user, at Phase 3's gate 14)* **(a): keep the wedge.** Flights
    are aimed at the traffic. Neither (b) nor (c) is taken: `WIDTH` stays 0.15, the ease
    band `max(r, 10 m)` and the stub 3 m.
- **OQ-16** — A building with parts: draw its parts only, or also the rest of its
  footprint (§2.16.4)? **RESOLVED.**
  - *The facts:* the parts of 414 of Midtown's 487 buildings with parts cover their
    footprint to within 1 m². 196,624 m² is uncovered in all, 16 % of their footprint, and
    160,808 m² of that is three underground buildings (`is_underground`, 10 m blocks
    today), one of them over OQ-4's queue. The rest, 35,817 m², is spread over 70
    buildings, and has no height of its own.
  - *The options:* (a) the parts only, decision 4 as written; (b) the parts, and the rest
    at the building's height, which puts back the slab this phase removes and needs a
    polygon difference (a crate, or a column in the fetch); (c) the rest at a fixed low
    height, which invents one.
  - *Recommendation:* (a). The user sees what it leaves at Phase 4's gate 15.
  - ~~*(design call: the user; blocks Phase 4's mesh counts (gate 6) and the predictions of
    gates 12 and 15, not the fetch or the reader.)*~~
  - *(answered 2026-10-04, user, before review)* **(a): the parts only.** The footprint no
    part covers is not drawn. Recorded as §2.16.1 decision 1 and §2.16.4. Gates 6, 12 and
    15 were predicted with (a) and do not change.
  - *(2026-10-04, user, at Phase 4's gate 15)* Stands: gate 15 passed, and nothing drawn
    from the parts changes.
- **OQ-17** — A part, or a raised base, wholly above its building's cut height `h′`: a lid
  at `h′`, or removed (§2.16.7)? **RESOLVED.**
  - *The facts:* §2.15.7's clamp puts it at `h′` with no walls: a lid. Over a lower part of
    the same building, it is in that part's roof plane and changes no pixel. Over nothing,
    an overhang, it covers the street at `h′`, down to 3 m. 19,841 m² of Midtown's raised
    parts lie over no ground part of their building, in 103 parts of 41 buildings. With
    see-through on, the city flight has such a lid in 726 of its 1,800 frames, up to
    2,567 m² in one frame; the orbit has none.
  - *The options:* (a) the lid: §2.15.7 as written, continuous, and no worse than Phase 3,
    which cuts the whole footprint to `h′` there today; (b) removed once `h′` falls to its
    base: the street under an overhang shows, but its roof vanishes in one frame, a pop
    §2.15.3 does not have, and each vertex must carry its volume's base.
  - *Recommendation:* (a).
  - ~~*(design call: the user; blocks Phase 4's gate 8 case "B cut" and gate 12's counts
    with see-through on, which are (a)'s; not the fetch, the reader or the mesh.)*~~
  - *(answered 2026-10-04, user, before review)* **(a): the lid at the cut height**,
    §2.15.7's clamp as built. Recorded as §2.16.1 decision 2 and §2.16.7. Gates 8 and 12
    were predicted with (a) and do not change.
  - *(2026-10-04, user, at Phase 4's gate 15)* Stands: gate 15 passed, and nothing drawn
    from the parts changes.
- **OQ-18** — What fills a two-way street's median gap (§2.17.5)? **RESOLVED.**
  - *The facts:* the engine sets a pair's two strips `median_gap` apart, and today the gap
    shows the background. Midtown's pairs: 3 at 0 m, 11 at 0.5 m, 4 at 3.5 m and 11 at
    4 m; urban_grid's 24 at 0.5 m. Whether a wide gap is painted or raised is not in the
    data.
  - *The options:*
    - (a) **fill every gap with the road's grey;** under 1 m, one double yellow line in
      its middle; 1 m or more, a flush median: a double yellow line inside each edge
      (MUTCD §3B.24);
    - (b) fill only gaps under 1 m, and leave 3.5–4 m gaps dark, as a raised median
      would be, until the ground phase draws curbs (decision 2);
    - (c) fill every gap grey, with one double yellow line in its middle whatever its
      width.
  - *Recommendation:* (a). It is the US marking for a painted median, it keeps a street
    one surface until curbs exist, and it is what the probe drew.
  - ~~*(design call: the user; blocks the median fills' and yellow lines' counts in gates
    5, 6 and 8 for Midtown's 15 wide pairs: with (b) Midtown has 11 fills and 28 yellow
    lines; with (c) 26 fills and 58 yellow lines. Nothing else.)*~~
  - *(answered 2026-10-05, user, before review)* **(a): the painted median.** Every gap is
    filled with the road's grey; under 1 m one double yellow line in its middle, 1 m or
    more a double yellow line inside each edge. Recorded as §2.17.1's answers and
    §2.17.5. Gates 5, 6 and 8 were predicted with (a) and do not change.
  - *(2026-10-08, user, at Phase 5's gate 15)* Stands: gate 15 passed, and nothing drawn
    changes.
  - *(2026-10-08, Phase 6 draft)* **The yellow part of (a) is replaced when Phase 6
    ships:** no double yellow line and no flush median's lines. The engine draws one
    0.15 m centre line, only on a pair with a 0 m gap (§2.18.3), so a pair with a gap
    shows its grey fill and no line. **The fill stays** (§2.18.1, decision 3).
  - *(2026-10-09, user, at Phase 6's gate 14)* **The fill part of (a) is replaced too when
    Phase 6 ships.** On the grey fill with no line the user said "there is no gap.. it
    looks like more road". A pair's gap shows as the engine's dashboard shows it: the
    background, with its junction ends rounded by `NetworkJson::median_noses` in the
    road's grey, and no fill (§2.18.1, decision 5 as changed; §2.18.16). Nothing of (a)
    remains drawn.
- **OQ-19** — How are markings drawn where they are under a pixel (§2.17.10)? **RESOLVED.**
  - *The facts:* a 0.15 m line is 0.09 px wide in Midtown's orthographic frame, 0.06 px
    in `view`'s launch fit, and 0.09–0.54 px through the city flight. Drawn at true
    colour, MSAA ×4 turns Midtown's lines, which run at an angle to the pixel grid, into
    a scatter of separate white and amber dots, and an amber dot reads as a slow
    vehicle. urban_grid's run along the grid and show as thin continuous lines. The orbit
    at 250 m (0.65 px) and anything closer show whole lines.
  - *The options:*
    - (a) **the fade:** each vertex's colour blends from the road's grey (under 0.1 px) to
      the marking's (from 0.5 px), opaque, per pose. Midtown's overview shows junctions,
      fills and stop lines, and no lane or yellow line; close up nothing changes. It
      costs a per-pose rebuild of the markings' colours, +5.1–5.2 ms a frame in a
      perspective render, with `view --bench` still at 60 fps (§2.17.11);
    - (b) **true colours always:** static meshes, nothing per frame, and the dots in every
      overview and high shot;
    - (c) **a minimum width on screen:** each line at least one pixel wide. Lines show at
      every height, but at Midtown's overview a 1.6 m line on a 3.25 m lane, and a flush
      median's four lines merge into a 6 m amber band. It is a per-pose geometry rebuild.
  - *Recommendation:* (a). It is the only one that shows a clean overview and true
    markings close up, and it is the probe's measured look.
  - ~~*(design call: the user; blocks gate 9 and the markings mesh's per-pose rebuild in
    gate 13 and 14 with (a). With (b), gate 9 and the rebuild go, and gate 15's overview
    has the dots. Gates 1–8 and 10–12 hold either way: gate 8's lines are 3 px wide.)*~~
  - *(answered 2026-10-05, user, before review)* **(a): the fade.** A marking's colour
    blends from the road's grey under 0.1 px to its own from 0.5 px, opaque, per pose.
    Recorded as §2.17.1's answers and §2.17.10. Gates 9, 13 and 14 were predicted with (a)
    and do not change.
  - *(2026-10-08, user, at Phase 5's gate 15)* Stands: gate 15 passed, and nothing drawn
    changes.
- **OQ-20** — Which colours do the engine's markings take here (§2.18.4)? **RESOLVED.**
  - *The facts:* the dashboard has two palettes, `dark` (its default) and `light`, which
    differ only in the centre line and the dead-end bar. It paints each marking with an
    opacity (0.5 lane dashes, 0.7 centre line, 0.9 stop lines, 0.6 arrows, 0.85 dead-end
    bar) over its road, (81, 81, 82) as shown. Our road is (92, 96, 104) and stays.
  - *The options:*
    - (a) **dark, each colour composited at its opacity over our road:** dashes (174,
      176, 180), centre line (199, 167, 77), stop lines (239, 239, 240), arrows (190, 191,
      195), dead-end bar (217, 72, 73). The dashboard's contrast with the road, opaque;
    - (b) dark, the colours as the dashboard shows them over its own road: dashes (168,
      168, 169), centre line (196, 162, 71), stop lines (238, 238, 238), arrows (185, 185,
      186), bar (215, 70, 70). The same pixels as the dashboard, 6–11 levels darker
      against our lighter road;
    - (c) dark, the raw colours at full strength: white dashes, stop lines and arrows
      (255, 255, 255), centre line (245, 197, 66), bar (239, 68, 68). Brighter than the
      dashboard shows anything;
    - (d) light, composited as (a): the centre line (180, 112, 35), an orange near the
      2–5 m/s boxes' (244, 132, 45), and the bar (201, 47, 48).
  - *Recommendation:* (a). It keeps the look the dashboard gives against its road, on
    ours, and it is what the probe drew (§2.18.15).
  - ~~*(design call: the user; blocks the colours in gates 5, 8 and 9 and gate 14's
    descriptions. Every count, position, mesh size and gate 7 hold either way.)*~~
  - *(answered 2026-10-08, user, before review)* **(a): the dark palette at the
    dashboard's opacities, over our road.** Recorded as §2.18.1's decision 4 and §2.18.4.
    Gates 8, 9 and 14 were predicted with (a) and do not change (gate 5 asserts no
    colour).
  - *(2026-10-09, user, at gate 14)* Stands: the colours pass as they look.

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
    - `read(path, &NetworkConfig) -> Result<Buildings>`, with every check of §2.4.3 in its
      order. `Buildings` holds each building's `id`, its projected and oriented polygons,
      its height and the rule that gave it (§2.4.2), the counts per rule, and the tallest
      height;
    - `building_mesh(&Building)`: one building's positions, normals and indices in `f64`
      world metres (§2.6), with its wall-quad and roof-triangle counts; roofs through
      `earcut`. `mesh_data(&Buildings)` appends every building's, in file order. Gate 9
      calls `building_mesh` per building.
  - **Drawing (`src/draw.rs`).**
    - The Bevy mesh from that data, baked at `(fx, fy)` and converted to `f32` there.
      `render` and `view` both spawn it with `NoFrustumCulling`
      (`bevy::camera::visibility::NoFrustumCulling`, not in the prelude);
    - the lit material and the sun (§2.7), with `SUN_AZIMUTH_DEG`, `SUN_ELEVATION_DEG` and
      the ambient level as named constants. `draw::sun_direction(azimuth_deg,
      elevation_deg) -> Vec3` gives the direction the light travels, in Bevy coordinates,
      and the sun's `Transform` faces along it (gate 12);
    - `draw::ortho_eye(tallest_m) -> (eye, far)` is §2.8's rule: `(500, 1000)` for a
      `tallest_m` of 0, which is what a job without buildings passes.
      `src/draw.rs:look_down` and `src/draw.rs:projection` take its two values.
      `src/draw.rs:perspective` is not edited.
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
      given, and sets their visibility from the flag every frame;
    - nothing else in the frame order changes.
  - **The fixture.** `scripts/fixture.sh midtown`, §2.9.
  - **`Cargo.toml`.** `earcut` 0.4 (§2.11). No Bevy feature changes.
  - **Tests.**
    - `tests/buildings.rs` (new): gates 6–12 and 14, with gate 7's accepted cases. The
      headless ones (6, 7's library cases, 8 on the shapes, 11's eye rule, 12's sun
      direction, 14) need no fixture. Those that need the Midtown fixture or the GPU are
      `#[ignore]`d, like `tests/camera.rs`.
    - The headless tests build their network in the test with `serde_yaml::from_str`
      (already a dependency). It has a `metadata.name`, Midtown's `map_origin`, two
      `endpoint` nodes at `point: [-130, -30]` and `[200, 30]`, and `links: []`. So
      `read`'s `map_origin` and extent checks pass on the shapes, and no file is needed.
    - `tests/shapes.geojson` (new): six hand-written shapes, given in metres about
      Midtown's origin. They are written as lng/lat to 7 decimals through
      `xy_to_lnglat`. `shape-cube`'s exterior is written clockwise and `shape-courtyard`'s
      hole counter-clockwise, the opposite of what the mesh needs, so the reader's
      reorientation is tested (gates 8 and 12). The other rings are counter-clockwise, with
      clockwise holes.

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
    - `scripts/gates-city.sh` (new) runs:
      - gates 3 and 4;
      - gate 5's offline checks: the report line in `scratch/midtown/fetch.log`, and the
        cache's bytes and SHA-256. Only with `REFETCH=1`, since they use the network: its
        second fetch and its missing-release case;
      - gate 7's CLI cases. Each `view` case runs under `perl -e 'alarm 60; exec @ARGV'`, so
        a window that opens fails the case instead of hanging;
      - gate 13.

      `scripts/gates.sh` is not edited. Gate 1's `--camera` reference is kept and
      compared by hand, after `scripts/gates.sh` has run: the non-`#` lines of its
      `scratch/out/camera.framemd5` against the reference, as its own reference check
      does.
      Gate 15 is recorded by hand, as vis-001's `--bench` was.
    - `tests/gates.rs`, `tests/view.rs`, `tests/slider.rs` and `tests/camera.rs` are not
      edited.
- **Exit gate.** On the development machine (Apple M3, macOS), on two fixtures:
  - vis-001's urban_grid (engine `df8aec0`, baseline, seed 42), for what must not change;
  - Midtown (`scripts/fixture.sh midtown`, §2.9) for the rest.

  Only fetches use the network: the fixture's step 5, and gate 5's second fetch and
  missing-release case. Every other check runs offline. The predictions are §2.12's
  measurements: the drafting ones, and review round 1's. Nothing of this repo was built.
  Distances are compared to 1e-9 m, unless a gate says otherwise.
  - **What must not change:**
  1. **`render` without `--buildings`.**
     - `scripts/gates.sh` passes, and its reference comparison gives **8700 of 8700**
       against `scratch/ref-8eb9052.framemd5` (§2.2 f).
     - The `--camera` path, too. Before the first code change, `scripts/gates.sh` runs at
       the base commit, as every vis-001 phase began. Its `--camera tests/flight.toml`
       frames are kept as `scratch/ref-camera-<base>.framemd5`. After the change, that
       render's `framemd5` equals it, **8700 of 8700**.
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
     - The working `target/` growth is recorded.
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
     - *Note (2026-10-05):* from vis-001 Phase 7 (the engine pin `90b39292`),
       `scripts/gates-city.sh` checks **202,241 rows** and compares with
       `scratch/midtown-2850-90b39292`, a second run at the new pin. This gate's recorded miss
       (280,875 rows, and the comparison with `scratch/midtown-2850`) stands as recorded.
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
     - a made-up point (review round 1), (lng −73.974, lat 40.7625) →
       (296.3801816977122, −247.1576725002842). Regrouped as `(lng − origin_lng) · (111320
       · cos …)`, its `x` would be 296.38018169771215.

     And with a made-up origin `[−0.1, 51.5]` (review round 1), (lng −0.09, lat 51.51) →
     (692.9832935049988, 1113.1999999997786). Here `to_radians()` would give `x` =
     692.9832935049986. At Midtown's origin the two agree, so this case is what pins that
     part of the order (§2.5).

     These are the engine formula's values in IEEE doubles, with its operation order (Python
     on this machine, which uses the same libm `cos` as Rust). `xy_to_lnglat` of each gives
     back the lng/lat to 1e-12°. No point here is taken from map data (§2.10). The bbox
     corners are the user's import request.
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
     - Gate 9's roof-area and outward-wall checks pass on the shapes too, including the cube
       and the courtyard's hole, which are written in the wrong orientation.
     - Midtown gives 4,277, 8 and 51, equal to the fetch report's.
  9. **The mesh, on Midtown's 4,336 buildings** (§2.6), through `building_mesh` per building:
     - all **4,336** are meshed;
     - for each building, the roof triangles' total area equals its footprint's area
       (exterior minus holes, by the shoelace formula) to 1e-9 relative: **0 failures**;
     - wall quads: **42,776**, one per ring edge;
     - every wall's normal points out of the solid: a point 1 mm along it from the edge's
       midpoint is outside the footprint (outside the exterior, or inside a hole). **0
       inward**;
     - every roof vertex is at its building's height;
     - roof triangles: **34,178** (earcut 0.4.11 on `f64`, review round 1).
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
      - The orthographic eye is 500 m up with a far of 1,000 m in both, since Midtown's
        472 m does not raise it (§2.8). That is `draw::ortho_eye` of each job's tallest
        height (`Job::buildings()`'s, 0 without), which is `(500, 1000)` for both.
        Headless, `ortho_eye` gives 500 for a tallest of 0, 472 and 490, and 505 for 495,
        with far = eye + 500.
      - A pixel is *inside a footprint* when its centre is inside the union of the
        projected footprints, holes outside (as in gate 10).
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
        Prediction: **roof > south wall > west wall** (0.866, 0.433 and 0.25 of the sun by
        Lambert, about 0.88, 0.50 and 0.36 by Bevy's Burley term, §2.7). Each of the three
        samples is neutral (R = G = B) and not ground, so a culled wall cannot pass by
        showing the ground behind it. The roof's red is below 255.
      - **The sun's side (headless):** `draw::sun_direction(SUN_AZIMUTH_DEG,
        SUN_ELEVATION_DEG)` is §2.7's formula. At 210° and 60° it equals Bevy `(0.25,
        −0.8660254037844386, −0.4330127018922194)`, which is `(1/4, −√3/2, −√3/4)`, to
        1e-6 per component. A sun mirrored to 150° would pass the order above, but not
        this.
      - **Courtyard:** look-at (0, 0), pitch 90, `height_m` 60. The image centre is ground;
        `project` of (0, 14, 20) on the roof is not.
      - **The L's notch:** look-at (90, 10), pitch 90, `height_m` 60. The image centre is
        ground; `project` of (70, −10, 14) on the roof is not.
  13. **Deterministic and offline.**
      - `render --buildings scratch/midtown/buildings.geojson --camera tests/city-flight.toml
        --from 300 --to 360 --speedup 1`, twice: `ffprobe` gives `1920,1080,30/1,1800`, and
        `framemd5` is equal, **1800 of 1800**.
      - A third run under `sandbox-exec -p '(version 1)(allow default)(deny network*)'`
        completes, equal to both. This is a prediction. While drafting, the profile was
        tried only on `curl` and `echo`, not on Bevy with Metal and an ffmpeg child.
      - The same window rendered twice without `--camera` (orthographic, roofs only) is
        equal too.
  14. **`B`, headless.**
      - On the plain state of vis-001 Phase 3, `b` flips `buildings_shown` from true to
        false, and a second `b` flips it back.
      - `t`, the clock, the pose, `k` and `follow` stay exactly as they were.
      - During a right-button orbit, a left drag and a scrub, `b` flips the flag and the
        gesture goes on unchanged.
  15. **Frame rate: recorded, not gated**, by vis-001 §2.9.8 decision 2, the user's.
      - `view --bench 20` runs on Midtown at the default window, with and without
        `--buildings`, and its JSON is recorded with the load average.
      - For comparison, not a prediction: urban_grid gave 60.00 fps at vis-001 Phase 4 and
        59.99 at Phase 5 (vsync).
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

      *(Passed by the user, 2026-10-02.)* The user checked in `view`: alignment, the tilt to
      25°, `B`, and roofs lighter than walls. They also checked
      `scratch/out/city/flight1.mp4` and `midtown-buildings.mp4`. The look stays as built,
      and colour tuning is later iteration. OQ-5 is answered: not sooner. The gate record is
      in `specs/reviews/vis-002.md`.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `render` without `--buildings`, urban_grid: default and `--camera` | 8700 of 8700 frames equal to each reference | 1 |
  | `Cargo.lock` packages added; HTTP clients; release crates | 1 (`earcut`); 0; 363 | 2 |
  | Fixture FCD | 280,872 rows, 985 vehicles, 1.1…1199.1 s | 3 |
  | Fetch bbox (W S E N) | −73.9938413 40.7544211 −73.9643086 40.7740495 | 4, 5 |
  | Buildings: by height / by num_floors / at 10 m | 4,336: 4,277 / 8 / 51 | 5, 8 |
  | Cache file | 2,140,989 bytes, one SHA-256 on every fetch | 5 |
  | Fetch time | 122–145 s (recorded) | 5 |
  | Projection | 5 exact values, at two origins | 6 |
  | Roof-area failures; inward walls; wall quads; roof triangles | 0; 0; 42,776; 34,178 | 9 |
  | Centreline inside footprints | 2.239 % (bar 3 %) | 10 |
  | Orthographic eye for Midtown, with buildings | 500 m, far 1,000 m (unchanged) | 11 |
  | Top-down coverage violations | 0 and 0 | 11 |
  | Shape luminance; sun direction at 210°, 60° | roof > south wall > west wall; Bevy (1/4, −√3/2, −√3/4) to 1e-6 | 12 |
  | Keyframed render with buildings, twice and sandboxed | 1800 of 1800 | 13 |

- **Not predicted, and so not gated:**
  - the look: the grey, the ambient level, the sun's illuminance and the shading of the
    shapes beyond gate 12's order;
  - `view`'s frame rate (gate 15; vis-001 §2.9.8 decision 2);
  - wall times: the fetch (gate 5), the engine run (gate 3) and the clean build (gate 2).
    Also the Midtown default render (9,000 frames) with `--buildings`, recorded once by
    hand at close-out beside the drafting record's 503.3 s without (§2.12). Other sessions load this machine,
    so times are recorded (vis-001);
  - the working `target/` growth (gate 2) and the building pixel count (gate 11).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-1`), one push. The commits:
    - the projection and extent half of `src/buildings.rs` (`lnglat_to_xy`,
      `xy_to_lnglat`, `network_extent`, `fetch_bbox`), `network-extent`, the fetch and the
      Midtown fixture (gates 3–6);
    - reading and mesh data (gate 7's library cases, gates 8–10);
    - drawing in `render` and `view`, and `B` (gates 1, 2, 7's CLI cases, 11–15);
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - a new `rules/buildings.md` (`max_lines: 60`) for §2.3–§2.8 as built: the fetch and
      its report, the cache format and its checks, the projection, the height rule, the
      mesh, the light, the orthographic eye rule and `B`. Its `sources` are
      `src/buildings.rs`, `src/bin/network-extent.rs`, `scripts/fetch-buildings.sh`,
      `src/draw.rs`, `src/view/state.rs` and `src/view/mod.rs`;
    - four rules are at their caps: `rules/render.md` (60/60), `rules/view.md` (60/60),
      `rules/camera.md` (60/60) and `rules/inputs.md` (50/50). Each gains a pointer to
      `rules/buildings.md`, reworded to fit; no `max_lines` is raised:
      - `render.md`: `--buildings` on `render`;
      - `view.md`: `--buildings` on `view`, and `B`;
      - `camera.md`: the orthographic eye rule, and `NoFrustumCulling` on the building mesh;
      - `inputs.md`: the `--buildings` check's place, after `--camera`'s;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - the README gains:
      - the fetch script with its prerequisites (DuckDB ≥ 1.5.1 with `httpfs` and
        `spatial`; the network, for the fetch only);
      - `--buildings` and `B`;
      - the Midtown fixture (it needs the user's project) and the new gate commands:
        `scripts/fixture.sh midtown`, `scripts/gates-city.sh` with `REFETCH=1`, and
        `cargo test --release --test buildings -- --include-ignored --test-threads=1`;
      - the ODbL credit (§2.2 l), exactly OQ-7's text (answered 2026-10-01). It covers
        Overture's buildings and the roads of an imported network. The README also says that a
        credit line in the video is a later phase, and that it must ship before any
        Midtown video is shown outside;
    - `CLAUDE.md` gains one line beside the licence: nothing derived from OSM or Overture
      (ODbL) is committed; fetched buildings and imported networks stay out of the repo;
    - status artifact: none needed, since this repo has none (`.spec-lint.yaml`'s
      `status_artifacts` is empty).
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and its
    cause.
  - Write this phase's `shipped` date.

### Phase 2 — Credit: a line built from the data, on every frame of an imported network's render
*Produces the observable: yes. `render` of an imported network writes the run's video with
its data's credit in the bottom-right corner of every frame, with or without `--buildings`.
`render` of urban_grid is byte-identical to Phase 1's (gate 1).*

Drafted 2026-10-02; the design is §2.14, and the user's decisions are §2.14.1. It builds on
Phase 1 and changes none of its behaviour on a network without `metadata.map_origin`, nor
anything in `view`. The plan, as decided:
- the cache carries its sources and release (§2.14.3);
- every non-OSM building source present is named, in §2.14.2's wording (OQ-9);
- TomTom is not named until the data shows it (OQ-10);
- a HERE report is an error (OQ-11);
- the font is Fira Sans Medium, committed, with `license` naming OFL-1.1 too (OQ-12);
- the README tells publishers what to put in a video's description (OQ-13).

**Its one deadline:** gate 4's re-fetch needs `2026-09-23.1` on S3, until about 2026-11-22
(§2.14.3). Drafting's probe file in `scratch/vis002p2-probe/fetch/` is the same file and
covers the gates that do not fetch. If the release is gone and the probe file is still
there:
- it is copied to `scratch/midtown/buildings.geojson` in place of gate 4's fetch;
- gate 4's fetch checks are a recorded miss, with that reason. Those are the
  `fetch.log` report line and the second fetch;
- gate 4's file checks (bytes, SHA-256, the dataset counts) and gates 5–14 run on the copy.

- **Scope:**
  - **The fetch** (`scripts/fetch-buildings.sh`, §2.14.3): the `sources` column, the
    `DESCRIPTION` layer option, the two HTTP settings, and the report's `sources` and
    `no_sources`. `scripts/fixture.sh` does not change: the gate run re-fetches with
    `REFETCH=1`.
  - **The line (`src/credit.rs`, new; no Bevy types).**
    - `credit::line(network, project_dir, buildings: Option<&Path>) -> Result<Option<String>>`:
      §2.14.2's rule, reading `import_report.json` and the buildings file's two members
      with every check of §2.14.7 in order. `None` without `map_origin`, with nothing read.
    - The dataset table as a constant, in its order.
    - `font_size(width, height)`, `margin(size)` and `outline_offset(size)` (§2.14.5); `FILL`
      and `OUTLINE` as sRGB constants.
    - `fit_size(size, width_px, avail_px) -> Result<u32>`: one pass of §2.14.5's shrink,
      with its 10 px floor. `avail_px` is `W − 2·margin(S)` for the first `S`.
  - **Drawing (`src/draw.rs`, `src/render.rs`).**
    - `draw::spawn_credit(world, line, camera: Entity, font: Handle<Font>, size, margin,
      offset) -> CreditNodes` and `draw::set_credit_size(world, &CreditNodes, size)`:
      §2.14.5's tree. It has `UiTargetCamera`, the outline's eight copies as the fill's
      siblings at `ZIndex(-1)`, and `LineBreak::NoWrap` on all nine.
    - The font: `assets/fonts/FiraSans-Medium.ttf` and `assets/fonts/OFL.txt` (new; OQ-12),
      embedded with `include_bytes!` and added through `Font::from_bytes`.
    - `src/render.rs:Renderer` takes `credit: Option<&str>` in `new` and `new_perspective`.
      It keeps the image camera's entity on both paths and spawns the tree only with a
      line. After the settle frames it reads the fill's computed width and shrinks the
      line until it fits, running the three settle renders again after each shrink
      (§2.14.5), or fails (§2.14.7). Then it reads the credit box and keeps the size.
  - **`render` (`src/lib.rs`).** `Job::prepare_with` builds the line after the buildings
    file, and `Job::prepare_without_credit` does not (§2.14.7). `Job::credit()`,
    `Job::credit_size()` and `Job::credit_box()` are added. `src/main.rs`, `RenderOptions` and the progress lines do
    not change.
  - **`Cargo.toml`:** `license = "(MIT OR Apache-2.0) AND OFL-1.1"` (OQ-12,
    answered (b)). No dependency or feature changes.
  - **Not edited:** `src/buildings.rs`, every file under `src/view/`, `Cargo.lock`,
    `scripts/gates.sh`, `tests/gates.rs`, `tests/view.rs`, `tests/slider.rs`
    and `tests/camera.rs`.
  - **Tests.**
    - `tests/credit.rs` (new), `#[ignore]`d as `tests/buildings.rs` does:
      - gates 6 and 8, headless and not ignored;
      - gate 3's `Buildings` equality and gate 7, headless, ignored (they need the
        Midtown fixture);
      - gates 10 and 11, ignored (the GPU and the fixture). Gate 6's inputs are made up in the
      test: a network from `serde_yaml::from_str` as in Phase 1, and the reports and
      buildings files written into a temporary directory. None of it is map data (§2.10).
    - Two made-up buildings files, written from `tests/shapes.geojson` at test time and
      not committed. Each has the description `Overture Maps buildings, release test`:
      - *six sources*: Esri, Google, Microsoft, USGS and gate 6's `Alpha Survey` and `Zeta
        Lab`, on the six shapes (105.39 em; gate 11's shrink). `tests/credit.rs` writes it
        to a temporary directory;
      - *long*: ten made-up dataset names on one shape, `Made-up dataset name number 00
        for tests` to `… number 09 for tests` (40 characters each; 241.28 em; gate 9's
        error). `scripts/gates-credit.sh` writes it under `scratch/out/credit/`.
    - `tests/buildings.rs`: gate 11's test excludes the credit box, and gate 12's helper
      `shape_frames` builds its two jobs with `Job::prepare_without_credit` (§2.14.8). No
      other test there changes.
    - `scripts/gates-city.sh`: gate 5's report, bytes and SHA-256 take Phase 2's values
      (gate 4). Nothing else in it changes. The gate run runs it once, after gate 4 and
      without `REFETCH`, since gate 4 makes the second fetch. Expected: everything passes
      but Phase 1's two recorded misses, gate 3's FCD rows (280,872 against 280,875) and
      gate 13's flight pair (OQ-8).
    - `scripts/gates-credit.sh` (new) runs gate 4's checks after its first two steps,
      with its second fetch only with `REFETCH=1`. It also runs gate 5's strip-and-compare,
      gate 9's CLI cases and gate 14. Gate 4's first two steps, the copy and the
      re-fetch, are run by hand, once, in that order, and recorded. Gate 9's report cases run on a copy of
      `scratch/midtown` under `scratch/out/credit/` whose `import_report.json` is replaced.
      The fixture itself is never edited.
- **Exit gate.** On the development machine (Apple M3, macOS), on two fixtures:
  - vis-001's urban_grid (engine `df8aec0`, baseline, seed 42), for what must not change;
  - Midtown (`scratch/midtown`, Phase 1's fixture), its cache fetched again by gate 4.

  Only gate 4 uses the network. The predictions are drafting's measurements (§2.14.10) and
  Phase 1's recorded results. Nothing of this repo was built.
  - **What must not change:**
  1. **`render` of urban_grid.**
     - `scripts/gates.sh` passes, and its reference comparison gives **8700 of 8700**
       against `scratch/ref-8eb9052.framemd5` (decision 2).
     - The `--camera` path, too. Before the first code change, `scripts/gates.sh` runs at
       the base commit, and its `--camera tests/flight.toml` frames are kept as
       `scratch/ref-camera-<base>.framemd5`. After the change that render's `framemd5`
       equals it, **8700 of 8700**.
     - With `--include-ignored --test-threads=1`, `tests/gates.rs` (5), `tests/view.rs`
       (10), `tests/slider.rs` (8) and `tests/camera.rs` (19) pass, none of them edited.
  2. **Build cost** (§2.14.9).
     - `Cargo.lock` is unchanged: **0** packages. `git diff <base> -- Cargo.toml` shows
       only the `license` line.
     - A clean release build, counted from its `Compiling` lines in a throwaway
       `CARGO_TARGET_DIR` under `scratch/` (Phase 1's gate 2 method), compiles **363**
       crates.
     - `cargo tree -e normal` lists no HTTP client, as in Phase 1's gate 2.
     - The release binary's growth is recorded (about the font's 457,248 bytes).
  3. **`view`.**
     - `git diff <base> -- src/view/ src/buildings.rs` is empty.
     - `buildings::read` gives equal `Buildings` for Phase 1's cache and Phase 2's: the
       same ids in order, polygons, heights, rules and counts, compared exactly
       (`tests/credit.rs`, `PartialEq`). So
       `view --buildings` draws the same buildings from either file.
  - **The fetch and the cache (network, once):**
  4. **The re-fetch.**
     - Before it, Phase 1's cache is copied to `scratch/buildings-vis002p1.geojson`.
       Its SHA-256 is still `561a615d…` (Phase 1's gate 5).
     - Then `REFETCH=1 scripts/fixture.sh midtown`. Its report line in `fetch.log` gives
       release `2026-09-23.1`, Phase 1's gate 4 bbox, and **4,336** buildings: **4,277** by
       `height`, **8** by `num_floors`, **51** at the default, all as in Phase 1. Its
       `sources` are `{"Microsoft ML Buildings": 29, "OpenStreetMap": 4327, "USGS Lidar":
       316}`, and `no_sources` is **0**.
     - Over the file, 4,000 buildings name one dataset and 336 name two, which is 4,336.
     - The file is **2,284,830 bytes**, SHA-256
       `f241ccbd9a2fba2e3c80fa28ec7522ca2a33b84628c18954952ad6bc1d4a1dfd`, the probe's
       file byte for byte.
     - **This changes Phase 1's gate 5 prediction** (2,140,989 bytes, `561a615d…`). The
       change is recorded here, and Phase 1's text is not rewritten.
     - With `REFETCH=1`, a second fetch to another `--out` gives the same SHA-256.
     - The fetch time is recorded. Drafting's probe took 1,253 s, on a slow link.
  5. **Phase 1's content, unchanged.**
     - Take out of the new file its `"description"` line and, from each feature, the
       `, "sources": …` member. What is left is **byte-identical** to
       `buildings-vis002p1.geojson` (checked on the probe's file, §2.14.10).
     - `tests/buildings.rs`'s gates 8, 9 and 10, unedited, pass on the new cache with
       Phase 1's numbers: 4,277/8/51; 42,776 quads, 34,178 roof triangles, 0 and 0; 2.239 %.
  - **The line — headless, offline:**
  6. **The rule** (§2.14.2) on made-up inputs. Each case gives exactly this line, or one
     error naming its file and cause:

     | Network | `import_report.json` | Buildings file | Line or error |
     |---|---|---|---|
     | no `map_origin` | present, not JSON | — | none; nothing read |
     | `map_origin` | absent | — | `© OpenStreetMap contributors (ODbL)` |
     | `map_origin` | `Osm` | — | the same |
     | `map_origin` | `"source": null` | — | the same |
     | `map_origin` | `Overture`, `R1` | — | `… (ODbL) · Overture Maps Foundation, release R1` |
     | `map_origin` | `Overture`, `R1` | release `R2`, all `["OpenStreetMap"]` | `… · Overture Maps Foundation, release R2` |
     | `map_origin` | absent | release `R2`, all `null` or `[]` | the same |
     | `map_origin` | `Overture`, `R1` | `R2`; one building `["Esri Community Maps", "OpenStreetMap"]` | `… release R2 · Esri Community Maps contributors (CC BY 4.0)` |
     | `map_origin` | `Overture`, `R1` | `R2`; `Zeta Lab`, `USGS Lidar`, `Microsoft ML Buildings`, `Alpha Survey`, `Google Open Buildings`, `Esri Community Maps` on six buildings | `… release R2 · Esri Community Maps contributors (CC BY 4.0) · Google Open Buildings (CC BY 4.0) · Microsoft ML Buildings (ODbL) · USGS Lidar · Alpha Survey · Zeta Lab` |
     | `map_origin` | not JSON; `source` a string; `Overture` without `release`; `Here`; `Foo` | — | an `import_report.json` error, one per case |
     | `map_origin` | `Overture`, `R1` | no `description`; `description` `"buildings"`; `sources` `"OpenStreetMap"` (a string); `sources` `[1]` | a `--buildings` error, one per case; the last two name the feature |

  7. **Midtown's lines**, from the fixture's own files:
     - `credit::line` without buildings gives exactly `© OpenStreetMap contributors (ODbL)
       · Overture Maps Foundation, release 2026-09-23.1`;
     - with `scratch/midtown/buildings.geojson` (gate 4's) it gives exactly `©
       OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1 ·
       Microsoft ML Buildings (ODbL) · USGS Lidar`.
     - Both are complete for Midtown's roads. Its 222 mapped segments name OpenStreetMap
       only, with no TomTom (§2.14.4, checked 2026-10-02). So no TomTom item is missing.
  8. **The numbers.**
     - `font_size` gives 13, 20 and 40 at 1280×720, 1920×1080 and 3840×2160, and 11 at
       1080×1920.
     - `outline_offset` gives 1, 2 and 3 at sizes 13, 20 and 40, and `margin` the size.
     - The WCAG contrast ratio of `FILL` against `OUTLINE` is 18.4:1, at least 7:1.
     - `fit_size`:
       - (13, 785, 1254) gives 13: it fits;
       - (20, 1961, 1880) gives 19;
       - (20, 4826, 1880) is an error, at 7 px;
       - (40, 9651, 3760) gives 15;
       - (7, 300, 626), at 640×360, is an error at 7 px: `S` is under 10 before any shrink.
  9. **Errors through `render`.** Each exits 1 with one stderr line (`error: <project>/
     import_report.json: …` or `error: --buildings …` or `error: the credit line would be
     …`), with no progress line and no file at `--out`:
     - on the Midtown copy, its report replaced in turn by: not JSON; `source.type`
       `Here`; `Overture` without `release`;
     - Midtown with `--buildings scratch/buildings-vis002p1.geojson` (no release);
     - Midtown with the made-up *long* file at 1920×1080: the fit's error, its `S'` under
       10 and recorded. Prediction: **7**, or 8 with kerning (§2.14.5's table);
     - urban_grid with `--buildings tests/shapes.geojson` still gives Phase 1's
       `metadata.map_origin` error, before any credit check.
  - **Through the GPU, offline:**
  10. **Present, and confined to its box.** Midtown at 1920×1080, the orthographic job,
      without and with `--buildings`: each built with `Job::prepare_with` and with
      `Job::prepare_without_credit`, and compared at `render_empty()` and at three
      `render_at` times (the window's first, middle and last frame times).
      - Every pixel that differs between the two lies inside `Job::credit_box()`. Outside
        it, the frames are byte-identical. Prediction: **0** pixels outside.
      - Inside, more than 0 pixels differ (the count is recorded).
      - `Job::credit()` is gate 7's line for that job, without or with buildings. `main.rs`
        renders through the same `Job::prepare_with`, so this is the CLI's line.
      - The box's right edge `x1` is `W − m + o` and its bottom edge `y1` is `H − m + o`,
        both exclusive: 1902 and 1062 here. Its top `y0` is **1034 ± 1** (§2.14.5). The box
        is the same at all four times.
      - Once more through a one-keyframe `--camera` file, `keyframes = [ { t = 300.000, x =
        0.00, y = 0.00, height_m = 900.00, yaw_deg = 0.00, pitch_deg = 60.00 } ]`, with
        `--buildings`, at `render_empty()` only, with no boxes to stack (OQ-8): 0 pixels
        outside the box.
  11. **Legible at 1280×720 and 3840×2160** (orthographic Midtown with `--buildings`,
      `render_empty()`):
      - `Job::credit_size()` is 13 and 40;
      - the rows of the box holding a pixel that differs from the frame without the line
        span **16 ± 2 px** and **49 ± 2 px**. That is the glyphs' 1.070 em plus the outline
        (§2.14.5), so capitals are 9.0 and 27.6 px tall;
      - the box is 60.37 em wide, ± 3 % (the estimate has no kerning), plus 2o: 785 + 2 px
        and 2,415 + 6 px, within `W − 2m`;
      - inside the box, the lightest pixel has every channel ≥ 220 and the darkest every
        channel ≤ 20: the fill and the outline are both reached;
      - **the shrink:** with the made-up *six sources* file at 1920×1080, the line is
        laid out at **17 or 18 px** (`Job::credit_size()`). The fill's width is at most
        `W − 2m`, the fit's bound. Against the same job without the line at
        `render_empty()`, **0** pixels change outside the box, as in gate 10.
  12. **Phase 1's gate 11, again** (§2.14.8). At 3840×2160 with the line drawn, the union
      of both frames' credit boxes is excluded from both checks. Prediction: **0** and **0**,
      as in Phase 1. The excluded pixel count is recorded.
  13. **Phase 1's gate 12, again**, through `Job::prepare_without_credit`: roof > south
      wall > west wall, each neutral and not ground, the roof below 255; the courtyard and
      the notch as in Phase 1.
  14. **Deterministic.** `render --project scratch/midtown --scenario baseline --seed 42
      --from 300 --to 360 --speedup 1`, orthographic, twice with `--buildings
      scratch/midtown/buildings.geojson` and twice without. `ffprobe` gives
      `1920,1080,30/1,1800` for each, and each pair's `framemd5` is equal, **1800 of
      1800**.
  15. **Recorded, not gated:** the default Midtown render with `--buildings` (9,000 frames)
      is timed once by hand, beside Phase 1's 172.7 s.
  - **The user's check:**
  16. **The user watches** the Midtown renders: orthographic with `--buildings` at
      1920×1080 and at 1280×720, and gate 13's flight from Phase 1 with the line. The
      flicker of OQ-8 is expected. The user reads the line over dark ground, roads and
      roofs, and says whether the size, margin and outline should change (iteration,
      §2.14.5). The wording is settled (OQ-9).

      *(Passed by the user, 2026-10-03.)* The user watched
      `scratch/out/credit/ortho-1080-buildings.mp4`, `ortho-720-buildings.mp4` and
      `scratch/out/city/flight1.mp4`. The look stays as built: the ratio 1/54, the margin and
      the outline. The 13 px line at 1280×720 is accepted. The gate record is in
      `specs/reviews/vis-002.md`.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | `render` of urban_grid: default and `--camera` | 8700 of 8700 frames equal to each reference | 1 |
  | Packages added; `Cargo.toml` lines changed; release crates | 0; `license` only; 363 | 2 |
  | `view`'s and the reader's files; `Buildings` from both caches | unchanged; equal | 3 |
  | Re-fetched cache: buildings by height / num_floors / default | 4,336: 4,277 / 8 / 51 (Phase 1's) | 4 |
  | Its sources; buildings by dataset count | OpenStreetMap 4,327, USGS Lidar 316, Microsoft 29, none 0; 4,000 with one and 336 with two | 4 |
  | Its bytes; SHA-256 | 2,284,830; `f241ccbd…` (the probe's file) | 4 |
  | The new file without its two new members | byte-identical to Phase 1's | 5 |
  | The rule's made-up cases (11 rows, 18 cases) | each line or error as tabled | 6 |
  | Midtown's line without / with buildings | `© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1` / the same `· Microsoft ML Buildings (ODbL) · USGS Lidar` | 7 |
  | Font size at 720p, 1080p, 2160p, portrait 1080×1920; contrast; `fit_size` | 13, 20, 40, 11 px; 18.4:1; 13, 19, error (7), 15, error (7) | 8 |
  | Pixels changed outside the credit box | 0 | 10, 11 |
  | Box edges at 1920×1080, `x1` and `y1` exclusive | right 1902, bottom 1062, top 1034 ± 1 | 10 |
  | The *long* line at 1920×1080 | an error, `S'` 7 (or 8) | 9 |
  | Ink span at 720p / 2160p; box width; the six-source line at 1080p | 16 ± 2 / 49 ± 2 px; 60.37 em ± 3 % + 2o; 17 or 18 px | 11 |
  | Phase 1's gate 11, credit box excluded | 0 and 0 | 12 |
  | Orthographic Midtown, twice, with and without buildings | 1800 of 1800 | 14 |

- **Not predicted, and so not gated:**
  - the look: `S`'s ratio, the margin, the outline's width and the two colours (§2.14.5),
    all for the user at gate 16;
  - the count of pixels the line changes (gates 10–12), the binary's growth (gate 2), and
    wall times: the fetch (gate 4), the build (gate 2) and the renders (gate 15).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-2`), one push. The commits:
    - the fetch's two members, the report's two keys, the re-fetch, and gate 5's new values
      in `scripts/gates-city.sh` (gates 4 and 5);
    - `src/credit.rs` and `tests/credit.rs`'s line tests (gates 6–8; gate 7, ignored,
      needs the Midtown fixture);
    - the font, the drawing, the `Job` API, the test edits and `scripts/gates-credit.sh`
      (gates 1–3 and 9–14);
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - a new `rules/credit.md` (`max_lines: 40`): the rule and the table, its inputs and
      checks, the drawing numbers, the font and its licence, and `Job`'s three credit
      accessors. Its `sources` are `src/credit.rs`, `src/draw.rs`, `src/render.rs`,
      `src/lib.rs` and `scripts/fetch-buildings.sh`;
    - `rules/buildings.md` (58/60): the fetch's `sources` and `description`, and the
      report's two keys, reworded to fit its cap;
    - `rules/render.md` (60/60) and `rules/inputs.md` (50/50) are at their caps. Each gains
      a pointer to `rules/credit.md`, reworded to fit, and no `max_lines` is raised:
      `render.md` for the line in the scene; `inputs.md` for the two checks after
      `--buildings`' and the fit after the settle frames. `render.md`'s Pipeline line,
      which lists `Job`'s gate API, also names `prepare_without_credit`, `credit()`,
      `credit_size()` and `credit_box()`, or points to `rules/credit.md` for them;
    - `rules/view.md` and `rules/camera.md`: none needed, since neither `view` nor the
      camera changes;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - the README:
      - the credit section says that the line is drawn, from what, and that a cache fetched
        before Phase 2 must be fetched again (`REFETCH=1`) before `render --buildings`
        reads it;
      - for whoever publishes a video (OQ-13): put the full credit, OQ-7's text with
        `openstreetmap.org/copyright`, in its description;
      - the fetch report's two keys;
      - the font, its licence and the crate's `(MIT OR Apache-2.0) AND OFL-1.1` (OQ-12);
      - the new gate commands: `scripts/gates-credit.sh`, with `REFETCH=1`, and `cargo
        test --release --test credit -- --include-ignored --test-threads=1`;
    - `CLAUDE.md`: the licence line names the font's exception and the crate's
      `(MIT OR Apache-2.0) AND OFL-1.1` (OQ-12);
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and its
    cause.
  - Write this phase's `shipped` date.

### Phase 3 — See-through: buildings in the way cut to stubs, in a keyframed render and in view
*Produces the observable: yes. `render --buildings <file> --camera <file.toml>` writes
the run's video with every building between the camera and the point it looks at cut to
a 3 m stub, so the street there and its traffic show. With `--no-see-through`, and in
every render without buildings or without `--camera`, the video is byte-identical to
today's (gates 1 and 2).*

Drafted 2026-10-03; the design is §2.15, and the user's decisions are §2.15.1. Phase 3
builds on Phase 2 and on vis-001 Phase 6, the perspective boxes as one mesh. With
`--no-see-through` it changes neither's output, and with see-through on it changes no
box. The CLI below is
OQ-14's answer: on by default, `--no-see-through` to turn it off (§2.15.6).

- **Scope:**
  - **The rule (`src/see_through.rs`, new; no Bevy types).**
    - `WIDTH` = 0.15, `EASE_MIN_M` = 10.0 and `STUB_M` = 3.0 (§2.15.2, §2.15.3);
    - `Wedge { g, l, r, ease }` and `wedge(&Pose) -> Wedge`. `g` comes from
      `src/camera.rs:Pose::eye`, and `r` uses `src/camera.rs:cos_deg`;
    - `distance(&Wedge, &Building) -> f64`, which is `δ`: the smaller of the footprint's
      distance to the triangle and its distance to `l` less `r` (at least 0). It is 0
      when they meet, and holes count as outside;
    - `height(&Wedge, &Building) -> f64`, which is §2.15.3's `h′`. For `δ ≥ ease` it is
      the building's own `height`, the same `f64`;
    - `heights(&Buildings, &Pose) -> Vec<f64>`, in file order.
  - **Drawing (`src/draw.rs`).**
    - `BuildingsCut::new(&Buildings, (fx, fy))` keeps `buildings::mesh_data`'s
      positions in `f64`, and each vertex's building index.
    - `BuildingsCut::mesh(&self, heights) -> Mesh` gives every vertex `z′ = min(z,
      heights[b])`. It converts to `f32` and Bevy axes as `draw::buildings_mesh` does,
      with the same usages, normals and indices. With every building's own height, its
      mesh equals `buildings_mesh(&mesh_data(b), fx, fy)`, attribute for attribute.
    - `draw::set_building_heights(world, mesh: Entity, &BuildingsCut, heights)` puts that
      mesh in place of the asset behind the entity's `Mesh3d` handle, with
      `Assets::insert`, as `draw::set_boxes` does.
    - `spawn_buildings`, `buildings_mesh` and every box function are unchanged.
  - **`render` (`src/render.rs`, `src/lib.rs`, `src/main.rs`).**
    - `src/render.rs:Renderer::build` keeps `spawn_buildings`' mesh entity (an `Entity`;
      today it drops the returned `BuildingEntities`). It keeps no `Buildings`.
    - `Renderer::set_see_through(&mut self, cut: Option<&Buildings>)`:
      - `Some(b)` on a perspective renderer with buildings turns it on. It clones `b` and
        builds the `BuildingsCut` once. It has no effect on an orthographic renderer or
        one without buildings.
      - It takes effect at the next `set_pose`, which computes `see_through::heights` and
        replaces the mesh when any height differs from the last drawn. Nothing is drawn
        until then.
      - `None` turns it off. Off from the start, nothing is built, kept or replaced. Off
        after on, the next `set_pose` draws every building at its own height if the last
        drawn differ, then drops the clone and the `BuildingsCut`.
    - `Job::set_see_through(&mut self, on: bool)` passes the job's own `Buildings` when
      `on`, else `None`. It has no effect on a job without buildings or without a flight,
      as the flag has none (§2.15.6). `Job::render_at` already calls `set_pose` before
      each render, so the cut follows the frame's pose.
      `Job::prepare*` builds a job with see-through off, so every test that builds a job
      today draws as today.
    - `render --no-see-through` is a clap flag. `src/main.rs` calls
      `Job::prepare_with` as today, then `set_see_through(true)` unless the flag is
      given. `RenderOptions` and every `Job::prepare*` signature are unchanged.
  - **`view` (`src/view/state.rs`, `src/view/mod.rs`, `src/main.rs`).**
    - `view --no-see-through` is a clap flag. `ViewOptions` gains `see_through`, true
      unless the flag is given.
    - `src/view/state.rs:Pressed` gains `x`. `src/view/state.rs:ViewState` gains
      `see_through`. It is false at `new`, so every test's state is unchanged, and
      `src/view/mod.rs:run` sets it from `ViewOptions` at launch. `x` flips it at any
      moment, as `b` flips `buildings_shown`, and changes nothing else.
    - With `--buildings`, `src/view/mod.rs:Viewer` keeps the `Buildings` (today `run`
      drops them after `spawn_buildings`), the last drawn heights, and a `BuildingsCut`
      built the first time `see_through` is on. So `view --no-see-through` builds none
      until `X`.
    - The window reads `KeyCode::KeyX`. With buildings, after the camera is set each
      frame, the wanted heights are `see_through::heights` at the state's pose when
      `see_through` and `buildings_shown` are both on, else each building's own. The mesh
      is replaced only when they differ from the last drawn, and on every frame under
      `--bench` with see-through on.
    - Nothing else in the frame order changes.
  - **Tests.**
    - `tests/see_through.rs` (new):
      - headless, not ignored: gates 5 and 11;
      - headless but needing the Midtown fixture, so ignored: gate 9;
      - through the GPU, ignored: gate 6 (no fixture) and gate 8 (Midtown).
    - `tests/see-through-flight.toml` (new), the probe's orbit (§2.15.9):
      ```toml
      keyframes = [
        { t = 300.000, x = -675.00, y = 375.00, height_m = 250.00, yaw_deg = 0.00, pitch_deg = 35.00 },
        { t = 320.000, x = -675.00, y = 375.00, height_m = 250.00, yaw_deg = 120.00, pitch_deg = 35.00 },
        { t = 340.000, x = -675.00, y = 375.00, height_m = 250.00, yaw_deg = 240.00, pitch_deg = 35.00 },
        { t = 360.000, x = -675.00, y = 375.00, height_m = 250.00, yaw_deg = 0.00, pitch_deg = 35.00 },
      ]
      ```
    - `scripts/gates-see-through.sh` (new, offline): gates 2, 7 and 10's CLI cases, and
      the renders for gate 14. It uses the Midtown fixture and, for gate 10, urban_grid,
      and writes to `scratch/out/see-through/`.
    - **Not edited:**
      - the test files `tests/gates.rs`, `view.rs`, `slider.rs`, `camera.rs`,
        `buildings.rs`, `credit.rs` and `ties.rs`;
      - every existing script;
      - `src/buildings.rs`, `src/credit.rs`, `src/camera.rs` and `src/keyframes.rs`;
      - `Cargo.toml` and `Cargo.lock`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid (engine `df8aec0`, baseline, seed 42) and on Midtown (`scratch/midtown`
  with its Phase 2 cache). Everything runs offline. The predictions come from §2.15.9's
  probe, which rendered through a byte-identical copy of the shipped renderer.

  **Baseline, before any change**, at `origin/main` (`2b25d5e`):
  - **The four Midtown references were copied while drafting.** They come from
    `scratch/out/ties/`, written by vis-001 Phase 6's gate run at `ef741d8`, whose `src/`
    is `2b25d5e`'s. They are `scratch/ref-ties-{ortho-city,ortho-roads,flight-city1,
    flight-roads1}-2b25d5e.framemd5`, with SHA-256 `3d516485…`, `1442054c…`,
    `d6ff4845…` and `ab292818…`. The two orthographic ones equal
    `ref-ortho-{city,roads}-92d09a0`. If one is missing or differs, run
    `scripts/gates-ties.sh` at `2b25d5e` and copy its outputs.
  - **Run** `scripts/gates.sh`, `scripts/gates-ties.sh`, `scripts/gates-credit.sh`,
    `scripts/gates-city.sh` and every test file with `--include-ignored
    --test-threads=1`, and keep their output.
    - The default render must give 8700 of 8700 against `ref-8eb9052`. The `--camera`
      render must give 8700 of 8700 against `ref-camera-ef741d8`, compared by hand on
      `scratch/out/camera.framemd5` as vis-001 Phase 6 did. Otherwise the run stops.
    - Record spec-lint and `Cargo.lock`'s package count.

  - **What must not change:**
  1. **urban_grid.** `scripts/gates.sh` passes. Its default render gives **8700 of 8700**
     against `ref-8eb9052`, and its `--camera` render **8700 of 8700** against
     `ref-camera-ef741d8`, compared by hand. `--test gates` passes 5 of 5.
  2. **Midtown with `--no-see-through`, and where nothing is cut.**
     `scripts/gates-see-through.sh` renders `--from 300 --to 360 --speedup 1` seven
     times:
     - **with `--no-see-through`**: orthographic with and without `--buildings`, and
       `--camera tests/city-flight.toml` with and without `--buildings`;
     - **without the flag, where see-through cannot act**: orthographic with and without
       `--buildings`, and `--camera tests/city-flight.toml` without `--buildings`.

     Each `framemd5` against its `ref-ties-…-2b25d5e` copy gives **1800 of 1800**: seven
     times. The orthographic render with `--buildings` goes against `ortho-city`, without
     against `ortho-roads`; the `--camera` render with `--buildings` against
     `flight-city1`, without against `flight-roads1`. These are CLI renders, so the credit
     line is in them. The three renders with the flag and nothing to cut also cover gate
     10's "accepted, with no effect".
  3. **The shared draw path's tests and scripts.**
     - Re-run with `--include-ignored --test-threads=1`: `--test view` 10 of 10,
       `--test slider` 8 of 8 and `--test camera` 19 of 19, and `--test buildings`,
       `--test credit` and `--test ties`. Every printed number equals the baseline's, and
       no file is edited. These build their jobs through `Job::prepare*`, which leave
       see-through off.
     - `scripts/gates-ties.sh`, `scripts/gates-credit.sh` and `scripts/gates-city.sh`
       run unedited. Their Midtown flights with `--buildings` now render cut by default
       (§2.15.6), and they compare those runs only with each other: gates-ties' gate 9
       gives **1800 of 1800** for each of its two pairs (city and roads), and
       gates-city's gate 13 gives **1800** for run 2 and for run 3 (sandboxed) against
       run 1. gates-ties and gates-credit pass. gates-city's only `FAIL` lines stay gate
       3's FCD comparison, OQ-4's recorded miss, so it exits non-zero as at baseline.
  4. **Build cost** (§2.15.8). `git diff 2b25d5e -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The rule — headless, offline:**
  5. **The wedge and the heights** (`tests/see_through.rs`). The buildings are made up in
     the test through `Building`'s fields, so no file is read:
     - **straight down** (`L` (0, 0), `height_m` 100, pitch 90): `r` is exactly 0 and `g`
       is exactly `l`. Each block here is a 20 m square, 30 m tall. One containing `L`
       gives **3**. One whose nearest edge is 5 m from `L` gives **16.5** (`3 +
       27·smoothstep(0.5)`). One whose nearest edge is 10 m or more away gives **its own
       height**, compared with `==`;
     - **gate 6's pose** (`L` (0, 0), `height_m` 60, yaw 0, pitch 35): `r` = 0.15 · 60 ·
       cos 35° (7.372 m), the ease band is 10, and `g` is `(0, −59.328…)`, equal to
       `Pose::eye`'s. Gate 6's two blocks give **[3, 20]**, the second compared with `==`;
     - a 2 m building inside the wedge gives **2**;
     - **a courtyard, straight down** (the first bullet's pose, where `W` is the point
       `L`): a 60 m square centred on `L`, 30 m tall, with a 16 m square hole centred on
       `L`. `L` lies in the hole, so the footprint does not contain it: `δ` is **8** (the
       hole's walls), not 0, and the height is **27.192** (`3 + 27·smoothstep(0.8)`),
       within 1e-9. At a tilted pose the wedge's triangle would cross the walls and give
       `δ` = 0, so this case is straight down only.
  - **The cut — through the GPU:**
  6. **The synthetic scene** (`tests/see_through.rs`, ignored, no fixture). It is the
     probe's, written out here because the probe is not committed:
     - one road strip from (−60, 0) to (60, 0), 7 m wide, sampled every 1 m;
     - two blocks: *in the way*, x −10…10, y −45…−25, `height` 30; and *off*, x 25…40,
       y −40…−25, `height` 20;
     - one box, vehicle 7 at (0, 0), heading 90°, 4.5 m long, 10 m/s;
     - `scene::Camera { cx: 0, cy: 0, k: 1 }` at 1280×720, and gate 5's second pose
       (`height_m` 60, yaw 0, pitch 35);
     - the scene goes through `Renderer::new_perspective` with a pool of 4 and no credit
       (`None`), as `tests/ties.rs` does. See-through is turned on with
       `set_see_through(Some(&buildings))`, then `set_pose(&pose)`. A box's pixels are
       those that differ between the frame with the box and `render(&[])`.

     The predictions:
     - see-through off: the box's pixels are **0**, because the block hides it;
     - on: **1,564**, the same count as the scene with no buildings;
     - on, against a scene built with the in-the-way block at 3 m: **0 pixels** apart;
     - every pixel that differs between on and off, both rendered with `render(&[])`,
       lies inside the in-the-way block's image rectangle (its eight corners projected,
       ±2 px): **0** outside.

     A cut that does nothing fails the second and third, so the test fails without the cut.
  7. **Midtown, deterministic.** `scripts/gates-see-through.sh` renders `--buildings`
     with see-through on (the default) twice with `--camera tests/city-flight.toml`, and
     twice with `--camera tests/see-through-flight.toml`, each `--from 300 --to 360
     --speedup 1`. Each is `1920,1080,30/1,1800`
     (`ffprobe`), and each pair is **1800 of 1800**. It also renders the orbit once with
     `--no-see-through`, for gate 14.
  8. **Midtown, the traffic shows** (`tests/see_through.rs`, ignored).
     - Each flight goes through `Job::prepare_without_credit` with `--buildings`, and
       `RenderOptions` of `--from 300 --to 360 --speedup 1` at 1920×1080 and 30 fps (not
       `tests/buildings.rs:midtown_options`' 300–301), once with see-through off and once
       with `set_see_through(true)`: two jobs. Every 15th frame (120) is rendered with
       `render_frame`, then `render_empty` at the same pose and heights.
     - Box pixels are counted in the centre (the middle quarter of the width and of the
       height), the middle (the middle half of each) and the whole frame.
     - The probe's counts, off → on:
       - orbit: **59,506 → 159,502**, **91,305 → 258,570** and **168,189 → 387,849**;
       - city flight: **3,928 → 4,719**, **42,502 → 45,322** and **90,710 → 99,748**.
     - Each count is predicted within 1 % of the probe's, since rounding in `δ` may move
       an edge sample. The orbit's centre on is at least 2.5 times its centre off.
  9. **Untouched buildings are the shipped buildings** (`tests/see_through.rs`, ignored,
     headless).
     - No GPU and no `Job`: as the probe did, the test reads the run with `run::load`
       (`scratch/midtown`, baseline, seed 42, 300–360 s), the flight with
       `keyframes::read` and `Flight::new`, and the buildings with `buildings::read`.
       Frame `n`'s pose is `flight.pose_at(clock::frame_time(300, n, 1, 30), …)`, and
       `(fx, fy)` is `scene::Camera::fit(&run.strips, 1920, 1080)`'s centre.
     - At every 30th frame (60) of both flights, every building with `δ ≥ e` keeps
       `h′ == h`. Its vertices in `BuildingsCut::mesh` equal, bit for bit, the same
       vertex range of `draw::buildings_mesh(&mesh_data(&buildings), fx, fy)`. A
       building's range follows from the lengths of `building_mesh` for the buildings
       before it, since `mesh_data` appends them in file order. With every height its
       own, the two meshes are equal attribute for attribute.
     - Over all 1800 frames of each flight, no building **in frame** changes its height
       by more than **10 m** between two consecutive frames. The change from frame
       `n − 1` to `n` counts when the building is in frame at frame `n`. The probe's largest such
       changes are 9.39 m (city) and 3.16 m (orbit), and both are recorded.
       - A building's centroid is the mean of the x and y of every vertex of every
         polygon's exterior ring, as `Polygon` stores them.
       - It is in frame at frame `n` when the point (centroid, z 0) lies more than
         0.1 m in front of the eye along `Pose::axes`' forward axis, and
         `camera::project` at that pose, 1920×1080, gives `0 ≤ px < 1920` and
         `0 ≤ py < 1080`. This is the probe's `geom::pixel` and `region`.
         `camera::project` alone has no depth test, so the test adds it.
     - The buildings lowered per frame are recorded. The probe gave a mean of 33.3
       (city) and 33.4 (orbit).
  10. **The flag's edges** (§2.15.6):
      - `render --see-through` with every argument a Midtown flight render needs
        (`--project`, `--scenario`, `--seed`, `--buildings`, `--camera`, `--out`) is a
        clap usage error, **exit 2**. The flag is `--no-see-through`, and the drafted
        opt-in flag does not exist;
      - on urban_grid, `render --buildings tests/shapes.geojson --camera
        tests/flight.toml` gives Phase 1's `metadata.map_origin` error, exit 1, with and
        without `--no-see-through`;
      - `view --no-see-through --bench 1` on urban_grid, with no buildings, exits 0 and
        prints its JSON line. The flag is accepted with no effect;
      - `render --no-see-through` without `--buildings` or without `--camera` is accepted
        and changes nothing, by gate 2's renders.
  11. **`X`, headless** (`tests/see_through.rs`), as Phase 1's gate 14 tests `B`:
      - on the plain state of vis-001 Phase 3, `x` flips `see_through` from false to
        true, and a second `x` flips it back;
      - `t`, the clock, the pose, `k`, `follow` and `buildings_shown` stay exactly as they
        were;
      - during a right-button orbit, a left drag and a scrub, `x` flips the flag and the
        gesture goes on unchanged.
  - **Recorded, with one bar:**
  12. **Render time.** Record the wall time of gate 7's see-through flights beside gate
      2's flights with `--no-see-through`. The probe added 3.1–4.1 ms a frame, about 5.5–7.4 s
      over 1800 frames.
  13. **`view --bench 20`** on Midtown at the default window, with `--buildings`
      (see-through on by default, rebuilding every frame, §2.15.6) and with `--buildings
      --no-see-through`. Record its JSON and the load average.
      - **A `mean_fps` below 30 with see-through on stops the build.** §2.15.5's
        fallback is then a scope change, and the phase goes back to review.
      - Prediction: at least 30, and likely 60 (vsync). Midtown with buildings gave 60.00
        fps at Phase 1, and the rebuild adds 3.1–4.1 ms to a 16.7 ms frame.
  - **The user's check:**
  14. **The user watches** gate 7's orbit, `see-through-orbit-on1.mp4` against
      `see-through-orbit-off.mp4` (`--no-see-through`), and its city flight
      `see-through-city-on1.mp4` against gate 2's flight with buildings and
      `--no-see-through`. Times are the video's, from 0:00.
      - **Orbit, from 0:26 to 0:32, 0:42 to 0:44 and 0:50 to 0:56, the middle of the
        frame.**
        - Off, towers fill it and no box shows there. In the probe, every box in the
          centre was hidden at each of those whole seconds (`geom4-orbit.txt`); at 0:33,
          0:45 and 0:57 a few already show.
        - On, the buildings between the camera and the middle are flat, light slabs a few
          metres tall. The crossing in the middle shows its roads and moving boxes.
        - Buildings away from the line to the middle stand at full height.
      - **Orbit, throughout.** As the camera circles, buildings sink as the line to the
        middle reaches them and rise once it has passed, over a second or more. None
        jumps between full height and a slab from one frame to the next. The last frame
        (t 359.967 s) is one frame short of the full turn that ends at 360 s, so the view
        and every building there are as at 0:00, to within one frame's motion.
      - **City flight.** From 0:00 to 0:10 the camera is high and nearly straight down,
        and only buildings near the middle of the frame change. In the probe's first
        frame, a tower of about 435 m standing within 10 m of the look-at point is
        lowered (the record's Phase 3 draft). Around 0:30, the towers in the lower
        middle of the frame, between the camera and the park's edge, are slabs, and
        streets show there.
      - **In `view`** with `--buildings`, see-through on from launch:
        - tilt to 30–40° over a busy street: the blocks between the camera and the
          centre are slabs;
        - orbit with a right-drag: they sink and rise smoothly;
        - `X` restores them to full height, `X` again cuts them, and `B` still hides
          every building;
        - `view --no-see-through` opens with every building at full height, and `X` cuts
          them.

      Then say whether `WIDTH`, the ease band or the stub should change (iteration,
      §2.15.2–§2.15.3), and answer OQ-15.

      *(Passed by the user, 2026-10-03.)* The user watched
      `scratch/out/see-through/see-through-orbit-{off,on1}.mp4` and
      `see-through-city-{off,on1}.mp4`. The look stays as built: `WIDTH` 0.15, the ease
      band `max(r, 10 m)` and the 3 m stub. OQ-15 is answered (a). The gate record is in
      `specs/reviews/vis-002.md`.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | urban_grid: default and `--camera` | 8700 of 8700 against `ref-8eb9052` and `ref-camera-ef741d8` | 1 |
  | Midtown with `--no-see-through`: ortho and flight, each with and without buildings; without the flag: both orthos and the flight without buildings | 1800 of 1800 against each `ref-ties-…-2b25d5e`, seven renders | 2 |
  | gates-ties' flight pair; gates-city's gate 13, now cut by default | 1800 of 1800; 1800 and 1800 | 3 |
  | Packages; `Cargo.toml` | 0; unchanged | 4 |
  | Heights straight down at 0, 5 and ≥ 10 m from `L`; gate 6's blocks | 3; 16.5; own; [3, 20] | 5 |
  | Synthetic box pixels off / on / no buildings; on against the stubbed file; changes outside the block | 0 / 1,564 / 1,564; 0; 0 | 6 |
  | Midtown renders with see-through on (the default), twice each | 1800 of 1800 | 7 |
  | Box pixels, orbit centre / middle / frame, off → on | 59,506 → 159,502 / 91,305 → 258,570 / 168,189 → 387,849, each ± 1 % | 8 |
  | Box pixels, city flight, off → on | 3,928 → 4,719 / 42,502 → 45,322 / 90,710 → 99,748, each ± 1 % | 8 |
  | Untouched buildings; largest change in frame between two frames | bit-identical vertices; ≤ 10 m (probe: 9.39 and 3.16 m) | 9 |
  | `--see-through`; urban_grid with buildings, with and without `--no-see-through`; `view --no-see-through --bench 1` without buildings | exit 2; the `map_origin` error, exit 1; exit 0 | 10 |
  | `view --bench` with see-through on (the default) | ≥ 30 fps (likely 60) | 13 |
- **Not predicted, and so not gated:**
  - the look: `WIDTH`, the ease band and the stub, for the user at gate 14;
  - render and build times (gate 12), and `view`'s frame rate above 30 (gate 13);
  - the number of buildings lowered (gate 9 records it).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-3`), one push. The commits:
    - `src/see_through.rs` and its headless tests (gates 5 and 9's rule half);
    - the drawing, `render`, `view`, the CLI, `tests/see-through-flight.toml`, the GPU
      tests and `scripts/gates-see-through.sh` (gates 1–4, 6–8 and 10–13). The default
      changes here, so gate 2 and gate 3's scripts run after this commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **a new `rules/see-through.md`** (`max_lines: 40`) covering the wedge, the heights
      and their constants, the drawing, the default and `--no-see-through`, and `X`. Its
      `sources` are
      `src/see_through.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`,
      `src/view/state.rs`, `src/view/mod.rs` and `src/main.rs`;
    - **three rules at their caps gain a pointer to it**, reworded to fit, with no
      `max_lines` raised: `rules/render.md` (60/60), whose CLI line gains
      `[--no-see-through]`; `rules/view.md` (60/60), with the CLI and `X`; and
      `rules/buildings.md` (60/60);
    - `rules/camera.md` and `rules/credit.md`: none needed, since neither the camera nor
      the line changes;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README** says that a flight with `--buildings` is cut by default, and gains
      `--no-see-through`, `X` and the new gate commands:
      `scripts/gates-see-through.sh` and `cargo test --release --test see_through --
      --include-ignored --test-threads=1`;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 4 — Building parts: towers on podiums and raised bases, from Overture's parts
*Produces the observable: yes. `render --buildings <file>`, with a cache fetched in Phase
4's form, writes the run's video with each building that has parts drawn from them: a
tower on its podium, an upper storey over a lower one, a raised base above the street.
With today's cache every frame is byte-identical to today's (gate 2).*

Drafted 2026-10-03; the design is §2.16, and the user's decisions are §2.16.1 (§2.15.1
decision 4). It builds on Phase 3 and changes nothing a cache without parts draws: not the
mesh, the cut, the credit or `view`. The plan: one cache file with the parts after the
buildings and `--source` (§2.16.3, decision 4), the parts only (decision 1, OQ-16),
§2.4.2's rule for tops and the same pattern for bases (§2.16.5), no underside (§2.16.6),
the lid for a part above the cut (decision 2, OQ-17), and no new credit item (§2.16.8).

**No deadline.** The data was saved while drafting (§2.16.1). Every gate reads it through
`--source`, offline, after `2026-09-23.1` has left S3 too.

- **Scope:**
  - **The fetch (`scripts/fetch-buildings.sh`), §2.16.3.**
    - The query: the buildings with `min_height` and `min_floor` and a `null`
      `building_id`, then their parts by `building_id` over the buildings' own extent,
      ordered by `building_id` (nulls first) and `id`; two DuckDB runs, the buildings
      handed over as `$TMP/b.parquet` (§2.16.3). A failed run is today's error, `release
      <r>: …`, with nothing at `--out`.
    - `--source <dir>`, a local directory; not given, S3 as today. A `--source` that is not
      a directory is an error, `error: --source: no directory <dir>`, with nothing at
      `--out`.
    - `--no-parts` (§2.16.1 decision 5): Phase 2's query and report, unchanged; it combines
      with `--source`.
    - The report's `raised` and `parts` (§2.16.3); every other key as today, counted over
      the features without a `building_id`.
  - **The fixture (`scripts/fixture.sh midtown`)**, §2.16.3. Step 5 passes `--no-parts`,
    and `--source scratch/overture-2026-09-23.1/theme=buildings` when that directory
    exists. Step 6 (new) writes `scratch/midtown/buildings-parts.geojson` only if it is
    absent or `REFETCH=1`, with the same `--source` rule; its report line goes to
    `scratch/midtown/fetch-parts.log`. Step 2's `find` also keeps
    `buildings-parts.geojson` and `fetch-parts.log`, so `FORCE=1` keeps them. The header
    comment says so. Nothing else changes.
  - **The Phase 1 and 2 network gates.** The second fetch in the `REFETCH=1` leg of
    `scripts/gates-city.sh` and of `scripts/gates-credit.sh` gains `--no-parts`: one
    changed line each, and their header comments unchanged.
  - **The mirror, once, by hand** (a build step, not a gate): in
    `scratch/overture-2026-09-23.1/`, `theme=buildings/type=building/` gets a copy of
    `building.parquet`, and `theme=buildings/type=building_part/` a copy of
    `building_part-of-buildings.parquet`. Their SHA-256 must be §2.16.10's.
  - **Reading and the mesh (`src/buildings.rs`).**
    - `Part { id, polygons, base, height, rule }`. `Building` gains `base: f64` (0 without
      a base) and `parts: Vec<Part>` (in file order), and `Building::top()`: the highest
      part's `height`, else the building's own `height` (§2.16.5).
    - `read` reads `building_id`, `min_height` and `min_floor`, runs §2.16.5's checks in
      its order, attaches each part to its building, and sets `Buildings::tallest` to the
      highest `top()`. `Counts` stays the buildings' height rules.
    - `building_mesh(b)`: without parts, today's walls with their lower edge at `b.base`
      and its roof at `b.height`; with parts, each part the same way, from its base to its
      `height`, in order: each part's walls, then its roof. No floor and no underside
      (§2.16.6). With base 0 and no parts it is today's `building_mesh`, vertex for vertex.
      `MeshData`'s doc comment, which says one building's walls come first, says this.
    - `mesh_data`, `lnglat_to_xy` and the rest are unchanged.
  - **See-through (`src/see_through.rs`).** `height` and `heights` take `b.top()` for `h`
    (§2.16.7). `wedge` and `distance` are unchanged.
  - **`render` and `view`.** `src/render.rs:Renderer::set_see_through` and
    `src/render.rs:Renderer::set_pose`, and `src/view/mod.rs`, take each building's
    `top()` where they take its `height` today, as the heights drawn without a cut.
    Nothing else changes: `draw::BuildingsCut`, `draw::spawn_buildings`, the credit, the
    CLI and every key.
  - **Tests.**
    - `tests/parts.rs` (new): gates 6, 7, 8, 10, 11's test half and 12; the Midtown and
      GPU ones ignored, as in `tests/see_through.rs`.
    - `scripts/gates-parts.sh` (new, offline): gates 2, 5, 9 and 11's CLI half, and the
      renders for gate 15, into `scratch/out/parts/`.
    - **Edited:** `tests/see_through.rs`, only to add `base: 0.0,` and `parts: vec![],` to
      its two `Building` literals (`rect` and gate 5's courtyard), since `Building` gains
      two fields: four added lines. Gate 3 checks that the diff is those four lines and
      that every printed number is the baseline's.
    - **Not edited:** every other test file, every other script (apart from the one line
      each in `gates-city.sh` and `gates-credit.sh`, above), `src/draw.rs`,
      `src/credit.rs`, `src/camera.rs`, `src/keyframes.rs`, `src/main.rs`, `Cargo.toml`
      and `Cargo.lock`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid (engine `df8aec0`, baseline, seed 42) and on Midtown (`scratch/midtown`).
  Everything runs offline. The predictions come from §2.16.10's data and probe, which
  rendered through a byte-identical copy of the shipped renderer.

  **Baseline, before any change**, at `origin/main` (its `src/`, `tests/`, `scripts/`,
  `assets/`, `Cargo.toml` and `Cargo.lock` must equal `fb53d7f`'s):
  - **The five Midtown references were copied while drafting**, from
    `scratch/out/see-through/`, which Phase 3's gate run wrote at `3e7e660`; `git diff
    3e7e660 fb53d7f` touches none of those paths. They are
    `scratch/ref-p4-<name>-fb53d7f.framemd5`, all `--from 300 --to 360 --speedup 1` with
    `--buildings scratch/midtown/buildings.geojson`:

    | `<name>` | The render | SHA-256 |
    |---|---|---|
    | `ortho-city` | orthographic | `3d516485…` (= `ref-ties-ortho-city-2b25d5e`) |
    | `city-on` | `--camera tests/city-flight.toml`, cut (the default) | `d7e8795b…` |
    | `city-off` | the same, `--no-see-through` | `d6ff4845…` (= `ref-ties-flight-city1-2b25d5e`) |
    | `orbit-on` | `--camera tests/see-through-flight.toml`, cut | `a905bea5…` |
    | `orbit-off` | the same, `--no-see-through` | `d2e4ec99…` |

    If one is missing or differs, run `scripts/gates-see-through.sh` at `fb53d7f` and copy
    its outputs (`default-ortho-city`, `see-through-{city,orbit}-{on1,off}`).
  - **Run** `scripts/gates.sh`, `scripts/gates-ties.sh`, `scripts/gates-credit.sh`,
    `scripts/gates-city.sh`, `scripts/gates-see-through.sh` and every test file with
    `--include-ignored --test-threads=1`, and keep their output.
    - `gates-see-through.sh`'s five renders above must give 1800 of 1800 against the
      copies, and `gates.sh`'s default render 8700 of 8700 against `ref-8eb9052`.
      Otherwise the run stops.
    - Record spec-lint and `Cargo.lock`'s package count (500).

  - **What must not change:**
  1. **urban_grid.** `scripts/gates.sh` passes. Its default render gives **8700 of 8700**
     against `ref-8eb9052`, and its `--camera` render **8700 of 8700** against
     `ref-camera-ef741d8`, compared by hand. `--test gates` passes 5 of 5.
  2. **Midtown with today's cache, byte-identical.** `scripts/gates-parts.sh` renders the
     five renders of the baseline table with `--buildings scratch/midtown/buildings.geojson`
     (today's form, §2.16.9), and each `framemd5` gives **1800 of 1800** against its
     `ref-p4-…-fb53d7f` copy: five times. These are CLI renders, credit line included.
  3. **The shared tests and scripts.**
     - Re-run with `--include-ignored --test-threads=1`: `--test view` 10 of 10, `slider`
       8 of 8, `camera` 19 of 19, `buildings` 13 of 13, `credit` 6 of 6, `ties` 2 of 2 and
       `see_through` 7 of 7, and `--lib` 4 of 4 (one, `place::turn_point_wiring`, is
       `#[ignore]`d without `--include-ignored`), as at baseline. Every printed
       number equals the baseline's, apart from build lines and times.
     - `git diff <baseline> -- tests/` shows only the four added lines of
       `tests/see_through.rs` named in the scope, and the new `tests/parts.rs`.
       `git diff <baseline> -- scripts/` shows `fetch-buildings.sh` and `fixture.sh`, the
       new `gates-parts.sh`, and in `gates-city.sh` and `gates-credit.sh` only the
       `--no-parts` of their second fetch.
     - `scripts/gates-ties.sh`, `gates-credit.sh`, `gates-city.sh` and
       `gates-see-through.sh` run on today's cache, offline, and print what they printed at
       baseline, apart from times. gates-city's only `FAIL` lines stay gate 3's FCD
       comparison (OQ-4).
  4. **Build cost** (§2.16.11). `git diff <baseline> -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The fetch and the cache — offline:**
  5. **The fetch from the saved data** (`scripts/gates-parts.sh`). With the mirror laid out
     (scope), `scripts/fetch-buildings.sh --project scratch/midtown --out <dir>/a.geojson
     --source scratch/overture-2026-09-23.1/theme=buildings`, twice, into two directories:
     - each report line is, apart from `seconds` and `out`:
       `{"release": "2026-09-23.1", "bbox": [-73.9938413, 40.7544211, -73.9643086,
       40.7740495], "buildings": 4336, "height": 4277, "num_floors": 8, "default": 51,
       "sources": {"Microsoft ML Buildings": 29, "OpenStreetMap": 4327, "USGS Lidar": 316},
       "no_sources": 0, "raised": 3, "parts": {"count": 4209, "buildings": 487, "height":
       4152, "num_floors": 17, "default": 40, "raised": 695, "sources": {"OpenStreetMap":
       4209}, "no_sources": 0}, "bytes": 4944032}`;
     - each file is **4,944,032 bytes**, SHA-256
       **`714e2f3a3ab612618100b82b698cc08d49a9badb96caa7afc31e8dbbdf159b14`**;
     - its first 4,336 features, with `"building_id": null, ` and `, "min_height": …,
       "min_floor": …` taken out and the last one's trailing comma dropped, are today's
       cache's feature lines byte for byte, and the header lines too;
     - `scratch/midtown/buildings-parts.geojson`, from the fixture's step 6, has the same
       SHA-256;
     - `--source scratch/no-such-dir` is one error line, a non-zero exit and nothing at
       `--out`;
     - **today's form, offline:** the same command with `--no-parts`, twice, gives
       **2,284,830 bytes, SHA-256 `f241ccbd…`**, today's cache, and the report line of
       `scratch/midtown/fetch.log` apart from `seconds` and `out` (no `raised`, no
       `parts`); `scratch/midtown/buildings.geojson`, after `REFETCH=1 scripts/fixture.sh
       midtown` with the mirror laid out, still has that SHA-256. That fixture run comes
       last in gate 5, after its reads of `fetch.log`; it rewrites `fetch.log` and
       `fetch-parts.log`, whose lines change only in `seconds`.

     While S3 still lists `2026-09-23.1` (until about 2026-11-22), the same command
     without `--source` is run once and its SHA-256 recorded. It is predicted equal, and
     it is not a gate, since it needs the network. So is `REFETCH=1
     scripts/gates-city.sh` and `REFETCH=1 scripts/gates-credit.sh`, whose second fetch
     with `--no-parts` is predicted `f241ccbd…`, as at Phases 1 and 2.
  6. **Reading and the mesh** (`tests/parts.rs`).
     - **Midtown with parts** (ignored, headless): `buildings::read` of
       `buildings-parts.geojson` gives **4,336 buildings, 487 with parts, 4,209 parts**.
       The parts' tops: 4,152 by `height`, 17 by `num_floors`, 40 at 10 m. Their bases:
       695 above 0, all by `min_height`. Three buildings have a base: 7.5 m
       (`2bd09890-…`), 7.0 m (`c92d28b5-…`, by `min_floor`) and 3.7 m (`de4ad6d6-…`, which
       has parts). `tallest` is 472.0. (`Part` and `Building` keep a base's value, not its
       rule; the test reads which rule gave each base from the file's properties.)
     - Its `mesh_data`: **336,835 vertices, 558,147 indices, 67,367 wall quads and 51,315
       roof triangles**.
     - Every one of the 3,847 buildings with no parts and base 0 has the same
       `building_mesh`, position for position, as the same building read from today's
       cache.
     - **Today's cache** through the same reader: every building has base 0 and no parts,
       and `mesh_data` gives Phase 1's **213,880 vertices, 359,190 indices, 42,776 wall
       quads and 34,178 roof triangles**.
  7. **The reader's new checks** (headless, hand-written cases in the test, on
     `tests/buildings.rs`'s made-up network with Midtown's `map_origin`). Each is one
     `buildings::read` error naming the feature, and the first in file order wins:
     - `"building_id": 7`: `building_id 7 is not a string`;
     - `"min_height": -1` on a building without parts, and `"min_floor": 1.5`: `min_height
       -1 is not a finite number of at least 0` and `min_floor 1.5 is not an integer of at
       least 0`;
     - `"min_height": -1` on a building with parts: the same error (§2.16.5: its base is
       checked, though not drawn);
     - a building of `height` 20 with `min_height` 20: `base 20 m is not below its top 20
       m`; a part of `height` 10 with `min_height` 12, likewise;
     - a part whose `building_id` is `"nobody"`: `part of nobody, which is not a building
       of the file`, reported after every feature's own checks pass.

     Accepted: a part with `min_floor` 2 and no `min_height` stands from 7.0 m; a building
     with parts whose own `min_height` is above its own `height` reads, since it is not
     drawn; `tests/shapes.geojson` reads exactly as in Phase 1's gate 8.
  - **The drawing — through the GPU:**
  8. **The synthetic scenes** (`tests/parts.rs`, ignored, no fixture). Phase 3 gate 6's
     road strip, box (vehicle 7 at (0, 0), heading 90°, 4.5 m, 10 m/s), `scene::Camera
     { cx: 0, cy: 0, k: 1 }` at 1280×720 and pose (`height_m` 60, yaw 0, pitch 35),
     through `Renderer::new_perspective` with a pool of 4 and no credit. A box's pixels
     are those that differ between the frame with the box and `render(&[])`. The
     buildings are built in the test:
     - **A, a tower on a podium**, between the eye and the box: the building x −30…30,
       y −40…−20, `height` 80, with two parts: the podium, the same footprint, 0 to 12 m;
       and the tower, x 15…30, y −40…−30, from 12 to 80 m;
     - **A without parts**: the same building with `parts` empty, as today's cache draws
       it;
     - **B, a raised base** over the box: x −8…8, y −10…10, `height` 20, `base` 8, no parts;
       and **B on the ground**, the same with `base` 0.

     The predictions, from the probe:
     - no buildings: **1,564** box pixels (Phase 3 gate 6's count);
     - A without parts **0**; A **1,564**;
     - A with `set_see_through(Some(&a))`: **1,564**, and its `render(&[])` is **0
       pixels** apart from that of a single building, the podium's footprint at 3 m;
     - B on the ground **0**; B **1,008**; B cut: **0** (the lid at 3 m, OQ-17's (a));
     - A's mesh has 40 vertices and 60 indices, and its tower's wall bottoms are at z 12;
       B's has 20 and 30, with its wall bottoms at z 8.

     A reader or a mesh that ignores the parts or the base gives 0 for A and for B: the
     test fails without them.
  9. **Midtown with parts, deterministic.** `scripts/gates-parts.sh` renders, with
     `--buildings scratch/midtown/buildings-parts.geojson` and `--from 300 --to 360
     --speedup 1`: `--camera tests/city-flight.toml` twice and
     `--camera tests/see-through-flight.toml` twice, cut (the default); each `ffprobe`
     `1920,1080,30/1,1800`, and each pair **1800 of 1800**. It also renders, once each,
     both flights with `--no-see-through`, for gate 15. Each
     cut render's frames that differ from today's (`ref-p4-city-on`, `ref-p4-orbit-on`)
     are counted and recorded.
  10. **Continuity, and buildings the cut leaves alone** (`tests/parts.rs`, ignored,
      headless; Phase 3 gate 9's method with `buildings-parts.geojson`).
      - At every 30th frame of both flights, every building with `δ ≥ e` keeps `h′ ==
        top()`, and its vertices in `BuildingsCut::mesh` equal its range of
        `draw::buildings_mesh(&mesh_data(&b), fx, fy)`, bit for bit.
      - Over all 1800 frames of each flight, no building in frame changes its `h′` by
        more than **10 m** between consecutive frames. The probe's largest are **9.39 m**
        (city, as today) and **3.10 m** (orbit; 3.16 m today), and both are recorded.
      - The buildings lowered a frame are recorded. The probe gave 33.3 (city) and 33.4
        (orbit), as today.
  11. **The credit** (`tests/parts.rs`, ignored, and `scripts/gates-parts.sh`).
      `Job::prepare_with` on Midtown with `buildings-parts.geojson` and with
      `buildings.geojson` both give `credit()` exactly `© OpenStreetMap contributors
      (ODbL) · Overture Maps Foundation, release 2026-09-23.1 · Microsoft ML Buildings
      (ODbL) · USGS Lidar`. Gate 9's renders exit 0.
  12. **The traffic** (`tests/parts.rs`, ignored), Phase 3 gate 8's count with
      `buildings-parts.geojson`: box pixels in the centre, the middle and the frame,
      every 15th frame, see-through off and on. The probe's counts, each predicted within
      1 %, off → on:
      - orbit: **62,082 → 159,502**, **92,016 → 259,247** and **147,200 → 387,679**;
      - city flight: **4,033 → 4,886**, **55,729 → 57,610** and **111,115 → 123,130**.

      Beside Phase 3's gate 8 on today's cache, the city flight shows 23 % more box
      pixels over the frame with see-through on (99,748 → 123,130), and 22 % more off
      (90,710 → 111,115). The orbit with see-through on is within 0.3 % of today's in
      each region. Recorded, not gated.
  - **Recorded, with one bar:**
  13. **Render time.** Record the wall time of gate 9's renders beside gate 2's. The
      probe's interleaved cost (§2.16.11) adds 2.6–2.8 ms a frame with the mesh rebuilt,
      about 5 s over 1800 frames.
  14. **`view --bench 20`** on Midtown at the default window, with `--buildings
      scratch/midtown/buildings-parts.geojson` (see-through on, rebuilding every frame)
      and with `--no-see-through`. Record its JSON and the load average.
      - **A `mean_fps` below 30 with see-through on stops the build.** §2.15.5's
        fallback, the vertex shader, is then a scope change, and the phase goes back to
        review.
      - Prediction: at least 30, and likely 60 (vsync). Phase 3's bench gave 59.43 fps
        rebuilding every frame, and the parts add about 2.8 ms to a frame (§2.16.11).
  - **The user's check:**
  15. **The user watches** gate 9's renders against gate 2's, each pair the same flight
      with today's cache and with parts: the city flight cut (the default), the orbit
      cut, and the city flight with `--no-see-through`. Times are the video's, from 0:00.
      What the probe's frames showed (§2.16.10) is given with each.
      - **City flight, 0:00 to 0:10**, high and nearly straight down. The roofs of the
        tall buildings show steps, where today each is one flat top. Just above and left
        of the middle of the frame, a traffic circle shows its whole ring road, where today
        a flat block shaped like an X covers half of it. 12–16 % of the frame's pixels
        differ, most in its lower half.
      - **City flight, around 0:30.** Left of the middle, a little above it, the circle's
        ring road shows with the boxes queued on it; today the X-shaped block hides its
        west half and the queue. In the left and right thirds, towers stand narrower on
        lower podiums, where today broad slabs stand as tall as the towers. About a
        third of the frame differs.
      - **City flight, around 0:40.** The left third: a narrower tower with lower
        buildings beside it, where today one broad slab fills it. The circle's ring road
        shows at the lower left.
      - **City flight, from 0:50 to the end**, over the park: almost nothing changes
        (1–2 % of pixels, at the top left).
      - **The `--no-see-through` pair** shows the same changes, with every building at
        full height.
      - **Orbit, throughout.** The crossing in the middle of the frame and its traffic
        look as today: the middle ninth of the frame differs in at most 3 % of its pixels
        at every sampled second, and gate 12's centre count with see-through on is
        today's (159,502). The changes are in
        the top third and at the sides. At 0:00, behind the crossing at the top middle, a
        tower rises from a lower base where today a wide slab stands. Around 0:20, a
        tower about 230 m tall stands in the top left third, where today a block about
        45 m tall stands among taller ones.
      - **The cut, with parts.** As in Phase 3, the buildings between the camera and the
        middle sink to slabs and rise again over a second or more. A tower on a podium
        sinks to one flat slab: nothing of the tower hangs above it. Where a building
        overhangs a street and is cut, the overhang lies at the slab's height over the
        street (OQ-17's (a)).
      - **In `view`** with `--buildings scratch/midtown/buildings-parts.geojson`: tilt to
        30–40° over a block of towers; they stand on their podiums. `X` and `B` work as
        in Phase 3.

      Then say whether anything drawn from the parts should change. OQ-16 and OQ-17 were
      answered before review (§2.16.1); what (a) leaves on screen is the user's to judge
      here, and a change to either is a scope change that goes back to review.
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | urban_grid: default and `--camera` | 8700 of 8700 against `ref-8eb9052` and `ref-camera-ef741d8` | 1 |
  | Midtown with today's cache: ortho, both flights cut and `--no-see-through` | 1800 of 1800 against each `ref-p4-…-fb53d7f`, five renders | 2 |
  | Other test files and scripts | baseline numbers; `tests/` diff is four added lines and `tests/parts.rs` | 3 |
  | Packages; `Cargo.toml` | 0; unchanged | 4 |
  | The fetch from the saved data | the report line above; 4,944,032 bytes, `714e2f3a…`, twice | 5 |
  | Midtown with parts: buildings, with parts, parts; mesh | 4,336, 487, 4,209; 336,835 vertices, 558,147 indices | 6 |
  | Today's cache, read by Phase 4 | 213,880 vertices, 359,190 indices | 6 |
  | New reader errors | as listed | 7 |
  | Synthetic box pixels: none; A without / with parts / cut; B ground / raised / cut | 1,564; 0 / 1,564 / 1,564; 0 / 1,008 / 0 | 8 |
  | Midtown flights with parts, twice each | 1800 of 1800 | 9 |
  | Largest in-frame step | ≤ 10 m (probe 9.39 and 3.10 m) | 10 |
  | The credit line, either cache | unchanged, as above | 11 |
  | Box pixels with parts, off → on, centre / middle / frame | orbit 62,082 → 159,502 / 92,016 → 259,247 / 147,200 → 387,679; city 4,033 → 4,886 / 55,729 → 57,610 / 111,115 → 123,130; each ± 1 % | 12 |
  | `view --bench` with parts, see-through on | ≥ 30 fps | 14 |
- **Not predicted, and so not gated:**
  - the look of what the parts leave uncovered (OQ-16) and of overhang lids (OQ-17), for
    the user at gate 15;
  - render times (gate 13) and `view`'s frame rate above 30 (gate 14);
  - the S3 fetch's bytes (gate 5's note).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-4`), one push. The commits:
    - the fetch, `--source`, `--no-parts`, the fixture's steps 5 and 6, the one line each
      in `gates-city.sh` and `gates-credit.sh`, the reader and the mesh, with
      `tests/parts.rs`'s headless gates (6 and 7) and `tests/see_through.rs`'s four
      lines;
    - see-through's `top()`, `render` and `view`, the GPU and Midtown tests, and
      `scripts/gates-parts.sh` (gates 2, 5 and 8–12);
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **a new `rules/parts.md`** (`max_lines: 40`) covering the cache's parts and bases,
      the query by `building_id`, `--source`, `--no-parts`, the report's new keys, the
      reader's checks, the volumes and their mesh, `top()` and the cut. Its `sources` are
      `scripts/fetch-buildings.sh`, `scripts/fixture.sh`, `src/buildings.rs`,
      `src/see_through.rs`, `src/render.rs` and `src/view/mod.rs`;
    - **two rules at their caps are corrected and gain a pointer to it**, reworded to fit,
      with no `max_lines` raised:
      - `rules/buildings.md` (60/60): its fetch lines (the columns kept, the usage line,
        the report's keys) and "a feature is one building" are now true of the
        `--no-parts` form and of the buildings of either form, and say so; its mesh lines
        hold for a building without parts;
      - `rules/see-through.md` (40/40), whose `h` is `top()`;
    - `rules/credit.md`: its "each building dataset" becomes each feature's, parts
      included, one line, no new item;
    - `rules/render.md`, `rules/view.md` and `rules/camera.md`: none needed, since neither
      the CLI, the keys nor the camera change;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README**: "What it fetches" names the parts and their columns, the report's
      new keys and `--no-parts`; "Older caches" says a cache fetched before Phase 4 draws
      as before until fetched again; "How they look" says a building with parts is drawn
      from them, each from its base; and it says what `--source` does and does not check.
      It gains the new gate commands: `scripts/gates-parts.sh` and `cargo
      test --release --test parts -- --include-ignored --test-threads=1`;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 5 — Streets: junction surfaces, lane lines, stop lines and the yellow centre line, by default
*Produces the observable: yes. `render` writes the run's video with every junction filled
in the road's grey, and white lane lines, white stop lines at signals and a double yellow
centre line on every two-way street, faded where they are under a pixel. With
`--no-streets` the video is byte-identical to today's (gates 1 and 2).*

Drafted 2026-10-05; the design is §2.17, and the user's decisions are §2.17.1. Phase 5
builds on vis-001 Phase 7's fixtures (engine `90b39292`) and on Phases 1–4 here. With
`--no-streets` it changes no output. With streets on it changes no box, building, camera,
credit line or road strip: it adds two meshes under the boxes. OQ-18 (the median) and
OQ-19 (the fade) are answered (a) each (§2.17.1, decisions 5 and 6), as the scope and
gates below were drafted.

- **Scope:**
  - **The streets (`src/streets.rs`, new; no Bevy types).**
    - The constants of §2.17.6 and §2.17.9: `LINE_M` 0.15, `DOUBLE_SPACE_M` 0.10,
      `DASH_M` 3.0, `GAP_M` 9.0, `STOP_M` 0.60, `FLUSH_MIN_M` 1.0, `WHITE`, `YELLOW`,
      `LIFT_YELLOW_M` 0.01, `LIFT_WHITE_M` 0.02; and the fade's `FADE_FROM_PX` 0.1 and
      `FADE_TO_PX` 0.5 (§2.17.10).
    - `Streets::build(&Placement) -> Streets`: the junction surfaces (each node's polygon,
      in node order, kept public with its node id after the closing copy is dropped, and
      triangulated with `earcut`), the two-way pairs, the median fills,
      and the markings as ribbons, each with its kind (yellow, lane, stop), link, lane
      and width, built as §2.17.4–§2.17.8 say, in link order, each ribbon sampled by
      §2.17.6's rule. Stop lines are kept per `(link, lane)`.
    - `Streets::surface(&self)` and `Streets::markings(&self)`: the two meshes' data in
      `f64` world metres (positions, indices; for the markings also each vertex's colour
      and line width), the yellow lines first, then the lane lines, then the stop lines.
    - `fade(width_m, mpp) -> f64`, §2.17.10's `α`; `colour(marking, α)`, the linear
      blend `(1 − α)·road + α·marking` of `scene::ROAD` toward it (§2.17.10); and
      `mpp_at(&Pose, h_px) -> impl Fn([f64; 3]) -> f64`, the perspective metres per
      pixel at a world point (§2.17.10). The orthographic render passes `|_| k`.
  - **Drawing (`src/draw.rs`).**
    - `StreetsDrawn` (new): the two entities, each an `Option` (§2.17.12: a mesh with no
      triangle is not spawned), the markings' mesh handle, and what
      recolouring needs: the markings' `f64` positions, marking colours and line widths,
      and the last drawn colours.
    - `spawn_streets(world, &Streets, road material, (fx, fy), mpp: impl Fn([f64; 3]) ->
      f64) -> StreetsDrawn` spawns the two entities of §2.17.11 (either one only if its
      mesh has a triangle, §2.17.12). The surface uses the road's material handle; the
      markings one unlit white material with `cull_mode: None` and `NoFrustumCulling`.
    - `set_street_colours(world, &mut StreetsDrawn, mpp: impl Fn([f64; 3]) -> f64)`
      replaces the markings' mesh asset with the same positions and indices and new
      colours (`Assets::insert`), as `draw::set_building_heights` does, when any colour
      differs from the last drawn.
    - `materials`, `road_mesh` and every box and building function are unchanged.
  - **`render` (`src/render.rs`, `src/lib.rs`, `src/main.rs`).**
    - `Renderer` keeps two more fields: the road's material handle (today a local of
      `Renderer::build`) and, on a perspective renderer, its current pose (the one
      `Renderer::new_perspective` was given, then each `set_pose`'s), plus
      `Option<StreetsDrawn>`.
    - `Renderer::set_streets(&mut self, streets: Option<&Streets>) -> Result<()>`: `Some`
      spawns the two entities, with colours for the orthographic `k` or the current
      pose, then settles again (three empty renders, as at build); `None` despawns them (no caller
      in Phase 5 but `Job::set_streets(false)`, and no gate). Off from the start,
      nothing is spawned.
    - On a perspective renderer with streets, `set_pose` keeps the pose, recomputes the
      markings' colours for it and replaces their mesh when any colour differs from the
      last drawn.
    - `Job::set_streets(&mut self, on: bool) -> Result<()>` builds `Streets` from the
      job's placement and passes it, or `None`. `Job::prepare*` build every job with
      streets off.
    - `render --no-streets` is a clap flag. `src/main.rs` calls `Job::prepare_with`, then
      `set_streets(true)?` unless the flag is given, then `set_see_through` as today.
      `RenderOptions` and every `Job::prepare*` signature are unchanged.
  - **`view` (`src/view/state.rs`, `src/view/mod.rs`, `src/main.rs`).**
    - `view --no-streets` is a clap flag; `ViewOptions` gains `streets`, true unless it is
      given.
    - `Pressed` gains `m`; `ViewState` gains `streets_shown`, false at `new` (so every
      test's state is unchanged) and set from `ViewOptions` by `run` at launch. `m` flips
      it at any moment, as `b` flips `buildings_shown`, and changes nothing else. The frame
      order becomes "… `B`; `X`; `M`; orbit …".
    - `run` builds the streets at launch on every network and spawns them with their
      visibility from `streets_shown`. The window reads `KeyCode::KeyM`. Each frame, after
      the camera is set, the visibility of each entity spawned follows `streets_shown`,
      and while it is on the markings' colours follow the pose (every frame under
      `--bench`).
  - **Scripts:** the 29 renders of §2.17.12 gain `--no-streets` and nothing else, in
    `scripts/gates.sh`, `gates-ties.sh`, `gates-see-through.sh` and `gates-parts.sh`; each
    script's header comment says so in one line.
  - **Tests.**
    - `tests/streets.rs` (new):
      - headless, not ignored: gates 5, 9 and 12;
      - headless, needing the fixtures, so ignored: gates 6 and 7;
      - through the GPU, ignored: gate 8 (no fixture).
    - `scripts/gates-streets.sh` (new, offline): gates 10 and 11, and the renders for
      gate 15, into `scratch/out/streets/`: gate 10's first render of each case is kept
      as `streets-ortho-city.mp4`, `streets-city.mp4`, `streets-orbit.mp4` and
      `streets-ug-camera.mp4`.
    - **Not edited:** every other test file; `scripts/gates-city.sh`,
      `scripts/gates-credit.sh` and `scripts/fixture.sh`; `src/buildings.rs`,
      `src/see_through.rs`, `src/credit.rs`, `src/camera.rs`, `src/keyframes.rs`,
      `src/scene.rs` and `src/place.rs`; `Cargo.toml` and `Cargo.lock`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid and Midtown at engine `90b39292` (FCD SHA-1s `3ad76744…` and
  `01becc86…`), baseline, seed 42, with Midtown's Phase 2 cache
  (`scratch/midtown/buildings.geojson`, `f241ccbd…`). Everything runs offline. The
  predictions come from §2.17.15's probe, which drew through a copy of the shipped
  renderer with the streets added behind a switch.

  **Baseline, before any change**, at `origin/main`:
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` (vis-001 Phase 7 gate 5)
    are present, with SHA-256 `cdad89d1…` (camera), `dcfdd2b7…` (default), `f6255241…`
    (ortho-city), `2139c319…` (ortho-roads), `d6641985…` (flight-city1 and city-off, the
    same bytes), `020475b7…` (flight-roads1), `d7118c3a…` (city-on), `af80a068…`
    (orbit-off) and `05d7d174…` (orbit-on). If one is missing or differs, the run stops.
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).

  - **What must not change:**
  1. **urban_grid, with `--no-streets`.** `scripts/gates.sh` passes. Its default render
     gives **8700 of 8700** against `ref-pin90b39292-default`, and its `--camera` render
     **8700 of 8700** against `ref-pin90b39292-camera`, compared by hand on
     `scratch/out/camera.framemd5`. `--test gates` passes 5 of 5.
  2. **Midtown, with `--no-streets`.** `gates-ties.sh` gate 2 (2 renders),
     `gates-see-through.sh` gate 2 (7) and `gates-parts.sh` gate 2 (5) each give **1800 of
     1800** against their references: 14 renders covering the eight Midtown references.
     The recorded lines of `gates-see-through.sh` gate 7 and `gates-parts.sh` gate 9 equal
     the baseline's.
  3. **What streets do not touch.**
     - Every test file but `tests/streets.rs`, unedited, passes with every printed number
       equal to the baseline's. They build their jobs through `Job::prepare*`, which leave
       streets off, or their renderers through `Renderer::new*`, which spawn none until
       `set_streets`; the two that run the binary (`tests/camera.rs`'s
       `gate5_keyframe_file_errors`, `tests/view.rs`'s `gate2_view_input_errors`) stop
       at an error before streets are built.
     - The renders that compare only with each other now draw streets and still give equal
       pairs: `gates-ties.sh` gate 9, **1800 of 1800** for each pair; `gates-city.sh` gate
       13, **1800** for flight 2 and flight 3 (sandboxed) against flight 1, and for its
       orthographic pair; `gates-credit.sh` gate 14, **1800** for each pair. All six
       scripts pass.
     - `git diff origin/main -- scripts/` changes 29 render lines, each by ` --no-streets`
       alone, plus one header line in each of the four scripts, and adds
       `gates-streets.sh`.
  4. **Build cost** (§2.17.14). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The streets — headless:**
  5. **A synthetic crossing** (`tests/streets.rs`, not ignored). The network is written in
     the test as YAML and read with `serde_yaml` into a `NetworkConfig`, then
     `Placement::new`:
     ```yaml
     schema_version: 1
     metadata: {name: streets synthetic, coordinate_system: metric, z_enabled: false}
     nodes:
       - {id: C, point: [0, 0], type: junction}
       - {id: W, point: [-50, 0], type: endpoint}
       - {id: E, point: [50, 0], type: endpoint}
       - {id: N, point: [0, 50], type: endpoint}
       - {id: S, point: [0, -50], type: endpoint}
     links:   # each also: median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]
       - {id: L_WC, from_node: W, to_node: C, geometry: [[-50, 0], [0, 0]]}
       - {id: L_CW, from_node: C, to_node: W, geometry: [[0, 0], [-50, 0]]}
       - {id: L_EC, from_node: E, to_node: C, geometry: [[50, 0], [0, 0]]}
       - {id: L_CE, from_node: C, to_node: E, geometry: [[0, 0], [50, 0]]}
       - {id: L_NC, from_node: N, to_node: C, geometry: [[0, 50], [0, 0]]}
       - {id: L_CS, from_node: C, to_node: S, geometry: [[0, 0], [0, -50]]}
     junctions:
       - {node_id: C, control: signal, geometry: {setback: 3}, stop_line_offsets: {L_EC: {'0': 5}}}
     ```
     An east–west two-way street, a one-way street southbound, one signal. The
     predictions (the probe's):
     - **one junction surface**, the engine's polygon of 44 vertices as given, its last a
       copy of its first, so 43 after that copy is dropped (36 distinct, with rounded
       corners), x ±6.5 m and y ±10.25 m at most; the sum of its triangles' areas
       equals the polygon's shoelace area (236.69 m²) to 1e-9 of it. Without the surface
       there is no triangle, and this fails;
     - **pairs** `(L_WC, L_CW)` and `(L_EC, L_CE)`; **4 yellow lines**, at
       `y ∈ [0.05, 0.20]` and `[−0.20, −0.05]` on each, from the junction's edge
       (`|x|` 6.5) to the end (`|x|` 50); **2 median fills**, `y ∈ [−0.25, 0.25]`. No
       yellow line on `L_NC` or `L_CS`;
     - **24 lane-line dashes**, 4 a link, each 0.15 m wide on the line between the two
       lanes (`y` = ±3.75, or `x` = 0), starting at each link's `s` 0, 12, 24 and 36;
       `L_EC`'s last ends at `s` 37.9, cut by its stop line;
     - **6 stop lines**, one per lane of `L_WC`, `L_EC` and `L_NC`. Each is 0.60 m deep and
       ends at the junction's edge (`L_WC` at `x` −6.5, `L_NC` at `y` 10.25), except
       `L_EC` lane 0, which ends 5 m back: `x` from 12.1 to 11.5, against lane 1's 7.1 to
       6.5. None on `L_CW`, `L_CE` or `L_CS`, which end at endpoints.

     Every number is computed in the test with `==` where it is exact (the counts, the
     links, the lanes) and within 1e-9 m elsewhere.
  6. **The fixtures' counts** (`tests/streets.rs`, ignored: needs both fixtures). Each
     count is also computed independently in the test from `NetworkConfig` (the
     junctions' polygons, the lanes of links into signals, the node-pair lookup, the gaps,
     and `ceil(end / 12 m)` dashes per lane boundary):

     | | Midtown | urban_grid |
     |---|---|---|
     | Junction surfaces | **106** | **9** |
     | Two-way pairs | **29** | **24** |
     | Median fills | **26** | **24** |
     | Yellow lines | **88** | **48** |
     | Stop lines (approach links) | **402** (188) | **71** (36) |
     | Lane-line dashes | **2,208** | **1,105** |
     | Surface mesh: vertices, triangles | **10,510**, **9,532** | **13,971**, **13,842** |
     | Markings mesh: vertices, triangles | **37,594**, **32,198** | **36,292**, **33,844** |

     **And against the engine's dashboard geometry** (§2.17.2): `NetworkJson::
     from_config_with_network_data(&placement.network, &placement.data)` on each fixture.
     Its `junction_fills` (106 and 9), with every trailing vertex equal to the first
     stripped (two a ring, §2.17.2), have each the same number of vertices as its
     surface's polygon (`==`) and equal it vertex for vertex: worst distance
     **0.000000 m**, within 1 cm. Every corner of the draft's stop lines' junction-facing
     edges (804 and 142) lies on the junction-facing edge of the engine's stop line for
     that link (its lane's, where it has one per lane): worst distance **0.000000 m**,
     within 1 cm. The engine's stop lines are matched as the probe matched them: a ring's
     junction-facing edge is `coords[1]`–`coords[2]`; a ring with a `lane` belongs to its
     `link_id` and lane; a full-width ring (`link_id` `None`) belongs to the link whose
     trimmed end (`Placement::place_lateral` at `link_length`, lateral 0) is nearest that
     edge's midpoint, and to each of its lanes; a ring with a `link_id` and no `lane`
     (a connector: urban_grid's one, on `L_J12_J11`) and every ring on a link that does
     not end at a signal (39 in Midtown) are left out.
  7. **The strips and the boxes** (`tests/streets.rs`, ignored, headless). From
     `run::load` on each fixture, over its whole run:
     - every strip-end corner at a junction polygon (922 and 144) is within **1 mm** of the
       boundary of that junction's built polygon (`Streets`' per-junction polygons);
     - of the box centres at every whole second (201,256 and 21,358), **all** lie on the
       strips or the built junction surfaces' triangles, and those in a junction span
       (17,706 and 1,231) all on the surfaces; on the strips alone **91.202 %** and
       **94.236 %**, as today;
     - **the first stopped box at a stop line** (§2.17.7's rule, from the FCD rows, the gap
       measured to the upstream edge of the built stop line for its `(link, lane)`): the
       engine's code predicts its front at the line's back edge, a gap of **0**. Measured,
       and predicted exactly here since the FCD is fixed: Midtown **2,491** episodes
       (48,899 rows), median gap **+0.042 m**, **2,441** within 0.10 m, 27 between 1.35
       and 1.55 m, **10** at −0.600 m (front at the junction's edge) and 13 others;
       urban_grid **129** (2,555), median **+0.042 m**, **123** within 0.10 m and 6
       between 1.35 and 1.55 m.
  - **The streets — through the GPU:**
  8. **The synthetic crossing, drawn** (`tests/streets.rs`, ignored, no fixture). Gate 5's
     network, through `Renderer::new` with `scene::Camera { cx: 0, cy: 0, k: 0.05 }` at
     1280×720, a pool of 1, no buildings, no credit and no box: once as built, and once
     after `set_streets(Some(&streets))`. At the pixel holding each world point, off → on:

     | Point (m) | What | Off | On |
     |---|---|---|---|
     | (0, 0) | the junction | background (18, 22, 30) | **road (92, 96, 104)** |
     | (−20, 0) | the median fill | background | **road** |
     | (−20, 0.125), (−20, −0.125) | the double yellow | background | **(230, 170, 20)** each |
     | (−8, 3.75) | a lane-line dash | road | **(235, 235, 235)** |
     | (−14, 3.75) | between dashes | road | road |
     | (−6.8, −2), (6.8, 5.5) | stop lines at the edge | road | **(235, 235, 235)** |
     | (6.8, 2) | `L_EC` lane 0 at the edge | road | **road**: its line is 5 m back |
     | (11.8, 2) | `L_EC` lane 0's line | road | **(235, 235, 235)** |
     | (0, 12.5), (0, 16) | the one-way's dash, its gap | road, road | **white**, road |
     | (3.3, 14) | the one-way's left edge (its driver's; south is down) | road | road |

     Over the whole frame: **112,608** pixels change; **6,316** are exactly white and
     **6,120** exactly yellow; and **0** pixels on the one-way street's arms (`|x| < 8`,
     `|y| > 9`) are yellowish (red minus blue over 40). Lines here are 3 px wide, so the
     fade leaves them whole. Without the feature every "on" above fails.
  9. **The fade** (`tests/streets.rs`, not ignored): `fade(0.15, 1.5)` (0.1 px) is
     **0**; `fade(0.15, 0.5)` (0.3 px) is **0.5** (within 1e-12); `fade(0.15, 0.3)` (0.5
     px) is **1**; the stop line at Midtown's `k` 1.6009 m/px, **0.767** (±0.001); and a
     line at urban_grid's 1.1481, **0.017** (±0.001). `colour` at `α` 0 is the road's grey
     and at 1 the marking's colour, each channel `==`. `mpp_at(&Pose::top_down(cx, cy,
     h), H)` at `(cx, cy, 0)` is `h / H` within 1e-9 of it, for `h` 60, 500 and 1800 m and
     `H` 720 and 1080; at a ground point 100 m from `(cx, cy)` it is larger; and at a
     pose with pitch 25°, of two ground points on the line of sight's ground track, the
     one farther from the eye gives the larger value.
  - **The streets — renders:**
  10. **On, deterministic** (`scripts/gates-streets.sh`). Each twice, each pair equal:
      - urban_grid, the default render and `--camera tests/flight.toml`: **8700 of 8700**;
      - Midtown `--from 300 --to 360 --speedup 1`: orthographic with and without
        `--buildings`; `--camera tests/city-flight.toml` with and without
        `--buildings`; `--camera tests/see-through-flight.toml` with `--buildings`:
        **1800 of 1800** each (`ffprobe` `1920,1080,30/1,1800`).

      **Streets are on by default:** each first render differs from its `--no-streets`
      reference in **at least one frame**, and the count of differing frames is recorded
      (the probe: every frame). The references, all `ref-pin90b39292-*`: `default` and
      `camera` (urban_grid); `ortho-city` and `ortho-roads` (orthographic, with and
      without buildings); `city-on` and `flight-roads1` (the city flight with buildings,
      see-through on as `render` defaults, and without); `orbit-on` (the see-through
      flight).
  11. **The flag's edges** (`scripts/gates-streets.sh`):
      - `render --streets` with every argument a Midtown render needs is a clap usage
        error, **exit 2**, and writes no video;
      - `view --no-streets --bench 1` on urban_grid exits **0** and prints its JSON line;
      - `render --no-streets` is accepted, by gates 1 and 2.
  12. **`M`, headless** (`tests/streets.rs`), as Phase 3's gate 11 tests `X`:
      - on the plain state of vis-001 Phase 3, `m` flips `streets_shown` from false to
        true, and a second `m` flips it back;
      - `t`, the clock, the pose, `k`, `follow`, `buildings_shown` and `see_through` stay
        exactly as they were;
      - during a right-button orbit, a left drag and a scrub, `m` flips the flag and the
        gesture goes on unchanged.
  - **Recorded, with one bar:**
  13. **Render time.** Record gate 10's wall times beside the same renders with
      `--no-streets` (gates 1 and 2). The probe's interleaved measure: +5.1–5.2 ms a frame
      in perspective (§2.17.11), about 9 s a Midtown flight; the orthographic render
      builds its colours once.
  14. **`view --bench 20`** on Midtown at the default window, with `--buildings` (streets
      and see-through on by default) and with `--buildings --no-streets`. Record the JSON
      and the load average.
      - **A `mean_fps` below 30 with streets on stops the build**, and the phase goes back
        to review.
      - Prediction: at least 30, and likely 60 (vsync). The probe gave 60.00 fps with
        streets and the fade, twice, as without (§2.17.11).
  - **The user's check:**
  15. **The user watches** gate 10's Midtown renders with streets, against gate 2's with
      `--no-streets`. Times are the video's, from 0:00; colours as §2.17.6.
      - **Midtown, orthographic with buildings (`streets-ortho-city.mp4`), throughout.**
        Where streets cross, the frame shows grey squares joined to the streets, where
        `--no-streets` shows dark squares between the grey bands. The dark seam down the
        middle of the wider streets is gone. No white or amber dot lies along a street.
        Short faint white ticks cross the streets at the edges of most crossings.
      - **The city flight with buildings (`streets-city.mp4`).** From 0:00 to 0:10, high
        and nearly straight down: as the orthographic video, and no dot shimmers along
        the streets as the camera descends. From 0:30 to 1:00, near the middle of the
        frame: white dashes along the streets, white bars across them at the crossings,
        and amber double lines along the middle of two-way streets; toward the top of the
        frame they fade into the grey, with no line popping on or off between frames.
      - **The orbit with buildings (`streets-orbit.mp4`), 0:00 to 1:00, the middle of the
        frame.** The crossing there is grey, and the boxes crossing it stand on grey, not
        on the dark background. Every street shows sharp white dashes; the approaches
        have a white bar across their lanes at the crossing's edge, and the first box
        queued at a red light stops with its front at the bar, touching it, with no grey
        between them and not over it; a two-way street has an amber double line down its middle.
        Nothing flickers between white and grey from frame to frame, and the boxes stand
        over the lines.
      - **urban_grid's `--camera` render (`streets-ug-camera.mp4`), from 0:55 to 2:11,**
        the follow at `height_m` 120 to 60: lane dashes, the double amber line beside the
        followed box, and white bars where it reaches a crossing, all sharp. At the
        centre crossing, on the approach from the east, one lane's bar stands about 5 m
        behind its neighbour's.
      - **In `view` on Midtown:** at launch, grey crossings with faint white ticks at their
        edges (the stop lines, `w_px` 0.25, about a third of their colour) and no lane or
        yellow line; scrolling in
        over a street, the lines fade in smoothly; `M` hides streets (the crossings go
        dark again) and shows them; `view --no-streets` opens as today, and `M` shows
        them.

      Then say whether OQ-18's and OQ-19's answers stand as they look, and whether the
      widths, the colours, the dash pattern or the fade's thresholds should change
      (iteration, §2.17.6, §2.17.10).
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | urban_grid with `--no-streets`: default and `--camera` | 8700 of 8700 against `ref-pin90b39292-default` and `-camera` | 1 |
  | Midtown with `--no-streets`: 14 renders | 1800 of 1800 against each of the eight Midtown references | 2 |
  | Other tests; the self-compared renders, now with streets | baseline numbers; 1800 of 1800 each | 3 |
  | Packages; `Cargo.toml` | 0; unchanged | 4 |
  | Synthetic: surfaces, pairs, yellow lines, fills, dashes, stop lines | 1; 2; 4; 2; 24; 6, `L_EC` lane 0 5 m back | 5 |
  | Midtown / urban_grid counts | as gate 6's table | 6 |
  | Against the engine's `NetworkJson`: fills, stop lines' junction-facing edges | worst 0.000000 m each | 6 |
  | Strip ends; box centres on the drawn road | ≤ 1 mm; all (today 91.202 % / 94.236 %) | 7 |
  | First stopped box's front to the line's back edge, Midtown / urban_grid | engine's code: 0; 2,491 / 129 episodes, median +0.042 m, 2,441 / 123 within 0.10 m, 10 / 0 at −0.600 m | 7 |
  | Synthetic pixels off → on; changed, white, yellow, yellow on the one-way | as gate 8's table; 112,608, 6,316, 6,120, 0 | 8 |
  | The fade; `colour`'s ends; `mpp_at` top-down | 0, 0.5, 1; 0.767; 0.017; exact; `h / H` | 9 |
  | Streets on, twice each: two urban_grid, five Midtown renders; against `--no-streets` | 8700 of 8700; 1800 of 1800; at least one frame differs (probe: every frame) | 10 |
  | `--streets`; `view --no-streets --bench 1` | exit 2; exit 0 | 11 |
  | `view --bench` with streets and buildings | ≥ 30 fps | 14 |
- **Not predicted, and so not gated:**
  - the look: the widths, the colours, the fade's two thresholds, OQ-18's median and
    OQ-19's fade, for the user at gate 15;
  - render times (gate 13), and `view`'s frame rate above 30 (gate 14).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-5`), one push. The commits:
    - `src/streets.rs` and its headless tests (gates 5–7 and 9);
    - the drawing, `render`, `view`, the CLI, the GPU test, the 29 script lines and
      `scripts/gates-streets.sh` (gates 1–4, 8 and 10–14). The default changes here, so
      gates 1–3 run after this commit;
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **a new `rules/streets.md`** (`max_lines: 40`) covering the surfaces, the pairs and
      fills, the markings and their constants, the stop lines, the lifts, the fade, the
      two meshes, the default, `--no-streets` and `M`. Its `sources` are
      `src/streets.rs`, `src/draw.rs`, `src/render.rs`, `src/lib.rs`,
      `src/view/state.rs`, `src/view/mod.rs` and `src/main.rs`;
    - **three rules at their caps are corrected and gain a pointer to it**, reworded to
      fit, with no `max_lines` raised:
      - `rules/render.md` (60/60): its CLI line gains `[--no-streets]`; its "Roads" item
        says streets are drawn over them unless the flag is given; its Pipeline bullet on
        `Job::prepare_with` says streets are off there too, and that `set_streets`
        settles again;
      - `rules/view.md` (60/60): the CLI, `M` and the frame order, with another line of
        its Drawing bullet tightened to make room;
      - `rules/camera.md` (60/60): "What each path draws" names the two street meshes on
        both paths, and the markings' `NoFrustumCulling`;
    - `rules/see-through.md`, `rules/buildings.md`, `rules/parts.md`, `rules/credit.md`,
      `rules/inputs.md`, `rules/motion.md` and `rules/slider.md`: none needed, since none
      of their subjects changes;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README** says that `render` and `view` draw streets by default (what, and the
      fade), gains `--no-streets`, `M` and the new gate commands:
      `scripts/gates-streets.sh` and `cargo test --release --test streets --
      --include-ignored --test-threads=1`; and says the four reference scripts pass
      `--no-streets`;
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and
    its cause.
  - Write this phase's `shipped` date.

### Phase 6 — Engine look and lane arrows: the dashboard's markings, from `NetworkJson`, by default
*Produces the observable: yes. `render` writes the run's video with the engine
dashboard's markings in place of Phase 5's US set: its lane dashes, its centre line, its
0.4 m stop lines at the approaches it marks, and its lane arrows, in its colours, faded
where they are under a pixel, over Phase 5's junction surfaces and median fills. With
`--no-streets` the video is byte-identical to today's (gates 1 and 2).*

Drafted 2026-10-08; the design is §2.18, and the user's decisions are §2.18.1. Phase 6
builds on Phase 5 (shipped 2026-10-08) at engine `90b39292`. With `--no-streets` it changes
no output. With streets on it changes only the markings mesh: no box, building, camera,
credit line, road strip, junction surface or median fill. OQ-20 (the colours) is answered
(a), and the median gaps stay as drafted (§2.18.1, decisions 4 and 5), as the scope and
gates below were drafted.

*(2026-10-09, changed by the user at gate 14)* Amended before it ships: the median fills
are not drawn, and each gap shows the background, rounded at its junction ends by
`NetworkJson::median_noses` in the road's grey. The scope's, gates' and predictions'
changes are this phase's last item, "Amended at gate 14"; the design is §2.18.16.

- **Scope:**
  - **The streets (`src/streets.rs`; no Bevy types).**
    - **Removed:** Phase 5's US markings, their constants (`LINE_M`, `DOUBLE_SPACE_M`,
      `DASH_M`, `GAP_M`, `STOP_M`, `FLUSH_MIN_M`, `WHITE`, `YELLOW`) and their code in
      `Streets::build` (§2.18.11). **Kept:** the junction surfaces, the pairs, the
      median fills, `Ribbon` and `ribbon` (which draw the fills), `fade`, `mpp_at`,
      `vertex_colours`, `FADE_FROM_PX` and `FADE_TO_PX`; the lifts' values, renamed
      `LIFT_LINES_M` 0.01 and `LIFT_TOP_M` 0.02 (§2.18.8).
    - **Copied from the front end, each with its file and line at `90b39292` in a
      comment** (§2.18.12): the dark palette's five marking colours (`LANE_MARKING`,
      `MEDIAN_LINE`, `STOP_LINE`, `ROAD_ARROW`, `DEAD_END_BAR`) and their five opacities
      (§2.18.4); `ROAD_FILL` is not copied, since only §2.18.4's dashboard-road column
      uses it and §2.18.12's diff reads it there;
      the glyph constants `LA_SW`, `LA_HW`, `LA_HL`, `LA_LEN`, `LA_TURN_SHAFT`,
      `LA_TURN_R`, the u-turn's radius and return leg, the fork and the dead-end bar
      (§2.18.5); and the glyph outlines, `glyph(arrow_type) -> (Vec<Vec<[f64; 2]>>,
      Option<Vec<[f64; 2]>>)` (white outlines; the red bar), as §2.18.5 says.
    - `Kind` becomes `Centre`, `Dash`, `Solid`, `Stop`, `Connector`, `Arrow`, `DeadEnd`.
      A `Marking` is its kind, its link (`Option<usize>`, in link order), its lane
      (`Option<u32>`), its arrow type (for an arrow or a bar), its polygon (no closing
      copy) and its `earcut` triangles. Stop lines and connectors take their link and
      lane as §2.18.6 gives them, and arrows and bars from their `LaneArrow`
      (`link_id` → its index, `lane`); centre lines, dashes and solid lines, which
      `NetworkJson` gives no link, have `None` for both. Each kind's colour is the copied colour
      composited over `scene::ROAD` at its opacity, rounded once (§2.18.4: `COLOURS`, a
      constant table, checked by gate 9), and each kind's fade width is §2.18.9's.
    - `Streets::build(&Placement)`: the surfaces, pairs and fills as now; then
      `NetworkJson::from_config_with_network_data(&pl.network, &pl.data)` once, and the
      markings from it in its order: `median_lines` (centre lines, edges split at
      `STRIP_STEP`), `lane_marking_dashes`, `solid_lane_lines` (edges split likewise),
      `stop_lines` (a full-width ring matched to its link and cut per lane; per-lane rings
      as they are; connectors whole: §2.18.6), then `lane_arrows`, each glyph's outlines
      and its bar. A full-width ring matched to no link within 1 mm is kept whole with
      no link and no lane (§2.18.6). `Streets::build`'s signature does not change.
    - `Streets::markings(&self)`: centre lines, dashes, solid lines at `LIFT_LINES_M`;
      stop lines, connectors, arrows, bars at `LIFT_TOP_M`; each kind in build order. Its
      data is what it is today (positions, a colour and a width per vertex, indices), so
      the drawing does not change.
    - `colour(marking, α)`: the blend as now, from linear colours computed once for the
      road and the five marking colours.
  - **Drawing, `render`, `view`, the CLI:** no change (§2.18.11), but for one comment.
    `--no-streets` and `M` as Phase 5. `src/main.rs`'s `--no-streets` doc comment, which
    is its `--help` text and names "lane lines, stop lines and the yellow centre line",
    is reworded to the dashboard's markings (lane dashes, stop lines, lane arrows, a
    centre line where a pair has no gap); no other line of `src/main.rs` changes.
  - **Tests.**
    - `tests/look.rs` (new):
      - headless, not ignored: gates 5 and 9;
      - headless, needing the fixtures, so ignored: gates 6 and 7;
      - through the GPU, ignored: gate 8 (no fixture).
    - `tests/streets.rs`: the assertions of §2.18.11's table that test the US markings
      are removed, with the code only they use. Kept assertions share statements with
      removed ones, so a kept line may change, but only to drop what is removed:
      `fade(LINE_M, 1.1481)` becomes `fade(0.15, 1.1481)` (`LINE_M` is gone from
      `src/streets.rs`); gate 6's counter tuple, its `got` and expected arrays and its
      `println!`s lose the removed counts (the yellow lines, stop lines, stop links,
      dashes and the markings mesh); `fill_worst <= 0.01 && stop_worst <= 0.01` loses its
      stop-line clause; gate 8's `[Row; 13]` becomes `[Row; 2]`; gate 5's and gate 9's
      `println!`s lose the removed values; and names no longer used leave the `use` lines.
      Nothing is added, and no kept value or tolerance changes. Every assertion it keeps
      passes with Phase 5's numbers.
    - `scripts/gates-look.sh` (new, offline): gate 10, and the renders for gate 14, into
      `scratch/out/look/`: gate 10's first render of each case is kept as
      `look-ortho-city.mp4`, `look-city.mp4`, `look-orbit.mp4` and `look-ug-camera.mp4`.
    - **Not edited:** every other test file; every other script, `scripts/gates-streets.sh`
      included; every source file but `src/streets.rs` and the one doc comment in
      `src/main.rs`; `Cargo.toml` and `Cargo.lock`.
- **Exit gate.** On the development machine (Apple M3, macOS, Bevy 0.19.1, ffmpeg 9.0.2),
  on urban_grid and Midtown at engine `90b39292` (FCD SHA-1s `3ad76744…` and
  `01becc86…`), baseline, seed 42, with Midtown's Phase 2 cache
  (`scratch/midtown/buildings.geojson`, `f241ccbd…`). Everything runs offline. The
  predictions come from §2.18.15's probe, which drew through a copy of the shipped
  renderer with Phase 6's markings swapped in behind a switch.

  **Baseline, before any change**, at `origin/main`:
  - **The ten references** `scratch/ref-pin90b39292-*.framemd5` are present with the
    SHA-256s of Phase 5's baseline (`cdad89d1…` camera, `dcfdd2b7…` default, `f6255241…`
    ortho-city, `2139c319…` ortho-roads, `d6641985…` flight-city1 and city-off,
    `020475b7…` flight-roads1, `d7118c3a…` city-on, `af80a068…` orbit-off, `05d7d174…`
    orbit-on). If one is missing or differs, the run stops.
  - **Phase 5's streets renders are copied** before anything runs, since
    `scripts/gates-streets.sh` writes into `scratch/out/streets/`: its seven `*.framemd5`
    (the first of each pair) and the four videos of Phase 5's gate 15
    (`streets-ortho-city.mp4`, `streets-city.mp4`, `streets-orbit.mp4`,
    `streets-ug-camera.mp4`) go to `scratch/out/streets-p5/`, and their SHA-256s are
    recorded.
  - **Run** every gate script and every test file with `--include-ignored
    --test-threads=1`, and keep their output. Record spec-lint and `Cargo.lock`'s package
    count (500).

  - **What must not change:**
  1. **urban_grid, with `--no-streets`.** `scripts/gates.sh` passes. Its default render
     gives **8700 of 8700** against `ref-pin90b39292-default`, and its `--camera` render
     **8700 of 8700** against `ref-pin90b39292-camera`, compared by hand on
     `scratch/out/camera.framemd5`. `--test gates` passes 5 of 5.
  2. **Midtown, with `--no-streets`.** `gates-ties.sh` gate 2 (2 renders),
     `gates-see-through.sh` gate 2 (7) and `gates-parts.sh` gate 2 (5) each give **1800 of
     1800** against their references: 14 renders covering the eight Midtown references.
     The recorded lines of `gates-see-through.sh` gate 7 and `gates-parts.sh` gate 9 equal
     the baseline's. (The probe: Midtown's ortho with `--no-streets`, 1800 of 1800
     against `ortho-roads`.)
  3. **What Phase 6 does not touch.**
     - Every test file but `tests/look.rs` and `tests/streets.rs`, unedited, passes with
       every printed number equal to the baseline's. `tests/streets.rs`, with §2.18.11's
       assertions removed, passes with every number it still prints equal to the
       baseline's: the surfaces, pairs, fills, surface mesh (10,510 and 9,532; 13,971 and
       13,842), strip ends, box centres, the fade's and `mpp_at`'s values, and gate 12.
     - The renders that compare only with each other still give equal pairs:
       `gates-ties.sh` gate 9, `gates-city.sh` gate 13 and `gates-credit.sh` gate 14,
       **1800 of 1800** each; `gates-streets.sh` gates 10 and 11 pass (seven cases twice,
       each pair equal and each differing from its `--no-streets` reference; `--streets`
       exits 2). All seven scripts pass.
     - `git diff origin/main --stat -- src/` is `src/streets.rs` and `src/main.rs`, and
       `src/main.rs`'s diff is the `--no-streets` doc comment alone; `-- scripts/` adds
       `gates-look.sh` alone; `-- tests/` adds `tests/look.rs`, and in `tests/streets.rs`
       every change is a removed line or a kept line narrowed as the scope's "Tests"
       allows, read hunk by hunk and listed in the record: no added assertion, and no
       kept value or tolerance changed.
  4. **Build cost** (§2.18.14). `git diff origin/main -- Cargo.toml Cargo.lock` is empty:
     **0 packages**, no feature.
  - **The markings — headless:**
  5. **A synthetic crossing with every arrow type** (`tests/look.rs`, not ignored). The
     network is written in the test as YAML, read with `serde_yaml` into a
     `NetworkConfig`, then `Placement::new`:
     ```yaml
     schema_version: 1
     metadata: {name: arrows synthetic, coordinate_system: metric, z_enabled: false}
     nodes:
       - {id: C, point: [0, 0], type: junction}
       - {id: W, point: [-60, 0], type: endpoint}
       - {id: E, point: [60, 0], type: endpoint}
       - {id: N, point: [0, 60], type: endpoint}
       - {id: S, point: [0, -60], type: endpoint}
     links:   # lane k: {id: k, width: 3.5, speed_limit: 13.9}; geometry: the two nodes' points
       - {id: L_WC, from_node: W, to_node: C, median_gap: 0,   lanes: 3}
       - {id: L_CW, from_node: C, to_node: W, median_gap: 0,   lanes: 2}
       - {id: L_EC, from_node: E, to_node: C, median_gap: 0.5, lanes: 3}
       - {id: L_CE, from_node: C, to_node: E, median_gap: 0.5, lanes: 2}
       - {id: L_NC, from_node: N, to_node: C, median_gap: 0,   lanes: 2}
       - {id: L_CN, from_node: C, to_node: N, median_gap: 0,   lanes: 2}
       - {id: L_SC, from_node: S, to_node: C, median_gap: 4,   lanes: 1}
       - {id: L_CS, from_node: C, to_node: S, median_gap: 4,   lanes: 2}
     junctions:
       - node_id: C
         control: signal
         geometry: {setback: 3}
         stop_line_offsets: {L_EC: {'0': 5}}
         movements:   # id: from_link → to_link, from_lanes → to_lanes, type
           # W_U L_WC→L_CW [0]→[0] u-turn;  W_L L_WC→L_CN [1]→[0] left
           # W_T L_WC→L_CE [1,2]→[0,1] through;  W_R L_WC→L_CS [2]→[1] right
           # E_L L_EC→L_CS [0]→[0] left;  E_U L_EC→L_CE [0]→[0] u-turn
           # E_T L_EC→L_CW [1]→[1] through;  E_R L_EC→L_CN [2]→[1] right
           # N_L L_NC→L_CE [0]→[0] left;  N_T L_NC→L_CS [0]→[0] through;  N_R L_NC→L_CW [0]→[1] right
           # S_L L_SC→L_CW [0]→[0] left;  S_R L_SC→L_CE [0]→[1] right
     ```
     (The test writes each link and movement out in full; this is its content. Lanes are
     `{id: 0, …}`, `{id: 1, …}` in order, and each movement is `{id, from_link, to_link,
     from_lanes, to_lanes, type}` with `type` one of `u-turn`, `left`, `through`, `right`,
     exactly as the probe's `scratch/vis002p6-probe/synth/network.yaml`.) Lanes per
     approach: `L_WC` u-turn / through+left / through+right; `L_EC` left+u-turn /
     through / right; `L_NC` all three / none; `L_SC` left+right. The predictions (the
     probe's, `scratch/vis002p6-probe/out/synth.txt`):
     - **2 centre lines,** on the two 0 m pairs: `y ∈ [−0.075, 0.075]` from `x` −60 to
       −12 (`L_WC`), and `x ∈ [−0.075, 0.075]` from `y` 13.75 to 60 (`L_NC`); **none** on
       the 0.5 m pair (east) or the 4 m pair (south), whose fills Phase 5 draws;
     - **68 lane dashes**, each 2.5 × 0.15 m; on `L_WC`'s first boundary at `y` −3.5,
       from `x` −60, −53.5, … −14.5, the last ending at −12.0, under the stop line;
     - **stop lines 0.40 m deep:** `L_WC`'s full-width ring (`x` −12.4 to −12.0, `y` −10.5
       to 0) cut into 3 lanes; `L_EC`'s three per-lane rings, lane 0 at `x` 15.0–15.4 and
       lanes 1–2 at 10.0–10.4; `L_NC`'s ring (`y` 13.75–14.15) cut into 2; `L_SC`'s one
       lane (`y` −13.9 to −13.5). **9 stop lines**, one per `(link, lane)`, and **1
       connector** on `L_EC` between lanes 0 and 1 (`y` 3.7–3.8, `x` 10.0–15.4);
     - **35 arrows:** one each of `through_left` (`L_WC` lane 1, at (−16, −5.25)),
       `through_right` (`L_WC` 2, (−16, −8.75)), `left` (`L_EC` 0, (19, 2)), `right`
       (`L_EC` 2, (14, 9)), `through_left_right` (`L_NC` 0, (−1.75, 17.75)), `dead_end`
       (`L_NC` 1, (−5.25, 17.75)) and `left_right` (`L_SC` 0, (3.75, −17.5)); 2 `u_turn`
       (`L_WC` 0 alone at (−16, −1.75); `L_EC` 0's separate one at (24, 2), 5 m behind
       its `left`); and 26 `through` (`L_EC` lane 1's arriving one at (14, 5.5), 8
       arriving on the exits near their ends, 17 departing at 4 m after each link's
       start). Headings 0, 180, −90 and 90 as each link runs;
     - **the mesh:** centre lines 2 (194 vertices, 190 triangles), dashes 68 (272, 136),
       stop lines 9 (36, 18), connector 1 (4, 2), arrows 40 outlines (513, 433), dead-end
       bar 1 (4, 2): **1,023 vertices, 781 triangles**;
     - **every drawn vertex** is a vertex of its `NetworkJson` ring or glyph placement
       exactly, or lies on that ring's edge within 1e-9 m (the cut points, the split
       points), and the cut quads' areas sum to their ring's within `1e-9 ×` its area
       (relative, as Phase 5's gate 5);
     - **each cut quad is its lane's:** `L_WC`'s lane 0 quad spans `y` −3.5 to 0 and
       its lane 2 `y` −10.5 to −7; `L_NC`'s lane 0 spans `x` −3.5 to 0 and its lane 1
       `x` −7 to −3.5 (lane 0 the leftmost, §2.18.6). A cut run from the wrong corner
       swaps them.

     Every number is computed in the test with `==` where it is exact (the counts, the
     types, the links and lanes) and within 1e-9 m elsewhere. **Without the feature it
     fails:** Phase 5's builder gives no arrow, a double yellow line on every pair and
     0.60 m stop lines at signals only.
  6. **The fixtures' counts, and against the engine** (`tests/look.rs`, ignored: needs both
     fixtures). Each count is also computed independently in the test from
     `NetworkConfig` and `NetworkData`: arrows per lane of each link at least 8 m long
     (the arc length of its `NetworkJson::links[].coords`, the offset polyline the
     engine measures, not `LinkData.length`), the departing ones on links of 25 m or
     more, each type from the lane's movements; stop lines by the engine's whole rule
     (`network_json.rs` l. 1553–1693 at the pin, §2.18.2): no waypoint; every lane of
     every approach to a signal; at a priority junction the approaches with a minor
     movement, and at any other unsignalised junction those with a lane path that is not
     transparent (`NetworkData::is_transparent_lane_path`), each then only on its lanes
     with such a path; no approach under 1 m long, and no ring, full-width or per lane,
     whose centre would fall before the link's start (`s_center < 0`, l. 1690 and
     1746–1749). Neither fixture has a
     priority junction or a waypoint, so those branches give 0 here and are copied for a
     pin move. The per-type arrow counts are `NetworkJson`'s (§2.18.3); the probe did not
     count types independently, so this test is the first to.

     | | Midtown | urban_grid |
     |---|---|---|
     | Centre lines | **3** | **0** |
     | Lane dashes | **3,921** | **2,044** |
     | Solid lane lines | **0** | **0** |
     | Stop rings: full-width / per lane / connector | **223 / 4 / 0** | **35 / 2 / 1** |
     | Stop lines per `(link, lane)`: at a signal / elsewhere (approach links) | **402 / 69** (188 / 38) | **71 / 0** (36 / 0) |
     | Arrows | **1,050** | **190** |
     | `through` / `through_left` / `through_right` / `left` / `right` / `left_right` / `through_left_right` / `u_turn` / `dead_end` | **788 / 111 / 119 / 13 / 12 / 2 / 5 / 0 / 0** | **121 / 29 / 30 / 2 / 2 / 1 / 5 / 0 / 0** |
     | Markings mesh: vertices, triangles | **33,662**, **22,288** | **11,946**, **7,194** |

     **And against the engine:**
     - every full-width ring's link is found within 1 mm (worst **0.000000 m**, 223 and
       35): none is left whole, and none is ambiguous;
     - every drawn vertex is a `NetworkJson` vertex or on its ring's edge, as gate 5;
     - every marking vertex lies within **1 cm** of the drawn road (the strips, the
       junction surfaces and the fills): worst **0.0063 m** in Midtown (stop lines),
       **0.0000 m** in urban_grid;
     - the streets built twice in one process are equal, value for value.
  7. **Where the first vehicle stops** (`tests/look.rs`, ignored, headless). Phase 5's
     rule (its gate 7) on each whole run, the gap measured to the back edge of the built
     stop line for the row's `(link, lane)`, 0.40 m from its junction-facing edge. The
     engine's code predicts **+0.2 m**. Measured, and predicted exactly here, since the
     FCD is fixed:
     - **at signals:** Midtown **2,490** episodes (48,896 rows), median **+0.242 m**,
       **2,441** within 0.10 m of +0.242, 27 between 1.55 and 1.75 m, **10** in the
       −0.4 m bin (front at the junction's edge) and 12 others; urban_grid **129**
       (2,555), median **+0.242 m**, **123** within, 6 between 1.55 and 1.75 m;
     - **at Midtown's new unsignalised stop lines:** **214** episodes (445 rows), median
       **+0.233 m**, **132** within 0.10 m of +0.242, 2 between 1.55 and 1.75 m, **67**
       in the −0.4 m bin and 13 others.
  - **The markings — through the GPU:**
  8. **The synthetic crossing, drawn** (`tests/look.rs`, ignored, no fixture). Gate 5's
     network, through `Renderer::new` with `scene::Camera { cx: 0, cy: 0, k: 0.05 }` at
     1280×720, a pool of 1, no buildings, no credit and no box: once as built, and once
     after `set_streets(Some(&streets))`. At the pixel holding each world point, off → on,
     with road (92, 96, 104), background (18, 22, 30), dash (174, 176, 180), centre
     (199, 167, 77), stop (239, 239, 240), arrow (190, 191, 195) and bar (217, 72, 73):

     | Point (m) | What | Off | On |
     |---|---|---|---|
     | (0, 0) | the junction | background | **road** |
     | (30, 0), (0, −15) | the 0.5 m and the 4 m fills, no line | background | **road** |
     | (−30, 0), (0, 15) | the two centre lines | road | **centre** |
     | (−26.25, −3.5), (−29.5, −3.5) | a dash, the gap after it | road, road | **dash**, road |
     | (−12.2, −3.5) | a dash's end under `L_WC`'s stop line | road | **stop** |
     | (−12.2, −5) | `L_WC`'s stop line | road | **stop** |
     | (10.2, 2), (15.2, 2) | `L_EC` lane 0 at the edge; its line 5 m back | road, road | road, **stop** |
     | (12, 3.75) | the connector | road | **stop** |
     | (−16, 1.75), (12.8, 5.5) | a departing `through`'s shaft; an arriving `through`'s head | road | **arrow** |
     | (13.5, 9.8), (12.8, 9) | `right`'s head; where a `through` head would be | road, road | **arrow**, road |
     | (18.5, 1.2), (24.9, 1.4) | `left`'s head; the separate `u_turn`'s head | road | **arrow** |
     | (−16.9, −1.15) | the lone `u_turn`'s head | road | **arrow** |
     | (−16, −4.45), (−14.8, −5.25) | `through_left`'s branch head and straight head | road | **arrow** |
     | (−16, −9.55) | `through_right`'s branch head | road | **arrow** |
     | (−0.95, 17.75), (−2.55, 17.75), (−1.75, 16.55) | `through_left_right`'s left, right and straight heads | road | **arrow** |
     | (−5.6, 16.45), (−5.25, 17.5) | `dead_end`'s bar; its shaft | road | **bar**, **arrow** |
     | (2.95, −17), (4.55, −17), (3.75, −16.3) | `left_right`'s heads; where a straight head would be | road | **arrow**, **arrow**, road |

     Over the whole frame: **243,827** pixels change; exactly **954** centre, **2,166**
     dash, **5,240** stop (stop lines and the connector), **6,546** arrow and **160** bar
     pixels. Lines here are 2–8 px wide (k 0.05), so the fade leaves them whole. Without
     the feature every coloured "on" above (centre, dash, stop, arrow, bar) fails, and so
     does every pixel count; the "on"s that are road pass either way.
  9. **The colours and the fade** (`tests/look.rs`, not ignored):
     - each kind's colour, composited from the copied hex and opacity over `scene::ROAD`,
       equals §2.18.4's table (`==` per channel), and `colour` at `α` 0 is the road's and
       at 1 each marking's linear colour (`==`);
     - `fade(0.40, 1.6009)` is **0.316** and `fade(0.30, 1.6009)` **0.122**;
       `fade(0.40, 2.401)` **0.074**; `fade(0.30, 1.1481)` **0.357** (each ±0.001); and each
       kind's fade width is §2.18.9's (0.15, 0.40, 0.30, 0.40, 0.10).
  - **The markings — renders:**
  10. **On, deterministic** (`scripts/gates-look.sh`). Each twice, each pair equal:
      - urban_grid, the default render and `--camera tests/flight.toml`: **8700 of 8700**;
      - Midtown `--from 300 --to 360 --speedup 1`: orthographic with and without
        `--buildings`; `--camera tests/city-flight.toml` with and without
        `--buildings`; `--camera tests/see-through-flight.toml` with `--buildings`:
        **1800 of 1800** each (`ffprobe` `1920,1080,30/1,1800`).

      **The look changed:** each first render differs from Phase 5's render of the same
      case (`scratch/out/streets-p5/*.framemd5`) in **at least one frame**, and from its
      `--no-streets` reference (as Phase 5's gate 10 lists them) in at least one; both
      counts are recorded. (The probe: every frame differs from Phase 5's, 1800 of 1800 and
      8700 of 8700, and from the references.)
  11. **The copy is traceable** (by reading, recorded). `src/streets.rs` names `90b39292`,
      and beside each value copied from `colors.ts` and `networkRenderer.ts` (§2.18.12's
      list) gives its file and line there; each line, read at the pin with
      `git show 90b39292:<file>`, holds that value.
  - **Recorded, with one bar:**
  12. **Render time.** Record gate 10's wall times beside Phase 5's gate 10 and gates 1–2's
      `--no-streets` renders. Prediction: as Phase 5's, since the markings mesh is
      smaller (§2.18.10) and `NetworkJson` adds 5 ms once.
  13. **`view --bench 20`** on Midtown at the default window, with `--buildings` (streets
      and see-through on by default) and with `--buildings --no-streets`. Record the JSON
      and the load average.
      - **A `mean_fps` below 30 with streets on stops the build**, and the phase goes back
        to review.
      - Prediction: at least 30, and likely 60 (vsync). The probe, at a load of 48–58: 51.18–58.49 fps, the median
        frame 15.5–16.5 ms, as Phase 5's look and `--no-streets` in the same runs
        (§2.18.10).
  - **The user's check:**
  14. **The user watches** gate 10's renders with Phase 6's markings
      (`scratch/out/look/`) against Phase 5's (`scratch/out/streets-p5/`). Times are the
      video's, from 0:00; colours as §2.18.4.
      - **Midtown, orthographic with buildings (`look-ortho-city.mp4` against
        `streets-ortho-city.mp4`), throughout.** The two look nearly the same: grey
        crossings joined to the streets, no dark seam down the wider streets. The short
        white ticks across the streets at the edges of crossings are fainter than Phase 5's
        (about a third of their colour, against three quarters), and some crossings that
        had none now have them. No white, grey or amber dot lies along a street, and no
        arrow can be made out.
      - **The city flight with buildings (`look-city.mp4` against `streets-city.mp4`).**
        From 0:00 to 0:10, high and nearly straight down: as the orthographic video. From
        0:25 to 0:35, on the cross street in the lower middle of the frame: where Phase 5
        shows sparse white dashes and an amber line down the middle, Phase 6 shows grey
        dashes about twice as dense, no amber line, and small light-grey arrows in each
        lane a few metres before each crossing's white bar and just after each crossing.
        Toward the top of the frame they fade into the grey, with nothing popping on or off
        between frames.
      - **The orbit with buildings (`look-orbit.mp4` against `streets-orbit.mp4`), 0:00
        to 1:00, the middle of the frame.** At the crossing, every approach lane has a
        light-grey arrow before the white bar, pointing at the crossing: straight, or
        straight with a short hook to the left or right. Just past the crossing each
        departing lane has a straight arrow pointing away. The bars are thinner than
        Phase 5's. The cross street has no amber line; its two directions are separated by
        grey only. The first box queued at a red light stops just short of the bar, never
        over it. Nothing flickers, and the boxes stand over the markings.
      - **urban_grid's `--camera` render (`look-ug-camera.mp4` against
        `streets-ug-camera.mp4`), from 0:55 to 2:11,** the follow at `height_m` 120 to 60:
        grey dashes, about twice as dense as Phase 5's; **no amber line anywhere**, the
        two directions of each street separated by grey only; at each crossing an arrow
        in every lane before the bar and a straight arrow in every lane after it. At the
        centre crossing, on the approach from the east, one lane's bar stands about 5 m
        behind its neighbour's, now joined to it by a thin white strip along the lane line.
      - **In `view` on Midtown:** at launch, grey crossings, their ticks barely visible
        (`α` 0.07), and no dash or arrow; scrolling in over a street, dashes, bars and
        arrows fade in smoothly; `M` hides the streets and shows them; `view --no-streets`
        opens as today, and `M` shows them.
      - **In every video, decided and watched** (§2.18.1, decision 5): a two-way street
        with a median gap shows grey between its two directions and no line; an amber
        centre line shows only on the few two-way streets with no gap (three in Midtown,
        none in urban_grid).

      Then say whether OQ-20's colours stand as they look (iteration, §2.18.4).

      *(2026-10-09, user)* **Passed on every point but one:** the space between a two-way
      street's directions ("there is no gap.. it looks like more road"). OQ-20's colours
      stand, and so do the two comment lines narrowed in `tests/streets.rs`. Decision 5 is
      changed at this gate, and gates 3, 5, 6, 8, 10 and 14 are amended (this phase's last
      item).
- **Predictions at a glance:**

  | What | Prediction | Gate |
  |---|---|---|
  | urban_grid with `--no-streets`: default and `--camera` | 8700 of 8700 against `ref-pin90b39292-default` and `-camera` | 1 |
  | Midtown with `--no-streets`: 14 renders | 1800 of 1800 against each of the eight Midtown references | 2 |
  | Other tests; `tests/streets.rs` trimmed; self-compared renders; `gates-streets.sh` | baseline numbers; kept numbers; 1800 of 1800; pass | 3 |
  | Packages; `Cargo.toml` | 0; unchanged | 4 |
  | Synthetic: centre lines, dashes, stop lines, connector, arrows by type, mesh | 2; 68; 9; 1; 35 (every type); 1,023 and 781 | 5 |
  | Midtown / urban_grid counts; against `NetworkJson`; on the road | as gate 6's table; exact; ≤ 1 cm (0.0063 m) | 6 |
  | First stopped box to the 0.4 m line's back edge, at signals | engine's code: +0.2 m; median +0.242 m both; 2,441 / 123 within 0.10 m | 7 |
  | At Midtown's unsignalised stop lines | 214 episodes, median +0.233 m, 67 at the junction's edge | 7 |
  | Synthetic pixels off → on; changed; exact colours | as gate 8's table; 243,827; 954, 2,166, 5,240, 6,546, 160 | 8 |
  | Colours; the fade per kind | §2.18.4's table; 0.316, 0.122, 0.074, 0.357 | 9 |
  | Seven renders twice; against Phase 5's and the references | equal; at least one frame differs | 10 |
  | Copied values at their lines | each holds | 11 |
  | `view --bench` with streets and buildings | ≥ 30 fps | 13 |
- **Not predicted, and so not gated:**
  - the look: the colours (OQ-20, answered (a)), the arrows' size on screen, how the
    fade's widths look (their values are gate 9's), for the user at gate 14;
  - solid lane lines: drawn by the same path as the centre lines (a ring, its edges split
    at `STRIP_STEP`), but no lane in either fixture or the synthetic crossing has
    `no_change_left`/`right`, so no gate draws one;
  - render times (gate 12), and `view`'s frame rate above 30 (gate 13).
- **Close-out (standing plan steps, the methodology's §3):**
  - **Commit plan:** one branch (`vis-002-phase-6`), one push. The commits:
    - `src/streets.rs`, `src/main.rs`'s doc comment, `tests/look.rs`'s headless gates
      (5, 6, 7, 9) and the removals in `tests/streets.rs`. The look changes here, since
      streets are on by default and the drawing is untouched;
    - `tests/look.rs`'s GPU gate 8 and `scripts/gates-look.sh` (gate 10 and gate 14's
      renders). Gates 1–4 and 8–13 run after this commit (8 through cargo, 11 and 13 by
      hand);
    - the gate run and its record;
    - the close-out.
  - **Reconciliation:**
    - **`rules/streets.md`** (`max_lines: 40`, kept) is corrected: "What is built" names
      `NetworkJson` and its five fields, the cut, the split and the glyphs; "The two
      meshes" the new kinds, lifts and colours; the fade's widths per kind. Its `sources`
      and `covers` gain the copied front-end values (the `covers` line), not new files.
      It is at its cap (40 of 40); the US markings' lines it drops make the room, and it
      is cut to fit rather than the cap raised;
    - `rules/render.md`, `rules/view.md` and `rules/camera.md`: none needed. Each says
      "streets (junctions, median fills, markings)" or "the two street meshes", which
      stays true;
    - `rules/see-through.md`, `rules/buildings.md`, `rules/parts.md`, `rules/credit.md`,
      `rules/inputs.md`, `rules/motion.md` and `rules/slider.md`: none needed;
    - `spec-lint --write-index` regenerates `rules/INDEX.md` and `specs/INDEX.md`;
    - **the README**: its streets paragraph, and the summary sentence "Its Phase 5 draws
      the streets: junctions filled in grey, and US road markings" (l. 16–17), say the
      markings are the engine dashboard's (dashes, centre line, stop lines, lane arrows)
      from `NetworkJson`, not US ones, and gains `scripts/gates-look.sh` and `cargo test
      --release --test look -- --include-ignored --test-threads=1`. The note on
      `scripts/gates-streets.sh` (l. 246, "Phase 5 gates 10, 11 and the gate 15
      renders") says its renders now show Phase 6's markings;
    - `src/main.rs`'s `--no-streets` help text: reworded in the scope (above);
    - `CLAUDE.md`: none needed, since no stanza changes;
    - status artifact: none needed, since this repo has none.
  - Record the gate results in `specs/reviews/vis-002.md`, with any missed prediction and
    its cause, and the assertions removed from `tests/streets.rs`.
  - Write this phase's `shipped` date, and turn §2.17.1's (decisions 3, 5, 8 and 9) and
    OQ-18's dated notes from "when it ships" to the date.
- **Amended at gate 14** *(2026-10-09, changed by the user at gate 14)*. Gate 14 passed on
  every point but the median gaps. Decision 5 changes (§2.18.1): a pair with a gap shows it
  as the dashboard does, the background with its junction ends rounded by
  `NetworkJson::median_noses` in the road's grey, and no median fill. The design is
  §2.18.16. Phase 6 is amended on `vis-002-phase-6` before it ships. Everything above
  stands except what this item replaces.
  - **What the amendment produces:** the run's video as above, with each two-way street's
    median gap dark between its two directions and rounded at each junction, where the
    first build drew it grey. With `--no-streets`, today's frames (gates 1 and 2,
    unchanged).
  - **Scope, in addition to the first build's:**
    - **`src/streets.rs`:**
      - removed: `Fill`, `Streets::fills`, the fill ribbon in `Streets::build`, and
        `Ribbon`, `ribbon` and `ribbon_mesh`, which only the fills used;
      - added: `Nose { polygon, triangles }` (the ring with no closing copy, and `earcut`),
        and `Streets::noses`, from `NetworkJson::median_noses` in its order, read from
        the one `NetworkJson` that `build` already makes;
      - `Streets::surface`: the junction surfaces, then the noses, at height 0;
      - `Pair` and `pairs` are kept, though nothing draws from them;
      - the module doc says so.
    - **`tests/look.rs`** (Phase 6's own):
      - gate 5's fill assertion becomes the noses' (below);
      - gate 6 adds the noses and the surface mesh;
      - gate 8's table and counts change (below).
    - **`tests/streets.rs`:** Phase 5's fill checks are removed, with the code only they
      use. A kept line may change only to drop what is removed:
      - gate 5:
        - the `st.fills.len()` assertion and the loop over `st.fills` go;
        - with them go `points`, `range`, `Line::s` and `Line::cross`, `Line`'s `start`
          and `dir` fields and their twelve initialiser lines;
        - `Line`'s doc comment loses the words naming them, and the comment `// Pairs and
          fills.` becomes `// Pairs.`;
        - the `println!` loses "2 fills";
      - gate 6:
        - the `fills` counter leaves the tuple, and its line goes;
        - `got`, both expected arrays and the `println!` lose the fill count;
        - `Expect` loses `fills` and `surface`, and the two literals lose those lines;
        - the surface-mesh assertion goes, since its values included the fills (the new
          values are `tests/look.rs`'s gate 6);
        - `let surface` goes, and the `println!` loses the surface values;
      - gate 8: the `(-20.0, 0.0, "the median fill", …)` row goes, so `[Row; 2]` becomes
        `[Row; 1]`;
      - names no longer used leave the `use` lines.

      Nothing is added, and no kept value or tolerance changes. Gate 7 (strip ends, box
      centres) is untouched: it never counted the fills.
    - **`scripts/gates-look.sh`** (Phase 6's own): gate 10 also compares each first render
      with the first build's copy in `scratch/out/look-v1/` (below).
    - **Not edited:** as above.
  - **Exit gate, as amended.** The same machine, fixtures, pin, FCDs and cache.
    - **The baseline is the branch's head when the amendment's build starts**, whose
      `src/`, `tests/` and `scripts/` equal `8db1742`'s. It runs as the
      first build's baseline did: the ten references; every gate script, `gates-look.sh`
      included, and every test file with `--include-ignored --test-threads=1`; spec-lint
      and the package count.
    - **First, before any script runs, copy this build's `scratch/out/look/` to
      `scratch/out/look-v1/`** (its fourteen `*.framemd5` and the four gate 14 videos), and
      record their SHA-256s. `gates-look.sh` writes into `scratch/out/look/`, so the
      first build's renders would be overwritten. Every `scratch/ref-*`,
      `scratch/out/streets-p5/` and `scratch/out/look-v1/` stays as it is.
    - Gates 1, 2, 4, 7, 9, 11, 12 and 13 are unchanged.
    - The others change as follows:
    - **Gate 3, what the amendment does not touch:**
      - every test file but `tests/look.rs` and `tests/streets.rs` prints the
        baseline's numbers;
      - `tests/streets.rs`, with Phase 5's fill checks removed, prints the baseline's
        numbers that remain: surfaces 106 / 9, pairs 29 / 24, `junction_fills` worst
        0.000000 m, strip ends, box centres, fade, `mpp_at`, gate 12;
      - the self-compared renders give equal pairs, and all eight scripts pass;
      - `git diff 8db1742 --stat -- src/` is `src/streets.rs`; `-- scripts/` is
        `gates-look.sh`; `-- tests/` is `tests/look.rs` and `tests/streets.rs`, the
        latter read hunk by hunk against the list above.
    - **Gate 5, the synthetic crossing,** in addition to the first build's predictions:
      - **no fill;**
      - **4 median noses** (40 vertices, 24 triangles), each vertex its `NetworkJson`
        ring's exactly:
        - two at the junction end of `L_EC`'s 0.5 m gap, `x` 10.0–10.25, `y` 0…0.25 and
          −0.25…0 (radius 0.25);
        - two at `L_SC`'s 4 m gap, `x` 0…2 and −2…0, `y` −15.5…−13.5 (radius 2);
      - the surface mesh **83 vertices, 58 triangles** (43 + 40; 34 + 24). The markings
        and their mesh are as before.

      Without the amendment it fails: the first build has 2 fills and no nose.
    - **Gate 6, the fixtures,** in addition: noses **94** (940 vertices, 564 triangles)
      and **72** (720, 432); the surface mesh **5,344 and 4,042** and **1,107 and 738**;
      no fill. Every marking vertex lies on the drawn road, now the strips, junction
      surfaces and noses: worst **0.0063 m** in Midtown (325 stop-line vertices just off)
      and **0.0000 m** in urban_grid. Built twice, equal.
    - **Gate 8, the synthetic crossing, drawn.** The table's rows change:

      | Point (m) | What | Off | On |
      |---|---|---|---|
      | (30, 0) | the 0.5 m gap | background | **background** (was road) |
      | (0, −15) | inside the 4 m gap's rounded end | background | **background** (was road) |
      | (1.8, −13.7), (−1.8, −13.7) | the 4 m gap's two noses | background | **road** |

      Every other row stands. Over the whole frame **233,007** pixels change (was 243,827).
      The exact counts stand: 954 centre, 2,166 dash, 5,240 stop, 6,546 arrow, 160 bar.
      Without the amendment, (30, 0), (0, −15) and the pixel count fail.
    - **Gate 10, on, deterministic, and not the first build's.** As before, and each first
      render also differs from the first build's
      (`scratch/out/look-v1/look-<case>.framemd5`) in **at least one frame**, the count
      recorded. The probe: the seven cases twice, each pair equal (8700 of 8700, 1800 of
      1800); each first render differs from the first build's in every frame (8700 of 8700
      and 1800 of 1800 each), and from Phase 5's and the reference likewise; Midtown's
      ortho with `--no-streets` 1800 of 1800 against `ref-pin90b39292-ortho-roads`.
    - **Gate 14, the user's check, rewritten.** The user watches the amended renders
      (`scratch/out/look/`) against the first build's (`scratch/out/look-v1/`). Times are
      the video's, from 0:00. The points passed at the first build's gate 14 (the
      markings, the arrows, the bars, the stopped boxes, the fade, `view`'s `M`) are
      looked at again only for change. What changes:
      - **Midtown, orthographic with buildings (`look-ortho-city.mp4` against
        `look-v1/look-ortho-city.mp4`), throughout:** along the 15 wide two-way streets
        a dark band, 2–3 px, runs between the two directions where the first build
        showed grey. Along the 11 with a 0.5 m gap, a thin dark line. Each stops at the
        crossings, which stay grey. The rest is as the first build.
      - **The city flight with buildings (`look-city.mp4`):**
        - from 0:00 to 0:10, high up, as the orthographic video: thin dark lines along
          the wider streets;
        - from 0:25 to 0:35, low over the cross street in the lower middle of the frame:
          a dark strip between its two directions, ending at each crossing in two
          rounded grey corners. No amber line in it;
        - the dashes, arrows and bars are as the first build.
      - **The orbit with buildings (`look-orbit.mp4`), 0:00 to 1:00, the middle of the
        frame:**
        - where the cross street has a gap, it is dark between its two directions, with
          rounded grey corners at the crossing;
        - the boxes drive on the grey, none over the dark gap.
      - **urban_grid's `--camera` render (`look-ug-camera.mp4`), from 0:55 to 2:11:**
        every street's two directions are parted by a thin dark line (its gaps are all
        0.5 m), rounded at each crossing, and no amber line anywhere.
      - **In `view` on Midtown:**
        - at launch, the thin dark lines and bands along the two-way streets;
        - scrolling in, the rounded ends at the crossings;
        - `M` hides the streets, and the gaps look as before: the same dark, since
          `--no-streets` shows the background there too;
        - `view --no-streets` opens as today.
      - **In every video:** an amber centre line only on Midtown's 3 two-way streets with
        no gap, none in urban_grid; every other two-way street shows its gap dark.

      Then say whether the gaps now read as gaps.
  - **Predictions, old → new:**

    | What | First build | Amended | Gate |
    |---|---|---|---|
    | Synthetic: fills; noses; surface mesh | 2; 0; not predicted (Phase 5's) | **0; 4 (40, 24); 83 and 58** | 5 |
    | Fixtures: fills; noses | 26 / 24; not drawn (94 / 72 in `NetworkJson`) | **0; 94 / 72** | 6 |
    | Surface mesh, Midtown / urban_grid | 10,510 and 9,532 / 13,971 and 13,842 (Phase 5's) | **5,344 and 4,042 / 1,107 and 738** | 6 |
    | Markings on the drawn road | ≤ 0.0063 m / 0.0000 m, against strips, surfaces, fills | **the same**, against strips, surfaces, noses (Midtown: 325 just off, was 316) | 6 |
    | Box centres on the drawn road (Phase 5's gate 7) | all; 0 in a gap | **all; 0 in a gap**, unchanged | Phase 5's 7 |
    | Synthetic pixels: (30, 0), (0, −15) | road | **background** | 8 |
    | Synthetic pixels: the 4 m gap's noses | — | **road** | 8 |
    | Synthetic pixels changed | 243,827 | **233,007** | 8 |
    | Exact colour pixels | 954, 2,166, 5,240, 6,546, 160 | **the same** | 8 |
    | Renders against the first build | — | **differ in ≥ 1 frame** (probe: every frame, each case) | 10 |
    | `view --bench 20`, streets on | ≥ 30 fps | **≥ 30 fps** (probe: 60.00 fps, median 16.66 ms, with streets on and with `--no-streets`, at a load of 7–8) | 13 |
  - **Close-out of the amendment (standing plan steps, the methodology's §3):**
    - **Commit plan:** the same branch, one push. The commits:
      - `src/streets.rs`, `tests/look.rs`'s gates 5, 6 and 8, the removals in
        `tests/streets.rs`, and `gates-look.sh`'s comparison with `look-v1`;
      - the amendment's gate run and its record;
      - the close-out.
    - **Reconciliation:**
      - `rules/streets.md` (cap 40, kept): the fills go and the noses come, in "What is
        built" and "The two meshes", and its `covers`;
      - `rules/render.md` l. 39, "(junctions, median fills, markings)", becomes
        "(junctions, median noses, markings)";
      - the README's streets paragraph loses "the gap between a two-way street's halves
        is filled too" and says the gap shows dark with rounded ends, as on the
        dashboard;
      - `spec-lint --write-index`;
      - `CLAUDE.md`: none needed.
    - Record the amendment's gate run in `specs/reviews/vis-002.md` as its own entry,
      with the `tests/streets.rs` hunks. Then gate 14. Then `shipped`, and the "when it
      ships" notes of §2.17.1 (decisions 3, 5, 8, 9), §2.17.5 and OQ-18 dated.
