---
title: signals
sources:
  - src/signals.rs
  - src/inputs.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/mod.rs
  - src/view/state.rs
  - src/main.rs
covers: >
  The run's signals: the timeline stepped from the engine's `SignalState` and its clock
  rule, the dots' place, look and size in both cameras, their depth and order, the values
  copied from the engine at 90b39292, `--no-signals`, `view`'s `L` and the two errors.
max_lines: 40
generated: 2026-10-18
---

# Signals

**What is true right now.** Corrected freely against the sources above.

## Timeline (`signals::Signals`, no Bevy types)
- `Signals::build`: `SignalState::new`, `advance(dt)` while `time < duration − 1e-9`,
  each approach's `link_signal_color` changes kept as `(k, colour)`, and `t[k]` summed as
  the engine sums it. Never the plan's seconds, `cycle_time` or `offset`.
- Approaches: `signalized_approach_links()`, in its sorted order; one dot each. At sim
  time `t` a frame shows step `k`, the last with `t[k] ≤ t + 1e-6` (`step_at`; 0 before
  the first step). `Colour::of`: `"green"`, `"amber"`, anything else red.

## Clock (`inputs::run_clock`, then `Signals::for_run`)
- No signal in the network: nothing read or drawn. Else the completed `runs` row for
  scenario and seed, read-only, `immutable=1`: `mode` = `micro` ("the run is <mode> …"),
  then `sim_params.timestep` and `runs.duration` finite, above 0 ("the run's timestep is
  not readable …: <why>"); both end "; --no-signals renders without signals".

## Dots (`signals::dots`, `draw::spawn_dots`, `draw::set_dots`)
- At `s = max(link_length − stop_line_offset, 0)` on lane 0 (`Placement::place`).
- Per dot: a black ring from `4.5·s` to `5.5·s` px and a fill to `4.5·s` px, `s =
  min(H, 9W/16)/1080`; 32 segments, 97 vertices and 96 triangles, ring first.
- Orthographic: flat at `LIFT_M` 0.03 m, radii in metres `·k`. Perspective: a disc facing
  the camera, its lowest point 0.03 m over its stop line, sized to project to exactly
  those radii; left out when its depth is ≤ 0.1 m.
- One mesh and entity, unlit, `cull_mode: None`, `NoFrustumCulling`, vertex colours via
  `streets::colour`, rebuilt each frame, hidden when empty. Over the road and markings,
  under boxes and buildings; between dots, the later in approach order.
- Copied from the engine at 90b39292 (each line named in `src/signals.rs`): the place
  from `frame_collect.rs`'s `collect_signal_state`, l. 537–561; `radius = 5` and
  `lineWidth = 1` from `vehicleRenderer.ts`, l. 158 and 169; `#22c55e`, `#f59e0b`,
  `#ef4444` and `#000000` from `colors.ts`, l. 84–86 and 121.

## Switches
- `Job::set_signals(on)`: every `prepare*` leaves them off; `render` turns them on unless
  `--no-signals`, which reads and builds nothing. `render_at` sets the dots before the boxes.
- `view` builds the timeline before the window; its error is `view`'s, but under
  `--no-signals` (dots start hidden) it is dropped and `L` draws nothing. `L` flips
  `signals_shown`, after `M`; nothing in the readout or the keyframe line.
