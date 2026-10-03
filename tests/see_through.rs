//! vis-002 Phase 3 exit gates 5 and 9 (`specs/city_spec.md`): the wedge and the heights,
//! headless, and Midtown's untouched buildings, which needs the fixture of
//! `scripts/fixture.sh midtown` and is ignored. Run with
//! `cargo test --release --test see_through -- --include-ignored --test-threads=1 --nocapture`.

use std::path::{Path, PathBuf};

use assimilator_video::buildings::{self, Building, Buildings, Counts, HeightRule, Polygon};
use assimilator_video::camera::{Pose, project};
use assimilator_video::clock;
use assimilator_video::keyframes::{self, Flight};
use assimilator_video::run::{self, LoadOptions};
use assimilator_video::see_through::{self, EASE_MIN_M, STUB_M, WIDTH};

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
    };
    let d = see_through::distance(&w, &b);
    let h = see_through::height(&w, &b);
    println!("gate5 courtyard: δ {d} height {h}");
    assert!((d - 8.0).abs() < 1e-9);
    assert!((h - 27.192).abs() < 1e-9, "3 + 27·smoothstep(0.8)");
}

// ── Gate 9: untouched buildings are the shipped buildings ────────────────────

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
    let own: Vec<f64> = b.buildings.iter().map(|x| x.height).collect();

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
            for (i, x) in b.buildings.iter().enumerate() {
                if see_through::distance(&w, x) >= w.ease {
                    assert_eq!(
                        hs[i].to_bits(),
                        x.height.to_bits(),
                        "frame {n} building {i}: h′ == h"
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
