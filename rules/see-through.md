---
title: see-through
sources:
  - src/see_through.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/state.rs
  - src/view/mod.rs
  - src/main.rs
covers: >
  See-through: which buildings stand between the camera and the point it looks at, the
  heights they are drawn at, how the building mesh is rebuilt, where it is on by default,
  `--no-see-through` and `view`'s `X`.
max_lines: 40
generated: 2026-10-04
---

# See-through

**What is true right now.** Corrected freely against the sources above.

## The wedge (`see_through::wedge`, no Bevy types)
- At a pose, `l = (cx, cy)` and `g` is `Pose::eye`'s ground point. The half-width is
  `r = WIDTH·height_m·cos_deg(pitch)` (`WIDTH` 0.15), exactly 0 straight down; the ease
  band is `e = max(r, EASE_MIN_M)` (10 m).
- The wedge is the hull of `g` and the disc of radius `r` about `l`: the triangle from `g`
  to its two tangent points, and the disc; only the disc when `|gl| ≤ r`.
- `distance` is `δ`, the ground distance from a building's footprint (every polygon,
  holes outside, even-odd) to the wedge: the smaller of its distance to the triangle and
  its distance to `l` less `r`, at least 0.

## The heights (`see_through::height`, `heights` in file order)
- `h` is `Building::top()` (`rules/parts.md`). `δ ≥ e`: `h`, the same `f64`.
- `δ < e`: `min(h, STUB_M + (h − STUB_M)·smoothstep(δ/e))`, `STUB_M` 3 m, `smoothstep(x) =
  x·x·(3 − 2x)`; under 3 m it keeps `h`. `δ` is the building's footprint's, never a part's.
- Continuous in the pose: no pop. A part wholly above `h′` lies as a lid at `h′`.

## Drawing (`draw::BuildingsCut`, `draw::set_building_heights`)
- `BuildingsCut::new` keeps each `building_mesh`'s `f64` positions in file order, each
  vertex's building (its parts' too), and normals and indices. `mesh(heights)` gives every
  vertex `z′ = min(z, heights[b])`, baked like `draw::buildings_mesh`; with tops it equals it.
- `set_building_heights` replaces the asset behind the buildings' `Mesh3d` (`Assets::insert`).
  Still one mesh, one draw, opaque; the boxes are never touched.

## Where it is on
- `render`: on with both `--buildings` and `--camera` (`Job::set_see_through(true)` after
  `prepare_with`). `Renderer::set_see_through(Some)` keeps a copy of the buildings; each
  `set_pose` replaces the mesh when a height differs from the last drawn. Never on an
  orthographic render or without buildings.
- `Job::prepare*` build every job with it off, so the library's renders are unchanged.
- `--no-see-through` (`render` and `view`) turns it off: no cut is built and no mesh replaced,
  so every frame is today's. Accepted with no effect where nothing could be cut.
- `view --buildings` starts with `ViewState::see_through` on; `X` (by position) flips it at
  any moment and changes nothing else. The cut is built the first time it is on. Heights
  follow the state's pose when it and `buildings_shown` are on, else each `top()`;
  the mesh is replaced on a change, and every frame of `--bench` while it is on.
