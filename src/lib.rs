//! assimilator-video: renders a video of one Assimilator simulation run (vis-001).
//!
//! The library exposes what the Phase 1 gates measure through: one frame rendered to its
//! RGBA buffer ([`Job::render_at`]), the camera's world-to-pixel transform
//! ([`scene::Camera::world_to_pixel`]) and its metres per pixel ([`Job::k`]). Phase 2
//! adds the boxes at any time with their track positions ([`Job::boxes_at`]) and the
//! motion report ([`Job::motion_report`]).

pub mod clock;
pub mod encode;
pub mod fcd;
pub mod inputs;
pub mod motion;
pub mod place;
pub mod render;
pub mod scene;

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::fcd::Fcd;
use crate::inputs::RunPaths;
use crate::motion::{Motion, MotionReport};
use crate::place::{Placed, Placement};
use crate::render::{Renderer, VehicleBox};
use crate::scene::Camera;

/// What `render` needs, after CLI parsing (vis-001 §2.4).
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub project: PathBuf,
    pub scenario: String,
    pub seed: u64,
    pub results: Option<PathBuf>,
    pub fcd: Option<PathBuf>,
    pub from: Option<f64>,
    pub to: Option<f64>,
    pub speedup: Option<f64>,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
}

/// The fixed clock of one render.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Clock {
    pub from: f64,
    pub to: f64,
    pub speedup: f64,
    pub fps: u32,
    pub frames: u64,
}

impl Clock {
    pub fn time_of(&self, n: u64) -> f64 {
        clock::frame_time(self.from, n, self.speedup, self.fps)
    }
}

/// A validated render: every input check has passed and every row is placed.
pub struct Job {
    pub clock: Clock,
    pub fcd: Fcd,
    pub placed: Vec<Placed>,
    pub placement: Placement,
    pub motion: Motion,
    renderer: Renderer,
}

impl Job {
    /// Run every check that can fail on the inputs, place every row, build every
    /// vehicle's track from the whole file, and build the scene. No frame is rendered.
    pub fn prepare(o: &RenderOptions) -> Result<Job> {
        if o.width == 0
            || o.height == 0
            || !o.width.is_multiple_of(2)
            || !o.height.is_multiple_of(2)
        {
            bail!(
                "--width and --height must be even and positive (got {}x{})",
                o.width,
                o.height
            );
        }
        if o.fps == 0 {
            bail!("--fps must be positive");
        }
        if let Some(s) = o.speedup
            && !(s.is_finite() && s > 0.0)
        {
            bail!("--speedup must be positive (got {s})");
        }
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
        let d = to - from;
        let speedup = o.speedup.unwrap_or_else(|| clock::default_speedup(d));
        let clock = Clock {
            from,
            to,
            speedup,
            fps: o.fps,
            frames: clock::frame_count(d, o.fps, speedup),
        };

        let fcd = fcd::read_window(&paths.fcd, from, to)?;
        let placement = Placement::new(network);
        let placed = placement.place_all(&fcd)?;
        let strips = scene::strips(&placement);
        if strips.is_empty() {
            bail!("the scenario's network has no drawable links");
        }
        let camera = Camera::fit(&strips, o.width, o.height);
        let motion = Motion::build(&fcd, &placement);
        let renderer = Renderer::new(&strips, camera, motion.max_drawn())?;
        Ok(Job {
            clock,
            fcd,
            placed,
            placement,
            motion,
            renderer,
        })
    }

    pub fn camera(&self) -> &Camera {
        self.renderer.camera()
    }

    /// Metres per pixel.
    pub fn k(&self) -> f64 {
        self.camera().k
    }

    /// The boxes shown at sim time `t`: every vehicle with `t_first − 1e-6 ≤ t ≤
    /// t_last + 1e-6`, where its track puts it at `t` (vis-001 §2.8), in `vehicle_id`
    /// order.
    pub fn boxes_at(&self, t: f64) -> Vec<VehicleBox> {
        self.motion
            .at(t, &self.fcd, &self.placement)
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

    /// What motion found in the whole file (§2.8.6's evidence).
    pub fn motion_report(&self) -> &MotionReport {
        self.motion.report()
    }

    /// Frame `n`'s lossless RGBA readback.
    pub fn render_frame(&mut self, n: u64) -> Result<Vec<u8>> {
        self.render_at(self.clock.time_of(n))
    }

    /// The RGBA readback of the scene at sim time `t`.
    pub fn render_at(&mut self, t: f64) -> Result<Vec<u8>> {
        let boxes = self.boxes_at(t);
        self.renderer.render(&boxes)
    }

    /// The same frame with no vehicles.
    pub fn render_empty(&mut self) -> Result<Vec<u8>> {
        self.renderer.render(&[])
    }
}
