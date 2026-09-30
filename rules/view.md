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
  wall-clock view clock, the camera (pan, zoom, centre bound), picking and following,
  the readout, the keyframe line and the hidden `--bench`.
max_lines: 60
generated: 2026-09-29
---

# View

**What is true right now.** Corrected freely against the sources above.

## CLI
`assimilator-video view --project <dir> --scenario <name> --seed <n> [--results] [--fcd]
[--from] [--to] [--width 1280] [--height 720]`. The size is the window's in logical pixels
(any positive size; 2× physical on a Retina display).
- `run::load` runs `render`'s checks after width and height, before any `App` is built;
  ffmpeg is not looked for. An error is one `error: …` line on stderr, exit 1, no window.
- Stdout carries only keyframe lines, one per `K`, flushed. Stderr carries only the
  error line or `--bench`'s JSON. Closing the window exits 0.
- One Bevy app on the main thread: `DefaultPlugins` with a primary window titled
  `assimilator-video view`, default present mode (vsync). One exclusive `Update` system
  per frame: Bevy input → `ViewInput` → `ViewState::frame` → camera, boxes, readout.

## Drawing (`src/draw.rs`, shared with `render`)
- The road mesh and the box pool (`motion.max_drawn()` boxes) are `render`'s, baked
  relative to the launch fit's centre `(fx, fy)`; the camera sits at
  `(cx − fx, 500, −(cy − fy))`, straight down, with an ortho projection of `W·k × H·k` m
  set every frame from the window's current logical size.
- The readout is a 14 px white UI text line at the top left.

## Clock (`ViewState`, no Bevy types)
- `t` starts at `from`, paused, at 1×. Playing: `t += s · min(Δ, 0.1 s)` of real time.
  `t` stays in `[from, to]`; reaching `to` pauses, and space at `to` restarts at `from`.
- Speed `s = 2^(i−3)`, 1/8× … 64×: `=`/keypad `+` doubles, `-`/keypad `−` halves, each
  stopping at its end. Held keys never repeat an action.
- `←`/`→` pause and step `1/30` s. With Shift: to the latest snapshot below `t − 1e-6` or
  the earliest above `t + 1e-6` (the snapshots include the one before `from`); none, no move.

## Camera
- Launch: `render`'s fit at the launch size (`k_fit`), and the fitted rectangle, the
  strips' bounding box plus 20 m. Neither changes on resize.
- Cursor `(px, py)` is over `(cx + (px − W/2)·k, cy − (py − H/2)·k)`.
- A left press becomes a drag at a net 4 px; the centre is then the press's centre plus
  `(−dx·k, +dy·k)`. A zoom during a drag re-anchors it. `WASD` pan `0.5·W·k` m/s.
- Scroll `n` lines (trackpad: 20 px a line): `k ← clamp(k·1.1^−n, 0.02, 2·k_fit)` about
  the cursor's world point, or about the centre while following.
- After pan and zoom the centre is clamped to the fitted rectangle.

## Pick and follow
- A release that never became a drag picks the box nearest the cursor's world point, by
  distance to its `length × 1.8 m` footprint at its heading, within `8·k` m; ties go to
  the higher `vehicle_id`. No box in reach changes nothing.
- Following sets the centre to the vehicle's placed point each frame it is drawn. When it
  is not drawn the centre holds and the follow stays armed. A drag, `WASD` or `Esc` stops it.
- Frame order: clock; `Esc`; pan and zoom; click; follow; `K`.

## Readout and keyframe line
- Readout: `t 64.10 s [9.1–299.1]  ×2  paused  following 103 (not drawn)`.
- `K`: `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }`, or
  `{ t = 200.000, follow = 103, height_m = 60.00 }` while following a drawn vehicle.
  `height_m = H·k`. `Keyframe::parse` accepts exactly these two forms.

## `--bench <s>` (hidden)
Starts at `t = 140` playing at 2× at the launch fit, skips 3 s, records `s` s of real frame
times, prints `{"frames", "mean_fps", "median_ms", "p99_ms", "worst_ms"}` and exits 0.
