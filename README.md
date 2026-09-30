# assimilator-visualizer

Renders a 3D video of one Assimilator simulation run, for presentations, from a CLI that
a harness can call.

Phase 1 (vis-001, `specs/visualizer_spec.md`) draws every FCD vehicle as a box moving
along its link, seen top-down, rendered headless with Bevy and encoded by ffmpeg.
Phase 2 makes the motion smooth between the 1 Hz FCD samples: boxes brake and
accelerate as the run did, follow the engine's turn path through each junction, slide
between lanes, and disappear at their last row.

```
assimilator-video render --project <dir> --scenario <name> --seed <n> --out <file.mp4>
                         [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                         [--speedup <x>] [--fps <n>] [--width <px>] [--height <px>]
```

- **Defaults.** `results.db` is `<project>/results.db`. FCD is
  `fcd/<scenario>_<seed>.parquet` beside it.
- **Window.** It runs from the first to the last FCD time. The default speed-up makes the
  video last that window, clamped to 30 s – 5 min, at 30 fps and 1920×1080. Width and
  height must be even.
- **Progress.** Stderr carries one JSON line per frame (`{"frame": n, "of": N}`), then
  `{"done": "<out>"}`.
- **Errors.** Any error is a single line and a non-zero exit. No file is left at `--out`.

## View a run

```
assimilator-video view --project <dir> --scenario <name> --seed <n>
                       [--results <file>] [--fcd <file>] [--from <s>] [--to <s>]
                       [--width <px>] [--height <px>]
```

`view` (Phase 3) opens a window over the same run, with the same roads and boxes moving
as `render` draws them. You drive the camera and the clock by hand, to find the moments
and framings worth rendering.
- **Inputs and checks** are `render`'s, and they run before the window opens. It does not
  need ffmpeg. `--width`×`--height` is the window in logical pixels, 1280×720 by default.
- **Output.** Stdout carries only keyframe lines, one per `K`, for example
  `{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }`. The line is
  `{ t = …, follow = <vehicle_id>, height_m = … }` while following a drawn vehicle.
  Closing the window exits 0.

| Key or mouse | Does |
|---|---|
| Space | play / pause (at the end, restart from the start) |
| `+` or `=` / `-` (by the character typed, any layout; keypad too) | double / halve the speed, 1/8× to 64× |
| `←` / `→` | pause and step 1/30 s |
| `Shift+←` / `Shift+→` | step to the previous / next FCD sample |
| drag, `W` `A` `S` `D` | pan |
| scroll | zoom about the cursor (0.02 m per pixel to half the network) |
| click a box | follow it; a drag, `WASD` or `Esc` stops following |
| `K` | print the camera as a keyframe line on stdout |

The top-left readout shows the time, the window, the speed, playing or paused, and the
vehicle followed, with "(not drawn)" while it is out of the run and "— Esc to stop".

The run must have FCD Parquet output
(`--set simulation.output.fcd.enabled=true` on `assimilator run`). The renderer reads the
project, `results.db` and the FCD in place, and never writes into the project.

## Prerequisites

- **Rust** (edition 2024; built with 1.97).
- **ffmpeg and ffprobe on `PATH`.** ffmpeg must have `libx264`. The gates also use
  `duckdb` and `python3`.
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
scripts/gates.sh                                                # gates 1, 2, 4
cargo test --release --test gates -- --ignored --test-threads=1 --nocapture   # gates 3, 6–11 + determinism
cargo test --release --test view -- --include-ignored --test-threads=1 --nocapture   # Phase 3 gates 2, 4–9
```

`scripts/gates.sh` also compares the default render's frames with
`scratch/ref-8eb9052.framemd5` when that file exists. That is Phase 3's gate 1: the file is
built once from a `git archive 8eb9052` build, as the spec says.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual
licensed as above, without any additional terms or conditions.
