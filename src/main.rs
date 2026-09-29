//! `assimilator-video render …` (vis-001 §2.4). Stderr carries one JSON progress object
//! per line and a final `done`, or a single error line; nothing else.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

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
    let Cmd::Render {
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
    } = cli.command;
    let Some(ffmpeg) = encode::find_ffmpeg() else {
        bail!("ffmpeg missing: no ffmpeg executable on PATH");
    };
    let opts = RenderOptions {
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
    };
    let mut job = Job::prepare(&opts)?;

    let n_frames = job.clock.frames;
    let mut enc = encode::Encoder::start(&ffmpeg, &out, width, height, fps)?;
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
