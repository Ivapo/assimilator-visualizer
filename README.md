# assimilator-visualizer

Renders a 3D video of one Assimilator simulation run, for presentations, from a CLI that
a harness can call.

Phase 1 (vis-001, `specs/visualizer_spec.md`) draws every FCD vehicle as a box moving
along its link, seen top-down, rendered headless with Bevy and encoded by ffmpeg.
Phase 2 makes the motion smooth between the 1 Hz FCD samples: boxes brake and
accelerate as the run did, follow the engine's turn path through each junction, slide
between lanes, and disappear at their last row. Phase 5 adds a 3D camera: `view` orbits
and tilts, and `render --camera` flies a perspective camera through keyframes.
vis-002 (`specs/city_spec.md`) Phase 1 puts the real buildings around an imported
network: Overture footprints, fetched once into a cache and drawn as grey, sunlit blocks.
Its Phase 2 draws the data's credit line in the corner of every frame of an imported
network's video.

```
assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
                         [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                         [--speedup <x>] [--fps <n>] [--width <px>] [--height <px>]
                         [--camera <file.toml>] [--buildings <file.geojson>]
```

- **Defaults.** `results.db` is `<project>/results.db`. FCD is
  `fcd/<scenario>_<seed>.parquet` beside it.
- **Window.** It runs from the first to the last FCD time. The default speed-up makes the
  video last that window, clamped to 30 s – 5 min, at 30 fps and 1920×1080. Width and
  height must be even.
- **Progress.** Stderr carries one JSON line per frame (`{"frame": n, "of": N}`), then
  `{"done": "<out>"}`.
- **Errors.** Any error is a single line and a non-zero exit. No file is left at `--out`.
- **Camera.** Without `--camera` the camera looks straight down on the whole network,
  orthographically. With `--camera <file.toml>` it is perspective and flies through the
  file's keyframes (below). `--camera` changes no other default.
- **Buildings.** `--buildings <file.geojson>` draws the buildings of a cache fetched for
  this network (below). Without it nothing about the video changes.
- **Credit.** A network with `metadata.map_origin` (an imported one) gets one line in the
  bottom-right corner of every frame, with or without `--buildings`, crediting its data
  (see [Map data](#map-data)). A drawn network such as `urban_grid` gets none. There is no
  switch to turn it off.

### The keyframe file

The lines `view` prints on `K`, in a TOML array, a comma after each:

```toml
keyframes = [
  { t = 20.000, x = 0.00, y = 300.00, height_m = 1240.00 },
  { t = 50.000, x = 0.00, y = 300.00, height_m = 1240.00 },
  { t = 64.100, follow = 1, height_m = 120.00, yaw_deg = 90.00, pitch_deg = 45.00 },
]
```

- `t` is sim seconds; `x`, `y` the point looked at (metres), or `follow` a vehicle id;
  `height_m` the world height visible there. `yaw_deg` (compass direction the camera
  faces, 0 = north up) and `pitch_deg` (90 = straight down, down to 25) are optional:
  without them the camera looks straight down, north up. `[[keyframes]]` tables also work.
- The camera passes through every keyframe on a smooth curve, eased in and out. Two
  keyframes with the same camera hold it still between them. A `follow` keyframe keeps its
  height, yaw and pitch and moves with the vehicle. Before the first keyframe and after the
  last, the camera holds.
- `t` must increase, and a followed vehicle must be drawn at its keyframe's `t`. Any
  error names the keyframe and stops before the first frame. The file's keyframes do not
  change the window: use `--from` and `--to` for a part of the run.

### Buildings

The buildings come from Overture Maps' `building` type, fetched once per network with
network access into a GeoJSON cache, then read offline by `render` and `view`:

```
scripts/fetch-buildings.sh --project <dir> --out <dir>/buildings.geojson
                           [--scenario baseline] [--margin 250] [--release 2026-09-23.1]
```

- **What it fetches.** Every building that meets the scenario's network extent plus
  `--margin` metres, whole, from the pinned Overture release, in lng/lat with its `id`,
  `height`, `num_floors` and `sources` (the datasets it came from). The file records its
  release in its `description`. It prints one JSON line: the release, the box, the number
  of buildings by height rule, by dataset (`sources`) and with none (`no_sources`), and the
  file's size. The same release gives the same bytes. Overture keeps only recent releases
  on S3, so keep the cache.
- **Older caches.** A cache fetched before vis-002 Phase 2 has no release or sources, so
  `render --buildings` rejects it (`view` still reads it). Fetch it again: for the Midtown
  fixture, `REFETCH=1 scripts/fixture.sh midtown`.
- **Which networks.** Only a georeferenced one, with `metadata.map_origin`, as an import
  writes. A drawn network such as `urban_grid` has no buildings, and `--buildings` on it
  is an error.
- **How they look.** Opaque grey blocks lit by one sun, roofs lighter than walls. A
  building's height is its `height`, else `num_floors` × 3.5 m, else 10 m. The camera may
  pass into a building; nothing collides.
- **Errors.** A file that is missing, malformed, in metres or fetched for another network
  is one `error: --buildings <file>: …` line before the first frame or the window.

## View a run

```
assimilator-video view --project <dir> --scenario <name> --seed <n>
                       [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                       [--width <px>] [--height <px>] [--buildings <file.geojson>]
```

`view` (Phase 3) opens a window over the same run, with the same roads and boxes moving
as `render` draws them. You drive the camera and the clock by hand, to find the moments
and framings worth rendering.
- **Inputs and checks** are `render`'s, and they run before the window opens. It does not
  need ffmpeg. `--width`×`--height` is the window in logical pixels, 1280×720 by default.
- **Output.** Stdout carries only keyframe lines, one per `K`, for example
  `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }`.
  The line has `follow = <vehicle_id>` instead of `x`, `y` while following a drawn
  vehicle. Closing the window exits 0.
- **Camera.** Perspective, starting straight down on the whole network, north up.

| Key or mouse | Does |
|---|---|
| Space | play / pause (at the end, restart from the start) |
| `+` or `=` / `-` (by the character typed, any layout; keypad too) | double / halve the speed, 1/8× to 64× |
| `←` / `→` | pause and step 1/30 s |
| `Shift+←` / `Shift+→` | step to the previous / next FCD sample |
| drag, `W` `A` `S` `D` | pan (the ground under the cursor stays under it) |
| right-drag, or `Ctrl` + drag | orbit (sideways) and tilt (up and down) about the centre |
| `Q` / `E` | turn 15° left / right |
| `R` / `F` | tilt 5° toward the horizon / toward straight down (25° to 90°) |
| scroll | zoom about the cursor (0.02 m per pixel to half the network) |
| click a box | follow it; a drag, `WASD` or `Esc` stops following (orbiting does not) |
| click or drag on the time slider | jump or scrub to that time (pauses while held, resumes on release) |
| `K` | print the camera as a keyframe line on stdout |
| `B` | hide / show the buildings (with `--buildings`) |

The **time slider** (Phase 4) is a thin bar along the bottom spanning the run's window,
with a handle at the current time and a tick at each whole minute (thinned on long runs).
A press on the bar never pans or picks, and scrolling over it does not zoom; a follow
stays on while you scrub. The bar hides when the window is under 64 pixels either way.

The readout, just above the slider, shows the time, the window, the speed, playing or
paused, and the vehicle followed, with "(not drawn)" while it is out of the run and
"— Esc to stop".

The run must have FCD Parquet output
(`--set simulation.output.fcd.enabled=true` on `assimilator run`). The renderer reads the
project, `results.db` and the FCD in place, and never writes into the project.

## Prerequisites

- **Rust** (edition 2024; built with 1.97).
- **ffmpeg and ffprobe on `PATH`.** ffmpeg must have `libx264`. The gates also use
  `duckdb` and `python3`.
- **For buildings only:** the DuckDB CLI ≥ 1.5.1 with its `httpfs` and `spatial`
  extensions (installed on first use), and network access for `fetch-buildings.sh`.
  `render` and `view` never use the network.
- **Read access to the private engine repo** `github.com/Ivapo/assimilator`.
  - This crate depends on the engine's `assimilator-config`, `assimilator-core` and
    `assimilator-geometry` crates as git dependencies. They are pinned at one rev in
    `Cargo.toml`, which the user moves on purpose.
  - Cargo's built-in git client cannot authenticate to a private repo, so the committed
    `.cargo/config.toml` sets `net.git-fetch-with-cli = true`. Cargo then fetches with
    your `git` and its credentials; check that `git ls-remote
    https://github.com/Ivapo/assimilator.git` works.
  - **Without engine access the crate does not compile, and no gate can run.** vis-001
    OQ-6 is the route to a public build.

```
cargo build --release
```

## Test fixture and gates

`scripts/fixture.sh` builds everything under `scratch/` (gitignored):
- it reads the engine URL and rev from `Cargo.toml`;
- it `cargo install`s the engine CLI at that rev into `scratch/engine`;
- it exports the engine's `urban_grid` example at the same rev with `git archive` from
  `${ENGINE_CHECKOUT:-../assimilator}`, which only reads it;
- it runs the example with FCD on, and derives the test FCD files.
Nothing is written into the engine checkout.

```
scripts/fixture.sh                                              # once
scripts/gates.sh                                                # gates 1, 2, 4; Phase 5 gate 10
cargo test --release --test gates -- --ignored --test-threads=1 --nocapture   # gates 3, 6–11 + determinism
cargo test --release --test view -- --include-ignored --test-threads=1 --nocapture   # Phase 3 gates 2, 4–9
cargo test --release --test slider -- --include-ignored --test-threads=1 --nocapture # Phase 4 gates 4–10
cargo test --release --test camera -- --include-ignored --test-threads=1 --nocapture # Phase 5 gates 4–9, 11, 12
```

`scripts/gates.sh` also compares the default render's frames with
`scratch/ref-8eb9052.framemd5` when that file exists. That is Phase 3's gate 1: the file is
built once from a `git archive 8eb9052` build, as the spec says.

vis-002's gates run on a second fixture, Midtown, built from the user's own project
(`MIDTOWN_PROJECT`, default `~/assimilator/projects/midtown-section`, read with
`git archive`), so only a machine with that project can run them:

```
scripts/fixture.sh midtown        # once: the project with its demand halved, the run, the one fetch
scripts/gates-city.sh             # gates 3, 4, 5, 7 (CLI) and 13, offline
REFETCH=1 scripts/gates-city.sh   # adds gate 5's second fetch and missing release (network)
cargo test --release --test buildings -- --include-ignored --test-threads=1 --nocapture   # gates 6–12, 14
scripts/gates-credit.sh           # Phase 2 gates 4 (its checks), 5, 9 (CLI) and 14, offline
REFETCH=1 scripts/gates-credit.sh # adds gate 4's second fetch (network)
cargo test --release --test credit -- --include-ignored --test-threads=1 --nocapture      # Phase 2 gates 3, 6–8, 10, 11
```

`FORCE=1 scripts/fixture.sh midtown` redoes the project and the run but keeps the fetched
`buildings.geojson`. The engine at the pin does not give the same Midtown traffic twice,
so the run in `scratch/midtown` is the fixture. Two checks of `gates-city.sh` are known
misses until their causes are fixed, and print `FAIL`: gate 3's comparison with the
2026-09-30 run (the engine, vis-002 OQ-4) and gate 13's flight comparison (the
`--camera` render, OQ-8).

## Map data

Building footprints and imported road networks come from Overture Maps (Overture Maps
Foundation, overturemaps.org). © OpenStreetMap contributors. Available under the Open
Database License (ODbL). Overture's buildings also include data from other sources
under their own licences, such as Esri Community Maps contributors, Microsoft Global
ML Building Footprints and Google Open Buildings; see
https://docs.overturemaps.org/attribution/.

None of that data is in this repository: fetched buildings and imported networks stay
outside it.

**In the video.** Every frame of an imported network's render carries one line in its
bottom-right corner, built from the data the video shows:
- `© OpenStreetMap contributors (ODbL)`, always;
- `Overture Maps Foundation, release <r>`, when the buildings or the roads came from
  Overture: the buildings cache's release, else the one in the project's
  `import_report.json`;
- each other building source present in the cache, such as `Microsoft ML Buildings
  (ODbL)` or `USGS Lidar`, with `Esri Community Maps contributors (CC BY 4.0)` and `Google
  Open Buildings (CC BY 4.0)` named only when their buildings are there.

For the Midtown fixture with its buildings, that is `© OpenStreetMap contributors (ODbL) ·
Overture Maps Foundation, release 2026-09-23.1 · Microsoft ML Buildings (ODbL) · USGS
Lidar`. An import from HERE, or any source type other than Overture or OSM, stops `render`
with an error until its credit is settled (vis-002 OQ-11).

**When you publish a video**, put the full credit in its description too: the paragraph
at the top of this section, and the link https://www.openstreetmap.org/copyright. The
line in the corner is not enough on its own for a video where the map is a major part
(OpenStreetMap's attribution guidelines). A closing card waits for the harness contract.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

**One exception: the font.** `assets/fonts/FiraSans-Medium.ttf` (Fira Sans Medium, which
`render` embeds to draw the credit line) is under the SIL Open Font License 1.1, in
`assets/fonts/OFL.txt`. So the crate's `license` is `(MIT OR Apache-2.0) AND OFL-1.1`.
The OFL does not cover the videos rendered with it.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
