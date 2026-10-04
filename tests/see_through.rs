//! vis-002 Phase 3 exit gates 5, 6, 8, 9 and 11 (`specs/city_spec.md`): the wedge and the
//! heights, the cut through the GPU, Midtown's traffic showing, untouched buildings and
//! `X`. Gates 5 and 11 are headless and run in plain `cargo test`. Gate 6 renders through
//! the GPU, gate 8 needs the Midtown fixture of `scripts/fixture.sh midtown` and the GPU,
//! and gate 9 needs the fixture; those are ignored. Run everything with
//! `cargo test --release --test see_through -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 7 and 10 are `scripts/gates-see-through.sh`.

use std::path::{Path, PathBuf};

use assimilator_video::buildings::{
    self, Building, Buildings, Counts, HeightRule, Polygon, building_mesh, mesh_data,
};
use assimilator_video::camera::{Pose, project};
use assimilator_video::clock;
use assimilator_video::draw::{self, BuildingsCut};
use assimilator_video::keyframes::{self, Flight};
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::run::{self, LoadOptions};
use assimilator_video::scene::{Camera, Strip};
use assimilator_video::see_through::{self, EASE_MIN_M, STUB_M, WIDTH};
use assimilator_video::view::state::{Fit, Follow, ViewInput, ViewState};
use assimilator_video::{Job, RenderOptions};
use bevy::mesh::{Indices, Mesh, VertexAttributeValues};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn midtown() -> PathBuf {
    root().join("scratch/midtown")
}

fn rect(id: &str, x0: f64, y0: f64, x1: f64, y1: f64, h: f64) -> Building {
    Building {
        id: id.into(),
        polygons: vec![Polygon {
            exterior: vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
            holes: vec![],
        }],
        height: h,
        rule: HeightRule::Height,
        base: 0.0,
        parts: vec![],
    }
}

fn city(bs: Vec<Building>) -> Buildings {
    let tallest = bs.iter().map(|b| b.height).fold(0.0, f64::max);
    Buildings {
        counts: Counts {
            height: bs.len(),
            num_floors: 0,
            default: 0,
        },
        buildings: bs,
        tallest,
    }
}

/// Straight down over (0, 0) at `height_m` 100.
fn straight_down() -> Pose {
    Pose {
        cx: 0.0,
        cy: 0.0,
        height_m: 100.0,
        yaw_deg: 0.0,
        pitch_deg: 90.0,
    }
}

/// Gate 6's pose.
fn tilted() -> Pose {
    Pose {
        cx: 0.0,
        cy: 0.0,
        height_m: 60.0,
        yaw_deg: 0.0,
        pitch_deg: 35.0,
    }
}

/// Gate 6's two blocks: in the way, and off.
fn gate6_blocks(h_in: f64, h_off: f64) -> Buildings {
    city(vec![
        rect("in-the-way", -10.0, -45.0, 10.0, -25.0, h_in),
        rect("off", 25.0, -40.0, 40.0, -25.0, h_off),
    ])
}

// ── Gate 5: the wedge and the heights ────────────────────────────────────────

#[test]
fn gate5_straight_down() {
    let pose = straight_down();
    let w = see_through::wedge(&pose);
    assert_eq!(w.r, 0.0, "r is exactly 0 straight down");
    assert_eq!(w.g, w.l, "g is exactly l");
    assert_eq!(w.ease, EASE_MIN_M);
    // 20 m squares, 30 m tall.
    let containing = rect("a", -10.0, -10.0, 10.0, 10.0, 30.0);
    let five = rect("b", 5.0, -10.0, 25.0, 10.0, 30.0);
    let ten = rect("c", 10.0, -10.0, 30.0, 10.0, 30.0);
    let far = rect("d", 40.0, 40.0, 60.0, 60.0, 30.0);
    let h = |b: &Building| see_through::height(&w, b);
    println!(
        "gate5 straight down: {} {} {} {}",
        h(&containing),
        h(&five),
        h(&ten),
        h(&far)
    );
    assert_eq!(h(&containing), STUB_M);
    assert_eq!(see_through::distance(&w, &five), 5.0);
    assert_eq!(h(&five), 16.5, "3 + 27·smoothstep(0.5)");
    assert_eq!(h(&ten).to_bits(), 30f64.to_bits(), "own height at δ = e");
    assert_eq!(h(&far).to_bits(), 30f64.to_bits(), "own height beyond e");
    let all = city(vec![containing, five, ten, far]);
    assert_eq!(
        see_through::heights(&all, &pose),
        vec![3.0, 16.5, 30.0, 30.0],
        "heights, in file order"
    );
}

#[test]
fn gate5_tilted() {
    let pose = tilted();
    let w = see_through::wedge(&pose);
    let eye = pose.eye();
    println!("gate5 tilted: g {:?} r {} ease {}", w.g, w.r, w.ease);
    assert_eq!(w.g, [eye[0], eye[1]], "g is Pose::eye's");
    assert!((w.g[1] + 59.328).abs() < 1e-3 && w.g[0].abs() < 1e-9);
    assert_eq!(w.r, WIDTH * 60.0 * 35f64.to_radians().cos());
    assert!((w.r - 7.372).abs() < 1e-3);
    assert_eq!(w.ease, 10.0);
    let hs = see_through::heights(&gate6_blocks(30.0, 20.0), &pose);
    println!("gate5 gate 6's blocks: {hs:?}");
    assert_eq!(hs[0], 3.0);
    assert_eq!(hs[1].to_bits(), 20f64.to_bits());
    // A 2 m building inside the wedge keeps its own height.
    let low = rect("low", -2.0, -2.0, 2.0, 2.0, 2.0);
    assert_eq!(see_through::height(&w, &low), 2.0);
}

#[test]
fn gate5_courtyard() {
    // Straight down, `W` is the point L, which lies in the hole.
    let pose = straight_down();
    let w = see_through::wedge(&pose);
    let b = Building {
        id: "courtyard".into(),
        polygons: vec![Polygon {
            exterior: vec![[-30.0, -30.0], [30.0, -30.0], [30.0, 30.0], [-30.0, 30.0]],
            holes: vec![vec![[-8.0, -8.0], [-8.0, 8.0], [8.0, 8.0], [8.0, -8.0]]],
        }],
        height: 30.0,
        rule: HeightRule::Height,
        base: 0.0,
        parts: vec![],
    };
    let d = see_through::distance(&w, &b);
    let h = see_through::height(&w, &b);
    println!("gate5 courtyard: δ {d} height {h}");
    assert!((d - 8.0).abs() < 1e-9);
    assert!((h - 27.192).abs() < 1e-9, "3 + 27·smoothstep(0.8)");
}

// ── Gate 6: the synthetic scene, through the GPU ─────────────────────────────

fn vbox(id: u64, x: f64, y: f64, heading: f64, length: f64, speed: f64) -> VehicleBox {
    VehicleBox {
        vehicle_id: id,
        at: Placed { x, y, heading },
        length,
        speed,
        track: TrackPos {
            piece: Piece::Link(0),
            along: 0.0,
            lateral: 0.0,
            odo: 0.0,
        },
    }
}

/// Pixels that differ between two RGBA frames.
fn differ(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

#[test]
#[ignore = "needs the GPU"]
fn gate6_synthetic() {
    let strips = vec![Strip {
        left: (0..=120).map(|i| [-60.0 + i as f64, 3.5]).collect(),
        right: (0..=120).map(|i| [-60.0 + i as f64, -3.5]).collect(),
    }];
    let (w, h) = (1280u32, 720u32);
    let cam = Camera {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        width: w,
        height: h,
    };
    let pose = tilted();
    let full = gate6_blocks(30.0, 20.0);
    let hs = see_through::heights(&full, &pose);
    println!("gate6 heights {hs:?}");
    let stubbed = gate6_blocks(hs[0], hs[1]);
    let b = [vbox(7, 0.0, 0.0, 90.0, 4.5, 10.0)];
    let new = |bs: Option<&Buildings>| {
        Renderer::new_perspective(&strips, cam, 4, &pose, bs, None).unwrap()
    };

    let mut r = new(Some(&full));
    let off_box = r.render(&b).unwrap();
    let off_empty = r.render(&[]).unwrap();
    r.set_see_through(Some(&full));
    r.set_pose(&pose);
    let on_box = r.render(&b).unwrap();
    let on_empty = r.render(&[]).unwrap();
    drop(r);
    let ref_box = new(Some(&stubbed)).render(&b).unwrap();
    let mut n = new(None);
    let none_box = n.render(&b).unwrap();
    let none_empty = n.render(&[]).unwrap();

    let (off, on, none) = (
        differ(&off_box, &off_empty),
        differ(&on_box, &on_empty),
        differ(&none_box, &none_empty),
    );
    let apart = differ(&on_box, &ref_box);
    // The in-the-way block's image rectangle: its eight corners projected, ±2 px.
    let corners: Vec<(f64, f64)> = [[-10.0, -45.0], [10.0, -45.0], [10.0, -25.0], [-10.0, -25.0]]
        .iter()
        .flat_map(|p| [0.0, 30.0].map(|z| project(&pose, w as f64, h as f64, [p[0], p[1], z])))
        .collect();
    let (x0, x1) = corners
        .iter()
        .fold((f64::MAX, f64::MIN), |a, c| (a.0.min(c.0), a.1.max(c.0)));
    let (y0, y1) = corners
        .iter()
        .fold((f64::MAX, f64::MIN), |a, c| (a.0.min(c.1), a.1.max(c.1)));
    let changed = differ(&on_empty, &off_empty);
    let outside = on_empty
        .chunks(4)
        .zip(off_empty.chunks(4))
        .enumerate()
        .filter(|(i, (a, b))| {
            let (px, py) = ((i % w as usize) as f64 + 0.5, (i / w as usize) as f64 + 0.5);
            a != b && !(px >= x0 - 2.0 && px <= x1 + 2.0 && py >= y0 - 2.0 && py <= y1 + 2.0)
        })
        .count();
    println!(
        "gate6 box pixels off {off} on {on} no buildings {none}; on vs stubbed file {apart} apart; \
         on vs off empty {changed} changed, {outside} outside x {x0:.0}..{x1:.0} y {y0:.0}..{y1:.0}"
    );
    assert_eq!(off, 0, "the block hides the box");
    assert_eq!(on, 1564);
    assert_eq!(none, 1564);
    assert_eq!(apart, 0, "the cut is the stubbed file's scene");
    assert_eq!(outside, 0, "every change lies in the block's image");
}

// ── Gate 8: Midtown, the traffic shows ───────────────────────────────────────

fn flight_options() -> RenderOptions {
    RenderOptions {
        project: midtown(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: Some(300.0),
        to: Some(360.0),
        speedup: Some(1.0),
        fps: 30,
        width: 1920,
        height: 1080,
    }
}

/// Box pixels in the centre, the middle and the whole frame, over every 15th frame.
fn box_pixels(flight: &Path, on: bool) -> [u64; 3] {
    let cache = midtown().join("buildings.geojson");
    let mut job =
        Job::prepare_without_credit(&flight_options(), Some(flight), Some(&cache)).unwrap();
    job.set_see_through(on);
    let mut tot = [0u64; 3];
    for n in (0..1800u64).step_by(15) {
        let a = job.render_frame(n).unwrap();
        let e = job.render_empty().unwrap();
        for (i, (x, y)) in a.chunks(4).zip(e.chunks(4)).enumerate() {
            if x != y {
                let (px, py) = ((i % 1920) as f64 + 0.5, (i / 1920) as f64 + 0.5);
                let mid = (px - 960.0).abs() <= 480.0 && (py - 540.0).abs() <= 270.0;
                let cen = (px - 960.0).abs() <= 240.0 && (py - 540.0).abs() <= 135.0;
                tot[0] += cen as u64;
                tot[1] += mid as u64;
                tot[2] += 1;
            }
        }
    }
    tot
}

fn within_1pct(name: &str, got: [u64; 3], want: [u64; 3]) -> bool {
    let mut ok = true;
    for (k, region) in ["centre", "middle", "frame"].iter().enumerate() {
        let dev = (got[k] as f64 - want[k] as f64) / want[k] as f64;
        println!(
            "gate8 {name} {region}: {} (probe {}, {:+.3} %)",
            got[k],
            want[k],
            100.0 * dev
        );
        ok &= dev.abs() <= 0.01;
    }
    ok
}

#[test]
#[ignore = "needs the Midtown fixture and the GPU"]
fn gate8_traffic_shows() {
    let orbit = root().join("tests/see-through-flight.toml");
    let city_flight = root().join("tests/city-flight.toml");
    let o_off = box_pixels(&orbit, false);
    let o_on = box_pixels(&orbit, true);
    let c_off = box_pixels(&city_flight, false);
    let c_on = box_pixels(&city_flight, true);
    let mut ok = within_1pct("orbit off", o_off, [59_506, 91_305, 168_189]);
    ok &= within_1pct("orbit on", o_on, [159_502, 258_570, 387_849]);
    ok &= within_1pct("city off", c_off, [3_928, 42_502, 90_710]);
    ok &= within_1pct("city on", c_on, [4_719, 45_322, 99_748]);
    let ratio = o_on[0] as f64 / o_off[0] as f64;
    println!("gate8 orbit centre on / off: {ratio:.3}");
    assert!(ok, "a count is more than 1 % from the probe's");
    assert!(ratio >= 2.5, "the orbit's centre on is at least 2.5 × off");
}

// ── Gate 9: untouched buildings are the shipped buildings ────────────────────

fn positions(m: &Mesh) -> Vec<[u32; 3]> {
    match m.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => {
            v.iter().map(|p| p.map(f32::to_bits)).collect()
        }
        _ => panic!("positions"),
    }
}

fn normals(m: &Mesh) -> Vec<[u32; 3]> {
    match m.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(v)) => {
            v.iter().map(|p| p.map(f32::to_bits)).collect()
        }
        _ => panic!("normals"),
    }
}

fn indices(m: &Mesh) -> Vec<u32> {
    match m.indices() {
        Some(Indices::U32(v)) => v.clone(),
        _ => panic!("u32 indices"),
    }
}

/// One flight over Midtown, headless: returns the largest in-frame step and the mean
/// number of buildings lowered a frame.
fn untouched(flight_file: &Path) -> (f64, f64) {
    let run = run::load(&LoadOptions {
        project: midtown(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: Some(300.0),
        to: Some(360.0),
    })
    .unwrap();
    let flight = Flight::new(keyframes::read(flight_file).unwrap(), &run.fcd);
    let b = buildings::read(&midtown().join("buildings.geojson"), &run.placement.network).unwrap();
    let fit = Camera::fit(&run.strips, 1920, 1080);
    let (fx, fy) = (fit.cx, fit.cy);

    let cut = BuildingsCut::new(&b, (fx, fy));
    let full = draw::buildings_mesh(&mesh_data(&b), fx, fy);
    let own: Vec<f64> = b.buildings.iter().map(|x| x.height).collect();
    let as_drawn = cut.mesh(&own);
    assert_eq!(
        positions(&as_drawn),
        positions(&full),
        "own heights: positions"
    );
    assert_eq!(normals(&as_drawn), normals(&full), "own heights: normals");
    assert_eq!(indices(&as_drawn), indices(&full), "own heights: indices");
    assert_eq!(as_drawn.primitive_topology(), full.primitive_topology());
    assert_eq!(as_drawn.asset_usage, full.asset_usage);
    assert!(
        as_drawn == full,
        "own heights: the mesh, attribute for attribute"
    );
    let full_pos = positions(&full);
    // Building i's vertex range, from the lengths of the buildings before it.
    let mut ranges = Vec::with_capacity(b.buildings.len());
    let mut at = 0;
    for x in &b.buildings {
        let n = building_mesh(x).positions.len();
        ranges.push(at..at + n);
        at += n;
    }
    assert_eq!(at, full_pos.len());

    let centroids: Vec<[f64; 2]> = b
        .buildings
        .iter()
        .map(|x| {
            let pts: Vec<&[f64; 2]> = x.polygons.iter().flat_map(|p| p.exterior.iter()).collect();
            let n = pts.len() as f64;
            [
                pts.iter().map(|p| p[0]).sum::<f64>() / n,
                pts.iter().map(|p| p[1]).sum::<f64>() / n,
            ]
        })
        .collect();
    let in_frame = |pose: &Pose, c: [f64; 2]| {
        let (a, _, _) = pose.axes();
        let e = pose.eye();
        let depth = (c[0] - e[0]) * a[0] + (c[1] - e[1]) * a[1] + (0.0 - e[2]) * a[2];
        if depth <= 0.1 {
            return false;
        }
        let (px, py) = project(pose, 1920.0, 1080.0, [c[0], c[1], 0.0]);
        (0.0..1920.0).contains(&px) && (0.0..1080.0).contains(&py)
    };

    let mut prev: Option<Vec<f64>> = None;
    let (mut worst, mut lowered) = (0.0f64, 0u64);
    for n in 0..1800u64 {
        let pose = flight.pose_at(
            clock::frame_time(300.0, n, 1.0, 30),
            &run.motion,
            &run.fcd,
            &run.placement,
        );
        let hs = see_through::heights(&b, &pose);
        lowered += hs.iter().zip(&own).filter(|(a, b)| a < b).count() as u64;
        if n % 30 == 0 {
            let w = see_through::wedge(&pose);
            let pos = positions(&cut.mesh(&hs));
            for (i, x) in b.buildings.iter().enumerate() {
                if see_through::distance(&w, x) >= w.ease {
                    assert_eq!(
                        hs[i].to_bits(),
                        x.height.to_bits(),
                        "frame {n} building {i}: h′ == h"
                    );
                    let r = ranges[i].clone();
                    assert_eq!(
                        pos[r.clone()],
                        full_pos[r],
                        "frame {n} building {i}: vertices"
                    );
                }
            }
        }
        if let Some(p) = &prev {
            for i in 0..hs.len() {
                if in_frame(&pose, centroids[i]) {
                    worst = worst.max((hs[i] - p[i]).abs());
                }
            }
        }
        prev = Some(hs);
    }
    (worst, lowered as f64 / 1800.0)
}

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate9_untouched() {
    for (name, file) in [
        ("city", root().join("tests/city-flight.toml")),
        ("orbit", root().join("tests/see-through-flight.toml")),
    ] {
        let (worst, mean) = untouched(&file);
        println!("gate9 {name}: largest in-frame step {worst:.2} m; lowered a frame {mean:.1}");
        assert!(
            worst <= 10.0,
            "{name}: a building in frame changed by {worst} m"
        );
    }
}

// ── Gate 11: `X`, headless ───────────────────────────────────────────────────

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

/// `input` with `x` pressed flips the flag and does exactly what `input` alone does.
fn step_x(s: &mut ViewState, input: ViewInput) {
    let mut without = s.clone();
    let line = without.frame(&input, no_boxes);
    let before = s.see_through;
    let mut with_x = input;
    with_x.pressed.x = true;
    assert_eq!(s.frame(&with_x, no_boxes), line, "the keyframe line");
    assert_eq!(s.see_through, !before, "x flips the flag");
    let mut flipped_back = s.clone();
    flipped_back.see_through = before;
    assert_eq!(flipped_back, without, "x changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.see_through;
    s.frame(&input, no_boxes);
    assert_eq!(s.see_through, before, "only x flips the flag");
}

#[test]
fn gate11_x() {
    let mut s = plain_state();
    assert!(!s.see_through, "off at new");
    assert!(s.buildings_shown);
    step_x(&mut s, at((640.0, 360.0)));
    assert!(s.see_through);
    step_x(&mut s, at((640.0, 360.0)));
    assert!(!s.see_through);

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_x(&mut s, k);
    step_x(&mut s, at((640.0, 360.0)));
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
    step_x(&mut s, at((700.0, 330.0)));
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
    step_x(&mut s, at((460.0, 340.0)));
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
    step_x(&mut s, at((300.0, 706.0)));
    assert!(s.scrub.is_some(), "the scrub goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((300.0, 706.0))
        },
    );
    assert!(s.scrub.is_none());
    assert!(s.buildings_shown, "x never touches buildings_shown");
}
