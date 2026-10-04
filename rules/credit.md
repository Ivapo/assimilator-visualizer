---
title: credit
sources:
  - src/credit.rs
  - src/draw.rs
  - src/render.rs
  - src/lib.rs
  - scripts/fetch-buildings.sh
covers: >
  The credit line of an imported network's render: the rule that builds it from the data,
  its inputs and checks, how it is drawn and fitted, the font and its licence, and `Job`'s
  credit accessors.
max_lines: 40
generated: 2026-10-04
---

# Credit

**What is true right now.** Corrected freely against the sources above.

## The line (`credit::line`)
- Only `render`, on a network with `metadata.map_origin`, with or without `--buildings`;
  otherwise nothing is read or spawned (urban_grid is unchanged). `view` draws none; no flag.
- Items joined by ` · `, no full stop: (1) `© OpenStreetMap contributors (ODbL)`, always;
  (2) `Overture Maps Foundation, release <r>` with buildings (their release, which wins) or
  when `<project>/import_report.json`'s `source.type` is `Overture` (its `release`); (3) each
  dataset a feature names (parts included) but `OpenStreetMap`, once: `Esri Community Maps
  contributors (CC BY 4.0)`, `Google Open Buildings (CC BY 4.0)`, `Microsoft ML Buildings
  (ODbL)`, `USGS Lidar` in that order, then unknown ones verbatim, in byte order. No TomTom.
- Release and datasets come from the fetch (`rules/buildings.md`). No report, or a null or
  missing `source`, adds nothing for the roads.

## Checks (after `--buildings`' own; `rules/inputs.md`)
- `error: <project>/import_report.json: …`: unreadable; not JSON or not an object; `source`
  neither null nor an object with a string `type`; `Overture` without a string `release`; a
  type other than `Overture` or `Osm` (`… has no credit (vis-002 OQ-11)`).
- `error: --buildings <file>: …`: no such `description` with a non-empty release without
  spaces (`no release: fetched before vis-002 Phase 2; …`); a feature's `sources` not
  absent, null or an array of strings (named by its `id`).
- After the settle frames, a final size under 10 px: `error: the credit line would be <S'>
  px in a <W>×<H> frame, under 10 px; render a larger frame`.

## Drawing (`draw::spawn_credit`, `render::Renderer`)
- `S = round(min(H, 9W/16) / 54)` px (13, 20, 40 at 720p, 1080p, 2160p); margin `m = S`
  from the right and bottom; outline offset `o = max(1, round(S/12))`.
- A root `Node` at `right: m, bottom: m` with `UiTargetCamera(<image camera>)`; the fill,
  sRGB (240, 240, 240), in its flow; eight black copies offset by `±o or 0` each way, its
  siblings at `ZIndex(-1)`. None wraps. The fit: after the settle renders, while the fill is
  wider than `W − 2m`, all nine take `credit::fit_size`'s `floor(S·(W − 2m)/w)` and settle
  again; `m` and `o` stay the first size's.
- Font: Fira Sans Medium 4.203 under the OFL 1.1, embedded from `assets/fonts/` (`OFL.txt`).
- `Job::prepare_with` builds the line; `prepare_without_credit` is the same job without it,
  for the gates. `credit()`, `credit_size()` (after the fit) and `credit_box()`: `[x0, y0,
  x1, y1]`, `x1`/`y1` exclusive, the fill's laid-out rectangle grown by `o`, rounded
  outward. No pixel outside it changes.
