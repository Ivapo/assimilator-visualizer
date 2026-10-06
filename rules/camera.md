---
title: camera
sources:
  - src/camera.rs
  - src/keyframes.rs
  - src/draw.rs
covers: >
  The 3D camera: the pose and its perspective projection, what the orthographic and
  perspective paths draw, the keyframe line and the keyframe file with its errors,
  and the flight `render --camera` takes through the keyframes.
max_lines: 60
generated: 2026-10-06
---

# Camera

**What is true right now.** Corrected freely against the sources above.

## Pose and projection (`src/camera.rs`, no Bevy types)
- `Pose { cx, cy, height_m, yaw_deg, pitch_deg }`: the look-at point on the ground; the world
  height seen there; yaw clockwise from north (0 = north up); pitch 90 straight down, in [25, 90].
- Perspective, vertical `FOV_DEG` = 45°. Distance `d = height_m / (2·tan 22.5°)`
  (1.2071067811865475·`h`); axis `a = (s(y)c(p), c(y)c(p), −s(p))`, up
  `u = (s(y)s(p), c(y)s(p), c(p))`, right `r = (c(y), −s(y), 0)`; eye `E = (cx, cy, 0) − d·a`.
- `project(pose, W, H, X)`: `(W/2 + f·x_c/z_c, H/2 − f·y_c/z_c)`, `f = (H/2)/tan 22.5°`.
  `ray_to_plane(pose, W, H, pixel, z)` is its inverse onto the plane at height `z`.
- `sin_deg`/`cos_deg` are exact (0, ±1) at multiples of 90°. `wrap360` is `rem_euclid`
  with 360 and −0 taken as 0. `short_way(y0, y1)` is in (−180, 180]; 180 turns clockwise.
- At pitch 90 it frames the ground an orthographic `height_m` does, to rounding.

## What each path draws (`src/draw.rs`)
- **Orthographic** (`render` without `--camera`): `look_down` + `projection` at `ortho_eye`'s
  height and far (`rules/buildings.md`), cuboids, 0.01 m a rank. Both draw streets (`rules/streets.md`).
- **Perspective** (`view`, `render --camera`): `draw::perspective(pose, fx, fy, aspect)`
  maps world `(x, y, z)` to Bevy `(x − fx, z, −(y − fy))`; eye `E` on the look-at point, up
  `u`, fov 45°, near 0.1, far 20·d. Buildings, boxes and street markings: `NoFrustumCulling`.
- Perspective boxes: `boxes_mesh`, one mesh and one draw in `vehicle_id` order: a depth tie
  goes to the higher id every run, and a box bit-equal in x, y, heading and length to a later
  one is not drawn. Speed colour × shade: top 1, sides 0.55, ends 0.4, bottom 0.25; lift 0.001 m.

## Keyframe line (`src/keyframes.rs`)
- `Keyframe { t, at: Centre(x, y) | Follow(id), height_m, yaw_deg, pitch_deg }`. `format`:
  `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }`
  or `{ t = …, follow = 103, height_m = …, yaw_deg = …, pitch_deg = … }`; `t` 3 decimals,
  the rest 2; a yaw that would print `360.00` prints `0.00`.
- `parse(line)` reads `k = <line>` with TOML (serde, `deny_unknown_fields`). Key order is
  free; `yaw_deg` (0) and `pitch_deg` (90) are optional, so Phase 3 lines still read.

## Keyframe file (`render --camera <file.toml>`)
- One key, `keyframes`: an array of the lines, or `[[keyframes]]` tables.
- `read(path)`, in order: unreadable; not TOML (with its line); another top-level key; no
  `keyframes`; empty. Per keyframe (named `keyframe N (t = …)`, N from 1): an unknown key
  or a wrong type; no `t`; no `height_m`; both `x`/`y` and `follow`; neither; `x` without
  `y` or `y` without `x`; a non-finite number; `follow` < 0; `height_m` ≤ 0; pitch outside
  [25, 90]. Then `t` must increase: "out of order" and "duplicate `t`" are distinct errors.
- `check_follows` (after the run loads): a followed vehicle must be in the FCD and drawn
  at its keyframe's `t`.
- Not errors: one keyframe (held); keyframes outside `[from, to]`. The window, frame count
  and every other flag's default do not depend on `--camera`.

## The flight (`Flight::new`, `pose_at`)
- `ln height_m`, the unwrapped yaw (short way) and the pitch: each a PCHIP in time,
  Fritsch–Butland slopes, 0 at the ends and where secants differ in sign or vanish.
- A segment whose two keyframes are equal in a channel returns the keyframe's own value,
  and so does a keyframe's own `t`: holds are bit-exact.
- Position: centripetal Catmull–Rom (Barry–Goldman) through each run of consecutive fixed
  keyframes with different positions, knots `√|ΔP|`, reflected end neighbours; time maps to
  `τ` by PCHIP with slope 0 at the run's ends. Equal fixed neighbours are a hold.
- Two keyframes following one vehicle: its placed point. Either end following: the
  smoothstep blend `(1 − w)·P_a(t) + w·P_b(t)`. A followed vehicle not drawn at `t` is held
  at the nearest end of its rows. Before the first and after the last keyframe the pose holds.
- `pose_at(t, &Motion, &Fcd, &Placement)` finds vehicles through `run::boxes_at`;
  `pose_at_by(t, boxes_at)` takes them from a closure (the constructed gates).
