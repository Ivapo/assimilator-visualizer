//! `network-extent` (vis-002 §2.3.1): the lng/lat box `scripts/fetch-buildings.sh` fetches
//! buildings for. It is the scenario's resolved network's own extent (every node and link
//! `geometry` point) plus a margin, turned into lng/lat with the inverse of the engine's
//! projection and rounded outward to 7 decimals. Stdout is one `W S E N` line; an error
//! is one line on stderr and exit 1.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::Parser;

use assimilator_video::{buildings, inputs};

#[derive(Parser)]
#[command(
    name = "network-extent",
    about = "Print the lng/lat box (W S E N) to fetch a network's buildings for"
)]
struct Cli {
    #[arg(long)]
    project: PathBuf,
    #[arg(long, default_value = "baseline")]
    scenario: String,
    /// Metres added around the network's extent.
    #[arg(long, default_value_t = 250.0)]
    margin: f64,
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
            eprintln!("{}", msg.lines().next().unwrap_or("invalid arguments"));
            return ExitCode::from(2);
        }
    };
    match run(&cli) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}", format!("{e:#}").replace('\n', " "));
            ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<String> {
    if !(cli.margin.is_finite() && cli.margin >= 0.0) {
        bail!(
            "--margin must be a number of metres, at least 0 (got {})",
            cli.margin
        );
    }
    let network = inputs::load_network(&cli.project, &cli.scenario)?;
    let bbox = buildings::fetch_bbox(&network, cli.margin).map_err(anyhow::Error::msg)?;
    Ok(bbox.map(buildings::format_e7).join(" "))
}
