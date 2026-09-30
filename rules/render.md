---
title: render
sources:
  - src/clock.rs
  - src/scene.rs
  - src/render.rs
  - src/draw.rs
  - src/run.rs
  - src/encode.rs
  - src/main.rs
  - src/lib.rs
covers: >
  The render clock, the scene and camera, the headless Bevy pipeline, the ffmpeg
  output and the CLI contract of `assimilator-video render`.
max_lines: 60
generated: 2026-09-29
---

# Render

**What is true right now.** Corrected freely against the sources above.

## CLI
`assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
[--results] [--fcd] [--from] [--to] [--speedup] [--fps 30] [--width 1920] [--height 1080]`.
- Width and height must be even. Stderr carries one `{"frame": n, "of": N}` per frame
  (n from 1), then `{"done": "<out>"}`.
- On error it prints a single `error: …` line and exits 1. A clap usage error exits 2
  with its first line.
- No logger: Bevy's `bevy_log` feature is off, so neither Bevy nor wgpu prints.

## Clock
- `D = to − from`. The default speedup is `D / clamp(D, 30, 300)`.
- `N = ceil(D · fps / speedup − 1e-6)`, with a minimum of 1.
- Frame `n` (from 0) shows `from + n · speedup / fps`, never wall-clock time.

## Scene (built by `src/draw.rs`, shared with `view`: `rules/view.md`)
- **Roads.** One flat mesh at height 0, in the network's link order. Each link is a strip
  `total_width` wide. Its centreline is sampled with `interpolate_with_lateral(link, s,
  0.0)` at `s = L·i/n`, `n = ceil(L / 1 m)`, and the edges are offset along the right
  normal `(cos h, −sin h)`.
- **Road material** is unlit and not culled. **Vehicles:** a pool of unit cuboids, up
  to the most drawn at once, filled in `vehicle_id` order; where: `rules/motion.md`.
  - Each box is scaled to length × 1.8 m × 1.5 m and centred on the placed point, lifted
    0.01 m per rank in `vehicle_id` order within the frame (up to 1.09 m at urban_grid's
    peak of 110). The depth test, not Bevy's binned draw order, decides overlaps: the
    higher id wins.
  - Yawed by `90° − heading` about +Y; unlit colour from 5 speed bins (< 2, 5, 9, 13, ∞ m/s).
- **Colours.** Background `#12161e`, road `#5c6068`, speed colours distinct from both.
  World to Bevy is `(x − cx, height, −(y − cy))`.

## Camera
- Top-down orthographic, north up, fitted to the strips' bounding box plus 20 m:
  `k = max(bw / width, bh / height)` metres per pixel.
- `world_to_pixel(x, y) = (W/2 + (x − cx)/k, H/2 − (y − cy)/k)`; pixel (i, j) covers
  `[i, i+1) × [j, j+1)`.
- urban_grid: the bbox is 1200 m, so k is 0.5741 at 3840×2160 and 1.1481 at 1920×1080.

## Pipeline
- Bevy 0.19 `DefaultPlugins` without `WinitPlugin`, `PipelinedRenderingPlugin` or
  `TerminalCtrlCHandlerPlugin`; `WindowPlugin { primary_window: None, DontExit }`. No
  window or event loop; pipelines compile synchronously; the loop is pumped by hand.
- The target is an `Rgba8UnormSrgb` image. The camera uses MSAA ×4, `Tonemapping::None`
  and `DebandDither::Disabled`.
- Per frame: set the boxes, spawn `Screenshot::image(target)`, then update and
  `poll(Wait)` until the observer has the readback (at most 200 updates). The readback
  is the lossless frame, `W·H·4` bytes, top row first.
- Three empty frames are rendered at start-up so assets and pipelines settle.
- `Job::prepare` checks size, fps and speedup, then `run::load` (`rules/inputs.md`). `Job`
  exposes `render_frame(n)`, `render_at(t)`, `render_empty()`, `camera()`, `k()`,
  `boxes_at(t)` (`run::boxes_at`, no renderer) and `motion_report()` for the gates.

## Output
- `ffmpeg -f rawvideo -pix_fmt rgba -s WxH -r fps -i - -c:v libx264 -pix_fmt yuv420p
  -f mp4 <out>.partial`. ffmpeg's stderr is captured.
- `<out>.partial` is renamed to `<out>` only after ffmpeg exits 0. On failure, or if the
  encoder is dropped, `.partial` is removed.
