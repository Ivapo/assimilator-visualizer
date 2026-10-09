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
generated: 2026-10-09
---

# Render

**What is true right now.** Corrected freely against the sources above.

## CLI
`assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
[--results] [--fcd] [--from] [--to] [--speedup] [--fps 30] [--width 1920] [--height 1080]
[--camera <file.toml>] [--buildings <file.geojson>] [--no-see-through] [--no-streets]`.
- Width and height must be even. Stderr: one `{"frame": n, "of": N}` per frame, then `{"done": "<out>"}`.
- On error, one `error: …` line and exit 1; a clap usage error exits 2 with its first line.
- No logger: Bevy's `bevy_log` feature is off, so neither Bevy nor wgpu prints.

## Clock
- `D = to − from`; default speedup `D / clamp(D, 30, 300)`; `N = ceil(D·fps/speedup − 1e-6)` ≥ 1.
- Frame `n` (from 0) shows `from + n · speedup / fps`, never wall-clock time.

## Scene (built by `src/draw.rs`, shared with `view`: `rules/view.md`)
- **Roads.** One flat mesh at height 0, in link order: each link a strip `total_width`
  wide, its centreline sampled with `interpolate_with_lateral(link, s, 0.0)` at `s = L·i/n`,
  `n = ceil(L / 1 m)`, edges offset along the right normal `(cos h, −sin h)`. Streets
  (junctions, median noses, markings) are drawn over them unless `--no-streets` (`rules/streets.md`).
- **Road material** is unlit and not culled. **Vehicles** in `vehicle_id` order (where:
  `rules/motion.md`): a pool of unit cuboids orthographic, one shaded mesh in perspective.
  - Each box is length × 1.8 m × 1.5 m on the placed point, lifted 0.01 m per rank (0.001 m
    in perspective). The higher id wins an overlap: straight down by the lift and the depth
    test; in perspective by the one draw's order, a bit-identical lower box not drawn.
  - Yawed by `90° − heading` about +Y; unlit colour from 5 speed bins (< 2, 5, 9, 13, ∞ m/s).
- **Colours.** Background `#12161e`, road `#5c6068`, speed colours distinct from both.
  World to Bevy is `(x − cx, height, −(y − cy))`.
- **Buildings**, only with `--buildings`: one lit mesh and a sun (`rules/buildings.md`); with
  `--camera` too, those in the way cut to stubs unless `--no-see-through` (`rules/see-through.md`).
- **Credit line**, only on a network with `map_origin`: bottom-right UI text (`rules/credit.md`).

## Camera
- Without `--camera`: top-down orthographic, north up, fitted to the strips' bounding box
  plus 20 m: `k = max(bw / width, bh / height)` metres per pixel. With it: perspective,
  set from the flight's pose every frame, baked at the same fit (`rules/camera.md`).
- `world_to_pixel(x, y) = (W/2 + (x − cx)/k, H/2 − (y − cy)/k)`; pixel (i, j) covers
  `[i, i+1) × [j, j+1)`. urban_grid: bbox 1200 m, so k = 0.5741 at 3840×2160, 1.1481 at 1920×1080.

## Pipeline
- Bevy 0.19 `DefaultPlugins` without `WinitPlugin`, `PipelinedRenderingPlugin` or
  `TerminalCtrlCHandlerPlugin`; `WindowPlugin { primary_window: None, DontExit }`. No
  window or event loop; pipelines compile synchronously; the loop is pumped by hand.
- The target is an `Rgba8UnormSrgb` image. The camera uses MSAA ×4, `Tonemapping::None`
  and `DebandDither::Disabled`. Three empty frames at start-up settle assets and pipelines.
- Per frame: set the boxes, spawn `Screenshot::image(target)`, then update and `poll(Wait)`
  until the observer has the lossless readback (≤ 200 updates): `W·H·4` bytes, top row first.
- `Job::prepare_with(o, camera, buildings)` (`prepare`, `prepare_with_camera` delegate):
  size, fps, speedup, `run::load` (`rules/inputs.md`), the keyframe file, the buildings,
  the credit line (`prepare_without_credit`: none); see-through and streets off (`set_streets`
  spawns them and settles again). Gates: `render_frame(n)`, `render_at(t)`, `render_empty()`,
  `camera()`/`k()`, `pose_at(t)`, `boxes_at(t)`, `buildings()`, `motion_report()`, `credit*()`.

## Output
- `ffmpeg -f rawvideo -pix_fmt rgba -s WxH -r fps -i - -c:v libx264 -pix_fmt yuv420p
  -f mp4 <out>.partial`. ffmpeg's stderr is captured.
- `<out>.partial` is renamed to `<out>` only after ffmpeg exits 0. On failure, or if the
  encoder is dropped, `.partial` is removed.
