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
  Streets: the junction surfaces, the two-way pairs and their median fills, the US
  markings and their constants, the stop lines, the lifts, the fade, the two meshes, the
  default, `--no-streets` and `view`'s `M`.
max_lines: 40
generated: 2026-10-06
---

# Streets

**What is true right now.** Corrected freely against the sources above.

## What is built (`Streets::build`, no Bevy types)
- Nodes, links and lanes in network order; the engine's maps are only looked up.
- **Junction surfaces:** each node's `NetworkData::junction_polygons` entry, in node order,
  a closing copy of the first vertex dropped, under 3 vertices skipped, `earcut` triangles.
- **Pairs:** `B` is `A`'s twin when `B.from = A.to` and `B.to = A.from` (the first such link);
  drawn once along `A`, the one first in link order. `g = (g_A + g_B)/2`; the gap lies
  between laterals `−w_A/2 − g` and `−w_A/2`. A gap over 0 is filled (a ribbon, road grey).
- **Yellow** (`LINE_M` 0.15, `DOUBLE_SPACE_M` 0.10), the pair's whole length: under
  `FLUSH_MIN_M` (1 m) one double line about the gap's middle; else a double line inside
  each edge, its outer line flush (a flush median).
- **Stop lines** only where a link ends at a `control: signal` junction: one per lane,
  across its width, from `s1 = L − lane_stop_line_offset(link, k)` back `STOP_M` (0.60).
- **Lane lines** between lanes `k`, `k+1`, at `offs[k] + w_k/2 + gap_after/2`: `DASH_M` 3 m
  every 12 m (`GAP_M` 9) from `s` 0 to the furthest-back stop line, a cut dash drawn short.
- A ribbon takes `max(1, ceil((s1 − s0)/1 m))` steps on `place_lateral(link, s, 0)`, offset
  along the right normal: 2 vertices a sample, 2 triangles a step, nothing shared.

## The two meshes (`Streets::surface`, `markings`; `draw::spawn_streets`)
- **Surface:** junctions then fills, height 0, the road's own material (a tie with the
  strips gives the same colour). **Markings:** yellow, lane, stop, each in link order;
  yellow at `LIFT_YELLOW_M` 0.01, white at `LIFT_WHITE_M` 0.02, under every box's base.
  Per-vertex colour on one unlit white material, `cull_mode: None`, `NoFrustumCulling`.
- `WHITE` (235, 235, 235), `YELLOW` (230, 170, 20); a mesh with no triangle is not spawned.

## The fade (`fade`, `colour`, `mpp_at`) and where it is on
- `α = smoothstep((w/mpp − FADE_FROM_PX)/(FADE_TO_PX − FADE_FROM_PX))`, 0.1 → 0.5 px, with
  `w` the line's width (0.60 for a stop line). Colour `(1 − α)·road + α·marking` in linear
  `f32`, from three linear colours computed once; opaque.
- `mpp` is `k` orthographic (once), or `|p − eye|·2·tan 22.5°/H` per vertex at a pose;
  `draw::set_street_colours` replaces the markings' mesh when a colour changes.
- `render`: `Job::set_streets(true)` after `prepare_with` unless `--no-streets`;
  `Renderer::set_streets` spawns and settles again; each `set_pose` recolours. `prepare*`
  leave it off. `--no-streets` builds and spawns nothing: today's frames.
- `view`: built at launch; `M` (by position) flips `streets_shown` (on unless
  `--no-streets`) and nothing else. While shown, colours follow the pose (every `--bench` frame).
