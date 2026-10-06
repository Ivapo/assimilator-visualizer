//! assimilator-video: renders a video of one Assimilator simulation run (vis-001).
//!
//! The library exposes what the Phase 1 gates measure through: one frame rendered to its
//! RGBA buffer ([`Job::render_at`]), the camera's world-to-pixel transform
//! ([`scene::Camera::world_to_pixel`]) and its metres per pixel ([`Job::k`]). Phase 2
//! adds the boxes at any time with their track positions ([`Job::boxes_at`]) and the
//! motion report ([`Job::motion_report`]). Phase 5 adds a keyframed render through a
//! perspective camera ([`Job::prepare_with_camera`]) and its pose at any time
//! ([`Job::pose_at`]). vis-002 adds the buildings of a `--buildings` file
//! ([`Job::prepare_with`], [`Job::buildings`]), and the credit line of an imported
//! network's render ([`Job::credit`], [`Job::credit_size`], [`Job::credit_box`]), with
//! the same job without it for the gates ([`Job::prepare_without_credit`]). vis-002
//! Phase 3 cuts the buildings in the camera's way to stubs ([`Job::set_see_through`]).

pub mod buildings;
pub mod camera;
pub mod clock;
pub mod credit;
pub mod draw;
pub mod encode;
pub mod fcd;
pub mod inputs;
pub mod keyframes;
pub mod motion;
pub mod place;
pub mod render;
pub mod run;
pub mod scene;
pub mod see_through;
pub mod streets;
pub mod view;

use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, bail};

use crate::buildings::Buildings;
use crate::camera::Pose;
use crate::keyframes::Flight;

use crate::fcd::Fcd;
use crate::motion::{Motion, MotionReport};
use crate::place::{Placed, Placement};
use crate::render::{Renderer, VehicleBox};
use crate::run::LoadOptions;
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
    /// The keyframed camera; `None` for the orthographic job.
    flight: Option<Flight>,
    /// The `--buildings` file's buildings; `None` without one.
    buildings: Option<Buildings>,
    /// The credit line drawn on every frame; `None` without one.
    credit: Option<String>,
}

impl Job {
    /// Run every check that can fail on the inputs, place every row, build every
    /// vehicle's track from the whole file, and build the scene. No frame is rendered.
    pub fn prepare(o: &RenderOptions) -> Result<Job> {
        Self::prepare_with(o, None, None)
    }

    /// [`Job::prepare`] with a keyframe file at `camera` (vis-001 §2.11.4).
    pub fn prepare_with_camera(o: &RenderOptions, camera: &Path) -> Result<Job> {
        Self::prepare_with(o, Some(camera), None)
    }

    /// [`Job::prepare`]'s run, then the keyframe file at `camera`, if any: read, checked
    /// against the run and built into a flight (vis-001 §2.11.4). Then the buildings file
    /// at `buildings`, if any: read and checked against the run's network (vis-002
    /// §2.4.3). Then the credit line, for a network with `metadata.map_origin` (vis-002
    /// §2.14.2): the project's `import_report.json` and the buildings file's provenance,
    /// checked. Then the scene: Phase 1's, through a perspective camera set from the
    /// flight each frame when there is one, with the buildings and the sun when given,
    /// and the line fitted to the frame (§2.14.5).
    pub fn prepare_with(
        o: &RenderOptions,
        camera: Option<&Path>,
        buildings: Option<&Path>,
    ) -> Result<Job> {
        Self::prepare_inner(o, camera, buildings, true)
    }

    /// [`Job::prepare_with`] with no credit line, whatever the network: for the gates
    /// (vis-002 §2.14.7). No flag reaches it.
    pub fn prepare_without_credit(
        o: &RenderOptions,
        camera: Option<&Path>,
        buildings: Option<&Path>,
    ) -> Result<Job> {
        Self::prepare_inner(o, camera, buildings, false)
    }

    fn prepare_inner(
        o: &RenderOptions,
        camera: Option<&Path>,
        buildings: Option<&Path>,
        with_credit: bool,
    ) -> Result<Job> {
        let (clock, run) = Self::load(o)?;
        let flight = match camera {
            None => None,
            Some(camera) => {
                let err = |e: String| anyhow!("--camera {}: {e}", camera.display());
                let keyframes = keyframes::read(camera).map_err(err)?;
                keyframes::check_follows(&keyframes, &run.motion, &run.fcd, &run.placement)
                    .map_err(err)?;
                Some(Flight::new(keyframes, &run.fcd))
            }
        };
        let buildings_file = buildings;
        let buildings = match buildings {
            None => None,
            Some(file) => Some(
                crate::buildings::read(file, &run.placement.network)
                    .map_err(|e| anyhow!("--buildings {}: {e}", file.display()))?,
            ),
        };
        let credit = if with_credit {
            credit::line(&run.placement.network, &o.project, buildings_file)?
        } else {
            None
        };
        let fit = Camera::fit(&run.strips, o.width, o.height);
        let pool = run.motion.max_drawn();
        let line = credit.as_deref();
        let renderer = match &flight {
            None => Renderer::new(&run.strips, fit, pool, buildings.as_ref(), line)?,
            Some(flight) => {
                let pose = flight.pose_at(clock.from, &run.motion, &run.fcd, &run.placement);
                Renderer::new_perspective(&run.strips, fit, pool, &pose, buildings.as_ref(), line)?
            }
        };
        Ok(Self::assemble(
            clock, run, renderer, flight, buildings, credit,
        ))
    }

    fn assemble(
        clock: Clock,
        run: run::Run,
        renderer: Renderer,
        flight: Option<Flight>,
        buildings: Option<Buildings>,
        credit: Option<String>,
    ) -> Job {
        let run::Run {
            fcd,
            placed,
            placement,
            motion,
            ..
        } = run;
        Job {
            clock,
            fcd,
            placed,
            placement,
            motion,
            renderer,
            flight,
            buildings,
            credit,
        }
    }

    /// Every check that can fail on the inputs, and the run loaded.
    fn load(o: &RenderOptions) -> Result<(Clock, run::Run)> {
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
        let run = run::load(&LoadOptions {
            project: o.project.clone(),
            scenario: o.scenario.clone(),
            seed: o.seed,
            results: o.results.clone(),
            fcd: o.fcd.clone(),
            from: o.from,
            to: o.to,
        })?;
        let (from, to) = (run.from, run.to);
        let d = to - from;
        let speedup = o.speedup.unwrap_or_else(|| clock::default_speedup(d));
        let clock = Clock {
            from,
            to,
            speedup,
            fps: o.fps,
            frames: clock::frame_count(d, o.fps, speedup),
        };
        Ok((clock, run))
    }

    /// The keyframed camera's pose at sim time `t`; `None` for the orthographic job.
    pub fn pose_at(&self, t: f64) -> Option<Pose> {
        self.flight
            .as_ref()
            .map(|f| f.pose_at(t, &self.motion, &self.fcd, &self.placement))
    }

    /// The fit: the scene's bake origin, and the orthographic job's camera.
    pub fn camera(&self) -> &Camera {
        self.renderer.camera()
    }

    /// The `--buildings` file's buildings; `None` without one.
    pub fn buildings(&self) -> Option<&Buildings> {
        self.buildings.as_ref()
    }

    /// See-through (vis-002 §2.15.6), from the next frame: on cuts the buildings in the
    /// way to stubs at each frame's pose. No effect on a job without buildings or without
    /// a flight. Every `prepare*` builds a job with it off; `render` turns it on.
    pub fn set_see_through(&mut self, on: bool) {
        let cut = if on { self.buildings.as_ref() } else { None };
        self.renderer.set_see_through(cut);
    }

    /// Streets (vis-002 §2.17.12): on builds them from the job's placement and draws them
    /// from the next frame; off removes them. Every `prepare*` builds a job with them off;
    /// `render` turns them on unless `--no-streets` is given.
    pub fn set_streets(&mut self, on: bool) -> Result<()> {
        let streets = on.then(|| crate::streets::Streets::build(&self.placement));
        self.renderer.set_streets(streets.as_ref())
    }

    /// The credit line drawn on every frame (vis-002 §2.14.2); `None` without one.
    pub fn credit(&self) -> Option<&str> {
        self.credit.as_deref()
    }

    /// The credit line's font size after the fit, px (§2.14.5); `None` without one.
    pub fn credit_size(&self) -> Option<u32> {
        self.renderer.credit_size()
    }

    /// The credit box, `[x0, y0, x1, y1]` in pixels with `x1` and `y1` exclusive: the
    /// fill text's rectangle grown by the outline offset (§2.14.5); `None` without a line.
    pub fn credit_box(&self) -> Option<[u32; 4]> {
        self.renderer.credit_box()
    }

    /// Metres per pixel.
    pub fn k(&self) -> f64 {
        self.camera().k
    }

    /// The boxes shown at sim time `t` ([`run::boxes_at`]).
    pub fn boxes_at(&self, t: f64) -> Vec<VehicleBox> {
        run::boxes_at(&self.motion, &self.fcd, &self.placement, t)
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
        if let Some(pose) = self.pose_at(t) {
            self.renderer.set_pose(&pose);
        }
        let boxes = self.boxes_at(t);
        self.renderer.render(&boxes)
    }

    /// The same frame with no vehicles.
    pub fn render_empty(&mut self) -> Result<Vec<u8>> {
        self.renderer.render(&[])
    }
}
