//! See-through (vis-002 §2.15): which buildings stand between the camera and the point it
//! looks at, and how far down each is drawn. Plain Rust with no Bevy types; `render` and
//! `view` draw the heights through `draw::BuildingsCut`.
//!
//! The wedge (§2.15.2) is the convex hull of the eye's ground point `g` and the disc of
//! radius `r` about the look-at point `l`. A building whose footprint lies less than the
//! ease band `e` from it is lowered toward a [`STUB_M`] stub, eased by `smoothstep`
//! (§2.15.3); every other building keeps its own height, bit for bit.

use crate::buildings::{Building, Buildings};
use crate::camera::{Pose, cos_deg};

/// The wedge's half-width at the look-at point, per metre of `height_m`, times
/// `cos(pitch)`.
pub const WIDTH: f64 = 0.15;
/// The ease band is `max(r, EASE_MIN_M)` metres.
pub const EASE_MIN_M: f64 = 10.0;
/// The height of a building touching the wedge, metres.
pub const STUB_M: f64 = 3.0;

type P2 = [f64; 2];

/// The wedge at one pose: the eye's ground point `g`, the look-at point `l`, the disc's
/// radius `r` and the ease band `ease`, all in world metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wedge {
    pub g: P2,
    pub l: P2,
    pub r: f64,
    pub ease: f64,
}

/// The wedge at `pose`: `g` is [`Pose::eye`]'s ground point, `r = WIDTH·height_m·cos(pitch)`
/// (exactly 0 straight down), and the ease band `max(r, EASE_MIN_M)`.
pub fn wedge(pose: &Pose) -> Wedge {
    let e = pose.eye();
    let r = WIDTH * pose.height_m * cos_deg(pose.pitch_deg);
    Wedge {
        g: [e[0], e[1]],
        l: [pose.cx, pose.cy],
        r,
        ease: r.max(EASE_MIN_M),
    }
}

impl Wedge {
    /// The triangle from `g` to the two points where lines from `g` touch the disc;
    /// `None` when `g` lies in the disc or on its edge, where the wedge is the disc.
    fn triangle(&self) -> Option<[P2; 3]> {
        let (g, l, r) = (self.g, self.l, self.r);
        let gl = ((l[0] - g[0]).powi(2) + (l[1] - g[1]).powi(2)).sqrt();
        if gl <= r {
            return None;
        }
        let (ux, uy) = ((l[0] - g[0]) / gl, (l[1] - g[1]) / gl);
        let t = (gl * gl - r * r).sqrt();
        let (s, c) = (r / gl, t / gl);
        let t1 = [g[0] + t * (ux * c - uy * s), g[1] + t * (uy * c + ux * s)];
        let t2 = [g[0] + t * (ux * c + uy * s), g[1] + t * (uy * c - ux * s)];
        Some([g, t1, t2])
    }
}

/// `δ`: the ground distance from the building's footprint (every polygon, holes outside)
/// to the wedge; 0 when they meet.
pub fn distance(w: &Wedge, b: &Building) -> f64 {
    delta(w, &w.triangle(), b, f64::INFINITY)
}

/// The height `b` is drawn at (§2.15.3): `min(h, s + (h − s)·smoothstep(δ/e))` for
/// `δ < e`, and the building's own `height` for `δ ≥ e`.
pub fn height(w: &Wedge, b: &Building) -> f64 {
    height_with(w, &w.triangle(), b)
}

/// Every building's [`height`] at `pose`, in file order.
pub fn heights(buildings: &Buildings, pose: &Pose) -> Vec<f64> {
    let w = wedge(pose);
    let tri = w.triangle();
    buildings
        .buildings
        .iter()
        .map(|b| height_with(&w, &tri, b))
        .collect()
}

fn height_with(w: &Wedge, tri: &Option<[P2; 3]>, b: &Building) -> f64 {
    let (h, e) = (b.height, w.ease);
    let d = delta(w, tri, b, e);
    let f = if d <= 0.0 {
        0.0
    } else if d < e {
        smoothstep(d / e)
    } else {
        1.0
    };
    if f >= 1.0 {
        return h;
    }
    h.min(STUB_M + (h - STUB_M).max(0.0) * f)
}

fn smoothstep(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// `δ` when it is at most `cutoff`; something larger than `cutoff` otherwise. A
/// bounding-box test skips the exact distances only when the box is already too far.
fn delta(w: &Wedge, tri: &Option<[P2; 3]>, b: &Building, cutoff: f64) -> f64 {
    let bb = bbox(b.polygons.iter().flat_map(|p| p.exterior.iter()));
    let l = w.l;
    let dx = (bb[0] - l[0]).max(l[0] - bb[2]).max(0.0);
    let dy = (bb[1] - l[1]).max(l[1] - bb[3]).max(0.0);
    let disc = if (dx * dx + dy * dy).sqrt() - w.r > cutoff {
        f64::INFINITY
    } else {
        (point_distance(l, b) - w.r).max(0.0)
    };
    let tri = match tri {
        None => f64::INFINITY,
        Some(t) => triangle_distance(b, t, bb, cutoff),
    };
    disc.min(tri)
}

fn bbox<'a>(pts: impl Iterator<Item = &'a P2>) -> [f64; 4] {
    pts.fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |a, p| {
        [
            a[0].min(p[0]),
            a[1].min(p[1]),
            a[2].max(p[0]),
            a[3].max(p[1]),
        ]
    })
}

/// Each ring's edges, closing back to its first vertex.
fn edges(ring: &[P2]) -> impl Iterator<Item = (P2, P2)> + '_ {
    let n = ring.len();
    (0..n).map(move |i| (ring[i], ring[(i + 1) % n]))
}

/// The footprint's distance to the triangle `t`; 0 when they meet.
fn triangle_distance(b: &Building, t: &[P2; 3], bb: [f64; 4], cutoff: f64) -> f64 {
    let sb = bbox(t.iter());
    let dx = (bb[0] - sb[2]).max(sb[0] - bb[2]).max(0.0);
    let dy = (bb[1] - sb[3]).max(sb[1] - bb[3]).max(0.0);
    if (dx * dx + dy * dy).sqrt() > cutoff {
        return f64::INFINITY;
    }
    let tri = [t.to_vec()];
    let mut best = f64::INFINITY;
    for p in &b.polygons {
        if inside(t[0], p.rings()) || inside(p.exterior[0], tri.iter()) {
            return 0.0;
        }
        for ring in p.rings() {
            for (a, c) in edges(ring) {
                for (s0, s1) in edges(t) {
                    let d = seg_seg(a, c, s0, s1);
                    if d == 0.0 {
                        return 0.0;
                    }
                    best = best.min(d);
                }
            }
        }
    }
    best
}

/// The distance from `q` to the footprint; 0 inside it.
fn point_distance(q: P2, b: &Building) -> f64 {
    let mut best = f64::INFINITY;
    for p in &b.polygons {
        if inside(q, p.rings()) {
            return 0.0;
        }
        for ring in p.rings() {
            for (a, c) in edges(ring) {
                best = best.min(point_segment(q, a, c));
            }
        }
    }
    best
}

/// Even-odd over every ring, so a point in a hole is outside.
fn inside<'a>(pt: P2, rings: impl Iterator<Item = &'a Vec<P2>>) -> bool {
    let mut inside = false;
    for ring in rings {
        let n = ring.len();
        let mut j = n - 1;
        for i in 0..n {
            let (a, b) = (ring[i], ring[j]);
            if (a[1] > pt[1]) != (b[1] > pt[1])
                && pt[0] < (b[0] - a[0]) * (pt[1] - a[1]) / (b[1] - a[1]) + a[0]
            {
                inside = !inside;
            }
            j = i;
        }
    }
    inside
}

/// Whether segments `p0–p1` and `q0–q1` cross or touch.
fn crosses(p0: P2, p1: P2, q0: P2, q1: P2) -> bool {
    let r = [p1[0] - p0[0], p1[1] - p0[1]];
    let s = [q1[0] - q0[0], q1[1] - q0[1]];
    let den = r[0] * s[1] - r[1] * s[0];
    if den == 0.0 {
        return false;
    }
    let qp = [q0[0] - p0[0], q0[1] - p0[1]];
    let t = (qp[0] * s[1] - qp[1] * s[0]) / den;
    let u = (qp[0] * r[1] - qp[1] * r[0]) / den;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn point_segment(p: P2, a: P2, b: P2) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}

fn seg_seg(a0: P2, a1: P2, b0: P2, b1: P2) -> f64 {
    if a0 != a1 && b0 != b1 && crosses(a0, a1, b0, b1) {
        return 0.0;
    }
    point_segment(a0, b0, b1)
        .min(point_segment(a1, b0, b1))
        .min(point_segment(b0, a0, a1))
        .min(point_segment(b1, a0, a1))
}
