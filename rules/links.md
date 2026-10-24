---
title: links
sources:
  - src/links.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/mod.rs
  - src/view/state.rs
  - src/main.rs
covers: >
  Link colours: the window and the run's FCD interval, the four measures and their units,
  the steps and their rounding, the palette, the tint and its lift, the cars' one colour,
  the legend, `--link-colours`, `--link-window`, `--link-lanes`, `view`'s `C` and the two
  window errors.
max_lines: 40
generated: 2026-10-23
---

# Links

**What is true right now.** Corrected freely against the sources above.

## Rows and window (`links::Links`, no Bevy types)
- `Links::new`: a unit per link in network order, or per lane with `--link-lanes`. `with_units`
  takes every row of `Fcd::vehicles` in (`time`, `vehicle_id`) order, to its link's unit or its
  lane's (past the last: the last), its limit its own lane's; names are only looked up.
- At `t` with `W` s: the rows with `t − W + 1e-6 < time ≤ t + 1e-6`, `n` their distinct FCD
  times; nothing is carried between frames. The interval `I`: the smallest gap between
  successive FCD times, `None` with one.

## Measures and steps (`values_at`, `step`, `colours_at`)
- `relative`: mean `speed / limit`; `speed`: mean `speed` (both `None`, today's road, with
  no row). `density`: rows / `n` / (length × lanes / 1000), length `Placement::link_length`;
  `stopped`: rows under 0.1 m/s / `n` (both 0 with no row).
- Steps, 0 best: `relative` on `⌊100·r + 0.5⌋` % at 25 / 50 / 75 and `speed` on `⌊3.6·v +
  0.5⌋` km/h at 10 / 20 / 30, faster better; `density` at 3 / 6 / 12 and `stopped` at 0.25 /
  0.5 / 1, lower better. An edge is in the step above. `PALETTE`, best → worst: `#2c7bb6`,
  `#9ecae1`, `#fdae61`, `#d7191c`.

## Drawing (`links::tint`, `draw::spawn_tint`, `set_tint`, `spawn_legend`)
- Each unit's strip on the road strip's samples, across its span (`±total_width/2`, or its
  lane's), at `LIFT_M` 0.005 m: over the road, under markings, dots and vehicles; junctions grey.
  One unlit mesh in unit order (`cull_mode: None`, `NoFrustumCulling`), replaced on a change.
- While on, every body is `CARS` (235, 235, 235): the pool's materials and the mesh's body colour.
- The legend, one row at (`m`, `m + 1.6·S`), or (`m`, `m`) with the HUD off, at the HUD's
  `S`, fill and outline: the title, then per step a `round(0.75·S)` px swatch with an `o`-px
  black border and its range, ascending; `0.4·S` after a text, `0.25·S` after a swatch.

## Switches and errors
- `render --link-colours [relative|speed|density|stopped]` (bare: `relative`), `--link-window
  <s>` (60) and `--link-lanes`, the last two needing it (exit 2). Off: nothing read or spawned.
  `Job::set_links` runs after `set_hud`; `render_at` colours after the dots, before the vehicles.
- Exit 1: `--link-window must be a positive number of seconds (got <W>)`; then, for `W < I −
  1e-6`, `--link-window <W> s is shorter than the run's FCD interval (<I> s); --link-window
  <I> colours by each frame's own FCD row` (`I` to 1 ms).
- `view`: checked at launch with `--link-colours` or `--link-window`, else a refused window
  leaves `C` inert. `C` (by position), after `H`, cycles off → each measure → off.
