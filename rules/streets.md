---
title: streets
sources:
  - src/streets.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/state.rs
  - src/view/mod.rs
  - src/main.rs
covers: >
  Streets: the junction surfaces, the two-way pairs and their median fills, the engine
  dashboard's markings drawn from `NetworkJson` (centre lines, dashes, solid lines, stop
  lines cut per lane, connectors, lane arrows), the colours and glyphs copied from its
  front end at 90b39292, the lifts, the fade, the two meshes, the default, `--no-streets`
  and `view`'s `M`.
max_lines: 40
generated: 2026-10-08
---

# Streets

**What is true right now.** Corrected freely against the sources above.

## What is built (`Streets::build`, no Bevy types)
- Network order, `NetworkJson`'s order in its fields; every map is only looked up.
- **Junction surfaces:** each node's `NetworkData::junction_polygons` entry, in node order,
  a closing copy of the first vertex dropped, under 3 vertices skipped, `earcut` triangles.
- **Pairs:** `B` is `A`'s twin when `B.from = A.to` and `B.to = A.from` (the first such link);
  drawn once along `A`, the one first in link order. `g = (g_A + g_B)/2`; a gap over 0 is
  filled between laterals `−w_A/2 − g` and `−w_A/2` (a ribbon, road grey, 1 m steps).
- **Markings:** `NetworkJson` built once; its `median_lines` (only on a 0 m gap),
  `lane_marking_dashes`, `solid_lane_lines`, `stop_lines`, `lane_arrows`, closing copies
  dropped, `earcut`; centre and solid lines' edges split at `STRIP_STEP` (1 m).
- **Stop lines:** a full-width ring is matched to the link whose `place_lateral(L −
  stop_line_offset)` lies nearest its front edge's midpoint (`[1]`–`[2]`), within 1 mm,
  and cut on its own edges into one quad per lane (lane 0 leftmost); unmatched, it is kept
  whole with no link. Per-lane rings keep their link and lane; connectors stay whole.
- **Arrows:** `glyph` copies `networkRenderer.ts`'s nine glyphs as simple polygons (and
  `dead_end`'s red bar), any other type straight; `place` puts them at `(x, y, heading)`.

## The two meshes (`Streets::surface`, `markings`; `draw::spawn_streets`)
- **Surface:** junctions then fills, height 0, the road's own material. **Markings:** by
  kind (centre, dash, solid at `LIFT_LINES_M` 0.01; stop, connector, arrow, bar at
  `LIFT_TOP_M` 0.02), each in build order, under every box's base. Per-vertex colour on
  one unlit white material, `cull_mode: None`, `NoFrustumCulling`; none if empty.
- Colours (`COLOURS`): the dark palette of `colors.ts` composited at its opacities over
  `scene::ROAD`, rounded once: lines (174, 176, 180), centre (199, 167, 77), stop (239,
  239, 240), arrow (190, 191, 195), bar (217, 72, 73). Each copy names its line there.

## The fade (`fade`, `colour`, `mpp_at`) and where it is on
- `α = smoothstep((w/mpp − 0.1)/(0.5 − 0.1))`, `w` by kind (`Kind::width`): lines 0.15,
  stop and bar 0.40, arrow 0.30 (its shaft), connector 0.10. `(1 − α)·road + α·marking`
  in linear `f32`, from six linear colours computed once; opaque.
- `mpp`: `k` orthographic, or `|p − eye|·2·tan 22.5°/H` per vertex at a pose (`view`: each
  frame); `draw::set_street_colours` replaces the markings' mesh when a colour changes.
- `render`: `Job::set_streets(true)` unless `--no-streets` (today's frames, nothing built);
  `Renderer::set_streets` spawns and settles again; each `set_pose` recolours.
- `view`: built at launch; `M` flips `streets_shown` (on unless `--no-streets`), only.
