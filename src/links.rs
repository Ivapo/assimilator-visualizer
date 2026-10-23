//! Link colours (vis-003 §2.21): each link, or each lane of it, coloured by a measure of
//! its FCD rows over a past window ending at the frame's time, binned into four steps of
//! one palette, and drawn as a tint over its road strip. No Bevy types: the units, the
//! window, the four measures, the steps, the palette, the tint's mesh data and the
//! legend's text; `src/draw.rs` draws them.

use std::collections::HashMap;

use anyhow::{Result, bail};

use crate::fcd::{Fcd, Row};
use crate::place::Placement;
use crate::scene::STRIP_STEP;

/// §2.21.3's bound on each end of the window, sim seconds (vis-001's).
pub const EPS: f64 = 1e-6;
/// A row under this speed is a stopped car, m/s (§2.21.4).
pub const STOPPED: f64 = 0.1;
/// The tint's height over the road, metres (§2.21.10): under every marking.
pub const LIFT_M: f64 = 0.005;
/// `--link-window`'s default, seconds.
pub const WINDOW_S: f64 = 60.0;
/// The palette, best → worst (OQ-9 (b), blue to red): `#2c7bb6`, `#9ecae1`, `#fdae61`,
/// `#d7191c`.
pub const PALETTE: [[u8; 3]; 4] = [
    [0x2c, 0x7b, 0xb6],
    [0x9e, 0xca, 0xe1],
    [0xfd, 0xae, 0x61],
    [0xd7, 0x19, 0x1c],
];
/// Every box's colour while link colours are on (OQ-8), sRGB.
pub const CARS: [u8; 3] = [235, 235, 235];

/// What a link is coloured by (§2.21.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// Speed as a share of the lane's limit.
    Relative,
    Speed,
    /// Cars per lane-km.
    Density,
    /// Stopped cars.
    Stopped,
}

impl Measure {
    pub const ALL: [Measure; 4] = [
        Measure::Relative,
        Measure::Speed,
        Measure::Density,
        Measure::Stopped,
    ];
    /// The CLI names, in `ALL`'s order.
    pub const NAMES: [&'static str; 4] = ["relative", "speed", "density", "stopped"];

    /// `--link-colours`'s name for it.
    pub fn name(self) -> &'static str {
        Self::NAMES[self as usize]
    }

    pub fn from_name(s: &str) -> Option<Measure> {
        Self::NAMES.iter().position(|n| *n == s).map(|i| Self::ALL[i])
    }

    /// `C` in `view`: off → `relative` → `speed` → `density` → `stopped` → off.
    pub fn next(m: Option<Measure>) -> Option<Measure> {
        match m {
            None => Some(Measure::Relative),
            Some(m) => Self::ALL.get(m as usize + 1).copied(),
        }
    }

    /// The steps' three edges, ascending in the binned value (§2.21.5): percent, km/h,
    /// cars per lane-km, stopped cars.
    pub fn edges(self) -> [f64; 3] {
        match self {
            Measure::Relative => [25.0, 50.0, 75.0],
            Measure::Speed => [10.0, 20.0, 30.0],
            Measure::Density => [3.0, 6.0, 12.0],
            Measure::Stopped => [0.25, 0.5, 1.0],
        }
    }

    /// The counts: a higher value is a worse step.
    pub fn higher_is_worse(self) -> bool {
        matches!(self, Measure::Density | Measure::Stopped)
    }

    /// The legend's title and its four ranges, in ascending value (§2.21.9).
    pub fn legend_text(self) -> (&'static str, [&'static str; 4]) {
        match self {
            Measure::Relative => ("speed / limit", ["< 25 %", "25–50 %", "50–75 %", "≥ 75 %"]),
            Measure::Speed => ("speed", ["< 10", "10–20", "20–30", "≥ 30 km/h"]),
            Measure::Density => ("cars / lane-km", ["< 3", "3–6", "6–12", "≥ 12"]),
            Measure::Stopped => ("stopped cars", ["< 0.25", "0.25–0.5", "0.5–1", "≥ 1"]),
        }
    }
}

/// The legend: its title, and each step's range and colour in ascending value.
pub type Legend = (&'static str, [(&'static str, [u8; 3]); 4]);

/// The legend for `m` (§2.21.9).
pub fn legend(m: Measure) -> Legend {
    let (title, labels) = m.legend_text();
    let pair = |i: usize| {
        let step = if m.higher_is_worse() { i } else { 3 - i };
        (labels[i], PALETTE[step])
    };
    (title, [pair(0), pair(1), pair(2), pair(3)])
}

/// One coloured unit: a link, or one lane of it (§2.21.4, the scope's fields).
#[derive(Debug, Clone, PartialEq)]
pub struct Unit {
    /// The link's name.
    pub link: String,
    /// `None` for a link; `Some(k)` for its lane `k`.
    pub lane: Option<u32>,
    /// The trimmed `Placement::link_length`, metres.
    pub length: f64,
    /// What density divides by: the link's lane count, or 1 for a lane.
    pub lanes: u32,
    /// The `speed_limit` of each lane it covers, in lane order (one for a lane), m/s.
    pub limits: Vec<f64>,
    /// Its lateral extent from the link's centreline, metres, right positive.
    pub span: [f64; 2],
}

/// The run's rows, each tied to its unit and limit, by time.
pub struct Links {
    pub units: Vec<Unit>,
    /// Every row of the file with a unit, in (`time`, `vehicle_id`) order: its unit, its
    /// speed and its lane's limit.
    rows: Vec<(u32, f64, f64)>,
    /// The distinct FCD times of the whole file, ascending, and where each one's rows
    /// start in `rows`; `starts` has one more entry, the end.
    times: Vec<f64>,
    starts: Vec<usize>,
    interval: Option<f64>,
}

impl Links {
    /// The units of the network, in its link order, lanes in order within a link: one per
    /// link, or one per lane when `per_lane`; then [`Links::with_units`].
    pub fn new(pl: &Placement, fcd: &Fcd, per_lane: bool) -> Links {
        let mut units = Vec::new();
        for link in &pl.network.links {
            let id = link.id.0.as_str();
            let length = pl.link_length(id);
            if per_lane {
                let offs = link.lane_center_offsets();
                for (k, l) in link.lanes.iter().enumerate() {
                    units.push(Unit {
                        link: id.into(),
                        lane: Some(k as u32),
                        length,
                        lanes: 1,
                        limits: vec![l.speed_limit],
                        span: [offs[k] - l.width / 2.0, offs[k] + l.width / 2.0],
                    });
                }
            } else {
                let half = link.total_width() / 2.0;
                units.push(Unit {
                    link: id.into(),
                    lane: None,
                    length,
                    lanes: link.lanes.len() as u32,
                    limits: link.lanes.iter().map(|l| l.speed_limit).collect(),
                    span: [-half, half],
                });
            }
        }
        Self::with_units(units, fcd)
    }

    /// Every row of the file from `Fcd::vehicles`, in (`time`, `vehicle_id`) order, each
    /// given to the unit of its `link_id` whose `lane` is `None` or the row's lane, a lane
    /// at or past the link's last going to the last; its limit is `limits[lane]` for a link
    /// (the same clamp) and `limits[0]` for a lane. The FCD times, and the interval: the
    /// smallest gap between successive times (§2.21.3).
    pub fn with_units(units: Vec<Unit>, fcd: &Fcd) -> Links {
        // Looked up by name only, never walked.
        let mut by_name: HashMap<&str, Vec<usize>> = HashMap::new();
        for (u, unit) in units.iter().enumerate() {
            by_name.entry(unit.link.as_str()).or_default().push(u);
        }
        // Each FCD link's units: its link unit, or its lane units by lane.
        enum Of {
            None,
            Link(usize),
            Lanes(Vec<Option<usize>>),
        }
        let of: Vec<Of> = fcd
            .links
            .iter()
            .map(|name| match by_name.get(name.as_str()) {
                None => Of::None,
                Some(us) => match us.iter().find(|&&u| units[u].lane.is_none()) {
                    Some(&u) => Of::Link(u),
                    None => {
                        let last = us.iter().filter_map(|&u| units[u].lane).max().unwrap_or(0);
                        let mut by_lane = vec![None; last as usize + 1];
                        for &u in us {
                            if let Some(k) = units[u].lane {
                                by_lane[k as usize] = Some(u);
                            }
                        }
                        Of::Lanes(by_lane)
                    }
                },
            })
            .collect();
        let mut all: Vec<Row> = fcd
            .vehicles
            .iter()
            .flat_map(|v| v.rows.iter().copied())
            .collect();
        all.sort_by(|a, b| a.time.total_cmp(&b.time).then(a.vehicle_id.cmp(&b.vehicle_id)));
        let mut rows = Vec::with_capacity(all.len());
        let mut times = Vec::new();
        let mut starts = Vec::new();
        for r in &all {
            if times.last() != Some(&r.time) {
                times.push(r.time);
                starts.push(rows.len());
            }
            let at = match &of[r.link as usize] {
                Of::None => None,
                Of::Link(u) => {
                    let lim = &units[*u].limits;
                    lim.get((r.lane as usize).min(lim.len().saturating_sub(1)))
                        .map(|&l| (*u, l))
                }
                Of::Lanes(by_lane) => by_lane[(r.lane as usize).min(by_lane.len() - 1)]
                    .and_then(|u| units[u].limits.first().map(|&l| (u, l))),
            };
            if let Some((u, limit)) = at {
                rows.push((u as u32, r.speed, limit));
            }
        }
        starts.push(rows.len());
        let interval = times
            .windows(2)
            .map(|w| w[1] - w[0])
            .reduce(f64::min);
        Links {
            units,
            rows,
            times,
            starts,
            interval,
        }
    }

    /// The run's FCD interval (§2.21.3); `None` for a file with one FCD time.
    pub fn interval(&self) -> Option<f64> {
        self.interval
    }

    /// The window's FCD times, as a range of `times`: `t − w + EPS < time ≤ t + EPS`.
    fn window(&self, t: f64, w: f64) -> (usize, usize) {
        let hi = self.times.partition_point(|&x| x <= t + EPS);
        let lo = self.times.partition_point(|&x| x <= t - w + EPS);
        (lo, hi.max(lo))
    }

    /// Each unit's value of `m` over the window `w` ending at `t` (§2.21.3, §2.21.4): the
    /// speed measures `None` with no row, the counts 0 with none.
    pub fn values_at(&self, t: f64, w: f64, m: Measure) -> Vec<Option<f64>> {
        let (lo, hi) = self.window(t, w);
        let n_units = self.units.len();
        let mut count = vec![0u64; n_units];
        let mut sum = vec![0.0f64; n_units];
        let mut stopped = vec![0u64; n_units];
        for &(u, speed, limit) in &self.rows[self.starts[lo]..self.starts[hi]] {
            let u = u as usize;
            count[u] += 1;
            match m {
                Measure::Relative => sum[u] += speed / limit,
                Measure::Speed => sum[u] += speed,
                Measure::Density => {}
                Measure::Stopped => {
                    if speed < STOPPED {
                        stopped[u] += 1;
                    }
                }
            }
        }
        let n = (hi - lo) as f64;
        (0..n_units)
            .map(|u| match m {
                Measure::Relative | Measure::Speed => {
                    (count[u] > 0).then(|| sum[u] / count[u] as f64)
                }
                Measure::Density => Some(if n > 0.0 {
                    let unit = &self.units[u];
                    count[u] as f64 / n / (unit.length * unit.lanes as f64 / 1000.0)
                } else {
                    0.0
                }),
                Measure::Stopped => Some(if n > 0.0 { stopped[u] as f64 / n } else { 0.0 }),
            })
            .collect()
    }

    /// Each unit's colour at `t`: its step's, or `None` with no value.
    pub fn colours_at(&self, t: f64, w: f64, m: Measure) -> Vec<Option<[u8; 3]>> {
        self.values_at(t, w, m)
            .into_iter()
            .map(|v| v.map(|v| PALETTE[step(m, v) as usize]))
            .collect()
    }
}

/// `--link-window`'s two errors (§2.21.11), in this order: not a positive number; shorter
/// than the run's FCD interval, less `EPS`. A run with one FCD time takes any positive
/// window.
pub fn check_window(w: f64, interval: Option<f64>) -> Result<()> {
    if !(w.is_finite() && w > 0.0) {
        bail!("--link-window must be a positive number of seconds (got {w})");
    }
    if let Some(i) = interval
        && w < i - EPS
    {
        bail!(
            "--link-window {w} s is shorter than the run's FCD interval ({i:.3} s); \
             --link-window {i:.3} colours by each frame's own FCD row"
        );
    }
    Ok(())
}

/// A value's step, 0 best … 3 worst (§2.21.5): the speed measures on the whole number a
/// reader sees, halves up (`relative` in percent, `speed` in km/h), the counts on their
/// value; an edge is in the step above it.
pub fn step(m: Measure, v: f64) -> u8 {
    let v = match m {
        Measure::Relative => (100.0 * v + 0.5).floor(),
        Measure::Speed => (3.6 * v + 0.5).floor(),
        Measure::Density | Measure::Stopped => v,
    };
    let i = m.edges().iter().position(|&e| v < e).unwrap_or(3) as u8;
    if m.higher_is_worse() { i } else { 3 - i }
}

/// The tint's mesh data: every vertex's position (world metres, `z` = [`LIFT_M`]) and its
/// unit, and each unit's triangles.
#[derive(Debug, Clone, Default)]
pub struct TintData {
    pub positions: Vec<[f64; 3]>,
    pub unit: Vec<u32>,
    pub tris: Vec<Vec<u32>>,
}

/// Each unit's strip (§2.21.7 (a)): the road strip's samples (`scene::strips`'s, at most
/// [`STRIP_STEP`] apart) at the unit's `span`, in unit order. A unit whose link has no
/// placed geometry has no triangle.
pub fn tint(pl: &Placement, links: &Links) -> TintData {
    let mut d = TintData::default();
    for (ui, u) in links.units.iter().enumerate() {
        let n = ((u.length / STRIP_STEP).ceil() as usize).max(1);
        let base = d.positions.len() as u32;
        let mut k = 0u32;
        for i in 0..=n {
            let s = u.length * i as f64 / n as f64;
            let Some(p) = pl.place_lateral(&u.link, s, 0.0) else {
                break;
            };
            let h = p.heading.to_radians();
            // Right of travel: (cos h, −sin h) for a heading clockwise from north.
            let (rx, ry) = (h.cos(), -h.sin());
            for l in u.span {
                d.positions.push([p.x + rx * l, p.y + ry * l, LIFT_M]);
                d.unit.push(ui as u32);
            }
            k += 1;
        }
        let mut t = Vec::new();
        for i in 0..k.saturating_sub(1) {
            let (a, b, c, e) = (base + 2 * i, base + 2 * i + 1, base + 2 * i + 2, base + 2 * i + 3);
            t.extend_from_slice(&[a, b, c, b, e, c]);
        }
        d.tris.push(t);
    }
    d
}
