---
title: hud
sources:
  - src/hud.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - src/view/mod.rs
  - src/view/state.rs
  - src/main.rs
covers: >
  The HUD in the top-left corner of every frame: the numbers and their rule, the clock and
  its format, the text, how it is drawn (font, tabular figures, size, outline, place),
  `--no-hud`, `view`'s `H` and `Job`'s HUD accessors.
max_lines: 40
generated: 2026-10-23
---

# HUD

**What is true right now.** Corrected freely against the sources above.

## The numbers (`hud::numbers`, no Bevy types)
- At sim time `t`: the snapshot `Fcd::snapshot_at(t)` shows (the latest with `time ≤ t + 1e-6`, as the
  boxes and the dots), held between rows. The count is its rows, every vehicle class; the mean is the
  plain mean of their `speed`, stopped vehicles included. Reads only the FCD `run::load` has read.
- `(0, None)` when no box is drawn at `t` (`drawn` is `!boxes.is_empty()`) or there is no
  snapshot: before the first row, after the last, and a step with no vehicle (no FCD row).

## The text (`hud::clock`, `hud::text`)
- Clock: `⌊t + 1e-6⌋` s, at least 0, as `m:ss` (`5:00`, `59:59`), `h:mm:ss` from an hour.
- One line joined by ` · `: `5:00 · 184 vehicles · 17 km/h`. "vehicle" when the count is 1;
  the speed `⌊3.6·v + 0.5⌋ km/h` (halves up), `– km/h` (an en dash) with none. No labels.

## Drawing (`draw::spawn_hud`, `set_hud_text`, `set_hud_size`)
- `S = max(1, round(min(H, 9W/16) / 36))` px (20, 30, 60 at 720p, 1080p, 2160p); margin `m
  = S` from the top and left; outline offset `o = max(1, round(S/12))`.
- As the credit line (`rules/credit.md`): a root `Node` at `left: m, top: m`; the fill, sRGB
  (240, 240, 240), in its flow; eight black copies at `±o or 0` each way, siblings at
  `ZIndex(-1)`. None wraps. Fira Sans Medium from `draw::CREDIT_FONT`, with the OpenType
  feature `tnum` (`FontFeatureTag::TABULAR_FIGURES`): a changed digit moves nothing after it.
- Spawned holding `hud::GLYPHS` (`0123456789:·– vehiclsmp/hk`), so the settle frames lay out
  every glyph once. Each frame's line is written to all nine texts, each only if it differs.
- Over everything, in both cameras; no flight moves it, and no pixel outside `hud_box()` changes.
- With link colours on, their legend sits under the line, at `m + 1.6·S` (`rules/links.md`).

## `render` (`Renderer::set_hud`, `Job`)
- `Job::set_hud(on)`: every `prepare*` leaves it off; `render` turns it on unless `--no-hud`,
  which spawns nothing (Phase 1's frames). On spawns the tree with `UiTargetCamera(<image
  camera>)` and settles again. `render_at(t)` sets the line after the boxes are taken.
- `hud_text()`: the line the last frame drew. `hud_box()`: `[x0, y0, x1, y1]`, `x1`/`y1`
  exclusive, the fill's laid-out rectangle grown by `o`, rounded outward. `None` while off.

## `view`
- Spawned at launch into the window, sized from its logical size and re-sized when it
  changes. `--no-hud` starts it hidden. `H` (by position) flips `hud_shown`, after `L`;
  nothing in the readout or the keyframe line. While shown, the line for `t` every frame.
