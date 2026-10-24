---
title: vehicles
sources:
  - src/vehicles.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/mod.rs
  - src/view/state.rs
  - src/main.rs
covers: >
  Vehicle shapes in the 3D views: the table read from the FCD and its errors, the kinds
  and the class names, each kind's parts and their sizes, the stretch, the paints, the
  brake rule, the headlamps, the identical key, `--no-shapes` and `view`'s `V`.
max_lines: 50
generated: 2026-10-23
---

# Vehicles

**What is true right now.** Corrected freely against the sources above.

## The table (`vehicles::read`, no Bevy types)
- From the run's FCD file (`rules/inputs.md`): every row's `time`, `vehicle_id`, `speed`,
  `acceleration` and `vehicle_class`, sorted by (`vehicle_id`, `time`). A vehicle's kind is
  its first row's; class names are mapped once per dictionary value, never through a map walk.
- `vehicle_class` is `Dictionary(Int32, Utf8)` or `Utf8`; `acceleration` is `Float64`. Without
  `vehicle_class` every vehicle is plain; without `acceleration` a row brakes only standing. A
  null reads as the column missing for its row; a null `time`, `vehicle_id` or `speed` is an error.
- Either column of another type, exit 1, before the encoder or the window:
  `schema mismatch: <file> column "vehicle_class" is <type>, expected Dictionary(Int32, Utf8) or Utf8`,
  or `… column "acceleration" is <type>, expected Float64`.

## Kinds and parts (`kind`, `parts(kind, length)`)
- `kind(name)` is exact: `car`, `truck`, `bus`, `bike`; any other name and none: `Plain`.
- Cuboids in the vehicle's frame (`x` forward along the heading, `y` up, `z` right; metres),
  stretched along `x` to the box's length `L`, widths and heights fixed, on `FLOOR` 0.05 m.
  Lamps and panes are `T` 0.02 m thick, proud of their face. The table is `parts`:
  - car (7): body 1.8 m wide to 0.85 m; cabin from −0.32·`L` to 0.16·`L`, to 1.5 m; the lamps;
  - truck (11): a dark chassis to 0.95 m; a cab `min(0.35·L, 2.3)` long to 3.1 m; a gap
    `min(0.03·L, 0.25)`; the box to 3.6 m, 2.5 m wide; windscreen, side windows; the lamps;
  - bus (10): body 2.55 m wide to 3.1 m; side window bands, windscreen, rear window; the lamps;
  - bike (2): a dark frame 0.08 m wide to 0.75 m and a rider to 1.65 m; no lamps;
  - plain (6): today's 1.8 × 1.5 m box with the car's lamps.
- The lamps: at the back a dark tail panel with two brake lamps on it; at the front two headlamps.

## Paint (`draw::shapes_mesh`) and the brake rule (`brake`, `Fleet::looks`)
- Body: the speed colour (`rules/render.md`), or `CARS` while link colours are on, × the face's
  shade (top 1, sides 0.55, ends 0.4, bottom 0.25); the cabin's top is body, its sides glass.
  `GLASS` (34, 42, 56) and `DARK` (38, 38, 42) shaded. Lamps unshaded: `BRAKE_ON` (239, 68, 68)
  while braking, `BRAKE_OFF` (92, 22, 26) otherwise; `HEAD_ON` (255, 240, 196) always.
- Braking: `acceleration < 0 || speed < STANDING` (0.01 m/s) of the row at or before `t`
  (`time ≤ t + 1e-6`), the first row before that. An id not in the table: plain, unlit.

## Drawing and switches
- Perspective only (`view`, `render --camera`): one mesh in `vehicle_id` order in the boxes'
  place, each part the shaded unit cuboid scaled into the vehicle's frame and placed as a box
  (yaw `90° − heading`, 0.001 m a rank). A box whose placed point, heading, length and kind
  equal a later box's bit for bit is not drawn. The orthographic render draws boxes;
  `Renderer::render_looks(boxes, Some(looks))` draws `set_shapes`, `render(boxes)` passes `None`.
- `render`: on unless `--no-shapes`, which gives today's frames byte for byte. `Job::set_shapes`
  (after `set_links`) reads the table, a no-op on the orthographic job, off from every
  `prepare*`; `Job::looks_at(t)` reads the looks back.
- `view`: the table is read at launch after the link window's check, flag or not; its error is
  `view`'s unless `--no-shapes`, where it leaves no table. `shapes_shown` (off at
  `ViewState::new`) is set from the flag; `V` (by position), after `C`, flips shapes and boxes.
