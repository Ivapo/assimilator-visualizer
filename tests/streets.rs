//! vis-002 Phase 5 exit gates 5, 6, 7, 8, 9 and 12 (`specs/city_spec.md`): the streets
//! built from a synthetic crossing and from both fixtures, checked against the engine's
//! own dashboard geometry and against the run's boxes, drawn through the GPU, faded, and
//! `M`. The headless ones without a fixture run in plain `cargo test`. Those that need the
//! fixtures of `scripts/fixture.sh` and `scripts/fixture.sh midtown`, or the GPU, are
//! ignored; run everything with
//! `cargo test --release --test streets -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 10 and 11 are `scripts/gates-streets.sh`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use assimilator_config::network::{ControlType, NetworkConfig};
use assimilator_config::types::{LaneIdx, LinkId};
use assimilator_geometry::network_json::NetworkJson;
use assimilator_video::camera::Pose;
use assimilator_video::motion::Piece;
use assimilator_video::place::Placement;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::scene;
use assimilator_video::streets::{
    self, DASH_M, GAP_M, Kind, LINE_M, Pair, STOP_M, Streets, WHITE, YELLOW, colour, fade, mpp_at,
};

type P = [f64; 2];
/// A link's engine stop-line edges: each with its lane (`None`: the whole approach).
type Edges = Vec<(Option<u32>, [P; 2])>;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

// ── Gate 5: a synthetic crossing ─────────────────────────────────────────────

const SYNTHETIC: &str = r#"
schema_version: 1
metadata: {name: streets synthetic, coordinate_system: metric, z_enabled: false}
nodes:
  - {id: C, point: [0, 0], type: junction}
  - {id: W, point: [-50, 0], type: endpoint}
  - {id: E, point: [50, 0], type: endpoint}
  - {id: N, point: [0, 50], type: endpoint}
  - {id: S, point: [0, -50], type: endpoint}
links:
  - {id: L_WC, from_node: W, to_node: C, geometry: [[-50, 0], [0, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CW, from_node: C, to_node: W, geometry: [[0, 0], [-50, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_EC, from_node: E, to_node: C, geometry: [[50, 0], [0, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CE, from_node: C, to_node: E, geometry: [[0, 0], [50, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_NC, from_node: N, to_node: C, geometry: [[0, 50], [0, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CS, from_node: C, to_node: S, geometry: [[0, 0], [0, -50]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
junctions:
  - {node_id: C, control: signal, geometry: {setback: 3}, stop_line_offsets: {L_EC: {'0': 5}}}
"#;

pub fn synthetic() -> Placement {
    let cfg: NetworkConfig = serde_yaml::from_str(SYNTHETIC).expect("the synthetic network");
    Placement::new(cfg)
}

/// Each synthetic link as drawn: its trimmed start, its direction of travel, its length,
/// and its strip's centre and right of travel on the cross axis (`y` for east–west links,
/// `x` for north–south ones).
struct Line {
    start: P,
    dir: P,
    len: f64,
    centre: f64,
    right: f64,
}

impl Line {
    fn s(&self, p: P) -> f64 {
        (p[0] - self.start[0]) * self.dir[0] + (p[1] - self.start[1]) * self.dir[1]
    }
    fn cross(&self, p: P) -> f64 {
        if self.dir[0] != 0.0 { p[1] } else { p[0] }
    }
    /// The cross coordinate of lateral `l` (right of travel positive).
    fn at(&self, l: f64) -> f64 {
        self.centre + self.right * l
    }
}

/// The links of [`SYNTHETIC`], in link order: the junction's polygon reaches x ±6.5 and
/// y ±10.25, and a two-way link's strip stands `w/2 + g/2` = 3.75 m right of its centreline.
fn lines() -> [Line; 6] {
    let ew = 43.5;
    let ns = 39.75;
    [
        Line {
            start: [-50.0, 0.0],
            dir: [1.0, 0.0],
            len: ew,
            centre: -3.75,
            right: -1.0,
        },
        Line {
            start: [-6.5, 0.0],
            dir: [-1.0, 0.0],
            len: ew,
            centre: 3.75,
            right: 1.0,
        },
        Line {
            start: [50.0, 0.0],
            dir: [-1.0, 0.0],
            len: ew,
            centre: 3.75,
            right: 1.0,
        },
        Line {
            start: [6.5, 0.0],
            dir: [1.0, 0.0],
            len: ew,
            centre: -3.75,
            right: -1.0,
        },
        Line {
            start: [0.0, 50.0],
            dir: [0.0, -1.0],
            len: ns,
            centre: 0.0,
            right: -1.0,
        },
        Line {
            start: [0.0, -10.25],
            dir: [0.0, -1.0],
            len: ns,
            centre: 0.0,
            right: -1.0,
        },
    ]
}

/// A ribbon's points.
fn points(r: &streets::Ribbon) -> impl Iterator<Item = P> + '_ {
    r.left.iter().chain(&r.right).copied()
}

/// `(min, max)` of `f` over a ribbon.
fn range(r: &streets::Ribbon, f: impl Fn(P) -> f64) -> (f64, f64) {
    points(r).fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| {
        (a.min(f(p)), b.max(f(p)))
    })
}

fn shoelace(r: &[P]) -> f64 {
    let n = r.len();
    (0..n)
        .map(|i| r[i][0] * r[(i + 1) % n][1] - r[(i + 1) % n][0] * r[i][1])
        .sum::<f64>()
        .abs()
        / 2.0
}

fn tri_area(a: P, b: P, c: P) -> f64 {
    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
}

#[test]
fn gate5_synthetic_crossing() {
    let pl = synthetic();
    let st = Streets::build(&pl);
    let ls = lines();
    for (l, line) in pl.network.links.iter().zip(&ls) {
        assert!(
            close(pl.link_length(&l.id.0), line.len),
            "{} length",
            l.id.0
        );
    }

    // One junction surface.
    let raw = pl
        .data
        .junction_polygons
        .get(&pl.network.nodes[0].id)
        .unwrap();
    assert_eq!(raw.len(), 44, "the engine's polygon as given");
    assert_eq!(
        raw.first(),
        raw.last(),
        "its last vertex a copy of its first"
    );
    assert_eq!(st.junctions.len(), 1);
    let j = &st.junctions[0];
    assert_eq!(j.node, "C");
    assert_eq!(j.polygon.len(), 43);
    let distinct: HashSet<(u64, u64)> = j
        .polygon
        .iter()
        .map(|p| (p[0].to_bits(), p[1].to_bits()))
        .collect();
    assert_eq!(distinct.len(), 36, "distinct vertices");
    let xmax = j.polygon.iter().map(|p| p[0].abs()).fold(0.0, f64::max);
    let ymax = j.polygon.iter().map(|p| p[1].abs()).fold(0.0, f64::max);
    assert!(
        close(xmax, 6.5) && close(ymax, 10.25),
        "extent {xmax} {ymax}"
    );
    let area = shoelace(&j.polygon);
    assert!((area - 236.69).abs() < 0.005, "area {area}");
    let tris: f64 = j
        .triangles
        .chunks(3)
        .map(|t| {
            tri_area(
                j.polygon[t[0] as usize],
                j.polygon[t[1] as usize],
                j.polygon[t[2] as usize],
            )
        })
        .sum();
    assert!(
        (tris - area).abs() <= 1e-9 * area,
        "triangles {tris} against {area}"
    );
    println!(
        "gate 5: polygon 44 → 43 ({} distinct), area {area:.4} m², {} triangles",
        distinct.len(),
        j.triangles.len() / 3
    );

    // Pairs, fills and yellow lines.
    assert_eq!(
        st.pairs,
        vec![
            Pair {
                a: 0,
                b: 1,
                gap: 0.5
            },
            Pair {
                a: 2,
                b: 3,
                gap: 0.5
            }
        ]
    );
    assert_eq!(st.fills.len(), 2);
    for f in &st.fills {
        let line = &ls[f.link];
        let (c0, c1) = range(&f.ribbon, |p| line.cross(p));
        assert!(close(c0, -0.25) && close(c1, 0.25), "fill {c0} {c1}");
        let (s0, s1) = range(&f.ribbon, |p| line.s(p));
        assert!(close(s0, 0.0) && close(s1, line.len));
    }
    let of = |k: Kind| st.markings.iter().filter(move |m| m.kind == k);
    let yellow: Vec<_> = of(Kind::Yellow).collect();
    assert_eq!(yellow.len(), 4);
    let mut sides = Vec::new();
    for (i, m) in yellow.iter().enumerate() {
        assert_eq!(
            m.link,
            [0, 0, 2, 2][i],
            "yellow only along each pair's first link"
        );
        assert_eq!((m.lane, m.width), (None, LINE_M));
        let (c0, c1) = range(&m.ribbon, |p| p[1]);
        let north = close(c0, 0.05) && close(c1, 0.20);
        let south = close(c0, -0.20) && close(c1, -0.05);
        assert!(north || south, "yellow at y {c0} … {c1}");
        sides.push(north);
        let xs = range(&m.ribbon, |p| p[0].abs());
        assert!(
            close(xs.0, 6.5) && close(xs.1, 50.0),
            "yellow from the edge to the end"
        );
    }
    assert!(
        sides[0] != sides[1] && sides[2] != sides[3],
        "one line each side of y 0 a pair"
    );

    // Lane-line dashes.
    let dashes: Vec<_> = of(Kind::Lane).collect();
    assert_eq!(dashes.len(), 24);
    for (li, line) in ls.iter().enumerate() {
        let mine: Vec<_> = dashes.iter().filter(|m| m.link == li).collect();
        assert_eq!(mine.len(), 4, "4 dashes on link {li}");
        // Each approach's dashes end at its stop lines; L_EC's at lane 0's, 5 m back.
        let end = match li {
            0 | 4 => line.len - STOP_M,
            2 => line.len - 5.0 - STOP_M,
            _ => line.len,
        };
        for (i, m) in mine.iter().enumerate() {
            assert_eq!(m.lane, Some(0));
            assert_eq!(m.width, LINE_M);
            let (c0, c1) = range(&m.ribbon, |p| line.cross(p));
            let c = line.at(0.0);
            assert!(
                close(c0, c - LINE_M / 2.0) && close(c1, c + LINE_M / 2.0),
                "dash across {c0} {c1}"
            );
            let (s0, s1) = range(&m.ribbon, |p| line.s(p));
            let a = i as f64 * (DASH_M + GAP_M);
            assert!(
                close(s0, a) && close(s1, (a + DASH_M).min(end)),
                "dash {li}/{i}: {s0} {s1}"
            );
        }
    }
    let ec_last = dashes.iter().rfind(|m| m.link == 2).unwrap();
    assert!(close(range(&ec_last.ribbon, |p| ls[2].s(p)).1, 37.9));

    // Stop lines.
    let stops: Vec<_> = of(Kind::Stop).collect();
    let got: Vec<(usize, Option<u32>)> = stops.iter().map(|m| (m.link, m.lane)).collect();
    assert_eq!(
        got,
        vec![
            (0, Some(0)),
            (0, Some(1)),
            (2, Some(0)),
            (2, Some(1)),
            (4, Some(0)),
            (4, Some(1))
        ]
    );
    for m in &stops {
        let line = &ls[m.link];
        let k = m.lane.unwrap() as f64;
        assert_eq!(m.width, STOP_M);
        let (s0, s1) = range(&m.ribbon, |p| line.s(p));
        let edge = if (m.link, m.lane) == (2, Some(0)) {
            line.len - 5.0
        } else {
            line.len
        };
        assert!(
            close(s1, edge) && close(s0, edge - STOP_M),
            "stop line {s0} {s1}"
        );
        let (c0, c1) = range(&m.ribbon, |p| line.cross(p));
        let (a, b) = (line.at(-3.5 + 3.5 * k), line.at(3.5 * k));
        assert!(
            close(c0, a.min(b)) && close(c1, a.max(b)),
            "across its lane {c0} {c1}"
        );
    }
    let x = |m: &streets::Marking| range(&m.ribbon, |p| p[0]);
    assert!(
        close(x(stops[0]).0, -7.1) && close(x(stops[0]).1, -6.5),
        "L_WC at the edge"
    );
    assert!(
        close(x(stops[2]).0, 11.5) && close(x(stops[2]).1, 12.1),
        "L_EC lane 0, 5 m back"
    );
    assert!(
        close(x(stops[3]).0, 6.5) && close(x(stops[3]).1, 7.1),
        "L_EC lane 1"
    );
    let y4 = range(&stops[4].ribbon, |p| p[1]);
    assert!(close(y4.0, 10.25) && close(y4.1, 10.85), "L_NC at the edge");
    println!("gate 5: 1 surface, 2 pairs, 4 yellow, 2 fills, 24 dashes, 6 stop lines: PASS");
}

// ── Gate 9: the fade ─────────────────────────────────────────────────────────

fn linear(c: u8) -> f32 {
    let c = c as f64 / 255.0;
    (if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }) as f32
}

#[test]
fn gate9_fade() {
    assert_eq!(fade(0.15, 1.5), 0.0);
    assert!((fade(0.15, 0.5) - 0.5).abs() <= 1e-12);
    assert_eq!(fade(0.15, 0.3), 1.0);
    let stop = fade(STOP_M, 1.6009);
    assert!((stop - 0.767).abs() <= 0.001, "Midtown's stop line {stop}");
    let ug = fade(LINE_M, 1.1481);
    assert!((ug - 0.017).abs() <= 0.001, "urban_grid's line {ug}");
    for m in [WHITE, YELLOW] {
        let (c0, c1) = (colour(m, 0.0), colour(m, 1.0));
        for i in 0..3 {
            assert_eq!(c0[i], linear(scene::ROAD[i]), "α 0 is the road");
            assert_eq!(c1[i], linear(m[i]), "α 1 is the marking");
        }
        assert_eq!((c0[3], c1[3]), (1.0, 1.0));
    }
    let (cx, cy) = (1234.5, -678.0);
    for h in [60.0, 500.0, 1800.0] {
        for hp in [720.0, 1080.0] {
            let f = mpp_at(&Pose::top_down(cx, cy, h), hp);
            let at = f([cx, cy, 0.0]);
            assert!(
                (at - h / hp).abs() <= 1e-9 * (h / hp),
                "top down {h} {hp}: {at}"
            );
            assert!(
                f([cx + 100.0, cy, 0.0]) > at,
                "farther from the look-at point"
            );
        }
    }
    let tilted = Pose {
        cx,
        cy,
        height_m: 500.0,
        yaw_deg: 0.0,
        pitch_deg: 25.0,
    };
    let f = mpp_at(&tilted, 1080.0);
    // Yaw 0 faces north: the eye is south of the look-at point.
    assert!(
        f([cx, cy + 50.0, 0.0]) > f([cx, cy - 50.0, 0.0]),
        "the farther one is larger"
    );
    println!(
        "gate 9: fade 0, 0.5, 1; stop {stop:.4}; urban_grid {ug:.4}; colour's ends exact; mpp_at: PASS"
    );
}

// ── Gates 6 and 7: the fixtures ──────────────────────────────────────────────

fn load(dir: &str) -> Run {
    run::load(&LoadOptions {
        project: root().join("scratch").join(dir),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: None,
        to: None,
    })
    .unwrap()
}

fn seg_dist(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((p[0] - a[0] - t * dx).powi(2) + (p[1] - a[1] - t * dy).powi(2)).sqrt()
}

fn ring_dist(p: P, r: &[P]) -> f64 {
    (0..r.len())
        .map(|i| seg_dist(p, r[i], r[(i + 1) % r.len()]))
        .fold(f64::INFINITY, f64::min)
}

struct Expect {
    junctions: usize,
    pairs: usize,
    fills: usize,
    yellow: usize,
    stops: usize,
    stop_links: usize,
    dashes: usize,
    surface: (usize, usize),
    markings: (usize, usize),
    corners: usize,
}

fn gate6(dir: &str, e: Expect) {
    let run = load(dir);
    let pl = &run.placement;
    let net = &pl.network;
    let st = Streets::build(pl);

    // Independently, from the config.
    let signal: HashSet<&str> = net
        .junctions
        .iter()
        .filter(|j| j.control == ControlType::Signal)
        .map(|j| j.node_id.0.as_str())
        .collect();
    let polys = net
        .nodes
        .iter()
        .filter(|n| pl.data.junction_polygons.contains_key(&n.id))
        .count();
    let mut first: HashMap<(&str, &str), usize> = HashMap::new();
    for (i, l) in net.links.iter().enumerate() {
        first.entry((&l.from_node.0, &l.to_node.0)).or_insert(i);
    }
    let (mut pairs, mut fills, mut yellow, mut stops, mut stop_links, mut dashes) =
        (0, 0, 0, 0, 0, 0);
    for (i, l) in net.links.iter().enumerate() {
        let len = pl.link_length(&l.id.0);
        if let Some(&j) = first.get(&(l.to_node.0.as_str(), l.from_node.0.as_str()))
            && i < j
        {
            pairs += 1;
            let g = (l.median_gap + net.links[j].median_gap) / 2.0;
            fills += (g > 0.0) as usize;
            yellow += if g >= 1.0 { 4 } else { 2 };
        }
        let mut end = len;
        if signal.contains(l.to_node.0.as_str()) {
            stop_links += 1;
            stops += l.lanes.len();
            for k in 0..l.lanes.len() {
                let off = pl
                    .data
                    .lane_stop_line_offset(&LinkId(l.id.0.clone()), LaneIdx(k as u32));
                end = end.min(len - off - 0.6);
            }
        }
        dashes += l.lanes.len().saturating_sub(1) * (end / 12.0).ceil().max(0.0) as usize;
    }
    let count = |k: Kind| st.markings.iter().filter(|m| m.kind == k).count();
    let links_with_stops: HashSet<usize> = st
        .markings
        .iter()
        .filter(|m| m.kind == Kind::Stop)
        .map(|m| m.link)
        .collect();
    let surface = st.surface();
    let markings = st.markings();
    let got = [
        st.junctions.len(),
        st.pairs.len(),
        st.fills.len(),
        count(Kind::Yellow),
        count(Kind::Stop),
        links_with_stops.len(),
        count(Kind::Lane),
    ];
    println!(
        "gate 6 {dir}: surfaces {} pairs {} fills {} yellow {} stop lines {} ({} links) dashes {}; surface {} vertices {} triangles; markings {} vertices {} triangles",
        got[0],
        got[1],
        got[2],
        got[3],
        got[4],
        got[5],
        got[6],
        surface.positions.len(),
        surface.indices.len() / 3,
        markings.positions.len(),
        markings.indices.len() / 3
    );
    assert_eq!(
        got,
        [polys, pairs, fills, yellow, stops, stop_links, dashes],
        "independent counts"
    );
    assert_eq!(
        got,
        [
            e.junctions,
            e.pairs,
            e.fills,
            e.yellow,
            e.stops,
            e.stop_links,
            e.dashes
        ],
        "predicted counts"
    );
    assert_eq!(
        (surface.positions.len(), surface.indices.len() / 3),
        e.surface
    );
    assert_eq!(
        (markings.positions.len(), markings.indices.len() / 3),
        e.markings
    );
    assert_eq!(markings.colours.len(), markings.positions.len());
    assert_eq!(markings.widths.len(), markings.positions.len());

    // Against the engine's dashboard geometry (§2.17.2).
    let nj = NetworkJson::from_config_with_network_data(net, &pl.data);
    let ours: HashMap<&str, &streets::Junction> =
        st.junctions.iter().map(|j| (j.node.as_str(), j)).collect();
    assert_eq!(nj.junction_fills.len(), e.junctions);
    let mut fill_worst: f64 = 0.0;
    for f in &nj.junction_fills {
        let mut ring = f.coords.clone();
        while ring.len() > 1 && ring.first() == ring.last() {
            ring.pop();
        }
        let j = ours[f.id.as_str()];
        assert_eq!(ring.len(), j.polygon.len(), "fill {} vertex count", f.id);
        for (a, b) in ring.iter().zip(&j.polygon) {
            fill_worst = fill_worst.max(((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt());
        }
    }
    let ends: Vec<P> = net
        .links
        .iter()
        .map(|l| {
            let e = pl
                .place_lateral(&l.id.0, pl.link_length(&l.id.0), 0.0)
                .unwrap();
            [e.x, e.y]
        })
        .collect();
    let nearest = |c: &[P]| -> usize {
        let mid = [(c[1][0] + c[2][0]) / 2.0, (c[1][1] + c[2][1]) / 2.0];
        let mut best = (f64::INFINITY, 0);
        for (i, e) in ends.iter().enumerate() {
            let d = ((e[0] - mid[0]).powi(2) + (e[1] - mid[1]).powi(2)).sqrt();
            if d < best.0 {
                best = (d, i);
            }
        }
        best.1
    };
    // Per link, the engine's junction-facing edges with their lane (`None`: full width).
    let mut eng: HashMap<usize, Edges> = HashMap::new();
    let (mut left_out_unsignalised, mut connectors) = (0, 0);
    for s in &nj.stop_lines {
        let c = &s.coords;
        if s.link_id.is_some() && s.lane.is_none() {
            connectors += 1;
            continue;
        }
        let li = match &s.link_id {
            Some(id) => net.links.iter().position(|l| &l.id.0 == id).unwrap(),
            None => nearest(c),
        };
        if !signal.contains(net.links[li].to_node.0.as_str()) {
            left_out_unsignalised += 1;
            continue;
        }
        eng.entry(li).or_default().push((s.lane, [c[1], c[2]]));
    }
    let (mut corners, mut stop_worst) = (0, 0.0f64);
    for m in st.markings.iter().filter(|m| m.kind == Kind::Stop) {
        let edges: Vec<_> = eng
            .get(&m.link)
            .map(|v| {
                v.iter()
                    .filter(|(l, _)| l.is_none() || *l == m.lane)
                    .collect()
            })
            .unwrap_or_default();
        assert!(
            !edges.is_empty(),
            "an engine stop line for {} lane {:?}",
            m.link,
            m.lane
        );
        for p in [
            *m.ribbon.left.last().unwrap(),
            *m.ribbon.right.last().unwrap(),
        ] {
            let d = edges
                .iter()
                .map(|(_, e)| seg_dist(p, e[0], e[1]))
                .fold(f64::INFINITY, f64::min);
            stop_worst = stop_worst.max(d);
            corners += 1;
        }
    }
    println!(
        "gate 6 {dir}: engine fills {} worst {fill_worst:.6} m; stop-line corners {corners} worst {stop_worst:.6} m (left out: {left_out_unsignalised} unsignalised, {connectors} connectors)",
        nj.junction_fills.len()
    );
    assert!(fill_worst <= 0.01 && stop_worst <= 0.01);
    assert_eq!(corners, e.corners);
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_counts() {
    gate6(
        "midtown",
        Expect {
            junctions: 106,
            pairs: 29,
            fills: 26,
            yellow: 88,
            stops: 402,
            stop_links: 188,
            dashes: 2208,
            surface: (10_510, 9_532),
            markings: (37_594, 32_198),
            corners: 804,
        },
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate6_urban_grid_counts() {
    gate6(
        "urban_grid",
        Expect {
            junctions: 9,
            pairs: 24,
            fills: 24,
            yellow: 48,
            stops: 71,
            stop_links: 36,
            dashes: 1105,
            surface: (13_971, 13_842),
            markings: (36_292, 33_844),
            corners: 142,
        },
    );
}

fn in_tri(p: P, a: P, b: P, c: P) -> bool {
    let s =
        |p1: P, p2: P, p3: P| (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1]);
    let (d1, d2, d3) = (s(p, a, b), s(p, b, c), s(p, c, a));
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// Triangles binned on a 10 m grid.
struct Grid {
    tris: Vec<[P; 3]>,
    cells: HashMap<(i64, i64), Vec<usize>>,
}

impl Grid {
    const CELL: f64 = 10.0;
    fn new(tris: Vec<[P; 3]>) -> Grid {
        let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            let f = |k: usize, m: fn(f64, f64) -> f64, z: f64| t.iter().map(|p| p[k]).fold(z, m);
            let (x0, x1) = (
                f(0, f64::min, f64::INFINITY),
                f(0, f64::max, f64::NEG_INFINITY),
            );
            let (y0, y1) = (
                f(1, f64::min, f64::INFINITY),
                f(1, f64::max, f64::NEG_INFINITY),
            );
            for gx in (x0 / Self::CELL).floor() as i64..=(x1 / Self::CELL).floor() as i64 {
                for gy in (y0 / Self::CELL).floor() as i64..=(y1 / Self::CELL).floor() as i64 {
                    cells.entry((gx, gy)).or_default().push(i);
                }
            }
        }
        Grid { tris, cells }
    }
    fn hit(&self, p: P) -> bool {
        let k = (
            (p[0] / Self::CELL).floor() as i64,
            (p[1] / Self::CELL).floor() as i64,
        );
        self.cells.get(&k).is_some_and(|v| {
            v.iter().any(|&i| {
                let t = self.tris[i];
                in_tri(p, t[0], t[1], t[2])
            })
        })
    }
}

fn strip_tris(l: &[P], r: &[P], out: &mut Vec<[P; 3]>) {
    for i in 0..l.len() - 1 {
        out.push([l[i], r[i], l[i + 1]]);
        out.push([r[i], r[i + 1], l[i + 1]]);
    }
}

struct Expect7 {
    corners: usize,
    centres: usize,
    strips_pct: f64,
    in_junction: usize,
    episodes: usize,
    rows: usize,
    within: usize,
    early: usize,
    at_edge: usize,
    other: usize,
}

fn gate7(dir: &str, e: Expect7) {
    let run = load(dir);
    let pl = &run.placement;
    let net = &pl.network;
    let st = Streets::build(pl);
    let polys: HashMap<&str, &[P]> = st
        .junctions
        .iter()
        .map(|j| (j.node.as_str(), j.polygon.as_slice()))
        .collect();
    assert_eq!(run.strips.len(), net.links.len(), "one strip a link");

    // Strip ends on their junction's built polygon.
    let (mut corners, mut worst) = (0, 0.0f64);
    for (l, s) in net.links.iter().zip(&run.strips) {
        for (node, cs) in [
            (
                &l.to_node,
                [*s.left.last().unwrap(), *s.right.last().unwrap()],
            ),
            (&l.from_node, [s.left[0], s.right[0]]),
        ] {
            let Some(r) = polys.get(node.0.as_str()) else {
                continue;
            };
            for c in cs {
                corners += 1;
                worst = worst.max(ring_dist(c, r));
            }
        }
    }
    println!("gate 7 {dir}: strip-end corners {corners}, worst {worst:.6} m from the polygon");
    assert_eq!(corners, e.corners);
    assert!(worst <= 0.001);

    // Box centres on the drawn road, every whole second.
    let mut strips_t = Vec::new();
    for s in &run.strips {
        strip_tris(&s.left, &s.right, &mut strips_t);
    }
    let mut junc_t = Vec::new();
    for j in &st.junctions {
        for t in j.triangles.chunks(3) {
            junc_t.push([
                j.polygon[t[0] as usize],
                j.polygon[t[1] as usize],
                j.polygon[t[2] as usize],
            ]);
        }
    }
    let (gs, gj) = (Grid::new(strips_t), Grid::new(junc_t));
    let (mut total, mut on_strips, mut on_road, mut in_j, mut in_j_on_surface) = (0, 0, 0, 0, 0);
    let mut t = run.from.ceil();
    while t <= run.to {
        for b in run.boxes_at(t) {
            let p = [b.at.x, b.at.y];
            let (s, j) = (gs.hit(p), gj.hit(p));
            total += 1;
            on_strips += s as usize;
            on_road += (s || j) as usize;
            if !matches!(b.track.piece, Piece::Link(_)) {
                in_j += 1;
                in_j_on_surface += j as usize;
            }
        }
        t += 1.0;
    }
    let pct = 100.0 * on_strips as f64 / total as f64;
    println!(
        "gate 7 {dir}: box centres {total}; on strips {pct:.3} %; on strips or surfaces {on_road}; in a junction span {in_j}, on the surfaces {in_j_on_surface}"
    );
    assert_eq!(total, e.centres);
    assert_eq!(on_road, total, "all on the drawn road");
    assert_eq!((in_j, in_j_on_surface), (e.in_junction, e.in_junction));
    assert!((pct - e.strips_pct).abs() < 0.0005, "on strips alone {pct}");

    // The first stopped box at a stop line (§2.17.7).
    let signal: HashSet<&str> = net
        .junctions
        .iter()
        .filter(|j| j.control == ControlType::Signal)
        .map(|j| j.node_id.0.as_str())
        .collect();
    let by_id: HashMap<&str, usize> = net
        .links
        .iter()
        .enumerate()
        .map(|(i, l)| (l.id.0.as_str(), i))
        .collect();
    // The upstream edge of the built stop line for each (link, lane).
    let back: HashMap<(usize, u32), f64> = st
        .markings
        .iter()
        .filter(|m| m.kind == Kind::Stop)
        .map(|m| {
            let id = &net.links[m.link].id.0;
            let lane = m.lane.unwrap();
            let s1 = pl.link_length(id)
                - pl.data
                    .lane_stop_line_offset(&LinkId(id.clone()), LaneIdx(lane));
            ((m.link, lane), s1 - m.width)
        })
        .collect();
    let fcd = &run.fcd;
    let (mut rows, mut episodes) = (0, Vec::new());
    let mut last: HashMap<u64, f64> = HashMap::new();
    for snap in &fcd.snapshots {
        let rs = &fcd.rows[snap.start..snap.end];
        let mut lead: HashMap<(u32, u32), f64> = HashMap::new();
        for r in rs {
            let e = lead.entry((r.link, r.lane)).or_insert(f64::NEG_INFINITY);
            *e = e.max(r.position);
        }
        for r in rs {
            if r.speed >= 0.1 {
                continue;
            }
            let li = by_id[fcd.links[r.link as usize].as_str()];
            if !signal.contains(net.links[li].to_node.0.as_str())
                || lead[&(r.link, r.lane)] > r.position
            {
                continue;
            }
            let g = back[&(li, r.lane)] - (r.position + r.length / 2.0);
            if g.abs() > 5.0 {
                continue;
            }
            rows += 1;
            let prev = last.insert(r.vehicle_id, snap.time);
            if prev.is_none_or(|p| snap.time - p > 1.5) {
                episodes.push(g);
            }
        }
    }
    episodes.sort_by(f64::total_cmp);
    let median = episodes[((episodes.len() - 1) as f64 * 0.5).round() as usize];
    let within = episodes.iter().filter(|g| g.abs() <= 0.10).count();
    let early = episodes
        .iter()
        .filter(|g| (1.35..=1.55).contains(*g))
        .count();
    // "At −0.600 m" is the probe's 0.1 m bucket (§2.17.15's histogram, `round(10·g)` −6):
    // fronts at the junction's edge, or within 5 cm of it. Exactly −0.600 is printed too.
    let at_edge = episodes
        .iter()
        .filter(|g| (*g * 10.0).round() == -6.0)
        .count();
    let exact = episodes
        .iter()
        .filter(|g| (*g + 0.6).abs() <= 0.0005)
        .count();
    let other = episodes.len() - within - early - at_edge;
    println!(
        "gate 7 {dir}: first stopped boxes {} episodes ({rows} rows), median gap {median:+.4} m; within 0.10 m {within}, 1.35–1.55 m {early}, at −0.600 m {at_edge} ({exact} within 0.5 mm of it), other {other}",
        episodes.len()
    );
    assert_eq!(
        (episodes.len(), rows, within, early, at_edge, other),
        (e.episodes, e.rows, e.within, e.early, e.at_edge, e.other)
    );
    assert!((median - 0.042).abs() < 0.0005, "median {median}");
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate7_midtown_strips_and_boxes() {
    gate7(
        "midtown",
        Expect7 {
            corners: 922,
            centres: 201_256,
            strips_pct: 91.202,
            in_junction: 17_706,
            episodes: 2_491,
            rows: 48_899,
            within: 2_441,
            early: 27,
            at_edge: 10,
            other: 13,
        },
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate7_urban_grid_strips_and_boxes() {
    gate7(
        "urban_grid",
        Expect7 {
            corners: 144,
            centres: 21_358,
            strips_pct: 94.236,
            in_junction: 1_231,
            episodes: 129,
            rows: 2_555,
            within: 123,
            early: 6,
            at_edge: 0,
            other: 0,
        },
    );
}
