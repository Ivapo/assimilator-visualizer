//! Streets (vis-002 §2.17, §2.18): the junctions' surfaces, the median gaps filled, and the
//! engine dashboard's markings — lane dashes, a centre line where a pair has no gap, solid
//! lane lines, 0.4 m stop lines and lane arrows — drawn from the engine's own
//! `NetworkJson`, with the dashboard's colours and arrow glyphs copied from its front end.
//! Plain Rust with no Bevy types; `draw` turns [`Streets::surface`] and
//! [`Streets::markings`] into the two meshes (§2.18.10), and [`fade`] fades a marking toward
//! the road where it is under a pixel (§2.17.10, §2.18.9).
//!
//! Every order here is the network's or `NetworkJson`'s: nodes, links and lanes are walked
//! in file order, `NetworkJson`'s fields in theirs, and every map is only looked up, never
//! walked.
//!
//! The values copied from the front end were read at the engine pin `90b39292`
//! (§2.18.12); each names its file under `web/src/canvas/` and its line there.

use std::collections::HashMap;
use std::sync::LazyLock;

use assimilator_config::types::LinkId;
use assimilator_geometry::network_json::NetworkJson;
use earcut::Earcut;

use crate::camera::{FOV_DEG, Pose};
use crate::place::Placement;
use crate::scene::{self, STRIP_STEP};

/// The markings' heights over the road, m (§2.18.8): the lines under the stop lines and
/// arrows, both under every box's base.
pub const LIFT_LINES_M: f64 = 0.01;
pub const LIFT_TOP_M: f64 = 0.02;
/// The fade (§2.17.10): a marking is the road's grey under this width on screen, px …
pub const FADE_FROM_PX: f64 = 0.1;
/// … and its own colour from this one up.
pub const FADE_TO_PX: f64 = 0.5;

// The dashboard's dark palette and its opacities (§2.18.4), `colors.ts`.
/// `LANE_MARKING` (colors.ts l. 75) at `LANE_MARKING_OPACITY` (l. 198): dashes, solid lines.
pub const LANE_MARKING: [u8; 3] = [0xff, 0xff, 0xff];
pub const LANE_MARKING_OPACITY: f64 = 0.5;
/// `MEDIAN_LINE` (colors.ts l. 76) at `MEDIAN_LINE_OPACITY` (l. 199): the centre line.
pub const MEDIAN_LINE: [u8; 3] = [0xf5, 0xc5, 0x42];
pub const MEDIAN_LINE_OPACITY: f64 = 0.7;
/// `STOP_LINE` (colors.ts l. 77) at `STOP_LINE_OPACITY` (l. 200): stop lines, connectors.
pub const STOP_LINE: [u8; 3] = [0xff, 0xff, 0xff];
pub const STOP_LINE_OPACITY: f64 = 0.9;
/// `ROAD_ARROW` (colors.ts l. 78) at `ROAD_ARROW_OPACITY` (l. 201): the arrows.
pub const ROAD_ARROW: [u8; 3] = [0xff, 0xff, 0xff];
pub const ROAD_ARROW_OPACITY: f64 = 0.6;
/// `DEAD_END_BAR` (colors.ts l. 79) at `DEAD_END_BAR_OPACITY` (l. 202): a dead end's bar.
pub const DEAD_END_BAR: [u8; 3] = [0xef, 0x44, 0x44];
pub const DEAD_END_BAR_OPACITY: f64 = 0.85;

/// The markings' colours, sRGB (§2.18.4): each colour above composited at its opacity over
/// [`scene::ROAD`] as the canvas does, `round(a·c + (1 − a)·road)` per channel, rounded
/// once; opaque. In order: lane lines, the centre line, stop lines, arrows, the bar.
pub const COLOURS: [[u8; 3]; 5] = [
    [174, 176, 180],
    [199, 167, 77],
    [239, 239, 240],
    [190, 191, 195],
    [217, 72, 73],
];

// The arrow glyphs (§2.18.5), `networkRenderer.ts`, in metres.
/// `LA_SW` (networkRenderer.ts l. 1249): a shaft's half-width.
pub const LA_SW: f64 = 0.15;
/// `LA_HW` (l. 1250): a head's half-width.
pub const LA_HW: f64 = 0.40;
/// `LA_HL` (l. 1251): a head's length.
pub const LA_HL: f64 = 0.70;
/// `LA_LEN` (l. 1252): a straight arrow, tail to tip.
pub const LA_LEN: f64 = 3.0;
/// `LA_TURN_SHAFT` (l. 1253): the straight shaft before a turn's curve.
pub const LA_TURN_SHAFT: f64 = 1.50;
/// `LA_TURN_R` (l. 1254): a turn's radius.
pub const LA_TURN_R: f64 = 0.50;
/// `drawUTurnArrow`'s radius `0.30 * s` (l. 1608) …
pub const UTURN_R: f64 = 0.30;
/// … and its return leg's end, `-shaftLen * 0.4` (l. 1624).
pub const UTURN_RETURN: f64 = 0.4;
/// The branches' fork, `tail + 1.00 * s` (`drawThroughTurn` l. 1455,
/// `drawThroughLeftRight` l. 1499).
pub const FORK: f64 = 1.00;
/// `drawDeadEndArrow`'s bar: half-width `0.50 * s` (l. 1396), depth `0.40 * s` (l. 1397).
pub const BAR_HW: f64 = 0.50;
pub const BAR_DEPTH: f64 = 0.40;
/// `sampleQuadBezier`'s and the u-turn arc's segments, `12` (l. 1437, 1470, 1515, 1612).
const SEGMENTS: usize = 12;

/// What a marking is, in the markings mesh's order (§2.18.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A centre line: `NetworkJson::median_lines`, on a pair with a 0 m gap.
    Centre,
    /// A lane-line dash: `lane_marking_dashes`.
    Dash,
    /// A solid lane line: `solid_lane_lines`.
    Solid,
    /// A stop line across one lane (or a whole approach matched to no link).
    Stop,
    /// The strip along a lane line between two staggered stop lines.
    Connector,
    /// One outline of a lane arrow's glyph.
    Arrow,
    /// A dead end's red bar.
    DeadEnd,
}

impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::Centre,
        Kind::Dash,
        Kind::Solid,
        Kind::Stop,
        Kind::Connector,
        Kind::Arrow,
        Kind::DeadEnd,
    ];

    /// Its colour (§2.18.4).
    pub fn colour(self) -> [u8; 3] {
        match self {
            Kind::Dash | Kind::Solid => COLOURS[0],
            Kind::Centre => COLOURS[1],
            Kind::Stop | Kind::Connector => COLOURS[2],
            Kind::Arrow => COLOURS[3],
            Kind::DeadEnd => COLOURS[4],
        }
    }

    /// The width the fade reads, m (§2.18.9): its narrow dimension; an arrow's shaft.
    pub fn width(self) -> f64 {
        match self {
            Kind::Centre | Kind::Dash | Kind::Solid => 0.15,
            Kind::Stop | Kind::DeadEnd => 0.40,
            Kind::Arrow => 0.30,
            Kind::Connector => 0.10,
        }
    }

    /// Its height over the road (§2.18.8).
    pub fn lift(self) -> f64 {
        match self {
            Kind::Centre | Kind::Dash | Kind::Solid => LIFT_LINES_M,
            _ => LIFT_TOP_M,
        }
    }
}

/// A strip along one link: its two edges, sample for sample. `left` is the smaller
/// lateral (left of travel), `right` the larger.
#[derive(Debug, Clone, PartialEq)]
pub struct Ribbon {
    pub left: Vec<[f64; 2]>,
    pub right: Vec<[f64; 2]>,
}

/// One marking: a simple polygon from `NetworkJson`, with no closing copy, and its
/// `earcut` triangles (indices into `polygon`).
#[derive(Debug, Clone, PartialEq)]
pub struct Marking {
    pub kind: Kind,
    /// A stop line's, connector's, arrow's or bar's link, an index in link order; `None`
    /// for a centre line, a dash or a solid line, which `NetworkJson` gives no link, and
    /// for a full-width stop line matched to no link (§2.18.6).
    pub link: Option<usize>,
    /// A stop line's, arrow's or bar's lane.
    pub lane: Option<u32>,
    /// An arrow's or bar's `arrow_type`.
    pub arrow: Option<String>,
    pub polygon: Vec<[f64; 2]>,
    pub triangles: Vec<u32>,
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
    /// In `NetworkJson`'s order: the centre lines, the dashes, the solid lines, the stop
    /// lines and connectors, then each arrow's outlines and its bar.
    pub markings: Vec<Marking>,
}

/// The surface mesh's data (§2.17.11): world metres at height 0, and triangles.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceData {
    pub positions: Vec<[f64; 3]>,
    pub indices: Vec<u32>,
}

/// The markings mesh's data: world metres with each marking's lift, and per vertex its
/// marking's colour and width.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MarkingsData {
    pub positions: Vec<[f64; 3]>,
    pub colours: Vec<[u8; 3]>,
    pub widths: Vec<f64>,
    pub indices: Vec<u32>,
}

/// A ribbon on `link` from `s0` to `s1` between laterals `a` < `b` (right of travel
/// positive), sampled in `n = max(1, ceil((s1 − s0) / STRIP_STEP))` equal steps. `None`
/// when `s1 ≤ s0` or the link cannot be placed.
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

/// A ring with its closing copies of the first vertex dropped.
fn open(ring: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let mut v = ring.to_vec();
    while v.len() > 1 && v.first() == v.last() {
        v.pop();
    }
    v
}

/// A closed polygon with every edge split into `max(1, ceil(l / STRIP_STEP))` equal pieces,
/// so the fade follows the distance along a long line (§2.18.9).
pub fn split_edges(poly: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = poly.len();
    let mut v = Vec::new();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + 1) % n]);
        let l = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        let k = ((l / STRIP_STEP).ceil() as usize).max(1);
        for j in 0..k {
            let t = j as f64 / k as f64;
            v.push([a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]);
        }
    }
    v
}

// ── The glyphs, in local metres: +x forward, +y right of travel (§2.18.5) ──────

/// A centreline point and its unit tangent, `[x, y, tx, ty]`.
type Centre = [f64; 4];

/// `sampleQuadBezier` (l. 1274): `n + 1` points with unit tangents.
fn quad_bezier(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], n: usize) -> Vec<Centre> {
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            let u = 1.0 - t;
            let x = u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0];
            let y = u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1];
            let dx = 2.0 * u * (p1[0] - p0[0]) + 2.0 * t * (p2[0] - p1[0]);
            let dy = 2.0 * u * (p1[1] - p0[1]) + 2.0 * t * (p2[1] - p1[1]);
            let l = (dx * dx + dy * dy).sqrt();
            if l > 1e-9 {
                [x, y, dx / l, dy / l]
            } else {
                [x, y, 1.0, 0.0]
            }
        })
        .collect()
}

/// `fillCurvedArrow`'s outline (l. 1296), with `buildCurveEdges` (l. 1258): the left
/// edge forward, the head, the right edge back.
fn curved(cl: &[Centre]) -> Vec<[f64; 2]> {
    let edge = |s: f64| -> Vec<[f64; 2]> {
        cl.iter()
            .map(|&[x, y, tx, ty]| [x + s * ty * LA_SW, y - s * tx * LA_SW])
            .collect()
    };
    let [bx, by, tx, ty] = *cl.last().expect("a centreline");
    let (px, py) = (ty, -tx);
    let mut v = edge(1.0);
    v.push([bx + px * LA_HW, by + py * LA_HW]);
    v.push([bx + tx * LA_HL, by + ty * LA_HL]);
    v.push([bx - px * LA_HW, by - py * LA_HW]);
    v.extend(edge(-1.0).into_iter().rev());
    v
}

/// `drawStraightArrow` (l. 1371).
fn straight() -> Vec<[f64; 2]> {
    let (tail, tip) = (-LA_LEN / 2.0, LA_LEN / 2.0);
    let neck = tip - LA_HL;
    vec![
        [tail, -LA_SW],
        [neck, -LA_SW],
        [neck, -LA_HW],
        [tip, 0.0],
        [neck, LA_HW],
        [neck, LA_SW],
        [tail, LA_SW],
    ]
}

/// `drawCurvedTurnArrow` (l. 1429): `dir` −1 left, +1 right.
fn turn(dir: f64) -> Vec<[f64; 2]> {
    let r = LA_TURN_R;
    let mut cl = vec![[-LA_TURN_SHAFT, 0.0, 1.0, 0.0]];
    cl.extend(quad_bezier([0.0, 0.0], [r, 0.0], [r, dir * r], SEGMENTS));
    curved(&cl)
}

/// The branch of `drawThroughTurn` (l. 1446) and `drawThroughLeftRight` (l. 1490).
fn branch(dir: f64) -> Vec<[f64; 2]> {
    let r = LA_TURN_R;
    let fork = -LA_LEN / 2.0 + FORK;
    curved(&quad_bezier(
        [fork, 0.0],
        [fork + r, 0.0],
        [fork + r, dir * r],
        SEGMENTS,
    ))
}

/// `drawUTurnArrow` (l. 1603).
fn u_turn() -> Vec<[f64; 2]> {
    let r = UTURN_R;
    let mut cl = vec![[-LA_TURN_SHAFT, 0.0, 1.0, 0.0]];
    for i in 0..=SEGMENTS {
        let a = std::f64::consts::FRAC_PI_2 - std::f64::consts::PI * i as f64 / SEGMENTS as f64;
        cl.push([r * a.cos(), -r + r * a.sin(), a.sin(), -a.cos()]);
    }
    cl.push([-LA_TURN_SHAFT * UTURN_RETURN, -2.0 * r, -1.0, 0.0]);
    curved(&cl)
}

/// A glyph: its white outlines, each a simple polygon, and a dead end's red bar.
pub type Glyph = (Vec<Vec<[f64; 2]>>, Option<Vec<[f64; 2]>>);

/// A lane arrow's glyph as `drawLaneArrow` (l. 1324) dispatches on `arrow_type`. Any type
/// not below draws as a straight arrow, as its `else` does.
pub fn glyph(arrow_type: &str) -> Glyph {
    match arrow_type {
        "through" => (vec![straight()], None),
        "left" => (vec![turn(-1.0)], None),
        "right" => (vec![turn(1.0)], None),
        "through_left" => (vec![straight(), branch(-1.0)], None),
        "through_right" => (vec![straight(), branch(1.0)], None),
        "through_left_right" => (vec![straight(), branch(-1.0), branch(1.0)], None),
        // `drawLeftRight` (l. 1536): one path that crosses itself, as its two halves.
        "left_right" => (vec![turn(-1.0), turn(1.0)], None),
        "u_turn" => (vec![u_turn()], None),
        // `drawDeadEndArrow` (l. 1393): a shaft to the bar, and the bar.
        "dead_end" => {
            let tail = -LA_LEN / 2.0;
            let bb = LA_LEN / 2.0 - BAR_DEPTH;
            (
                vec![vec![
                    [tail, -LA_SW],
                    [bb, -LA_SW],
                    [bb, LA_SW],
                    [tail, LA_SW],
                ]],
                Some(vec![
                    [bb, -BAR_HW],
                    [bb + BAR_DEPTH, -BAR_HW],
                    [bb + BAR_DEPTH, BAR_HW],
                    [bb, BAR_HW],
                ]),
            )
        }
        _ => (vec![straight()], None),
    }
}

/// A glyph point `(u, v)` placed at `at` with `heading` in degrees (`atan2(dy, dx)`):
/// `at + u·(cos h, sin h) + v·(sin h, −cos h)`, the canvas's `rotate(−heading)` on a
/// y-down screen.
pub fn place(at: [f64; 2], heading_deg: f64, p: [f64; 2]) -> [f64; 2] {
    let h = heading_deg.to_radians();
    let (c, s) = (h.cos(), h.sin());
    [at[0] + p[0] * c + p[1] * s, at[1] + p[0] * s - p[1] * c]
}

impl Streets {
    /// The streets of a placed network (§2.17.4, §2.17.5, §2.18).
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

        // Looked up only: a link's twin (the first in link order), a link's index.
        let mut by_nodes: HashMap<(&str, &str), usize> = HashMap::new();
        for (i, l) in net.links.iter().enumerate() {
            by_nodes
                .entry((l.from_node.0.as_str(), l.to_node.0.as_str()))
                .or_insert(i);
        }
        let index: HashMap<&str, usize> = net
            .links
            .iter()
            .enumerate()
            .map(|(i, l)| (l.id.0.as_str(), i))
            .collect();

        // Two-way pairs and their median fills, along the link that comes first.
        for (i, link) in net.links.iter().enumerate() {
            let twin = by_nodes.get(&(link.to_node.0.as_str(), link.from_node.0.as_str()));
            if let Some(&j) = twin
                && i < j
            {
                let id = link.id.0.as_str();
                let w = link.total_width();
                let g = (link.median_gap + net.links[j].median_gap) / 2.0;
                // The gap lies left of `a`'s left edge.
                if g > 0.0
                    && let Some(ribbon) =
                        ribbon(pl, id, 0.0, pl.link_length(id), -w / 2.0 - g, -w / 2.0)
                {
                    st.fills.push(Fill { link: i, ribbon });
                }
                st.pairs.push(Pair { a: i, b: j, gap: g });
            }
        }

        // The markings, from the engine's dashboard geometry, in its order (§2.18.2).
        let nj = NetworkJson::from_config_with_network_data(net, data);
        let mut mark = |kind, link, lane, arrow: Option<&str>, polygon: Vec<[f64; 2]>| {
            let mut triangles = Vec::new();
            earcut.earcut(polygon.iter().copied(), &[] as &[u32], &mut triangles);
            st.markings.push(Marking {
                kind,
                link,
                lane,
                arrow: arrow.map(str::to_string),
                polygon,
                triangles,
            });
        };
        for m in &nj.median_lines {
            mark(
                Kind::Centre,
                None,
                None,
                None,
                split_edges(&open(&m.coords)),
            );
        }
        for m in &nj.lane_marking_dashes {
            mark(Kind::Dash, None, None, None, open(&m.coords));
        }
        for m in &nj.solid_lane_lines {
            mark(Kind::Solid, None, None, None, split_edges(&open(&m.coords)));
        }

        // Each link's trimmed end at lateral 0, where a full-width stop line's
        // junction-facing edge has its midpoint (§2.18.6).
        let ends: Vec<Option<[f64; 2]>> = net
            .links
            .iter()
            .map(|l| {
                let id = l.id.0.as_str();
                let off = data.stop_line_offset(&LinkId(id.to_string()));
                pl.place_lateral(id, pl.link_length(id) - off, 0.0)
                    .map(|p| [p.x, p.y])
            })
            .collect();
        for s in &nj.stop_lines {
            let ring = open(&s.coords);
            match (&s.link_id, s.lane) {
                (Some(id), Some(k)) => mark(
                    Kind::Stop,
                    index.get(id.as_str()).copied(),
                    Some(k),
                    None,
                    ring,
                ),
                (Some(id), None) => mark(
                    Kind::Connector,
                    index.get(id.as_str()).copied(),
                    None,
                    None,
                    ring,
                ),
                (None, _) => match full_width_link(&ring, &ends) {
                    Some(i) => {
                        for (k, quad) in cut(&ring, &net.links[i]).into_iter().enumerate() {
                            mark(Kind::Stop, Some(i), Some(k as u32), None, quad);
                        }
                    }
                    None => mark(Kind::Stop, None, None, None, ring),
                },
            }
        }

        for a in &nj.lane_arrows {
            let link = index.get(a.link_id.as_str()).copied();
            let at = [a.x, a.y];
            let (white, bar) = glyph(&a.arrow_type);
            for outline in white {
                let placed = outline.iter().map(|&p| place(at, a.heading, p)).collect();
                mark(Kind::Arrow, link, Some(a.lane), Some(&a.arrow_type), placed);
            }
            if let Some(bar) = bar {
                let placed = bar.iter().map(|&p| place(at, a.heading, p)).collect();
                mark(
                    Kind::DeadEnd,
                    link,
                    Some(a.lane),
                    Some(&a.arrow_type),
                    placed,
                );
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

    /// The markings mesh's data: kind by kind in [`Kind::ALL`]'s order, each in build
    /// order, at its kind's lift, colour and width.
    pub fn markings(&self) -> MarkingsData {
        let mut d = MarkingsData::default();
        for kind in Kind::ALL {
            let (lift, colour, width) = (kind.lift(), kind.colour(), kind.width());
            for m in self.markings.iter().filter(|m| m.kind == kind) {
                let base = d.positions.len() as u32;
                d.positions
                    .extend(m.polygon.iter().map(|p| [p[0], p[1], lift]));
                d.colours
                    .extend(std::iter::repeat_n(colour, m.polygon.len()));
                d.widths.extend(std::iter::repeat_n(width, m.polygon.len()));
                d.indices.extend(m.triangles.iter().map(|t| base + t));
            }
        }
        d
    }
}

/// A full-width stop line's link: the one whose trimmed end lies nearest the midpoint of
/// its junction-facing edge (`ring[1]`–`ring[2]`), the first in link order on a tie, if
/// within 1 mm (§2.18.6).
fn full_width_link(ring: &[[f64; 2]], ends: &[Option<[f64; 2]>]) -> Option<usize> {
    if ring.len() != 4 {
        return None;
    }
    let mid = [
        (ring[1][0] + ring[2][0]) / 2.0,
        (ring[1][1] + ring[2][1]) / 2.0,
    ];
    let mut best = (f64::INFINITY, None);
    for (i, e) in ends.iter().enumerate() {
        if let Some(e) = e {
            let d = ((e[0] - mid[0]).powi(2) + (e[1] - mid[1]).powi(2)).sqrt();
            if d < best.0 {
                best = (d, Some(i));
            }
        }
    }
    if best.0 <= 0.001 { best.1 } else { None }
}

/// A full-width stop line cut at its link's lane boundaries, one quad per lane in lane
/// order (§2.18.6). Its corners as `network_json.rs` builds them: `[0]` back right, `[1]`
/// front right, `[2]` front left, `[3]` back left. Each cut point lies on the ring's own
/// back or front edge, at lateral `l` as the fraction `(l + W/2) / W` from its left corner;
/// each quad keeps the ring's winding.
fn cut(ring: &[[f64; 2]], link: &assimilator_config::network::LinkConfig) -> Vec<Vec<[f64; 2]>> {
    let hw = link.total_width() / 2.0;
    let offs = link.lane_center_offsets();
    let at = |left: [f64; 2], right: [f64; 2], l: f64| {
        let t = (l + hw) / (2.0 * hw);
        [
            left[0] + (right[0] - left[0]) * t,
            left[1] + (right[1] - left[1]) * t,
        ]
    };
    let (back, front) = (|l| at(ring[3], ring[0], l), |l| at(ring[2], ring[1], l));
    link.lanes
        .iter()
        .enumerate()
        .map(|(k, lane)| {
            let (a, b) = (offs[k] - lane.width / 2.0, offs[k] + lane.width / 2.0);
            vec![back(b), front(b), front(a), back(a)]
        })
        .collect()
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

/// §2.17.10's `α` for a marking `width_m` wide where a pixel is `mpp` metres: 0 under
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

/// The road's linear colour, then the five markings' ([`COLOURS`]), computed once.
static LINEAR: LazyLock<[[f32; 3]; 6]> = LazyLock::new(|| {
    let mut v = [linear3(scene::ROAD); 6];
    for (i, c) in COLOURS.iter().enumerate() {
        v[i + 1] = linear3(*c);
    }
    v
});

/// A marking's colour at `α`, linear RGBA: `(1 − α)·road + α·marking` per channel in
/// `f32`, exactly the road's at 0 and the marking's at 1. Opaque.
pub fn colour(marking: [u8; 3], alpha: f64) -> [f32; 4] {
    let lin = &*LINEAR;
    let road = lin[0];
    let m = COLOURS
        .iter()
        .position(|c| *c == marking)
        .map_or_else(|| linear3(marking), |i| lin[i + 1]);
    let a = alpha as f32;
    let mix = |i: usize| (1.0 - a) * road[i] + a * m[i];
    [mix(0), mix(1), mix(2), 1.0]
}

/// Each marking vertex's colour, faded by its marking's width on screen at `mpp` (metres
/// per pixel at a world point).
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
