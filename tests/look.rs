//! vis-002 Phase 6 exit gates 5, 6, 7, 8 and 9 (`specs/city_spec.md`): the engine
//! dashboard's markings built from a synthetic crossing with every arrow type and from both
//! fixtures, checked against `NetworkJson`, counted independently from the config, against
//! the run's stopped boxes, drawn through the GPU, and coloured and faded. The headless ones
//! without a fixture run in plain `cargo test`. Those that need the fixtures of
//! `scripts/fixture.sh` and `scripts/fixture.sh midtown`, or the GPU, are ignored; run
//! everything with
//! `cargo test --release --test look -- --include-ignored --test-threads=1 --nocapture`.
//! Gate 10 is `scripts/gates-look.sh`.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use assimilator_config::network::{
    ControlType, MovementPriority, MovementType, NetworkConfig, NodeType, UnsignalizedRule,
};
use assimilator_config::types::{LaneIdx, LinkId};
use assimilator_geometry::network_json::NetworkJson;
use assimilator_video::place::Placement;
use assimilator_video::render::Renderer;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::scene;
use assimilator_video::streets::{
    self, COLOURS, DEAD_END_BAR, DEAD_END_BAR_OPACITY, Kind, LANE_MARKING, LANE_MARKING_OPACITY,
    MEDIAN_LINE, MEDIAN_LINE_OPACITY, Marking, ROAD_ARROW, ROAD_ARROW_OPACITY, STOP_LINE,
    STOP_LINE_OPACITY, Streets, colour, fade,
};

type P = [f64; 2];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
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

fn open(ring: &[P]) -> Vec<P> {
    let mut v = ring.to_vec();
    while v.len() > 1 && v.first() == v.last() {
        v.pop();
    }
    v
}

/// `(min, max)` of `f` over a polygon.
fn range(poly: &[P], f: impl Fn(P) -> f64) -> (f64, f64) {
    poly.iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &p| {
            (a.min(f(p)), b.max(f(p)))
        })
}

fn xr(m: &Marking) -> (f64, f64) {
    range(&m.polygon, |p| p[0])
}

fn yr(m: &Marking) -> (f64, f64) {
    range(&m.polygon, |p| p[1])
}

fn spans(r: (f64, f64), a: f64, b: f64) -> bool {
    close(r.0, a) && close(r.1, b)
}

/// Each marking traced back to `NetworkJson`, field by field in its order (§2.18.2): every
/// drawn vertex lies on its ring's boundary (a ring vertex, a cut point or a split point),
/// a cut ring's quads sum to its area, and an arrow's outlines are its glyph placed. Also
/// each marking's triangles sum to its shoelace area. Returns the worst vertex distance,
/// the worst relative area difference, and how many full-width rings were left whole.
fn trace(pl: &Placement, st: &Streets, nj: &NetworkJson) -> (f64, f64, usize) {
    let mut ms = st.markings.iter();
    let (mut worst, mut area_worst, mut whole) = (0.0f64, 0.0f64, 0);
    let mut on_ring = |m: &Marking, ring: &[P]| {
        for &v in &m.polygon {
            worst = worst.max(ring_dist(v, ring));
        }
    };
    let mut lines: Vec<(Kind, &Vec<P>)> = Vec::new();
    lines.extend(nj.median_lines.iter().map(|m| (Kind::Centre, &m.coords)));
    lines.extend(
        nj.lane_marking_dashes
            .iter()
            .map(|m| (Kind::Dash, &m.coords)),
    );
    lines.extend(nj.solid_lane_lines.iter().map(|m| (Kind::Solid, &m.coords)));
    for (kind, c) in lines {
        let m = ms.next().expect("a marking for each ring");
        assert_eq!(m.kind, kind);
        let ring = open(c);
        on_ring(m, &ring);
        assert!(
            ring.iter().all(|v| m.polygon.contains(v)),
            "every ring vertex kept"
        );
    }
    for s in &nj.stop_lines {
        let ring = open(&s.coords);
        let n = match (&s.link_id, s.lane) {
            (Some(_), _) => 1,
            (None, _) => {
                let first = ms.clone().next().expect("a marking for the ring");
                match first.link {
                    Some(i) => {
                        assert_eq!(first.lane, Some(0), "a cut ring starts at lane 0");
                        pl.network.links[i].lanes.len()
                    }
                    None => {
                        whole += 1;
                        1
                    }
                }
            }
        };
        let mut sum = 0.0;
        for _ in 0..n {
            let m = ms.next().expect("a marking for each stop ring");
            let want = if s.link_id.is_some() && s.lane.is_none() {
                Kind::Connector
            } else {
                Kind::Stop
            };
            assert_eq!(m.kind, want);
            on_ring(m, &ring);
            sum += shoelace(&m.polygon);
        }
        let a = shoelace(&ring);
        area_worst = area_worst.max((sum - a).abs() / a);
    }
    for a in &nj.lane_arrows {
        let (white, bar) = streets::glyph(&a.arrow_type);
        for (kind, outline) in white
            .iter()
            .map(|o| (Kind::Arrow, o))
            .chain(bar.iter().map(|o| (Kind::DeadEnd, o)))
        {
            let m = ms.next().expect("a marking for each outline");
            assert_eq!(m.kind, kind);
            assert_eq!(m.arrow.as_deref(), Some(a.arrow_type.as_str()));
            assert_eq!(m.lane, Some(a.lane));
            let placed: Vec<P> = outline
                .iter()
                .map(|&p| streets::place([a.x, a.y], a.heading, p))
                .collect();
            assert_eq!(m.polygon, placed, "the glyph placed, exactly");
        }
    }
    assert!(ms.next().is_none(), "no marking left over");
    for m in &st.markings {
        let t: f64 = m
            .triangles
            .chunks(3)
            .map(|t| {
                tri_area(
                    m.polygon[t[0] as usize],
                    m.polygon[t[1] as usize],
                    m.polygon[t[2] as usize],
                )
            })
            .sum();
        let s = shoelace(&m.polygon);
        assert!(
            (t - s).abs() <= 1e-9 * s.max(1.0),
            "triangles {t} against {s}"
        );
    }
    (worst, area_worst, whole)
}

// ── Gate 5: a synthetic crossing with every arrow type ──────────────────────

const SYNTHETIC: &str = r#"
schema_version: 1
metadata: {name: arrows synthetic, coordinate_system: metric, z_enabled: false}
nodes:
  - {id: C, point: [0, 0], type: junction}
  - {id: W, point: [-60, 0], type: endpoint}
  - {id: E, point: [60, 0], type: endpoint}
  - {id: N, point: [0, 60], type: endpoint}
  - {id: S, point: [0, -60], type: endpoint}
links:
  - {id: L_WC, from_node: W, to_node: C, geometry: [[-60, 0], [0, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}, {id: 2, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CW, from_node: C, to_node: W, geometry: [[0, 0], [-60, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_EC, from_node: E, to_node: C, geometry: [[60, 0], [0, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}, {id: 2, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CE, from_node: C, to_node: E, geometry: [[0, 0], [60, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_NC, from_node: N, to_node: C, geometry: [[0, 60], [0, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CN, from_node: C, to_node: N, geometry: [[0, 0], [0, 60]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_SC, from_node: S, to_node: C, geometry: [[0, -60], [0, 0]], median_gap: 4, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CS, from_node: C, to_node: S, geometry: [[0, 0], [0, -60]], median_gap: 4, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
junctions:
  - node_id: C
    control: signal
    geometry: {setback: 3}
    stop_line_offsets: {L_EC: {'0': 5}}
    movements:
      - {id: W_U, from_link: L_WC, to_link: L_CW, from_lanes: [0], to_lanes: [0], type: u-turn}
      - {id: W_L, from_link: L_WC, to_link: L_CN, from_lanes: [1], to_lanes: [0], type: left}
      - {id: W_T, from_link: L_WC, to_link: L_CE, from_lanes: [1, 2], to_lanes: [0, 1], type: through}
      - {id: W_R, from_link: L_WC, to_link: L_CS, from_lanes: [2], to_lanes: [1], type: right}
      - {id: E_L, from_link: L_EC, to_link: L_CS, from_lanes: [0], to_lanes: [0], type: left}
      - {id: E_U, from_link: L_EC, to_link: L_CE, from_lanes: [0], to_lanes: [0], type: u-turn}
      - {id: E_T, from_link: L_EC, to_link: L_CW, from_lanes: [1], to_lanes: [1], type: through}
      - {id: E_R, from_link: L_EC, to_link: L_CN, from_lanes: [2], to_lanes: [1], type: right}
      - {id: N_L, from_link: L_NC, to_link: L_CE, from_lanes: [0], to_lanes: [0], type: left}
      - {id: N_T, from_link: L_NC, to_link: L_CS, from_lanes: [0], to_lanes: [0], type: through}
      - {id: N_R, from_link: L_NC, to_link: L_CW, from_lanes: [0], to_lanes: [1], type: right}
      - {id: S_L, from_link: L_SC, to_link: L_CW, from_lanes: [0], to_lanes: [0], type: left}
      - {id: S_R, from_link: L_SC, to_link: L_CE, from_lanes: [0], to_lanes: [1], type: right}
"#;

pub fn synthetic() -> Placement {
    let cfg: NetworkConfig = serde_yaml::from_str(SYNTHETIC).expect("the synthetic network");
    Placement::new(cfg)
}

/// Counts, vertices and triangles of the markings of one kind.
fn mesh_of(st: &Streets, kind: Kind) -> (usize, usize, usize) {
    st.markings
        .iter()
        .filter(|m| m.kind == kind)
        .fold((0, 0, 0), |(n, v, t), m| {
            (n + 1, v + m.polygon.len(), t + m.triangles.len() / 3)
        })
}

#[test]
fn gate5_synthetic_arrows() {
    let pl = synthetic();
    let st = Streets::build(&pl);
    let nj = NetworkJson::from_config_with_network_data(&pl.network, &pl.data);
    let of = |k: Kind| st.markings.iter().filter(move |m| m.kind == k);

    // The surfaces, pairs and fills stay Phase 5's: two fills, on the 0.5 m and 4 m pairs.
    assert_eq!(st.junctions.len(), 1);
    let pairs: Vec<_> = st.pairs.iter().map(|p| (p.a, p.b, p.gap)).collect();
    assert_eq!(
        pairs,
        vec![(0, 1, 0.0), (2, 3, 0.5), (4, 5, 0.0), (6, 7, 4.0)]
    );
    assert_eq!(
        st.fills.iter().map(|f| f.link).collect::<Vec<_>>(),
        vec![2, 6]
    );

    // Two centre lines, on the two 0 m pairs.
    let centre: Vec<_> = of(Kind::Centre).collect();
    assert_eq!(centre.len(), 2);
    assert!(spans(yr(centre[0]), -0.075, 0.075) && spans(xr(centre[0]), -60.0, -12.0));
    assert!(spans(xr(centre[1]), -0.075, 0.075) && spans(yr(centre[1]), 13.75, 60.0));
    for m in &centre {
        assert_eq!((m.link, m.lane, m.arrow.as_deref()), (None, None, None));
    }

    // 68 dashes, 2.5 × 0.15 m; L_WC's first boundary at y −3.5 from x −60 every 6.5 m.
    let dashes: Vec<_> = of(Kind::Dash).collect();
    assert_eq!(dashes.len(), 68);
    for m in &dashes {
        let (x, y) = (xr(m), yr(m));
        let (dx, dy) = (x.1 - x.0, y.1 - y.0);
        assert!(
            (close(dx, 2.5) && close(dy, 0.15)) || (close(dx, 0.15) && close(dy, 2.5)),
            "dash {dx} × {dy}"
        );
        assert_eq!((m.link, m.lane), (None, None));
    }
    let first: Vec<(f64, f64)> = dashes
        .iter()
        .filter(|m| spans(yr(m), -3.575, -3.425))
        .map(|m| xr(m))
        .collect();
    let want: Vec<(f64, f64)> = (0..8)
        .map(|k| (-60.0 + 6.5 * k as f64, -57.5 + 6.5 * k as f64))
        .collect();
    assert_eq!(first.len(), 8);
    for (g, w) in first.iter().zip(&want) {
        assert!(spans(*g, w.0, w.1), "dash {g:?} against {w:?}");
    }

    // Stop lines, 0.40 m deep, one per (link, lane); one connector.
    let stops: Vec<_> = of(Kind::Stop).collect();
    let got: Vec<(Option<usize>, Option<u32>)> = stops.iter().map(|m| (m.link, m.lane)).collect();
    let want: Vec<(Option<usize>, Option<u32>)> = [
        (0, 0),
        (0, 1),
        (0, 2),
        (2, 0),
        (2, 1),
        (2, 2),
        (4, 0),
        (4, 1),
        (6, 0),
    ]
    .iter()
    .map(|&(l, k)| (Some(l), Some(k)))
    .collect();
    assert_eq!(got, want);
    // L_WC's ring, x −12.4 … −12.0, y −10.5 … 0, cut into 3: lane 0 the leftmost.
    for (i, y) in [(0, (-3.5, 0.0)), (1, (-7.0, -3.5)), (2, (-10.5, -7.0))] {
        assert!(
            spans(xr(stops[i]), -12.4, -12.0) && spans(yr(stops[i]), y.0, y.1),
            "L_WC lane {i}"
        );
    }
    assert!(spans(xr(stops[3]), 15.0, 15.4), "L_EC lane 0, 5 m back");
    assert!(
        spans(xr(stops[4]), 10.0, 10.4) && spans(xr(stops[5]), 10.0, 10.4),
        "L_EC lanes 1, 2"
    );
    // L_NC's ring, y 13.75 … 14.15, cut into 2.
    for (i, x) in [(6, (-3.5, 0.0)), (7, (-7.0, -3.5))] {
        assert!(
            spans(yr(stops[i]), 13.75, 14.15) && spans(xr(stops[i]), x.0, x.1),
            "L_NC stop {i}"
        );
    }
    assert!(
        spans(yr(stops[8]), -13.9, -13.5) && spans(xr(stops[8]), 2.0, 5.5),
        "L_SC"
    );
    let conn: Vec<_> = of(Kind::Connector).collect();
    assert_eq!(conn.len(), 1);
    assert_eq!((conn[0].link, conn[0].lane), (Some(2), None));
    assert!(
        spans(yr(conn[0]), 3.7, 3.8) && spans(xr(conn[0]), 10.0, 15.4),
        "the connector"
    );

    // 35 arrows, as `NetworkJson` places them, each glyph its type's.
    let arrows: Vec<(&str, u32, &str, f64, f64, f64)> = nj
        .lane_arrows
        .iter()
        .map(|a| {
            (
                a.link_id.as_str(),
                a.lane,
                a.arrow_type.as_str(),
                a.x,
                a.y,
                a.heading,
            )
        })
        .collect();
    #[rustfmt::skip]
    let want: [(&str, u32, &str, f64, f64, f64); 35] = [
        ("L_WC", 0, "u_turn", -16.0, -1.75, 0.0), ("L_WC", 0, "through", -56.0, -1.75, 0.0),
        ("L_WC", 1, "through_left", -16.0, -5.25, 0.0), ("L_WC", 1, "through", -56.0, -5.25, 0.0),
        ("L_WC", 2, "through_right", -16.0, -8.75, 0.0), ("L_WC", 2, "through", -56.0, -8.75, 0.0),
        ("L_CW", 0, "through", -56.0, 1.75, 180.0), ("L_CW", 0, "through", -16.0, 1.75, 180.0),
        ("L_CW", 1, "through", -56.0, 5.25, 180.0), ("L_CW", 1, "through", -16.0, 5.25, 180.0),
        ("L_EC", 0, "left", 19.0, 2.0, 180.0), ("L_EC", 0, "u_turn", 24.0, 2.0, 180.0),
        ("L_EC", 0, "through", 56.0, 2.0, 180.0),
        ("L_EC", 1, "through", 14.0, 5.5, 180.0), ("L_EC", 1, "through", 56.0, 5.5, 180.0),
        ("L_EC", 2, "right", 14.0, 9.0, 180.0), ("L_EC", 2, "through", 56.0, 9.0, 180.0),
        ("L_CE", 0, "through", 56.0, -2.0, 0.0), ("L_CE", 0, "through", 14.0, -2.0, 0.0),
        ("L_CE", 1, "through", 56.0, -5.5, 0.0), ("L_CE", 1, "through", 14.0, -5.5, 0.0),
        ("L_NC", 0, "through_left_right", -1.75, 17.75, -90.0), ("L_NC", 0, "through", -1.75, 56.0, -90.0),
        ("L_NC", 1, "dead_end", -5.25, 17.75, -90.0), ("L_NC", 1, "through", -5.25, 56.0, -90.0),
        ("L_CN", 0, "through", 1.75, 56.0, 90.0), ("L_CN", 0, "through", 1.75, 17.75, 90.0),
        ("L_CN", 1, "through", 5.25, 56.0, 90.0), ("L_CN", 1, "through", 5.25, 17.75, 90.0),
        ("L_SC", 0, "left_right", 3.75, -17.5, 90.0), ("L_SC", 0, "through", 3.75, -56.0, 90.0),
        ("L_CS", 0, "through", -3.75, -56.0, -90.0), ("L_CS", 0, "through", -3.75, -17.5, -90.0),
        ("L_CS", 1, "through", -7.25, -56.0, -90.0), ("L_CS", 1, "through", -7.25, -17.5, -90.0),
    ];
    assert_eq!(arrows.len(), 35);
    for (g, w) in arrows.iter().zip(&want) {
        assert!(
            (g.0, g.1, g.2) == (w.0, w.1, w.2)
                && (g.3 - w.3).abs() <= 1e-9
                && (g.4 - w.4).abs() <= 1e-9
                && (g.5 - w.5).abs() <= 1e-9,
            "arrow {g:?} against {w:?}"
        );
    }
    let mut types: Vec<(&str, usize)> = Vec::new();
    for t in [
        "through",
        "through_left",
        "through_right",
        "left",
        "right",
        "left_right",
        "through_left_right",
        "u_turn",
        "dead_end",
    ] {
        types.push((t, arrows.iter().filter(|a| a.2 == t).count()));
    }
    assert_eq!(
        types,
        vec![
            ("through", 26),
            ("through_left", 1),
            ("through_right", 1),
            ("left", 1),
            ("right", 1),
            ("left_right", 1),
            ("through_left_right", 1),
            ("u_turn", 2),
            ("dead_end", 1)
        ]
    );
    // Each outline carries its arrow's link (an index) and lane.
    let index: HashMap<&str, usize> = pl
        .network
        .links
        .iter()
        .enumerate()
        .map(|(i, l)| (l.id.0.as_str(), i))
        .collect();
    let mut outlines = st
        .markings
        .iter()
        .filter(|m| matches!(m.kind, Kind::Arrow | Kind::DeadEnd));
    for a in &nj.lane_arrows {
        let (white, bar) = streets::glyph(&a.arrow_type);
        for _ in 0..white.len() + bar.iter().count() {
            let m = outlines.next().unwrap();
            assert_eq!(
                (m.link, m.lane),
                (Some(index[a.link_id.as_str()]), Some(a.lane))
            );
        }
    }

    // The mesh.
    let kinds = [
        (Kind::Centre, (2, 194, 190)),
        (Kind::Dash, (68, 272, 136)),
        (Kind::Solid, (0, 0, 0)),
        (Kind::Stop, (9, 36, 18)),
        (Kind::Connector, (1, 4, 2)),
        (Kind::Arrow, (40, 513, 433)),
        (Kind::DeadEnd, (1, 4, 2)),
    ];
    for (k, want) in kinds {
        assert_eq!(mesh_of(&st, k), want, "{k:?}");
    }
    let md = st.markings();
    assert_eq!((md.positions.len(), md.indices.len() / 3), (1_023, 781));
    assert_eq!((md.colours.len(), md.widths.len()), (1_023, 1_023));

    // Every vertex back to `NetworkJson`.
    let (worst, area, whole) = trace(&pl, &st, &nj);
    assert!(worst <= 1e-9, "a vertex {worst} m off its ring");
    assert!(area <= 1e-9, "cut areas {area}");
    assert_eq!(whole, 0);
    println!(
        "gate 5: 2 centre lines, 68 dashes, 9 stop lines, 1 connector, 35 arrows (every type), mesh {} vertices {} triangles; vertices on their rings within {worst:.1e} m, cut areas within {area:.1e}: PASS",
        md.positions.len(),
        md.indices.len() / 3
    );
}

// ── Gate 9: the colours and the fade ─────────────────────────────────────────

fn linear(c: u8) -> f32 {
    let c = c as f64 / 255.0;
    (if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }) as f32
}

/// The canvas's compositing, in sRGB, rounded once (§2.18.4).
fn over(c: [u8; 3], a: f64, under: [u8; 3]) -> [u8; 3] {
    let m = |i: usize| (a * c[i] as f64 + (1.0 - a) * under[i] as f64).round() as u8;
    [m(0), m(1), m(2)]
}

#[test]
fn gate9_colours_and_fade() {
    let road = scene::ROAD;
    let table = [
        (
            Kind::Dash,
            LANE_MARKING,
            LANE_MARKING_OPACITY,
            [174, 176, 180],
            0.15,
        ),
        (
            Kind::Solid,
            LANE_MARKING,
            LANE_MARKING_OPACITY,
            [174, 176, 180],
            0.15,
        ),
        (
            Kind::Centre,
            MEDIAN_LINE,
            MEDIAN_LINE_OPACITY,
            [199, 167, 77],
            0.15,
        ),
        (
            Kind::Stop,
            STOP_LINE,
            STOP_LINE_OPACITY,
            [239, 239, 240],
            0.40,
        ),
        (
            Kind::Connector,
            STOP_LINE,
            STOP_LINE_OPACITY,
            [239, 239, 240],
            0.10,
        ),
        (
            Kind::Arrow,
            ROAD_ARROW,
            ROAD_ARROW_OPACITY,
            [190, 191, 195],
            0.30,
        ),
        (
            Kind::DeadEnd,
            DEAD_END_BAR,
            DEAD_END_BAR_OPACITY,
            [217, 72, 73],
            0.40,
        ),
    ];
    for (k, hex, a, want, w) in table {
        assert_eq!(over(hex, a, road), want, "{k:?} composited");
        assert_eq!(k.colour(), want, "{k:?}'s colour");
        assert_eq!(k.width(), w, "{k:?}'s fade width");
    }
    for c in COLOURS {
        let (c0, c1) = (colour(c, 0.0), colour(c, 1.0));
        for i in 0..3 {
            assert_eq!(c0[i], linear(road[i]), "α 0 is the road");
            assert_eq!(c1[i], linear(c[i]), "α 1 is the marking");
        }
        assert_eq!((c0[3], c1[3]), (1.0, 1.0));
    }
    let f = [
        ("stop, Midtown ortho", fade(0.40, 1.6009), 0.316),
        ("arrow, Midtown ortho", fade(0.30, 1.6009), 0.122),
        ("stop, view at launch", fade(0.40, 2.401), 0.074),
        ("arrow, urban_grid ortho", fade(0.30, 1.1481), 0.357),
    ];
    for (what, got, want) in f {
        assert!((got - want).abs() <= 0.001, "{what}: {got}");
    }
    println!(
        "gate 9: colours as §2.18.4; colour's ends exact; widths 0.15, 0.40, 0.30, 0.40, 0.10; fade {:.4}, {:.4}, {:.4}, {:.4}: PASS",
        f[0].1, f[1].1, f[2].1, f[3].1
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

fn arc(pts: &[P]) -> f64 {
    pts.windows(2)
        .map(|w| ((w[1][0] - w[0][0]).powi(2) + (w[1][1] - w[0][1]).powi(2)).sqrt())
        .sum()
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
    fn cell(p: P) -> (i64, i64) {
        (
            (p[0] / Self::CELL).floor() as i64,
            (p[1] / Self::CELL).floor() as i64,
        )
    }
    fn new(tris: Vec<[P; 3]>) -> Grid {
        let mut cells: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
        for (i, t) in tris.iter().enumerate() {
            let (x0, x1) = range(t, |p| p[0]);
            let (y0, y1) = range(t, |p| p[1]);
            let (a, b) = (Self::cell([x0, y0]), Self::cell([x1, y1]));
            for gx in a.0..=b.0 {
                for gy in a.1..=b.1 {
                    cells.entry((gx, gy)).or_default().push(i);
                }
            }
        }
        Grid { tris, cells }
    }
    /// 0 inside a triangle, else the distance to the nearest triangle edge in the cells
    /// around `p`.
    fn dist(&self, p: P) -> f64 {
        let k = Self::cell(p);
        let mut best = f64::INFINITY;
        for dx in -1..=1 {
            for dy in -1..=1 {
                for &i in self.cells.get(&(k.0 + dx, k.1 + dy)).into_iter().flatten() {
                    let t = self.tris[i];
                    if in_tri(p, t[0], t[1], t[2]) {
                        return 0.0;
                    }
                    for j in 0..3 {
                        best = best.min(seg_dist(p, t[j], t[(j + 1) % 3]));
                    }
                }
            }
        }
        best
    }
}

const TYPES: [&str; 9] = [
    "through",
    "through_left",
    "through_right",
    "left",
    "right",
    "left_right",
    "through_left_right",
    "u_turn",
    "dead_end",
];

/// The arrow types `NetworkJson` should emit, counted independently from the config: one
/// arriving arrow per lane of a link at least 8 m long (its offset polyline,
/// `NetworkJson::links[].coords`), typed by the lane's movements, a separate u-turn where
/// it shares the lane with others, and a departing `through` on a link of 25 m or more.
fn arrow_types(pl: &Placement, nj: &NetworkJson) -> Vec<String> {
    let net = &pl.network;
    let waypoint: HashSet<&str> = net
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Waypoint))
        .map(|n| n.id.0.as_str())
        .collect();
    let junction: HashSet<&str> = net
        .junctions
        .iter()
        .map(|j| j.node_id.0.as_str())
        .filter(|n| !waypoint.contains(n))
        .collect();
    let mut moves: HashMap<(&str, u32), Vec<MovementType>> = HashMap::new();
    for j in &net.junctions {
        for m in &j.movements {
            for fl in &m.from_lanes {
                moves
                    .entry((m.from_link.0.as_str(), fl.0))
                    .or_default()
                    .push(m.movement_type);
            }
        }
    }
    let mut out = Vec::new();
    for (l, ld) in net.links.iter().zip(&nj.links) {
        assert_eq!(l.id.0, ld.id, "NetworkJson's links in link order");
        let len = arc(&ld.coords);
        if len < 8.0 {
            continue;
        }
        for k in 0..l.lanes.len() as u32 {
            let (main, separate) = match moves.get(&(l.id.0.as_str(), k)) {
                Some(t) => {
                    let has = |x: MovementType| t.contains(&x);
                    let (th, le, ri, u) = (
                        has(MovementType::Through),
                        has(MovementType::Left),
                        has(MovementType::Right),
                        has(MovementType::UTurn),
                    );
                    let main = match (th, le, ri) {
                        (true, false, false) => "through",
                        (false, true, false) => "left",
                        (false, false, true) => "right",
                        (true, true, false) => "through_left",
                        (true, false, true) => "through_right",
                        (false, true, true) => "left_right",
                        (true, true, true) => "through_left_right",
                        (false, false, false) if u => "u_turn",
                        _ => "through",
                    };
                    (main, u && (th || le || ri))
                }
                None if junction.contains(l.to_node.0.as_str()) => ("dead_end", false),
                None => ("through", false),
            };
            out.push(main.to_string());
            if separate {
                out.push("u_turn".to_string());
            }
            if len >= 25.0 {
                out.push("through".to_string());
            }
        }
    }
    out
}

/// The stop lines `NetworkJson` should emit, counted independently by the engine's rule
/// (`network_json.rs` l. 1553–1749 at the pin): rings full-width / per lane / connector,
/// and per approach link the lanes it marks.
struct StopCount {
    full: usize,
    per_lane: usize,
    connectors: usize,
    /// (link index, lanes marked), in link order.
    marked: Vec<(usize, usize)>,
}

fn stop_count(pl: &Placement, nj: &NetworkJson) -> StopCount {
    let (net, nd) = (&pl.network, &pl.data);
    let waypoint: HashSet<&str> = net
        .nodes
        .iter()
        .filter(|n| matches!(n.node_type, NodeType::Waypoint))
        .map(|n| n.id.0.as_str())
        .collect();
    // A lane at an unsignalised junction is marked if any of its lane paths is not
    // transparent.
    let mut needs: HashMap<(&str, u32), bool> = HashMap::new();
    for j in net
        .junctions
        .iter()
        .filter(|j| j.control != ControlType::Signal)
    {
        for m in &j.movements {
            for &fl in &m.from_lanes {
                let e = needs.entry((m.from_link.0.as_str(), fl.0)).or_insert(false);
                *e |= !nd.is_transparent_lane_path(&j.node_id, &m.id, fl);
            }
        }
    }
    let mut c = StopCount {
        full: 0,
        per_lane: 0,
        connectors: 0,
        marked: Vec::new(),
    };
    for (i, (l, ld)) in net.links.iter().zip(&nj.links).enumerate() {
        let Some(j) = net
            .junctions
            .iter()
            .find(|j| j.node_id == l.to_node && !waypoint.contains(j.node_id.0.as_str()))
        else {
            continue;
        };
        let from_l = j.movements.iter().filter(|m| m.from_link == l.id);
        let approach = match j.control {
            ControlType::Signal => true,
            ControlType::Unsignalized => match j.rule {
                Some(UnsignalizedRule::Priority) => from_l
                    .clone()
                    .any(|m| m.priority == MovementPriority::Minor),
                _ => from_l.clone().any(|m| {
                    m.from_lanes
                        .iter()
                        .any(|&fl| !nd.is_transparent_lane_path(&j.node_id, &m.id, fl))
                }),
            },
        };
        let len = arc(&ld.coords);
        if !approach || len < 1.0 {
            continue;
        }
        let n = l.lanes.len();
        let visible: Vec<bool> = (0..n as u32)
            .map(|k| {
                j.control == ControlType::Signal
                    || needs.get(&(l.id.0.as_str(), k)).copied().unwrap_or(false)
            })
            .collect();
        let shown = visible.iter().filter(|v| **v).count();
        if shown == 0 {
            continue;
        }
        let id = LinkId(l.id.0.clone());
        let delta = (0..n as u32).any(|k| {
            nd.lane_stop_line_deltas
                .get(&(id.clone(), LaneIdx(k)))
                .is_some_and(|d| d.abs() > f64::EPSILON)
        });
        let gap = l.lanes.iter().any(|x| x.gap_after > 0.0);
        if shown == n && !delta && !gap {
            if len - nd.stop_line_offset(&id) - 0.20 < 0.0 {
                continue;
            }
            c.full += 1;
            c.marked.push((i, n));
            continue;
        }
        let centres: Vec<Option<f64>> = (0..n)
            .map(|k| {
                let s = len - nd.lane_stop_line_offset(&id, LaneIdx(k as u32)) - 0.20;
                (visible[k] && s >= 0.0).then_some(s)
            })
            .collect();
        let marked = centres.iter().flatten().count();
        c.per_lane += marked;
        if marked > 0 {
            c.marked.push((i, marked));
        }
        for k in 0..n.saturating_sub(1) {
            if l.lanes[k].gap_after > 0.0 {
                continue;
            }
            if let (Some(a), Some(b)) = (centres[k], centres[k + 1])
                && (a - b).abs() >= 0.05
            {
                c.connectors += 1;
            }
        }
    }
    c
}

struct Expect6 {
    centre: usize,
    dashes: usize,
    solid: usize,
    rings: (usize, usize, usize),
    /// (link, lane) stop lines at a signal / elsewhere; their approach links likewise.
    stops: (usize, usize),
    stop_links: (usize, usize),
    arrows: usize,
    types: [usize; 9],
    mesh: (usize, usize),
    road_worst: f64,
}

fn gate6(dir: &str, e: Expect6) {
    let run = load(dir);
    let pl = &run.placement;
    let net = &pl.network;
    let st = Streets::build(pl);
    let nj = NetworkJson::from_config_with_network_data(net, &pl.data);
    let signal: HashSet<&str> = net
        .junctions
        .iter()
        .filter(|j| j.control == ControlType::Signal)
        .map(|j| j.node_id.0.as_str())
        .collect();
    let at_signal = |i: usize| signal.contains(net.links[i].to_node.0.as_str());
    let count = |k: Kind| st.markings.iter().filter(|m| m.kind == k).count();

    // The rings as `NetworkJson` gives them, and independently.
    let rings = (
        nj.stop_lines.iter().filter(|s| s.link_id.is_none()).count(),
        nj.stop_lines.iter().filter(|s| s.lane.is_some()).count(),
        nj.stop_lines
            .iter()
            .filter(|s| s.link_id.is_some() && s.lane.is_none())
            .count(),
    );
    let ind = stop_count(pl, &nj);
    // The (link, lane) stop lines drawn, by approach.
    let mut drawn: Vec<(usize, usize)> = Vec::new();
    for m in st.markings.iter().filter(|m| m.kind == Kind::Stop) {
        let i = m.link.expect("every stop line has its link");
        assert!(m.lane.is_some(), "every stop line has its lane");
        match drawn.last_mut() {
            Some((l, n)) if *l == i => *n += 1,
            _ => drawn.push((i, 1)),
        }
    }
    let split = |v: &[(usize, usize)]| {
        let (mut s, mut o) = ((0, 0), (0, 0));
        for &(i, n) in v {
            if at_signal(i) {
                s = (s.0 + n, s.1 + 1);
            } else {
                o = (o.0 + n, o.1 + 1);
            }
        }
        ((s.0, o.0), (s.1, o.1))
    };
    let (stops, stop_links) = split(&drawn);

    // Arrows by type, as given and independently.
    let given: Vec<&str> = nj
        .lane_arrows
        .iter()
        .map(|a| a.arrow_type.as_str())
        .collect();
    let ind_types = arrow_types(pl, &nj);
    let by_type = |v: &[&str]| TYPES.map(|t| v.iter().filter(|x| **x == t).count());
    let types = by_type(&given);
    let ind_by_type = by_type(&ind_types.iter().map(String::as_str).collect::<Vec<_>>());
    assert!(
        given.iter().all(|t| TYPES.contains(t)),
        "an arrow type outside the nine"
    );
    let md = st.markings();
    println!(
        "gate 6 {dir}: centre lines {} dashes {} solid {}; rings {}/{}/{} (independently {}/{}/{}); stop lines at a signal {} elsewhere {} (links {} / {}); arrows {} (independently {}); by type {types:?} (independently {ind_by_type:?}); markings mesh {} vertices {} triangles",
        count(Kind::Centre),
        count(Kind::Dash),
        count(Kind::Solid),
        rings.0,
        rings.1,
        rings.2,
        ind.full,
        ind.per_lane,
        ind.connectors,
        stops.0,
        stops.1,
        stop_links.0,
        stop_links.1,
        given.len(),
        ind_types.len(),
        md.positions.len(),
        md.indices.len() / 3
    );
    assert_eq!(
        rings,
        (ind.full, ind.per_lane, ind.connectors),
        "rings, independent"
    );
    assert_eq!(drawn, ind.marked, "stop lines per approach, independent");
    assert_eq!(ind_types, given, "arrow types in order, independent");
    assert_eq!(
        (count(Kind::Centre), count(Kind::Dash), count(Kind::Solid)),
        (e.centre, e.dashes, e.solid)
    );
    assert_eq!(rings, e.rings);
    assert_eq!((stops, stop_links), (e.stops, e.stop_links));
    assert_eq!((given.len(), types), (e.arrows, e.types));
    assert_eq!((md.positions.len(), md.indices.len() / 3), e.mesh);
    assert_eq!(md.colours.len(), md.positions.len());
    assert_eq!(md.widths.len(), md.positions.len());

    // Each full-width ring's link: exactly one link end within 1 mm, at worst how far.
    let ends: Vec<P> = net
        .links
        .iter()
        .map(|l| {
            let off = pl.data.stop_line_offset(&LinkId(l.id.0.clone()));
            let p = pl
                .place_lateral(&l.id.0, pl.link_length(&l.id.0) - off, 0.0)
                .unwrap();
            [p.x, p.y]
        })
        .collect();
    let (mut match_worst, mut ambiguous, mut unmatched) = (0.0f64, 0, 0);
    for s in nj.stop_lines.iter().filter(|s| s.link_id.is_none()) {
        let c = &s.coords;
        let mid = [(c[1][0] + c[2][0]) / 2.0, (c[1][1] + c[2][1]) / 2.0];
        let d: Vec<f64> = ends
            .iter()
            .map(|e| ((e[0] - mid[0]).powi(2) + (e[1] - mid[1]).powi(2)).sqrt())
            .collect();
        let within = d.iter().filter(|x| **x <= 0.001).count();
        match within {
            0 => unmatched += 1,
            1 => {}
            _ => ambiguous += 1,
        }
        match_worst = match_worst.max(d.iter().copied().fold(f64::INFINITY, f64::min));
    }

    // Every vertex back to `NetworkJson`.
    let (worst, area, whole) = trace(pl, &st, &nj);

    // Every marking vertex on the drawn road: the strips, the junction surfaces, the fills.
    let mut road = Vec::new();
    for s in &run.strips {
        for i in 0..s.left.len() - 1 {
            road.push([s.left[i], s.right[i], s.left[i + 1]]);
            road.push([s.right[i], s.right[i + 1], s.left[i + 1]]);
        }
    }
    let sd = st.surface();
    for t in sd.indices.chunks(3) {
        let q = |i: u32| [sd.positions[i as usize][0], sd.positions[i as usize][1]];
        road.push([q(t[0]), q(t[1]), q(t[2])]);
    }
    let g = Grid::new(road);
    let mut by_kind: Vec<(Kind, usize, usize, f64)> = Vec::new();
    for k in Kind::ALL {
        let (mut n, mut off, mut w) = (0, 0, 0.0f64);
        for m in st.markings.iter().filter(|m| m.kind == k) {
            for &v in &m.polygon {
                let d = g.dist(v);
                n += 1;
                off += (d > 0.0) as usize;
                w = w.max(d);
            }
        }
        by_kind.push((k, n, off, w));
    }
    let road_worst = by_kind.iter().map(|x| x.3).fold(0.0, f64::max);
    let twice = Streets::build(pl) == st;
    println!(
        "gate 6 {dir}: full-width rings matched worst {match_worst:.6} m, unmatched {unmatched}, ambiguous {ambiguous}, left whole {whole}; vertices on their rings within {worst:.1e} m, cut areas within {area:.1e}; on the drawn road worst {road_worst:.4} m ({}); built twice equal {twice}",
        by_kind
            .iter()
            .filter(|x| x.1 > 0)
            .map(|(k, n, off, w)| format!("{k:?} {n} vertices, {off} off, worst {w:.4}"))
            .collect::<Vec<_>>()
            .join("; ")
    );
    assert!(match_worst <= 0.001 && unmatched == 0 && ambiguous == 0 && whole == 0);
    assert!(worst <= 1e-9 && area <= 1e-9);
    assert!(road_worst <= 0.01, "within 1 cm of the drawn road");
    assert!(
        (road_worst - e.road_worst).abs() < 0.00005,
        "worst {road_worst} against {}",
        e.road_worst
    );
    assert!(twice, "the streets built twice in one process are equal");
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_counts() {
    gate6(
        "midtown",
        Expect6 {
            centre: 3,
            dashes: 3_921,
            solid: 0,
            rings: (223, 4, 0),
            stops: (402, 69),
            stop_links: (188, 38),
            arrows: 1_050,
            types: [788, 111, 119, 13, 12, 2, 5, 0, 0],
            mesh: (33_662, 22_288),
            road_worst: 0.0063,
        },
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate6_urban_grid_counts() {
    gate6(
        "urban_grid",
        Expect6 {
            centre: 0,
            dashes: 2_044,
            solid: 0,
            rings: (35, 2, 1),
            stops: (71, 0),
            stop_links: (36, 0),
            arrows: 190,
            types: [121, 29, 30, 2, 2, 1, 5, 0, 0],
            mesh: (11_946, 7_194),
            road_worst: 0.0,
        },
    );
}

/// One set of first stopped boxes: episodes, rows, the median gap, those within 0.10 m of
/// +0.242, between 1.55 and 1.75 m, in the −0.4 m bin, and the rest.
#[derive(Debug, PartialEq)]
struct Expect7 {
    episodes: usize,
    rows: usize,
    median: f64,
    within: usize,
    early: usize,
    at_edge: usize,
    other: usize,
}

/// §2.18.7, by Phase 5's rule (its gate 7): a stopped box (speed under 0.1 m/s) first in
/// its lane, on a lane with a built stop line, its front's gap to the line's back edge,
/// 0.40 m from its junction-facing edge, within 5 m; an episode a vehicle's run of such
/// rows with no break over 1.5 s.
fn gate7(dir: &str, sig: Expect7, unsig: Option<Expect7>) {
    let run = load(dir);
    let pl = &run.placement;
    let net = &pl.network;
    let st = Streets::build(pl);
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
    // The back edge of the built stop line for each (link, lane), s along the link.
    let back: HashMap<(usize, u32), f64> = st
        .markings
        .iter()
        .filter(|m| m.kind == Kind::Stop)
        .map(|m| {
            let (li, lane) = (m.link.unwrap(), m.lane.unwrap());
            let id = &net.links[li].id.0;
            let s1 = pl.link_length(id)
                - pl.data
                    .lane_stop_line_offset(&LinkId(id.clone()), LaneIdx(lane));
            ((li, lane), s1 - Kind::Stop.width())
        })
        .collect();
    for (label, at_signal, e) in [("at signals", true, Some(sig)), ("elsewhere", false, unsig)] {
        let fcd = &run.fcd;
        let (mut rows, mut eps) = (0, Vec::new());
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
                let Some(b) = back.get(&(li, r.lane)) else {
                    continue;
                };
                if signal.contains(net.links[li].to_node.0.as_str()) != at_signal
                    || lead[&(r.link, r.lane)] > r.position
                {
                    continue;
                }
                let g = b - (r.position + r.length / 2.0);
                if g.abs() > 5.0 {
                    continue;
                }
                rows += 1;
                let prev = last.insert(r.vehicle_id, snap.time);
                if prev.is_none_or(|p| snap.time - p > 1.5) {
                    eps.push(g);
                }
            }
        }
        let Some(e) = e else {
            println!("gate 7 {dir} {label}: {} episodes ({rows} rows)", eps.len());
            assert_eq!((eps.len(), rows), (0, 0));
            continue;
        };
        eps.sort_by(f64::total_cmp);
        let median = eps[((eps.len() - 1) as f64 * 0.5).round() as usize];
        let within = eps.iter().filter(|g| (*g - 0.242).abs() <= 0.10).count();
        let early: Vec<f64> = eps
            .iter()
            .copied()
            .filter(|g| (1.55..=1.75).contains(g))
            .collect();
        // "The −0.4 m bin" is the probe's 0.1 m bin, `round(10·(g + 0.4)) == 0`: fronts at
        // the junction's edge, or within 5 cm of it. Each bin prints its exact count too.
        let at_edge = eps
            .iter()
            .filter(|g| ((*g + 0.4) * 10.0).round() == 0.0)
            .count();
        let exact = |c: f64| eps.iter().filter(|g| (*g - c).abs() <= 0.0005).count();
        let other = eps.len() - within - early.len() - at_edge;
        println!(
            "gate 7 {dir} {label}: first stopped boxes {} episodes ({rows} rows), median gap {median:+.4} m; within 0.10 m of +0.242 {within} ({} within 0.5 mm of it); 1.55–1.75 m {} (from {:+.4} to {:+.4}); in the −0.4 m bin {at_edge} ({} within 0.5 mm of −0.400); other {other}",
            eps.len(),
            exact(0.242),
            early.len(),
            early.first().copied().unwrap_or(f64::NAN),
            early.last().copied().unwrap_or(f64::NAN),
            exact(-0.4)
        );
        assert_eq!(
            (eps.len(), rows, within, early.len(), at_edge, other),
            (e.episodes, e.rows, e.within, e.early, e.at_edge, e.other)
        );
        assert!((median - e.median).abs() < 0.0005, "median {median}");
    }
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate7_midtown_first_stopped() {
    gate7(
        "midtown",
        Expect7 {
            episodes: 2_490,
            rows: 48_896,
            median: 0.242,
            within: 2_441,
            early: 27,
            at_edge: 10,
            other: 12,
        },
        Some(Expect7 {
            episodes: 214,
            rows: 445,
            median: 0.233,
            within: 132,
            early: 2,
            at_edge: 67,
            other: 13,
        }),
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate7_urban_grid_first_stopped() {
    gate7(
        "urban_grid",
        Expect7 {
            episodes: 129,
            rows: 2_555,
            median: 0.242,
            within: 123,
            early: 6,
            at_edge: 0,
            other: 0,
        },
        None,
    );
}

// ── Gate 8: the synthetic crossing, drawn ────────────────────────────────────

#[test]
#[ignore = "renders through the GPU"]
fn gate8_synthetic_drawn() {
    let pl = synthetic();
    let st = Streets::build(&pl);
    let strips = scene::strips(&pl);
    let cam = scene::Camera {
        cx: 0.0,
        cy: 0.0,
        k: 0.05,
        width: 1280,
        height: 720,
    };
    let mut r = Renderer::new(&strips, cam, 1, None, None).unwrap();
    let off = r.render(&[]).unwrap();
    r.set_streets(Some(&st)).unwrap();
    let on = r.render(&[]).unwrap();
    let px = |f: &[u8], x: f64, y: f64| {
        let (i, j) = cam.world_to_pixel(x, y);
        let o = ((j.floor() as usize) * 1280 + i.floor() as usize) * 4;
        [f[o], f[o + 1], f[o + 2]]
    };
    let (bg, road) = (scene::BACKGROUND, scene::ROAD);
    let (dash, centre, stop, arrow, bar) = (
        Kind::Dash.colour(),
        Kind::Centre.colour(),
        Kind::Stop.colour(),
        Kind::Arrow.colour(),
        Kind::DeadEnd.colour(),
    );
    // A world point, what is there, and its colour off and on.
    type Row = (f64, f64, &'static str, [u8; 3], [u8; 3]);
    #[rustfmt::skip]
    let table: [Row; 30] = [
        (0.0, 0.0, "the junction", bg, road),
        (30.0, 0.0, "the 0.5 m fill, no line", bg, road),
        (0.0, -15.0, "the 4 m fill, no line", bg, road),
        (-30.0, 0.0, "L_WC's centre line", road, centre),
        (0.0, 15.0, "L_NC's centre line", road, centre),
        (-26.25, -3.5, "a dash", road, dash),
        (-29.5, -3.5, "the gap after it", road, road),
        (-12.2, -3.5, "a dash's end under L_WC's stop line", road, stop),
        (-12.2, -5.0, "L_WC's stop line", road, stop),
        (10.2, 2.0, "L_EC lane 0 at the edge", road, road),
        (15.2, 2.0, "L_EC lane 0's line, 5 m back", road, stop),
        (12.0, 3.75, "the connector", road, stop),
        (-16.0, 1.75, "a departing through's shaft", road, arrow),
        (12.8, 5.5, "an arriving through's head", road, arrow),
        (13.5, 9.8, "right's head", road, arrow),
        (12.8, 9.0, "where a through head would be", road, road),
        (18.5, 1.2, "left's head", road, arrow),
        (24.9, 1.4, "the separate u_turn's head", road, arrow),
        (-16.9, -1.15, "the lone u_turn's head", road, arrow),
        (-16.0, -4.45, "through_left's branch head", road, arrow),
        (-14.8, -5.25, "through_left's straight head", road, arrow),
        (-16.0, -9.55, "through_right's branch head", road, arrow),
        (-0.95, 17.75, "through_left_right's left head", road, arrow),
        (-2.55, 17.75, "through_left_right's right head", road, arrow),
        (-1.75, 16.55, "through_left_right's straight head", road, arrow),
        (-5.6, 16.45, "dead_end's bar", road, bar),
        (-5.25, 17.5, "dead_end's shaft", road, arrow),
        (2.95, -17.0, "left_right's left head", road, arrow),
        (4.55, -17.0, "left_right's right head", road, arrow),
        (3.75, -16.3, "where a straight head would be", road, road),
    ];
    for (x, y, what, want_off, want_on) in table {
        let (a, b) = (px(&off, x, y), px(&on, x, y));
        println!("gate 8 ({x}, {y}) {what}: off {a:?} on {b:?}");
        assert_eq!((a, b), (want_off, want_on), "({x}, {y}) {what}");
    }
    let changed = off
        .chunks(4)
        .zip(on.chunks(4))
        .filter(|(a, b)| a != b)
        .count();
    let exactly = |c: [u8; 3]| on.chunks(4).filter(|p| p[..3] == c).count();
    let counts = [
        exactly(centre),
        exactly(dash),
        exactly(stop),
        exactly(arrow),
        exactly(bar),
    ];
    println!(
        "gate 8: {changed} pixels change; exactly {} centre, {} dash, {} stop, {} arrow, {} bar",
        counts[0], counts[1], counts[2], counts[3], counts[4]
    );
    assert_eq!(
        (changed, counts),
        (243_827, [954, 2_166, 5_240, 6_546, 160])
    );
}
