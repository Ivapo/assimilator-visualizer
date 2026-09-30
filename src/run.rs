//! Loading one finished run, shared by `render` and `view` (vis-001 §2.9.6): every check
//! that can fail on the inputs, in `rules/inputs.md`'s order, every row placed, and
//! every vehicle's track built from the whole file. No Bevy.

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::fcd::{self, Fcd};
use crate::inputs::{self, RunPaths};
use crate::motion::Motion;
use crate::place::{Placed, Placement};
use crate::render::VehicleBox;
use crate::scene::{self, Strip};

/// Which run, and which window of it.
#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub project: PathBuf,
    pub scenario: String,
    pub seed: u64,
    pub results: Option<PathBuf>,
    pub fcd: Option<PathBuf>,
    pub from: Option<f64>,
    pub to: Option<f64>,
}

/// A loaded run: the window `[from, to]`, the FCD, the placed rows, the placement, the
/// motion and the road strips.
pub struct Run {
    pub from: f64,
    pub to: f64,
    pub fcd: Fcd,
    pub placed: Vec<Placed>,
    pub placement: Placement,
    pub motion: Motion,
    pub strips: Vec<Strip>,
}

pub fn load(o: &LoadOptions) -> Result<Run> {
    let paths = RunPaths::new(
        &o.project,
        &o.scenario,
        o.seed,
        o.results.clone(),
        o.fcd.clone(),
    );
    let network = inputs::load_network(&paths.project_dir, &paths.scenario)?;
    inputs::require_completed_run(&paths.results, &paths.scenario, paths.seed)?;
    if !paths.fcd.is_file() {
        bail!("missing file: {}", paths.fcd.display());
    }
    let (first, last) = fcd::time_span(&paths.fcd)?;
    let from = o.from.unwrap_or(first);
    let to = o.to.unwrap_or(last);
    if from.is_nan() || to.is_nan() || to <= from {
        bail!("empty window: --to ({to}) must be after --from ({from})");
    }

    let fcd = fcd::read_window(&paths.fcd, from, to)?;
    let placement = Placement::new(network);
    let placed = placement.place_all(&fcd)?;
    let strips = scene::strips(&placement);
    if strips.is_empty() {
        bail!("the scenario's network has no drawable links");
    }
    let motion = Motion::build(&fcd, &placement);
    Ok(Run {
        from,
        to,
        fcd,
        placed,
        placement,
        motion,
        strips,
    })
}

impl Run {
    /// The boxes shown at sim time `t` ([`boxes_at`]).
    pub fn boxes_at(&self, t: f64) -> Vec<VehicleBox> {
        boxes_at(&self.motion, &self.fcd, &self.placement, t)
    }
}

/// The boxes shown at sim time `t`: every vehicle with `t_first − 1e-6 ≤ t ≤
/// t_last + 1e-6`, where its track puts it at `t` (vis-001 §2.8), in `vehicle_id`
/// order.
pub fn boxes_at(motion: &Motion, fcd: &Fcd, placement: &Placement, t: f64) -> Vec<VehicleBox> {
    motion
        .at(t, fcd, placement)
        .into_iter()
        .map(|(vehicle_id, tr)| VehicleBox {
            vehicle_id,
            at: tr.at,
            length: tr.length,
            speed: tr.speed,
            track: tr.pos,
        })
        .collect()
}
