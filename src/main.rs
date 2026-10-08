//! `assimilator-video render …` (vis-001 §2.4). Stderr carries one JSON progress object
//! per line and a final `done`, or a single error line; nothing else.
//!
//! `assimilator-video view …` (vis-001 §2.9) opens a window over a finished run. Stdout
//! carries only keyframe lines; stderr only the error line, or `--bench`'s JSON.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

use assimilator_video::run::LoadOptions;
use assimilator_video::view::{self, ViewOptions};
use assimilator_video::{Job, RenderOptions, encode};

#[derive(Parser)]
#[command(
    name = "assimilator-video",
    version,
    about = "Render a video of one Assimilator run"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render one run to an MP4.
    Render {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        scenario: String,
        #[arg(long)]
        seed: u64,
        #[arg(long)]
        out: PathBuf,
        /// Default: <project>/results.db.
        #[arg(long)]
        results: Option<PathBuf>,
        /// Default: fcd/<scenario>_<seed>.parquet beside results.db.
        #[arg(long)]
        fcd: Option<PathBuf>,
        /// Window start, sim seconds. Default: the first FCD time.
        #[arg(long)]
        from: Option<f64>,
        /// Window end, sim seconds. Default: the last FCD time.
        #[arg(long)]
        to: Option<f64>,
        /// Sim seconds per video second. Default: the window clamped to 30 s – 5 min.
        #[arg(long)]
        speedup: Option<f64>,
        #[arg(long, default_value_t = 30)]
        fps: u32,
        #[arg(long, default_value_t = 1920)]
        width: u32,
        #[arg(long, default_value_t = 1080)]
        height: u32,
        /// A keyframe file (TOML): fly a perspective camera through its keyframes.
        /// Default: the top-down camera fitted to the network.
        #[arg(long)]
        camera: Option<PathBuf>,
        /// A buildings.geojson cache (scripts/fetch-buildings.sh): draw the buildings
        /// around a georeferenced network.
        #[arg(long)]
        buildings: Option<PathBuf>,
        /// Draw every building at full height. Default with `--buildings` and `--camera`:
        /// the buildings between the camera and the point it looks at are cut to stubs.
        #[arg(long)]
        no_see_through: bool,
        /// Draw the roads as plain strips. Default: junctions filled, and lane lines, stop
        /// lines and the yellow centre line, faded where they are under a pixel.
        #[arg(long)]
        no_streets: bool,
    },
    /// Open a window over one finished run; `K` prints a keyframe line on stdout.
    View {
        #[arg(long)]
        project: PathBuf,
        #[arg(long)]
        scenario: String,
        #[arg(long)]
        seed: u64,
        /// Default: <project>/results.db.
        #[arg(long)]
        results: Option<PathBuf>,
        /// Default: fcd/<scenario>_<seed>.parquet beside results.db.
        #[arg(long)]
        fcd: Option<PathBuf>,
        /// Window start, sim seconds. Default: the first FCD time.
        #[arg(long)]
        from: Option<f64>,
        /// Window end, sim seconds. Default: the last FCD time.
        #[arg(long)]
        to: Option<f64>,
        /// Window width, logical pixels.
        #[arg(long, default_value_t = 1280)]
        width: u32,
        /// Window height, logical pixels.
        #[arg(long, default_value_t = 720)]
        height: u32,
        /// A buildings.geojson cache (scripts/fetch-buildings.sh): draw the buildings
        /// around a georeferenced network; `B` hides and shows them.
        #[arg(long)]
        buildings: Option<PathBuf>,
        /// Start with every building at full height; `X` cuts the buildings in the way
        /// to stubs, as it does by default.
        #[arg(long)]
        no_see_through: bool,
        /// Start with the roads as plain strips; `M` shows the streets, as they are by
        /// default.
        #[arg(long)]
        no_streets: bool,
        /// Record `s` seconds of frame times, print them as JSON on stderr and exit.
        #[arg(long, hide = true)]
        bench: Option<f64>,
    },
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) if !e.use_stderr() => {
            let _ = e.print();
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            let msg = e.to_string();
            let first = msg.lines().next().unwrap_or("invalid arguments");
            eprintln!("{first}");
            return ExitCode::from(2);
        }
    };
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let msg = format!("{e:#}").replace('\n', " ");
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Cmd::Render {
            project,
            scenario,
            seed,
            out,
            results,
            fcd,
            from,
            to,
            speedup,
            fps,
            width,
            height,
            camera,
            buildings,
            no_see_through,
            no_streets,
        } => render(
            RenderOptions {
                project,
                scenario,
                seed,
                results,
                fcd,
                from,
                to,
                speedup,
                fps,
                width,
                height,
            },
            out,
            camera,
            buildings,
            !no_see_through,
            !no_streets,
        ),
        // No ffmpeg: `view` encodes nothing.
        Cmd::View {
            project,
            scenario,
            seed,
            results,
            fcd,
            from,
            to,
            width,
            height,
            buildings,
            no_see_through,
            no_streets,
            bench,
        } => view::run(&ViewOptions {
            load: LoadOptions {
                project,
                scenario,
                seed,
                results,
                fcd,
                from,
                to,
            },
            buildings,
            see_through: !no_see_through,
            streets: !no_streets,
            width,
            height,
            bench,
        }),
    }
}

fn render(
    opts: RenderOptions,
    out: PathBuf,
    camera: Option<PathBuf>,
    buildings: Option<PathBuf>,
    see_through: bool,
    streets: bool,
) -> Result<()> {
    let Some(ffmpeg) = encode::find_ffmpeg() else {
        bail!("ffmpeg missing: no ffmpeg executable on PATH");
    };
    let mut job = Job::prepare_with(&opts, camera.as_deref(), buildings.as_deref())?;
    if streets {
        job.set_streets(true)?;
    }
    job.set_see_through(see_through);

    let n_frames = job.clock.frames;
    let mut enc = encode::Encoder::start(&ffmpeg, &out, opts.width, opts.height, opts.fps)?;
    for n in 0..n_frames {
        let rgba = job.render_frame(n)?;
        enc.write_frame(&rgba)?;
        eprintln!("{{\"frame\": {}, \"of\": {}}}", n + 1, n_frames);
    }
    enc.finish()?;
    eprintln!(
        "{{\"done\": {}}}",
        serde_json::to_string(&out.to_string_lossy())?
    );
    Ok(())
}
