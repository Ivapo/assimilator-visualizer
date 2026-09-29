---
title: motion
sources:
  - src/motion.rs
  - src/place.rs
  - src/lib.rs
covers: >
  How each vehicle's FCD rows become a position at any time: Hermite along links,
  junction spans on the engine's turn paths, lane slides, when a vehicle is drawn,
  and the motion report.
max_lines: 60
generated: 2026-09-29
---

# Motion

**What is true right now.** Corrected freely against the sources above.

## Tracks, built before the first frame
- `Motion::build` takes every vehicle's rows from the **whole file** (`Fcd::vehicles`),
  in `time` order, not only the window's. Nothing in it can fail: a missing movement is
  drawn as a chord and counted.
- A track is a time-ordered list of along-link intervals and junction spans. `odo` is
  the distance along the drawn path since the first row, lateral motion excluded.

## Along a link (consecutive rows on one link, outside a span)
- Cubic Hermite in `u = (t − t_i)/Δt` through `s_i`, `s_{i+1}`, with tangents `v·Δt`.
- `Δs = 0`: both tangents 0, so the box stands. `Δs < 0`: it holds at `s_i`, then jumps at
  `t_{i+1}` (counted as `backward`). `Δs > 0` with `α² + β² > 9`: both tangents scale by
  `3/√(α² + β²)` (counted as `clamped`).
- Point: `interpolate_with_lateral(link, s, lateral)`. The lateral offset is the lanes'
  `lane_offset` blended by smoothstep `3u² − 2u³`. The heading is the link's.

## Through a junction (a span, rows A…D)
- D is a row whose link differs from the row before it. `p_f` is D−1's position. A is the
  last row of that visit below `p_f`, else the visit's first row. A transit with no row D
  is not a span, and its frozen rows stand.
- Movement: `resolve_route_pair(to_node(a), a, b)`.
  - Junction or waypoint movement: `lane_turn_path(entry, dep)` (step 1); else
    `lane_turn_path(entry, resolve_departure_lane(...))` (step 2); else `turn_path`, the
    centreline (step 3).
  - Transparent, or nothing found: the chord `place(a, L_a, entry)` → `place(b, 0, dep)`.
- Path: approach `[s_A, L_a]`, then the turn path (`length`), then departure `[0, s_D]`.
  `G` is their sum. `I(t)` is the trapezoid integral of FCD speed from `t_A`.
  `d(t) = G·I(t)/I`, or `G·(t − t_A)/(t_D − t_A)` when `I < 1 mm`.
- `r = G/I`. `r′ = (G − (L_a − p_f))/I` is out of band outside `[0.8, 1.25]` (counted,
  drawn the same).
- Lateral: the base is the path's lane on each link (0 for a centreline). A's lane slides
  to the entry lane over `[t_A, t_{A+1}]`, and the path's departure lane slides to D's
  over `[t_{D−1}, t_D]`. On the turn path or chord the slide is applied perpendicular
  (`apply_perpendicular_offset`).
- A turn-path point is `spatial_conflict::interpolate_pos`. Its heading is
  `interpolate_heading`'s θ as `(90° − θ°)` wrapped to 0–360 (`Placement::turn_point`).

## Drawn
- A vehicle is drawn when `t_first − 1e-6 ≤ t ≤ t_last + 1e-6`, with `t` clamped into
  its rows, and never extrapolated. The renderer's box pool is the largest number drawn
  at once (`Motion::max_drawn`).
- `Job::boxes_at(t)` gives, in `vehicle_id` order: `vehicle_id`, the placed point, the
  length, the FCD speed (linear between rows, for the colour) and a `TrackPos`.
  - `TrackPos` is a piece: `Link`, `Turn {a, b, entry, dep}` (the path's own departure
    lane, `None` on a centreline) or `Chord`.
  - It also carries `along`, `lateral` and `odo`.

## Motion report (`Job::motion_report()`, never printed)
Along-link intervals (clamped, backward), spans with how the path was found, `G`, `I`,
`r`, `r′` and the shortfall, the out-of-band count, and lane slides (along, entry,
departure). urban_grid, seed 42:
- 19 508 intervals, 317 clamped;
- 422 spans: 421 step 1, 1 step 2;
- `r` 1.024–1.196, `r′` 0.970–1.050;
- slides: 135 along, 9 entry, 1 departure.
