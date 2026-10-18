//! The run's signals (vis-003 Phase 1): the engine's own signal code stepped once through
//! the run into a timeline (§2.6), and a dot at every signalised approach's stop line,
//! drawn as the engine's dashboard draws it (§2.9–§2.11). No Bevy types.
//!
//! Copied from the engine at the pin `90b39292` (vis-001 §2.2.2), each with its file and
//! line there (vis-003 §2.16 says what a pin move re-reads):
//! - which approaches, where: `crates/geometry/src/frame_collect.rs`,
//!   `collect_signal_state` (l. 537–561): one per `signalized_approach_links()`, in that
//!   order (l. 542), at `s = max(link_length − stop_line_offset, 0)` (l. 549) on lane 0
//!   (l. 550), left out where it cannot be placed;
//! - the look: `web/src/canvas/vehicleRenderer.ts`, `drawSignals` (l. 151), and the dark
//!   palette of `web/src/canvas/colors.ts`, beside each constant below.

use std::path::Path;

use anyhow::Result;
use assimilator_config::types::LinkId;
use assimilator_core::systems::signal::SignalState;

use crate::camera::{self, Pose};
use crate::inputs;
use crate::place::Placement;

/// `SIGNAL_RED` `#ef4444`, `colors.ts` l. 84.
pub const RED: [u8; 3] = [0xef, 0x44, 0x44];
/// `SIGNAL_AMBER` `#f59e0b`, `colors.ts` l. 85.
pub const AMBER: [u8; 3] = [0xf5, 0x9e, 0x0b];
/// `SIGNAL_GREEN` `#22c55e`, `colors.ts` l. 86.
pub const GREEN: [u8; 3] = [0x22, 0xc5, 0x5e];
/// `VEHICLE_STROKE` `#000000`, `colors.ts` l. 121: the stroke of `vehicleRenderer.ts`
/// l. 168.
pub const OUTLINE: [u8; 3] = [0x00, 0x00, 0x00];
/// The fill's radius at 1080 lines, px: `radius = 5` (`vehicleRenderer.ts` l. 158) less
/// half the stroke's `lineWidth = 1` (l. 169), drawn after the fill (l. 166).
pub const FILL_PX: f64 = 4.5;
/// The outline's outer radius at 1080 lines, px: `radius = 5` plus half the stroke.
pub const OUTER_PX: f64 = 5.5;
/// A dot's lowest point above the road, m: over the markings (0.02), under the boxes.
pub const LIFT_M: f64 = 0.03;
/// Segments of a dot's circle.
pub const SEGMENTS: usize = 32;

/// An approach's colour, as `link_signal_color` names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colour {
    Green,
    Amber,
    Red,
}

impl Colour {
    /// The engine's string; anything else is red, as `SIGNAL_COLORS[s.state] ??
    /// SIGNAL_RED` (`vehicleRenderer.ts` l. 144–148 and 166).
    pub fn of(s: &str) -> Colour {
        match s {
            "green" => Colour::Green,
            "amber" => Colour::Amber,
            _ => Colour::Red,
        }
    }

    /// Its fill, sRGB.
    pub fn srgb(self) -> [u8; 3] {
        match self {
            Colour::Green => GREEN,
            Colour::Amber => AMBER,
            Colour::Red => RED,
        }
    }
}

/// One signalised approach link and its dot.
#[derive(Debug, Clone, PartialEq)]
pub struct Approach {
    pub node: String,
    pub link: String,
    /// The dot's point on the road, world metres.
    pub at: [f64; 2],
    /// `(k, colour)` from `k = 0`: the colour from step `k` until the next entry.
    pub changes: Vec<(u32, Colour)>,
}

/// The run's signal states: every approach's changes, and `t[k]`, the clock after `k`
/// steps (`t[0] = 0`), summed as the engine sums it.
#[derive(Debug, Clone, PartialEq)]
pub struct Signals {
    pub approaches: Vec<Approach>,
    pub t: Vec<f64>,
}

impl Signals {
    /// Step the engine's `SignalState` from `new` through the run, as its loop does (while
    /// `time < duration − 1e-9`), and keep each approach's changes of `link_signal_color`
    /// (§2.6). The approaches are `signalized_approach_links()`'s, in its sorted order.
    pub fn build(pl: &Placement, dt: f64, duration: f64) -> Signals {
        let nd = &pl.data;
        let mut state = SignalState::new(nd);
        let mut ids: Vec<LinkId> = Vec::new();
        let mut approaches: Vec<Approach> = Vec::new();
        for (node, link) in state.signalized_approach_links() {
            let s = (nd.link_length(&link) - nd.stop_line_offset(&link)).max(0.0);
            let Some(p) = pl.place(&link.0, s, 0) else {
                continue;
            };
            approaches.push(Approach {
                node: node.0.clone(),
                link: link.0.clone(),
                at: [p.x, p.y],
                changes: vec![(0, Colour::of(state.link_signal_color(nd, &link)))],
            });
            ids.push(link);
        }
        let mut t = vec![0.0f64];
        let mut time = 0.0f64;
        let mut k = 0u32;
        while time < duration - 1e-9 {
            state.advance(dt);
            time += dt;
            k += 1;
            t.push(time);
            for (a, id) in approaches.iter_mut().zip(&ids) {
                let c = Colour::of(state.link_signal_color(nd, id));
                if a.changes.last().map(|x| x.1) != Some(c) {
                    a.changes.push((k, c));
                }
            }
        }
        Signals { approaches, t }
    }

    /// The run's timeline, or `None` with nothing read on a network with no signal
    /// (§2.13): the clock from `results.db` ([`inputs::run_clock`]), then [`Signals::build`].
    pub fn for_run(
        pl: &Placement,
        results: &Path,
        scenario: &str,
        seed: u64,
    ) -> Result<Option<Signals>> {
        if SignalState::new(&pl.data)
            .signalized_approach_links()
            .is_empty()
        {
            return Ok(None);
        }
        let (dt, duration) = inputs::run_clock(results, scenario, seed)?;
        Ok(Some(Signals::build(pl, dt, duration)))
    }

    /// The step shown at sim time `t`: the last `k` with `t[k] ≤ t + 1e-6`, 0 before the
    /// first step.
    pub fn step_at(&self, t: f64) -> usize {
        self.t.partition_point(|&x| x <= t + 1e-6).max(1) - 1
    }

    /// Every approach's colour at sim time `t` ([`Signals::step_at`]), in approach order.
    pub fn colours_at(&self, t: f64) -> Vec<Colour> {
        let k = self.step_at(t);
        self.approaches
            .iter()
            .map(|a| {
                let j = a.changes.partition_point(|&(s, _)| s as usize <= k);
                a.changes[j - 1].1
            })
            .collect()
    }
}

/// The camera a frame's dots are sized for.
#[derive(Debug, Clone, Copy)]
pub enum Eye<'a> {
    /// The orthographic camera at `k` metres per pixel.
    Ortho(f64),
    /// A perspective camera at a pose.
    Pose(&'a Pose),
}

/// The dots' one mesh: world metres `(x, y, z)`, an sRGB colour per vertex, triangles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DotsData {
    pub positions: Vec<[f64; 3]>,
    pub colours: Vec<[u8; 3]>,
    pub indices: Vec<u32>,
}

/// A frame's pixel scale, as the credit line's: 1 at 1080 lines, `min(h, 9w/16) / 1080`.
pub fn scale(w: f64, h: f64) -> f64 {
    h.min(9.0 * w / 16.0) / 1080.0
}

/// The dots for `colours` (in approach order) in a `w × h` frame (§2.11). Per dot, in
/// approach order: the outline ring (32 inner and 32 outer vertices, 64 triangles), then
/// the fill (its centre and 32 rim vertices, 32 triangles). Orthographic: flat at
/// [`LIFT_M`], radii `FILL_PX·s·k` and `OUTER_PX·s·k`. Perspective: facing the camera,
/// its lowest point [`LIFT_M`] above the stop line, sized so that it projects to a circle
/// of exactly those pixel radii; a dot behind the camera is left out.
pub fn dots(sig: &Signals, colours: &[Colour], eye: Eye, w: f64, h: f64) -> DotsData {
    let sc = scale(w, h);
    let (rf, ro) = (FILL_PX * sc, OUTER_PX * sc);
    let n = SEGMENTS as u32;
    let mut d = DotsData::default();
    for (a, c) in sig.approaches.iter().zip(colours) {
        // The centre, the disc's two in-plane axes, and metres per pixel on it.
        let (ctr, ax_r, ax_u, mpp) = match eye {
            Eye::Ortho(k) => (
                [a.at[0], a.at[1], LIFT_M],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                k,
            ),
            Eye::Pose(pose) => {
                let (ax, u, r) = pose.axes();
                let e = pose.eye();
                let f = (h / 2.0) / (camera::FOV_DEG / 2.0).to_radians().tan();
                let zc = (a.at[0] - e[0]) * ax[0] + (a.at[1] - e[1]) * ax[1] + (0.0 - e[2]) * ax[2];
                let sp = -ax[2]; // sin(pitch)
                let cp = u[2]; // cos(pitch)
                let r_out = ro * (zc - LIFT_M * sp) / (f + ro * cp * sp);
                // The disc's view depth is f·ρ/R.
                if !(r_out > 0.0) || zc - (LIFT_M + r_out * cp) * sp <= 0.1 {
                    continue;
                }
                ([a.at[0], a.at[1], LIFT_M + r_out * cp], r, u, r_out / ro)
            }
        };
        let pt = |rad: f64, i: usize| {
            let th = 2.0 * std::f64::consts::PI * i as f64 / SEGMENTS as f64;
            let (x, y) = (th.cos() * rad * mpp, th.sin() * rad * mpp);
            [
                ctr[0] + x * ax_r[0] + y * ax_u[0],
                ctr[1] + x * ax_r[1] + y * ax_u[1],
                ctr[2] + x * ax_r[2] + y * ax_u[2],
            ]
        };
        let base = d.positions.len() as u32;
        for rad in [rf, ro] {
            for i in 0..SEGMENTS {
                d.positions.push(pt(rad, i));
                d.colours.push(OUTLINE);
            }
        }
        for i in 0..n {
            let j = (i + 1) % n;
            let (a0, a1, b0, b1) = (base + i, base + j, base + n + i, base + n + j);
            d.indices.extend([a0, b0, b1, a0, b1, a1]);
        }
        let fill = c.srgb();
        let cbase = d.positions.len() as u32;
        d.positions.push(ctr);
        d.colours.push(fill);
        for i in 0..SEGMENTS {
            d.positions.push(pt(rf, i));
            d.colours.push(fill);
        }
        for i in 0..n {
            let j = (i + 1) % n;
            d.indices.extend([cbase, cbase + 1 + i, cbase + 1 + j]);
        }
    }
    d
}
