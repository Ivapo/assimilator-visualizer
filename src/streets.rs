//! Streets (vis-002 §2.17): the junctions' surfaces, the median gaps filled, and the US
//! markings — white broken lane lines, white stop lines at signals and a double yellow
//! centre line on every two-way pair — as ribbons along the placed links. Plain Rust with
//! no Bevy types; `draw` turns [`Streets::surface`] and [`Streets::markings`] into the two
//! meshes (§2.17.11), and [`fade`] fades a marking toward the road where it is under a
//! pixel (§2.17.10).
//!
//! Every order here is the network's: nodes, links and lanes are walked in file order,
//! and the engine's maps are only looked up, never walked.

use std::collections::HashMap;
use std::sync::LazyLock;

use assimilator_config::network::ControlType;
use assimilator_config::types::{LaneIdx, LinkId};
use earcut::Earcut;

use crate::camera::{FOV_DEG, Pose};
use crate::place::Placement;
use crate::scene::{self, STRIP_STEP};

/// A line's width, m (§2.17.6): MUTCD's 6 in.
pub const LINE_M: f64 = 0.15;
/// The space between a double line's two lines, m.
pub const DOUBLE_SPACE_M: f64 = 0.10;
/// A lane line's dash and the gap after it, m: 10 ft and 30 ft.
pub const DASH_M: f64 = 3.0;
pub const GAP_M: f64 = 9.0;
/// A stop line's depth along the lane, m: 24 in.
pub const STOP_M: f64 = 0.60;
/// A median gap at least this wide is a flush median, a double yellow inside each edge.
pub const FLUSH_MIN_M: f64 = 1.0;
/// The markings' colours, sRGB.
pub const WHITE: [u8; 3] = [235, 235, 235];
pub const YELLOW: [u8; 3] = [230, 170, 20];
/// The markings' heights over the road, m (§2.17.9): white over yellow, both under every
/// box's base.
pub const LIFT_YELLOW_M: f64 = 0.01;
pub const LIFT_WHITE_M: f64 = 0.02;
/// The fade (§2.17.10): a marking is the road's grey under this width on screen, px …
pub const FADE_FROM_PX: f64 = 0.1;
/// … and its own colour from this one up.
pub const FADE_TO_PX: f64 = 0.5;

/// What a marking is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// One line of a double yellow centre line.
    Yellow,
    /// One dash of a lane line.
    Lane,
    /// A stop line across one lane.
    Stop,
}

/// A strip along one link: its two edges, sample for sample. `left` is the smaller
/// lateral (left of travel), `right` the larger.
#[derive(Debug, Clone, PartialEq)]
pub struct Ribbon {
    pub left: Vec<[f64; 2]>,
    pub right: Vec<[f64; 2]>,
}

/// One marking along link `link` (an index in link order).
#[derive(Debug, Clone, PartialEq)]
pub struct Marking {
    pub kind: Kind,
    pub link: usize,
    /// A stop line's lane; a lane line's lane `k`, the line being between `k` and `k + 1`;
    /// `None` on a yellow line.
    pub lane: Option<u32>,
    /// The line's width, m: [`LINE_M`], or [`STOP_M`] for a stop line (§2.17.10).
    pub width: f64,
    pub ribbon: Ribbon,
}

/// One junction's surface: its node's polygon with the closing copy dropped, and its
/// `earcut` triangles (indices into `polygon`).
#[derive(Debug, Clone, PartialEq)]
pub struct Junction {
    pub node: String,
    pub polygon: Vec<[f64; 2]>,
    pub triangles: Vec<u32>,
}

/// A two-way pair: `a` comes first in link order, `b` is its twin; `gap` is
/// `(g_A + g_B) / 2`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    pub a: usize,
    pub b: usize,
    pub gap: f64,
}

/// A median gap, filled with the road's grey (§2.17.5), along its pair's `a`.
#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub link: usize,
    pub ribbon: Ribbon,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Streets {
    /// In node order.
    pub junctions: Vec<Junction>,
    /// In link order of `a`.
    pub pairs: Vec<Pair>,
    pub fills: Vec<Fill>,
    /// In link order; on each link the yellow lines, the stop lines lane by lane, then
    /// the lane lines line by line.
    pub markings: Vec<Marking>,
}

/// The surface mesh's data (§2.17.11): world metres at height 0, and triangles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceData {
    pub positions: Vec<[f64; 3]>,
    pub indices: Vec<u32>,
}

/// The markings mesh's data: world metres with each marking's lift, and per vertex its
/// marking's colour and line width.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MarkingsData {
    pub positions: Vec<[f64; 3]>,
    pub colours: Vec<[u8; 3]>,
    pub widths: Vec<f64>,
    pub indices: Vec<u32>,
}

/// A ribbon on `link` from `s0` to `s1` between laterals `a` < `b` (right of travel
/// positive), sampled as §2.17.6 says: `n = max(1, ceil((s1 − s0) / STRIP_STEP))` equal
/// steps. `None` when `s1 ≤ s0` or the link cannot be placed.
pub fn ribbon(pl: &Placement, link: &str, s0: f64, s1: f64, a: f64, b: f64) -> Option<Ribbon> {
    if s1 <= s0 {
        return None;
    }
    let n = (((s1 - s0) / STRIP_STEP).ceil() as usize).max(1);
    let mut r = Ribbon {
        left: Vec::with_capacity(n + 1),
        right: Vec::with_capacity(n + 1),
    };
    for i in 0..=n {
        let s = s0 + (s1 - s0) * i as f64 / n as f64;
        let p = pl.place_lateral(link, s, 0.0)?;
        let h = p.heading.to_radians();
        // Right of travel, as `scene::strips`.
        let (rx, ry) = (h.cos(), -h.sin());
        r.left.push([p.x + rx * a, p.y + ry * a]);
        r.right.push([p.x + rx * b, p.y + ry * b]);
    }
    Some(r)
}

impl Streets {
    /// The streets of a placed network (§2.17.4–§2.17.8).
    pub fn build(pl: &Placement) -> Streets {
        let net = &pl.network;
        let data = &pl.data;
        let mut st = Streets::default();

        // Junction surfaces, in node order.
        let mut earcut = Earcut::<f64>::new();
        for node in &net.nodes {
            let Some(poly) = data.junction_polygons.get(&node.id) else {
                continue;
            };
            let mut polygon: Vec<[f64; 2]> = poly.iter().map(|p| [p.0, p.1]).collect();
            if polygon.len() > 1 && polygon.first() == polygon.last() {
                polygon.pop();
            }
            if polygon.len() < 3 {
                continue;
            }
            let mut triangles = Vec::new();
            earcut.earcut(polygon.iter().copied(), &[] as &[u32], &mut triangles);
            st.junctions.push(Junction {
                node: node.id.0.clone(),
                polygon,
                triangles,
            });
        }

        // Looked up only: a link's twin (the first in link order), a node's control.
        let mut by_nodes: HashMap<(&str, &str), usize> = HashMap::new();
        for (i, l) in net.links.iter().enumerate() {
            by_nodes
                .entry((l.from_node.0.as_str(), l.to_node.0.as_str()))
                .or_insert(i);
        }
        let control: HashMap<&str, ControlType> = net
            .junctions
            .iter()
            .map(|j| (j.node_id.0.as_str(), j.control))
            .collect();

        for (i, link) in net.links.iter().enumerate() {
            let id = link.id.0.as_str();
            let len = pl.link_length(id);
            let w = link.total_width();
            let offs = link.lane_center_offsets();
            let mut mark = |kind, lane, width, r: Option<Ribbon>| {
                if let Some(ribbon) = r {
                    st.markings.push(Marking {
                        kind,
                        link: i,
                        lane,
                        width,
                        ribbon,
                    });
                }
            };

            // A two-way pair, drawn once, along its link that comes first (§2.17.8).
            let twin = by_nodes.get(&(link.to_node.0.as_str(), link.from_node.0.as_str()));
            let mut fill = None;
            if let Some(&j) = twin
                && i < j
            {
                let g = (link.median_gap + net.links[j].median_gap) / 2.0;
                // The gap lies left of `a`'s left edge.
                let (g0, g1) = (-w / 2.0 - g, -w / 2.0);
                if g > 0.0 {
                    fill = ribbon(pl, id, 0.0, len, g0, g1);
                }
                let (lw, sp) = (LINE_M, DOUBLE_SPACE_M);
                let yellow = |a: f64, b: f64| ribbon(pl, id, 0.0, len, a, b);
                let lines = if g >= FLUSH_MIN_M {
                    vec![
                        yellow(g0, g0 + lw),
                        yellow(g0 + lw + sp, g0 + 2.0 * lw + sp),
                        yellow(g1 - 2.0 * lw - sp, g1 - lw - sp),
                        yellow(g1 - lw, g1),
                    ]
                } else {
                    let c = (g0 + g1) / 2.0;
                    vec![
                        yellow(c - sp / 2.0 - lw, c - sp / 2.0),
                        yellow(c + sp / 2.0, c + sp / 2.0 + lw),
                    ]
                };
                for r in lines {
                    mark(Kind::Yellow, None, LINE_M, r);
                }
                st.pairs.push(Pair { a: i, b: j, gap: g });
            }

            // Stop lines at a signal, one per lane, where the engine stops (§2.17.7).
            let mut lines_end = len;
            if control.get(link.to_node.0.as_str()) == Some(&ControlType::Signal) {
                for (k, lane) in link.lanes.iter().enumerate() {
                    let off =
                        data.lane_stop_line_offset(&LinkId(id.to_string()), LaneIdx(k as u32));
                    let s1 = len - off;
                    let s0 = (s1 - STOP_M).max(0.0);
                    lines_end = lines_end.min(s0);
                    let (a, b) = (offs[k] - lane.width / 2.0, offs[k] + lane.width / 2.0);
                    mark(
                        Kind::Stop,
                        Some(k as u32),
                        STOP_M,
                        ribbon(pl, id, s0, s1, a, b),
                    );
                }
            }

            // Lane lines: broken, from the link's start to its stop lines (§2.17.6).
            let boundaries = link.lanes.len().saturating_sub(1);
            for (k, lane) in link.lanes.iter().enumerate().take(boundaries) {
                let e = offs[k] + lane.width / 2.0 + lane.gap_after / 2.0;
                let mut s = 0.0;
                while s < lines_end {
                    let s1 = (s + DASH_M).min(lines_end);
                    let r = ribbon(pl, id, s, s1, e - LINE_M / 2.0, e + LINE_M / 2.0);
                    mark(Kind::Lane, Some(k as u32), LINE_M, r);
                    s += DASH_M + GAP_M;
                }
            }

            if let Some(ribbon) = fill {
                st.fills.push(Fill { link: i, ribbon });
            }
        }
        st
    }

    /// The surface mesh's data: the junction surfaces, then the median fills.
    pub fn surface(&self) -> SurfaceData {
        let mut d = SurfaceData::default();
        for j in &self.junctions {
            let base = d.positions.len() as u32;
            d.positions
                .extend(j.polygon.iter().map(|p| [p[0], p[1], 0.0]));
            d.indices.extend(j.triangles.iter().map(|t| base + t));
        }
        for f in &self.fills {
            ribbon_mesh(&f.ribbon, 0.0, &mut d.positions, &mut d.indices);
        }
        d
    }

    /// The markings mesh's data: the yellow lines, then the lane lines, then the stop
    /// lines, each in link order.
    pub fn markings(&self) -> MarkingsData {
        let mut d = MarkingsData::default();
        for kind in [Kind::Yellow, Kind::Lane, Kind::Stop] {
            let (lift, colour) = match kind {
                Kind::Yellow => (LIFT_YELLOW_M, YELLOW),
                Kind::Lane | Kind::Stop => (LIFT_WHITE_M, WHITE),
            };
            for m in self.markings.iter().filter(|m| m.kind == kind) {
                let n = d.positions.len();
                ribbon_mesh(&m.ribbon, lift, &mut d.positions, &mut d.indices);
                let added = d.positions.len() - n;
                d.colours.extend(std::iter::repeat_n(colour, added));
                d.widths.extend(std::iter::repeat_n(m.width, added));
            }
        }
        d
    }
}

/// A ribbon's 2 vertices a sample and 2 triangles a step, no vertex shared.
fn ribbon_mesh(r: &Ribbon, z: f64, positions: &mut Vec<[f64; 3]>, indices: &mut Vec<u32>) {
    let base = positions.len() as u32;
    for (l, rr) in r.left.iter().zip(&r.right) {
        positions.push([l[0], l[1], z]);
        positions.push([rr[0], rr[1], z]);
    }
    for i in 0..(r.left.len() as u32).saturating_sub(1) {
        let (l0, r0, l1, r1) = (
            base + 2 * i,
            base + 2 * i + 1,
            base + 2 * i + 2,
            base + 2 * i + 3,
        );
        indices.extend_from_slice(&[l0, r0, l1, r0, r1, l1]);
    }
}

/// §2.17.10's `α` for a line `width_m` wide where a pixel is `mpp` metres: 0 under
/// [`FADE_FROM_PX`] on screen, 1 from [`FADE_TO_PX`] up, smoothstep between.
pub fn fade(width_m: f64, mpp: f64) -> f64 {
    let x = ((width_m / mpp - FADE_FROM_PX) / (FADE_TO_PX - FADE_FROM_PX)).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// An sRGB channel in linear light.
fn linear(c: u8) -> f32 {
    let c = c as f64 / 255.0;
    (if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }) as f32
}

fn linear3(c: [u8; 3]) -> [f32; 3] {
    [linear(c[0]), linear(c[1]), linear(c[2])]
}

/// The road's, the white's and the yellow's linear colours, computed once.
static LINEAR: LazyLock<[[f32; 3]; 3]> =
    LazyLock::new(|| [linear3(scene::ROAD), linear3(WHITE), linear3(YELLOW)]);

/// A marking's colour at `α`, linear RGBA: `(1 − α)·road + α·marking` per channel in
/// `f32`, exactly the road's at 0 and the marking's at 1. Opaque.
pub fn colour(marking: [u8; 3], alpha: f64) -> [f32; 4] {
    let [road, white, yellow] = *LINEAR;
    let m = if marking == WHITE {
        white
    } else if marking == YELLOW {
        yellow
    } else {
        linear3(marking)
    };
    let a = alpha as f32;
    let mix = |i: usize| (1.0 - a) * road[i] + a * m[i];
    [mix(0), mix(1), mix(2), 1.0]
}

/// Each marking vertex's colour, faded by its line's width on screen at `mpp` (metres per
/// pixel at a world point).
pub fn vertex_colours(
    positions: &[[f64; 3]],
    colours: &[[u8; 3]],
    widths: &[f64],
    mpp: impl Fn([f64; 3]) -> f64,
) -> Vec<[f32; 4]> {
    positions
        .iter()
        .zip(colours)
        .zip(widths)
        .map(|((p, c), w)| colour(*c, fade(*w, mpp(*p))))
        .collect()
}

/// The perspective metres per pixel at a world point seen from `pose` in a frame `h_px`
/// pixels tall: the pixel's size across the ray, `|p − eye| · 2·tan(φ/2) / H`.
pub fn mpp_at(pose: &Pose, h_px: f64) -> impl Fn([f64; 3]) -> f64 + use<> {
    let e = pose.eye();
    let t = 2.0 * (FOV_DEG / 2.0).to_radians().tan() / h_px;
    move |p: [f64; 3]| {
        ((p[0] - e[0]).powi(2) + (p[1] - e[1]).powi(2) + (p[2] - e[2]).powi(2)).sqrt() * t
    }
}
