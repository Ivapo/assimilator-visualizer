//! Keyframes (vis-001 §2.11.4, §2.11.5): the line `view`'s `K` prints, the file
//! `render --camera` reads, both with TOML's reader, and the flight through them. Plain
//! Rust with no Bevy types.

use serde::Deserialize;

use crate::camera::{PITCH_MAX, PITCH_MIN, Pose, short_way, wrap360};
use crate::fcd::Fcd;
use crate::motion::Motion;
use crate::place::Placement;
use crate::render::VehicleBox;
use crate::run;

/// Where a keyframe's camera is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    Centre(f64, f64),
    Follow(u64),
}

/// One keyframe line (§2.9.5, §2.11.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub t: f64,
    pub at: At,
    pub height_m: f64,
    /// In [0, 360); 0 when the line has none.
    pub yaw_deg: f64,
    /// In [`PITCH_MIN`, `PITCH_MAX`]; 90 when the line has none.
    pub pitch_deg: f64,
}

/// One keyframe as TOML gives it, before any check.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    t: Option<f64>,
    x: Option<f64>,
    y: Option<f64>,
    follow: Option<i64>,
    height_m: Option<f64>,
    yaw_deg: Option<f64>,
    pitch_deg: Option<f64>,
}

/// A line read as the value of one key.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct One {
    k: Raw,
}

/// A TOML error as one line, with no source excerpt.
fn toml_error(e: &toml::de::Error) -> String {
    e.message().trim().replace('\n', " ")
}

impl Keyframe {
    pub fn format(&self) -> String {
        let mut yaw = format!("{:.2}", self.yaw_deg);
        if yaw == "360.00" {
            yaw = "0.00".into();
        }
        let tail = format!(
            "height_m = {:.2}, yaw_deg = {yaw}, pitch_deg = {:.2}",
            self.height_m, self.pitch_deg
        );
        match self.at {
            At::Centre(x, y) => format!(
                "{{ t = {:.3}, x = {x:.2}, y = {y:.2}, {tail} }}",
                self.t
            ),
            At::Follow(id) => format!("{{ t = {:.3}, follow = {id}, {tail} }}", self.t),
        }
    }

    /// Read one line with the file's reader (`k = <line>`), and apply every check that
    /// needs no run and no other keyframe.
    pub fn parse(line: &str) -> Result<Keyframe, String> {
        let one: One = toml::from_str(&format!("k = {line}")).map_err(|e| toml_error(&e))?;
        check(one.k)
    }
}

/// The checks of §2.11.4 on one keyframe that need no run, in the order listed there.
fn check(r: Raw) -> Result<Keyframe, String> {
    let t = r.t.ok_or("no `t`")?;
    let height_m = r.height_m.ok_or("no `height_m`")?;
    let at = match (r.x, r.y, r.follow) {
        (Some(_), _, Some(_)) | (_, Some(_), Some(_)) => {
            return Err("both `x`/`y` and `follow`".into());
        }
        (None, None, None) => return Err("neither `x`/`y` nor `follow`".into()),
        (Some(_), None, None) => return Err("`x` without `y`".into()),
        (None, Some(_), None) => return Err("`y` without `x`".into()),
        (Some(x), Some(y), None) => At::Centre(x, y),
        (None, None, Some(id)) => At::Follow(id.max(0) as u64),
    };
    let finite = |name: &str, v: f64| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err(format!("`{name}` must be finite (got {v})"))
        }
    };
    finite("t", t)?;
    finite("height_m", height_m)?;
    if let At::Centre(x, y) = at {
        finite("x", x)?;
        finite("y", y)?;
    }
    let yaw = finite("yaw_deg", r.yaw_deg.unwrap_or(0.0))?;
    let pitch = finite("pitch_deg", r.pitch_deg.unwrap_or(PITCH_MAX))?;
    if let Some(id) = r.follow
        && id < 0
    {
        return Err(format!("`follow` must be a vehicle id ≥ 0 (got {id})"));
    }
    if height_m <= 0.0 {
        return Err(format!("`height_m` must be positive (got {height_m})"));
    }
    if !(PITCH_MIN..=PITCH_MAX).contains(&pitch) {
        return Err(format!(
            "`pitch_deg` must be in [{PITCH_MIN}, {PITCH_MAX}] (got {pitch})"
        ));
    }
    Ok(Keyframe {
        t,
        at,
        height_m,
        yaw_deg: wrap360(yaw),
        pitch_deg: pitch,
    })
}

// ── The keyframe file (§2.11.4) ──────────────────────────────────────────────

/// The file: one key, `keyframes`, an array of the lines (or `[[keyframes]]` tables).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    keyframes: Option<Vec<toml::Table>>,
}

/// How a keyframe is named in an error: its index from 1, and its `t` when it has one.
fn name(i: usize, t: Option<f64>) -> String {
    match t {
        Some(t) => format!("keyframe {} (t = {t})", i + 1),
        None => format!("keyframe {}", i + 1),
    }
}

/// Read a keyframe file and apply every check of §2.11.4 that needs no run, in the order
/// listed there. The follows are checked against the run by [`check_follows`].
pub fn read(path: &std::path::Path) -> Result<Vec<Keyframe>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read: {e}"))?;
    parse_file(&text)
}

/// [`read`] on the file's text.
pub fn parse_file(text: &str) -> Result<Vec<Keyframe>, String> {
    let table: toml::Table = toml::from_str(text).map_err(|e| {
        let line = e
            .span()
            .map(|s| text[..s.start.min(text.len())].matches('\n').count() + 1);
        match line {
            Some(l) => format!("not TOML (line {l}): {}", toml_error(&e)),
            None => format!("not TOML: {}", toml_error(&e)),
        }
    })?;
    let file: File = table.try_into().map_err(|e| toml_error(&e))?;
    let tables = file.keyframes.ok_or("no `keyframes`")?;
    if tables.is_empty() {
        return Err("`keyframes` is empty".into());
    }
    let mut out: Vec<Keyframe> = Vec::with_capacity(tables.len());
    for (i, table) in tables.into_iter().enumerate() {
        let t = table.get("t").and_then(|v| match v {
            toml::Value::Float(f) => Some(*f),
            toml::Value::Integer(n) => Some(*n as f64),
            _ => None,
        });
        let raw: Raw = table
            .try_into()
            .map_err(|e| format!("{}: {}", name(i, t), toml_error(&e)))?;
        let kf = check(raw).map_err(|e| format!("{}: {e}", name(i, t)))?;
        out.push(kf);
    }
    for i in 1..out.len() {
        let (a, b) = (out[i - 1].t, out[i].t);
        if b == a {
            return Err(format!(
                "{}: duplicate `t`, the same as keyframe {}",
                name(i, Some(b)),
                i
            ));
        }
        if b < a {
            return Err(format!(
                "{}: out of order, before keyframe {} (t = {a}); `t` must increase",
                name(i, Some(b)),
                i
            ));
        }
    }
    Ok(out)
}

/// The last check of §2.11.4: every `follow` names a vehicle drawn at its keyframe's `t`.
pub fn check_follows(
    keyframes: &[Keyframe],
    motion: &Motion,
    fcd: &Fcd,
    placement: &Placement,
) -> Result<(), String> {
    for (i, kf) in keyframes.iter().enumerate() {
        let At::Follow(id) = kf.at else { continue };
        if !fcd.vehicles.iter().any(|v| v.vehicle_id == id) {
            return Err(format!(
                "{}: follows vehicle {id}, which is not in the FCD",
                name(i, Some(kf.t))
            ));
        }
        if !run::boxes_at(motion, fcd, placement, kf.t)
            .iter()
            .any(|b| b.vehicle_id == id)
        {
            return Err(format!(
                "{}: follows vehicle {id}, which is not drawn at that time",
                name(i, Some(kf.t))
            ));
        }
    }
    Ok(())
}

// ── The flight (§2.11.5) ─────────────────────────────────────────────────────

/// A channel's value at `t`: a keyframe's own value, or an evaluated one.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Val {
    Knot(usize),
    Value(f64),
}

/// A monotone cubic Hermite (PCHIP, Fritsch–Butland weights) through `(t[i], v[i])`,
/// slope 0 at both ends. Indices are keyframe indices, offset by `first`.
#[derive(Debug, Clone)]
struct Pchip {
    first: usize,
    t: Vec<f64>,
    v: Vec<f64>,
    m: Vec<f64>,
}

impl Pchip {
    fn new(first: usize, t: Vec<f64>, v: Vec<f64>) -> Pchip {
        let n = t.len();
        let mut m = vec![0.0; n];
        for j in 1..n.saturating_sub(1) {
            let (h0, h1) = (t[j] - t[j - 1], t[j + 1] - t[j]);
            let (d0, d1) = ((v[j] - v[j - 1]) / h0, (v[j + 1] - v[j]) / h1);
            if d0 * d1 > 0.0 {
                let (w1, w2) = (2.0 * h1 + h0, h1 + 2.0 * h0);
                m[j] = (w1 + w2) / (w1 / d0 + w2 / d1);
            }
        }
        Pchip { first, t, v, m }
    }

    /// The segment `i` with `t[i] ≤ t < t[i+1]`, clamped to the ends.
    fn eval(&self, t: f64) -> Val {
        let n = self.t.len();
        if t <= self.t[0] {
            return Val::Knot(self.first);
        }
        if t >= self.t[n - 1] {
            return Val::Knot(self.first + n - 1);
        }
        let i = self.t.partition_point(|&x| x <= t) - 1;
        if t == self.t[i] {
            return Val::Knot(self.first + i);
        }
        // A segment whose two ends are equal is constant: the keyframe's own value.
        if self.v[i] == self.v[i + 1] {
            return Val::Knot(self.first + i);
        }
        let h = self.t[i + 1] - self.t[i];
        let s = (t - self.t[i]) / h;
        let (s2, s3) = (s * s, s * s * s);
        let h00 = 2.0 * s3 - 3.0 * s2 + 1.0;
        let h10 = s3 - 2.0 * s2 + s;
        let h01 = -2.0 * s3 + 3.0 * s2;
        let h11 = s3 - s2;
        Val::Value(
            h00 * self.v[i]
                + h10 * h * self.m[i]
                + h01 * self.v[i + 1]
                + h11 * h * self.m[i + 1],
        )
    }
}

/// One centripetal Catmull–Rom run: keyframes `a..=b`, all fixed, consecutive positions
/// different; the knots include the reflected neighbours at both ends.
#[derive(Debug, Clone)]
struct Run {
    a: usize,
    /// `P[a − 1] … P[b + 1]`, the ends reflected.
    p: Vec<(f64, f64)>,
    /// Their knots `τ`.
    tau: Vec<f64>,
    /// Time to `τ`.
    time: Pchip,
}

/// How the look-at point moves between keyframes `i` and `i + 1`.
#[derive(Debug, Clone, Copy)]
enum Seg {
    /// Both keyframes at the same fixed point.
    Hold,
    /// Inside Catmull–Rom run `r`.
    Curve(usize),
    /// Both follow this vehicle.
    Vehicle(u64),
    /// Either end follows: a smoothstep blend.
    Blend,
}

/// The camera's path through the keyframes (§2.11.5), built once.
#[derive(Debug, Clone)]
pub struct Flight {
    keyframes: Vec<Keyframe>,
    ln_h: Pchip,
    yaw: Pchip,
    pitch: Pchip,
    segs: Vec<Seg>,
    runs: Vec<Run>,
    /// Each followed vehicle's drawn interval, `(id, t_first, t_last)`.
    drawn: Vec<(u64, f64, f64)>,
}

fn smoothstep(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn dist(p: (f64, f64), q: (f64, f64)) -> f64 {
    (q.0 - p.0).hypot(q.1 - p.1)
}

impl Flight {
    /// Build every channel from `keyframes` (checked by [`read`], so `t` increases), and
    /// each followed vehicle's drawn interval from `fcd`.
    pub fn new(keyframes: Vec<Keyframe>, fcd: &Fcd) -> Flight {
        let drawn = keyframes
            .iter()
            .filter_map(|k| match k.at {
                At::Follow(id) => fcd.vehicles.iter().find(|v| v.vehicle_id == id),
                At::Centre(..) => None,
            })
            .filter_map(|v| Some((v.vehicle_id, v.rows.first()?.time, v.rows.last()?.time)))
            .collect();
        Self::with_drawn(keyframes, drawn)
    }

    /// [`Flight::new`] with each followed vehicle's drawn interval given, `(id, t_first,
    /// t_last)`: no FCD is needed when nothing follows.
    pub fn with_drawn(keyframes: Vec<Keyframe>, drawn: Vec<(u64, f64, f64)>) -> Flight {
        let n = keyframes.len();
        let ts: Vec<f64> = keyframes.iter().map(|k| k.t).collect();
        let ln_h = Pchip::new(0, ts.clone(), keyframes.iter().map(|k| k.height_m.ln()).collect());
        let mut unwrapped = Vec::with_capacity(n);
        for (j, k) in keyframes.iter().enumerate() {
            unwrapped.push(match j {
                0 => k.yaw_deg,
                _ => unwrapped[j - 1] + short_way(keyframes[j - 1].yaw_deg, k.yaw_deg),
            });
        }
        let yaw = Pchip::new(0, ts.clone(), unwrapped);
        let pitch = Pchip::new(0, ts.clone(), keyframes.iter().map(|k| k.pitch_deg).collect());

        let mut segs = Vec::with_capacity(n.saturating_sub(1));
        let mut runs: Vec<Run> = Vec::new();
        let mut run_start: Option<usize> = None;
        let close_run = |runs: &mut Vec<Run>, a: usize, b: usize| {
            let pts: Vec<(f64, f64)> = (a..=b)
                .map(|j| match keyframes[j].at {
                    At::Centre(x, y) => (x, y),
                    At::Follow(_) => unreachable!("a run is fixed keyframes"),
                })
                .collect();
            let mut tau = vec![0.0];
            for j in 1..pts.len() {
                tau.push(tau[j - 1] + dist(pts[j - 1], pts[j]).sqrt());
            }
            let m = pts.len();
            let (p0, p1) = (pts[0], pts[1]);
            let (q0, q1) = (pts[m - 1], pts[m - 2]);
            let mut p = vec![(2.0 * p0.0 - p1.0, 2.0 * p0.1 - p1.1)];
            p.extend_from_slice(&pts);
            p.push((2.0 * q0.0 - q1.0, 2.0 * q0.1 - q1.1));
            let mut kt = vec![-tau[1]];
            kt.extend_from_slice(&tau);
            kt.push(2.0 * tau[m - 1] - tau[m - 2]);
            runs.push(Run {
                a,
                p,
                tau: kt,
                time: Pchip::new(a, ts[a..=b].to_vec(), tau),
            });
        };
        for i in 0..n.saturating_sub(1) {
            let seg = match (keyframes[i].at, keyframes[i + 1].at) {
                (At::Follow(u), At::Follow(v)) if u == v => Seg::Vehicle(u),
                (At::Follow(_), _) | (_, At::Follow(_)) => Seg::Blend,
                (At::Centre(x0, y0), At::Centre(x1, y1)) if x0 == x1 && y0 == y1 => Seg::Hold,
                _ => {
                    if run_start.is_none() {
                        run_start = Some(i);
                    }
                    Seg::Curve(runs.len())
                }
            };
            if !matches!(seg, Seg::Curve(_))
                && let Some(a) = run_start.take()
            {
                close_run(&mut runs, a, i);
            }
            segs.push(seg);
        }
        if let Some(a) = run_start.take() {
            close_run(&mut runs, a, n - 1);
        }

        Flight {
            keyframes,
            ln_h,
            yaw,
            pitch,
            segs,
            runs,
            drawn,
        }
    }

    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// The pose at sim time `t`, a vehicle's placed point from [`run::boxes_at`].
    pub fn pose_at(&self, t: f64, motion: &Motion, fcd: &Fcd, placement: &Placement) -> Pose {
        self.pose_at_by(t, &|t| run::boxes_at(motion, fcd, placement, t))
    }

    /// The pose at sim time `t`, with the boxes shown at any time from `boxes_at`.
    pub fn pose_at_by(&self, t: f64, boxes_at: &dyn Fn(f64) -> Vec<VehicleBox>) -> Pose {
        let k = &self.keyframes;
        let height_m = match self.ln_h.eval(t) {
            Val::Knot(j) => k[j].height_m,
            Val::Value(v) => v.exp(),
        };
        let yaw_deg = match self.yaw.eval(t) {
            Val::Knot(j) => k[j].yaw_deg,
            Val::Value(v) => wrap360(v),
        };
        let pitch_deg = match self.pitch.eval(t) {
            Val::Knot(j) => k[j].pitch_deg,
            Val::Value(v) => v,
        };
        let (cx, cy) = self.look_at(t, boxes_at);
        Pose {
            cx,
            cy,
            height_m,
            yaw_deg,
            pitch_deg,
        }
    }

    /// Keyframe `j`'s look-at point at `t`: its fixed point, or its vehicle's.
    fn point_of(&self, j: usize, t: f64, boxes_at: &dyn Fn(f64) -> Vec<VehicleBox>) -> (f64, f64) {
        match self.keyframes[j].at {
            At::Centre(x, y) => (x, y),
            At::Follow(id) => self.vehicle_point(id, t, boxes_at),
        }
    }

    /// The vehicle's placed point at `t`; when it is not drawn, the nearest end of its
    /// drawn interval (held where it was last seen, §2.11.5).
    fn vehicle_point(&self, id: u64, t: f64, boxes_at: &dyn Fn(f64) -> Vec<VehicleBox>) -> (f64, f64) {
        let find = |t: f64| {
            boxes_at(t)
                .iter()
                .find(|b| b.vehicle_id == id)
                .map(|b| (b.at.x, b.at.y))
        };
        if let Some(p) = find(t) {
            return p;
        }
        let &(_, first, last) = self
            .drawn
            .iter()
            .find(|&&(d, _, _)| d == id)
            .expect("a followed vehicle is in the FCD (check_follows)");
        find(t.clamp(first, last)).expect("a vehicle is drawn over its own rows")
    }

    fn look_at(&self, t: f64, boxes_at: &dyn Fn(f64) -> Vec<VehicleBox>) -> (f64, f64) {
        let k = &self.keyframes;
        let n = k.len();
        if t <= k[0].t {
            return self.point_of(0, t, boxes_at);
        }
        if t >= k[n - 1].t {
            return self.point_of(n - 1, t, boxes_at);
        }
        let i = k.partition_point(|kf| kf.t <= t) - 1;
        if t == k[i].t {
            return self.point_of(i, t, boxes_at);
        }
        match self.segs[i] {
            Seg::Hold => self.point_of(i, t, boxes_at),
            Seg::Vehicle(id) => self.vehicle_point(id, t, boxes_at),
            Seg::Blend => {
                let w = smoothstep((t - k[i].t) / (k[i + 1].t - k[i].t));
                let (a, b) = (
                    self.point_of(i, t, boxes_at),
                    self.point_of(i + 1, t, boxes_at),
                );
                ((1.0 - w) * a.0 + w * b.0, (1.0 - w) * a.1 + w * b.1)
            }
            Seg::Curve(r) => self.runs[r].eval(t, k),
        }
    }

    /// The Catmull–Rom parameter `τ` at `t` on run `r`'s time map (for the gates).
    pub fn tau_at(&self, t: f64) -> Option<f64> {
        let k = &self.keyframes;
        let i = k.partition_point(|kf| kf.t <= t).checked_sub(1)?;
        match self.segs.get(i)? {
            Seg::Curve(r) => {
                let run = &self.runs[*r];
                Some(match run.time.eval(t) {
                    Val::Knot(j) => run.tau[j - run.a + 1],
                    Val::Value(v) => v,
                })
            }
            _ => None,
        }
    }
}

impl Run {
    fn eval(&self, t: f64, k: &[Keyframe]) -> (f64, f64) {
        let tau = match self.time.eval(t) {
            Val::Knot(j) => {
                let At::Centre(x, y) = k[j].at else {
                    unreachable!("a run is fixed keyframes")
                };
                return (x, y);
            }
            Val::Value(v) => v,
        };
        // Segment `s` of the run, between its points `s` and `s + 1` (offset 1 in `p`).
        let s = (self.tau[1..self.tau.len() - 1].partition_point(|&x| x <= tau))
            .clamp(1, self.tau.len() - 3)
            - 1;
        let (p0, p1, p2, p3) = (self.p[s], self.p[s + 1], self.p[s + 2], self.p[s + 3]);
        let (t0, t1, t2, t3) = (self.tau[s], self.tau[s + 1], self.tau[s + 2], self.tau[s + 3]);
        let lerp = |p: (f64, f64), q: (f64, f64), ta: f64, tb: f64| {
            let (u, v) = ((tb - tau) / (tb - ta), (tau - ta) / (tb - ta));
            (u * p.0 + v * q.0, u * p.1 + v * q.1)
        };
        // Barry–Goldman.
        let a1 = lerp(p0, p1, t0, t1);
        let a2 = lerp(p1, p2, t1, t2);
        let a3 = lerp(p2, p3, t2, t3);
        let b1 = lerp(a1, a2, t0, t2);
        let b2 = lerp(a2, a3, t1, t3);
        lerp(b1, b2, t1, t2)
    }
}
