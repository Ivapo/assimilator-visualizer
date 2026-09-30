---
title: slider
sources:
  - src/view/slider.rs
  - src/view/state.rs
  - src/view/mod.rs
covers: >
  `view`'s time slider: the bar's geometry, x ↔ t, the ticks, how a press on the bar
  scrubs and what it takes from the camera and the clock, and how the window draws it.
max_lines: 40
generated: 2026-09-30
---

# Slider

**What is true right now.** Corrected freely against the sources above.

## Geometry (`slider::Bar`, no Bevy types; logical pixels, origin top left)
- From the window's current size `W × H` every frame; no bar when `W < 64` or `H < 64`.
- Track `x0 = 16` to `x1 = W − 16`, 4 px tall on `y_bar = H − 14`. Handle 12 × 12 on
  `(x(t), y_bar)`. Ticks 1 × 10 on `y_bar`, over the track and under the handle.
- Hit area `[x0 − 8, x1 + 8] × [H − 28, H]`. The readout sits at left 16, bottom 30.
- `x(t) = (1 − u)·x0 + u·x1`, `u = (t − from)/(to − from)`; `t(x) = (1 − u)·from + u·to`,
  `u = clamp((x − x0)/(x1 − x0), 0, 1)`. Both ends are exact; no snapping.

## Ticks
At every `m·P` in `[from, to]`. `P` is the first of 1, 5, 10, 15, 30, 60 min, 2, 3, 6, 12,
24 h with `P·(x1 − x0)/(to − from) ≥ 8` px; none fits, no ticks. Unlabelled.

## Input (`ViewState::frame`)
- Order: size and bar; bar press; clock; scrub; `Esc`; pan and zoom; click; follow; `K`.
- A left press with `cursor` in the hit area starts a scrub (`scrub = Some(was playing)`)
  and pauses; it makes no `Press`, so it never pans or picks. A press elsewhere is a
  pan or a click even if it later crosses the bar.
- Each scrub frame sets `t ← t(x)` from `pointer`, else `cursor`, else holds; `y` is
  ignored. Space, `←`, `→` are ignored while scrubbing; `+`/`−` still work.
- Release ends it, as does the bar disappearing (`t` held): playing resumes if it was
  playing and `t < to`. Scroll over the hit area or during a scrub does nothing.
- A scrub does not stop a follow; follow re-centres at the scrubbed `t`, or holds.

## Window (`src/view/mod.rs`)
- `track_pointer` runs before `frame` and keeps the primary window's last `CursorMoved`
  of the frame as `ViewInput::pointer`: logical and not bounded to the window (winit on
  macOS sends it outside while a button is held); `None` when the cursor did not move.
- Track, handle and a tick pool (grown on demand) are absolutely positioned `bevy_ui`
  `Node`s with `BackgroundColor` and `ZIndex`; each frame they are placed from
  `ViewState::bar()`, written only on change, and hidden (`Display::None`) when unused.
