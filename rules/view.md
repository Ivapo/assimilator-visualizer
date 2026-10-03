---
title: view
sources:
  - src/view/mod.rs
  - src/view/state.rs
  - src/run.rs
  - src/draw.rs
  - src/main.rs
covers: >
  The `assimilator-video view` window over a finished run: its CLI and checks, the
  wall-clock view clock, the camera (orbit and tilt, pan, zoom, centre bound), picking
  and following, the readout, the keyframe line and the hidden `--bench`. The time
  slider is `rules/slider.md`; the pose and the line's format are `rules/camera.md`.
max_lines: 60
generated: 2026-10-03
---

# View

**What is true right now.** Corrected freely against the sources above.

## CLI
`assimilator-video view --project <dir> --scenario <name> --seed <n> [--results] [--fcd]
[--from] [--to] [--width 1280] [--height 720] [--buildings <file>]`: logical, positive.
- `run::load` runs `render`'s checks after width and height, then `--buildings`'s, before
  any `App` is built; no ffmpeg. An error is one `error: …` line, exit 1, no window.
- Stdout carries only keyframe lines, one per `K`, flushed. Stderr carries only the
  error line or `--bench`'s JSON. Closing the window exits 0.
- One Bevy app on the main thread (a window titled `assimilator-video view`, vsync). Per
  frame `track_pointer`, then input → `ViewInput` → `ViewState::frame` → the drawing.
- Hidden `--bench <s>`: from `t = 140` at 2× at the launch fit, skips 3 s, records `s` s of
  frame times, prints `{"frames", "mean_fps", "median_ms", "p99_ms", "worst_ms"}`, exits 0.

## Drawing (`src/draw.rs`, shared with `render`)
- The road mesh and the boxes' one mesh (`draw::set_boxes` each frame, 0.001 m lift) are
  baked relative to the launch fit's centre `(fx, fy)`. The camera is perspective at the
  state's pose (`height_m = H·k`), set every frame by `draw::perspective`.
- The readout is a 14 px white UI text line at left 16, bottom 30, just above the slider.
- `--buildings` adds their mesh and sun; `B` hides and shows them (`rules/buildings.md`).

## Clock (`ViewState`, no Bevy types)
- `t` starts at `from`, paused, at 1×. Playing: `t += s · min(Δ, 0.1 s)` of real time.
  `t` stays in `[from, to]`; reaching `to` pauses, and space at `to` restarts at `from`.
- Speed `s = 2^(i−3)`, 1/8× … 64×. Keys go by logical character (`ButtonInput<Key>`), so
  any layout works: `+`/`=` doubles, `-` halves; keypad `+`/`−` by position. Each stops at
  its end, and held keys never repeat an action.
- `←`/`→` pause and step `1/30` s. With Shift: to the latest snapshot below `t − 1e-6` or
  the earliest above `t + 1e-6` (the snapshots include the one before `from`); none, no move.

## Camera
- Launch: `render`'s fit at the launch size (`k_fit`), yaw 0, pitch 90; the fitted
  rectangle (strips' bbox + 20 m), fixed across resizes, bounds the centre.
- **Orbit:** a right press, or Ctrl + left (fixed at the press), off the bar, with no
  orbit, left press or scrub on; to that button's release `yaw ← wrap(yaw₀ − 0.25·dx)`,
  `pitch ← clamp(pitch₀ + 0.25·dy, 25, 90)`, with no pan, pick or scrub. `Q`/`E` ∓15° yaw,
  `R`/`F` −/+5° pitch, by position; not during an orbit, a left press or a scrub.
- The cursor's ground point is `ray_to_plane(pose, cursor, 0)`. A left press drags at a net
  4 px, keeping the press's ground point under the cursor; a zoom re-anchors it. `WASD` pan
  `0.5·W·k` m/s along the image's up and right on the ground.
- Scroll `n` lines (trackpad: 20 px a line): `k ← clamp(k·1.1^−n, 0.02, 2·k_fit)` about
  the cursor's ground point, or about the centre while following.

## Pick and follow
- A release that never became a drag picks the box nearest the cursor's point at 0.80 m,
  by distance to its `length × 1.8 m` footprint at its heading, within `8·k` m; ties go to
  the higher `vehicle_id`. No box in reach changes nothing.
- Following sets the centre to the vehicle's placed point each frame it is drawn. When it
  is not drawn the centre holds and the follow stays armed. A drag, `WASD` or `Esc` stops
  it; an orbit or a key step does not.
- Frame order: bar press; clock; scrub; `Esc`; `B`; orbit, `Q E R F`; pan, zoom; click;
  follow; `K`.

## Readout and keyframe line
- Readout: `t 64.10 s [9.1–299.1]  ×2  paused  following 103 (not drawn) — Esc to stop`
  (the Esc hint whenever following; "(not drawn)" only on hold).
- `K` prints the full camera (`rules/camera.md`); `follow` only for a drawn vehicle.
