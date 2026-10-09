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

use assimilator_config::network::NetworkConfig;
use assimilator_geometry::network_json::NetworkJson;
use assimilator_video::camera::Pose;
use assimilator_video::motion::Piece;
use assimilator_video::place::Placement;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::scene;
use assimilator_video::streets::{self, Pair, Streets, fade, mpp_at};
use assimilator_video::view::state::Follow;
use assimilator_video::view::{Fit, ViewInput, ViewState};

type P = [f64; 2];

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

/// Each synthetic link as drawn: its length.
struct Line {
    len: f64,
}

/// The links of [`SYNTHETIC`], in link order: the junction's polygon reaches x ±6.5 and
/// y ±10.25, and a two-way link's strip stands `w/2 + g/2` = 3.75 m right of its centreline.
fn lines() -> [Line; 6] {
    let ew = 43.5;
    let ns = 39.75;
    [
        Line { len: ew },
        Line { len: ew },
        Line { len: ew },
        Line { len: ew },
        Line { len: ns },
        Line { len: ns },
    ]
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

    // Pairs.
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
    println!("gate 5: 1 surface, 2 pairs: PASS");
}

// ── Gate 9: the fade ─────────────────────────────────────────────────────────

#[test]
fn gate9_fade() {
    assert_eq!(fade(0.15, 1.5), 0.0);
    assert!((fade(0.15, 0.5) - 0.5).abs() <= 1e-12);
    assert_eq!(fade(0.15, 0.3), 1.0);
    let ug = fade(0.15, 1.1481);
    assert!((ug - 0.017).abs() <= 0.001, "urban_grid's line {ug}");
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
    println!("gate 9: fade 0, 0.5, 1; urban_grid {ug:.4}; mpp_at: PASS");
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
}

fn gate6(dir: &str, e: Expect) {
    let run = load(dir);
    let pl = &run.placement;
    let net = &pl.network;
    let st = Streets::build(pl);

    // Independently, from the config.
    let polys = net
        .nodes
        .iter()
        .filter(|n| pl.data.junction_polygons.contains_key(&n.id))
        .count();
    let mut first: HashMap<(&str, &str), usize> = HashMap::new();
    for (i, l) in net.links.iter().enumerate() {
        first.entry((&l.from_node.0, &l.to_node.0)).or_insert(i);
    }
    let mut pairs = 0;
    for (i, l) in net.links.iter().enumerate() {
        if let Some(&j) = first.get(&(l.to_node.0.as_str(), l.from_node.0.as_str()))
            && i < j
        {
            pairs += 1;
        }
    }
    let got = [st.junctions.len(), st.pairs.len()];
    println!("gate 6 {dir}: surfaces {} pairs {}", got[0], got[1]);
    assert_eq!(got, [polys, pairs], "independent counts");
    assert_eq!(got, [e.junctions, e.pairs], "predicted counts");

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
    println!(
        "gate 6 {dir}: engine fills {} worst {fill_worst:.6} m",
        nj.junction_fills.len()
    );
    assert!(fill_worst <= 0.01);
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_counts() {
    gate6(
        "midtown",
        Expect {
            junctions: 106,
            pairs: 29,
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
        },
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
    // A world point, what is there, and its colour off and on.
    type Row = (f64, f64, &'static str, [u8; 3], [u8; 3]);
    let table: [Row; 1] = [(0.0, 0.0, "the junction", bg, road)];
    for (x, y, what, want_off, want_on) in table {
        let (a, b) = (px(&off, x, y), px(&on, x, y));
        println!("gate 8 ({x}, {y}) {what}: off {a:?} on {b:?}");
        assert_eq!((a, b), (want_off, want_on), "({x}, {y}) {what}");
    }
}

// ── Gate 12: `M`, headless ───────────────────────────────────────────────────

const W: f64 = 1280.0;
const H: f64 = 720.0;
const DT: f64 = 1.0 / 60.0;

/// vis-001 Phase 3's `plain_state`: `[0, 300]`, centre (0, 0), `k` = 1, 1280×720.
fn plain_state() -> ViewState {
    let fit = Fit {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        rect: (-1e4, -1e4, 1e4, 1e4),
        size: (W, H),
    };
    ViewState::new(0.0, 300.0, vec![], fit)
}

fn at(c: (f64, f64)) -> ViewInput {
    ViewInput {
        cursor: Some(c),
        pointer: Some(c),
        size: (W, H),
        dt: DT,
        ..Default::default()
    }
}

fn no_boxes(_: f64) -> Vec<VehicleBox> {
    vec![]
}

/// `input` with `m` pressed flips the flag and does exactly what `input` alone does.
fn step_m(s: &mut ViewState, input: ViewInput) {
    let mut without = s.clone();
    let line = without.frame(&input, no_boxes);
    let before = s.streets_shown;
    let mut with_m = input;
    with_m.pressed.m = true;
    assert_eq!(s.frame(&with_m, no_boxes), line, "the keyframe line");
    assert_eq!(s.streets_shown, !before, "m flips the flag");
    let mut flipped_back = s.clone();
    flipped_back.streets_shown = before;
    assert_eq!(flipped_back, without, "m changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.streets_shown;
    s.frame(&input, no_boxes);
    assert_eq!(s.streets_shown, before, "only m flips the flag");
}

#[test]
fn gate12_m() {
    let mut s = plain_state();
    assert!(!s.streets_shown, "off at new");
    let (buildings, see_through) = (s.buildings_shown, s.see_through);
    step_m(&mut s, at((640.0, 360.0)));
    assert!(s.streets_shown);
    step_m(&mut s, at((640.0, 360.0)));
    assert!(!s.streets_shown);

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_m(&mut s, k);
    step_m(&mut s, at((640.0, 360.0)));
    s.playing = false;
    s.follow = None;

    // A right-button orbit.
    step(
        &mut s,
        ViewInput {
            right_press: true,
            ..at((640.0, 360.0))
        },
    );
    assert!(s.orbit.is_some());
    step_m(&mut s, at((700.0, 330.0)));
    assert!(s.orbit.is_some(), "the orbit goes on");
    step(
        &mut s,
        ViewInput {
            right_release: true,
            ..at((700.0, 330.0))
        },
    );
    assert!(s.orbit.is_none());

    // A left drag.
    step(
        &mut s,
        ViewInput {
            press: true,
            ..at((400.0, 300.0))
        },
    );
    step_m(&mut s, at((460.0, 340.0)));
    assert!(s.press.is_some(), "the drag goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((460.0, 340.0))
        },
    );

    // A scrub.
    step(
        &mut s,
        ViewInput {
            press: true,
            ..at((640.0, 706.0))
        },
    );
    assert!(s.scrub.is_some());
    step_m(&mut s, at((300.0, 706.0)));
    assert!(s.scrub.is_some(), "the scrub goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((300.0, 706.0))
        },
    );
    assert!(s.scrub.is_none());
    assert_eq!(
        (s.buildings_shown, s.see_through),
        (buildings, see_through),
        "m never touches buildings_shown or see_through"
    );
    println!(
        "gate 12: m flips streets_shown and nothing else, through an orbit, a drag and a scrub: PASS"
    );
}
