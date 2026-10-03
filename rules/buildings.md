---
title: buildings
sources:
  - src/buildings.rs
  - src/bin/network-extent.rs
  - scripts/fetch-buildings.sh
  - src/draw.rs
  - src/view/state.rs
  - src/view/mod.rs
covers: >
  Buildings around a georeferenced network: the fetch script and its report, the cache
  format and its checks, the projection, the height rule, the mesh, the light, the
  orthographic eye rule and `view`'s `B`.
max_lines: 60
generated: 2026-10-02
---

# Buildings

**What is true right now.** Corrected freely against the sources above.

## The fetch (`scripts/fetch-buildings.sh`, the only step that uses the network)
`scripts/fetch-buildings.sh --project <dir> --out <file.geojson> [--scenario baseline]
[--margin 250] [--release 2026-09-23.1]`, with the DuckDB CLI (httpfs, spatial).
- The box is `network-extent`'s line `W S E N`: the scenario's resolved network's extent
  (every node and link `geometry` point) plus `--margin` m, through the inverse projection,
  rounded outward to 7 decimals. A network with no `map_origin` is one error line, exit 1.
- DuckDB (HTTP timeout 120 s, 8 retries) reads the release's `theme=buildings/type=building`
  on `s3://overturemaps-us-west-2`: every building whose bbox meets the box, whole, as `id,
  height, num_floors, sources, geometry` (`sources`: its datasets, distinct and sorted),
  `ORDER BY id`, to GeoJSON (CRS84, 7 decimals), the release in `description` (`rules/credit.md`).
- It writes `buildings.geojson` in a temporary directory beside `--out` (GDAL names the
  collection after the file) and moves it: the same release and box give the same bytes.
- Stdout is one JSON line, `release`, `bbox`, `buildings`, `height`, `num_floors`,
  `default`, `sources` (dataset → buildings naming it), `no_sources`, `bytes`, `seconds`,
  `out`, counted over the written file. An error, a release not on S3 included, is one
  stderr line and a non-zero exit, with nothing at `--out`.

## The cache and its checks (`buildings::read`)
- A `FeatureCollection`. A feature is one building: a string `properties.id` and a
  `Polygon` or `MultiPolygon` of `[lng, lat, …]` rings. Other members are ignored here.
- `render` reads it after the run and `--camera`'s file; `view` after the run. Each error is
  `error: --buildings <file>: …`, exit 1. In order: unreadable; not JSON; not a
  FeatureCollection with `features`. Then per feature in file order, named by its `id`
  (else `feature N`): no string id; no geometry, or not a polygon type; a ring under 4
  positions, not closed, or with a position that is not two or more numbers; a longitude
  outside ±180 or a latitude outside ±90; a used `height` not positive and finite, or a used
  `num_floors` not an integer ≥ 1. Then no `metadata.map_origin`. Then no building's bbox
  meets the network's extent (no margin). Every feature is drawn; none is clipped.

## Projection and height
- `lnglat_to_xy(map_origin, lng, lat)` is the engine's `wgs84_to_metric`, copied with its
  operation order: `x = (lng − lng0)·111320·cos(lat0·π/180)`, `y = (lat − lat0)·111320`.
  `xy_to_lnglat` is its inverse.
- Height is `height`, else `num_floors` × 3.5 m, else 10 m; the counts per rule and the
  tallest are kept.
- Each ring is projected and loses its closing position; exteriors are turned
  counter-clockwise and holes clockwise, seen from above.

## Mesh (`building_mesh`, f64 world metres)
- Walls: one quad per ring edge, holes included, `z` 0 to the height, its normal to the
  right of the ring (out of the solid), wound counter-clockwise from outside.
- Roofs: each polygon through `earcut` at `z` = height, normal up. No floor.
- `draw::buildings_mesh` bakes one Bevy mesh at the fit's centre like the roads, converting
  to f32 there, with `NoFrustumCulling`. The camera fit ignores buildings.

## Look (`draw::spawn_buildings`, only with `--buildings`)
- One lit material, neutral grey sRGB (188, 188, 188), roughness 1, reflectance 0.
- A `DirectionalLight` of 4,000 lux, no shadows, along `sun_direction(210°, 60°)`, Bevy
  `(1/4, −√3/2, −√3/4)`; `GlobalAmbientLight` 250. Roads and boxes stay unlit.
- Without `--buildings` nothing is spawned. The orthographic eye (`draw::ortho_eye`) is
  `max(500, tallest + 10)` m up with far = eye + 500 m: (500, 1000) without buildings.

## `B` in `view`
`B` (by position) flips `ViewState::buildings_shown`, true at launch, at any moment, and
changes nothing else. Each frame the mesh's and the sun's visibility follow it.
