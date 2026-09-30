//! vis-001 Phase 5 exit gates 4–12: the pose, the keyframe line and file, the flight and
//! `view`'s orbit and tilt. Gates 4, 6, 11 and 12 are headless and run in plain
//! `cargo test`. Those that need the fixture of `scripts/fixture.sh` or the GPU are
//! ignored; run everything with
//! `cargo test --release --test camera -- --include-ignored --test-threads=1 --nocapture`.

use std::path::{Path, PathBuf};
use std::process::Command;

use assimilator_video::camera::{self, Pose, distance, project, ray_to_plane};
use assimilator_video::keyframes::{self, Flight};
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::VehicleBox;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::scene::{self, Camera};
use assimilator_video::{Job, RenderOptions, clock};
use assimilator_video::view::state::{
    At, Fit, Follow, Held, Keyframe, OrbitButton, Pressed, ViewInput, ViewState,
};

/// Distances in metres, angles in degrees, pixels and times: set by formulas.
const TOL: f64 = 1e-9;
const W: f64 = 1280.0;
const H: f64 = 720.0;
const DT: f64 = 1.0 / 60.0;

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b} (tol {tol})");
}

fn close2(a: (f64, f64), b: (f64, f64), tol: f64, what: &str) {
    close(a.0, b.0, tol, &format!("{what} x"));
    close(a.1, b.1, tol, &format!("{what} y"));
}

// ── Gate 4: projection ───────────────────────────────────────────────────────

#[test]
fn gate4_projection() {
    for h in [1.0, 60.0, 720.0, 1240.0] {
        close(distance(h), 1.2071067811865475 * h, TOL, "distance");
    }

    let (rw, rh) = (1920.0, 1080.0);
    let fit = Pose::top_down(0.0, 300.0, 1240.0);
    let ortho = Camera {
        cx: 0.0,
        cy: 300.0,
        k: 1240.0 / 1080.0,
        width: 1920,
        height: 1080,
    };
    let x = [100.0, 350.0, 0.0];
    let p = project(&fit, rw, rh, x);
    println!("gate4 default pose: (100, 350, 0) at ({}, {})", p.0, p.1);
    close2(p, (1047.0967741935483, 496.4516129032258), TOL, "default pose");
    close2(p, ortho.world_to_pixel(100.0, 350.0), TOL, "world_to_pixel");

    let yaw90 = Pose {
        yaw_deg: 90.0,
        ..fit
    };
    let q = project(&yaw90, rw, rh, [100.0, 300.0, 0.0]);
    println!("gate4 yaw 90: (100, 300, 0) at ({}, {})", q.0, q.1);
    close2(q, (960.0, 452.9032258064516), TOL, "yaw 90");

    let tilt = Pose {
        cx: 0.0,
        cy: 0.0,
        height_m: 240.0,
        yaw_deg: 0.0,
        pitch_deg: 30.0,
    };
    let r = project(&tilt, rw, rh, [0.0, 100.0, 0.0]);
    println!("gate4 pitch 30: (0, 100, 0) at ({}, {})", r.0, r.1);
    close2(r, (960.0, 366.7808893062154), TOL, "pitch 30");

    for (pose, x, px) in [
        (fit, [100.0, 350.0], p),
        (yaw90, [100.0, 300.0], q),
        (tilt, [0.0, 100.0], r),
    ] {
        close2(
            ray_to_plane(&pose, rw, rh, px, 0.0),
            (x[0], x[1]),
            TOL,
            "ray_to_plane ∘ project",
        );
    }

    let mut s = plain_state();
    let g = s.world((740.0, 300.0));
    println!("gate4 plain state: cursor (740, 300) over ({}, {})", g.0, g.1);
    close2(g, (100.0, 60.0), TOL, "plain state");
    s.yaw_deg = 30.0;
    s.pitch_deg = 40.0;
    let g = s.world((740.0, 300.0));
    println!("gate4 yaw 30, pitch 40: over ({}, {})", g.0, g.1);
    close2(g, (145.222180146317, 33.60236249663478), TOL, "yaw 30, pitch 40");
}

#[test]
fn degrees_exact_at_right_angles() {
    for (d, s, c) in [
        (0.0, 0.0, 1.0),
        (90.0, 1.0, 0.0),
        (180.0, 0.0, -1.0),
        (270.0, -1.0, 0.0),
        (360.0, 0.0, 1.0),
        (-90.0, -1.0, 0.0),
    ] {
        assert_eq!(camera::sin_deg(d), s, "sin {d}");
        assert_eq!(camera::cos_deg(d), c, "cos {d}");
    }
    assert_eq!(camera::wrap360(-25.0), 335.0);
    assert_eq!(camera::wrap360(365.0), 5.0);
    assert_eq!(camera::wrap360(-1e-20), 0.0);
    assert_eq!(camera::wrap360(-0.0).to_bits(), 0.0f64.to_bits());
    assert_eq!(camera::short_way(0.0, 180.0), 180.0);
    assert_eq!(camera::short_way(180.0, 0.0), 180.0);
    assert_eq!(camera::short_way(90.0, 350.0), -100.0);
}

// ── Gate 11: orbit, tilt and the generalised camera ──────────────────────────

/// Phase 3's `plain_state`: `[0, 300]`, centre (0, 0), `k` = 1, 1280×720.
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

fn idle() -> ViewInput {
    ViewInput {
        size: (W, H),
        dt: DT,
        ..Default::default()
    }
}

fn at(c: (f64, f64)) -> ViewInput {
    ViewInput {
        cursor: Some(c),
        ..idle()
    }
}

fn vbox(id: u64, x: f64, y: f64, heading: f64, length: f64) -> VehicleBox {
    VehicleBox {
        vehicle_id: id,
        at: Placed { x, y, heading },
        length,
        speed: 0.0,
        track: TrackPos {
            piece: Piece::Link(0),
            along: 0.0,
            lateral: 0.0,
            odo: 0.0,
        },
    }
}

/// Every line gates 11–12 see: it round-trips, and gives back the pose to within half
/// its last decimal.
fn check_line(s: &ViewState) {
    let line = s.keyframe_line();
    let kf = Keyframe::parse(&line).unwrap_or_else(|e| panic!("{line}: {e}"));
    assert_eq!(kf.format(), line);
    close(kf.t, s.t, 0.0005 + TOL, "line t");
    close(kf.height_m, s.size.1 * s.k, 0.005 + TOL, "line height_m");
    close(kf.yaw_deg, s.yaw_deg, 0.005 + TOL, "line yaw_deg");
    close(kf.pitch_deg, s.pitch_deg, 0.005 + TOL, "line pitch_deg");
    match kf.at {
        At::Follow(id) => assert_eq!(s.follow.map(|f| f.vehicle_id), Some(id)),
        At::Centre(x, y) => {
            close(x, s.cx, 0.005 + TOL, "line x");
            close(y, s.cy, 0.005 + TOL, "line y");
        }
    }
}

fn step(s: &mut ViewState, boxes: &[VehicleBox], input: ViewInput) -> Option<String> {
    let line = s.frame(&input, |_| boxes.to_vec());
    check_line(s);
    line
}

fn key(f: impl FnOnce(&mut Pressed)) -> ViewInput {
    let mut i = at((640.0, 360.0));
    f(&mut i.pressed);
    i
}

fn angles(s: &ViewState) -> (f64, f64) {
    (s.yaw_deg, s.pitch_deg)
}

/// A right press at `from`, then the cursor at `to`.
fn right_drag(s: &mut ViewState, boxes: &[VehicleBox], from: (f64, f64), to: (f64, f64)) {
    step(
        s,
        boxes,
        ViewInput {
            right_press: true,
            ..at(from)
        },
    );
    step(s, boxes, at(to));
}

#[test]
fn gate11_orbit_right_drag() {
    let mut s = plain_state();
    right_drag(&mut s, &[], (640.0, 360.0), (740.0, 300.0));
    println!("gate11 right drag: yaw {} pitch {}", s.yaw_deg, s.pitch_deg);
    assert_eq!(angles(&s), (335.0, 75.0));
    assert_eq!((s.cx, s.cy, s.k), (0.0, 0.0, 1.0));
    assert_eq!(s.orbit.map(|o| o.button), Some(OrbitButton::Right));
    step(
        &mut s,
        &[],
        ViewInput {
            right_release: true,
            ..at((740.0, 300.0))
        },
    );
    assert_eq!(s.orbit, None);
    assert_eq!(angles(&s), (335.0, 75.0));
    let line = step(&mut s, &[], key(|p| p.k = true)).expect("K prints");
    // Gate 12's orbit line.
    println!("gate12 orbit line: {line}");
    assert_eq!(
        line,
        "{ t = 0.000, x = 0.00, y = 0.00, height_m = 720.00, yaw_deg = 335.00, pitch_deg = 75.00 }"
    );

    // Dragging down from straight down stays there.
    let mut s = plain_state();
    right_drag(&mut s, &[], (640.0, 360.0), (640.0, 460.0));
    assert_eq!(s.pitch_deg, 90.0);
}

#[test]
fn gate11_key_steps() {
    let mut s = plain_state();
    step(&mut s, &[], key(|p| p.e = true));
    assert_eq!(s.yaw_deg, 15.0);
    step(&mut s, &[], key(|p| p.q = true));
    assert_eq!(s.yaw_deg, 0.0);
    s.yaw_deg = 350.0;
    step(&mut s, &[], key(|p| p.e = true));
    assert_eq!(s.yaw_deg, 5.0);
    step(&mut s, &[], key(|p| p.q = true));
    assert_eq!(s.yaw_deg, 350.0);

    let mut s = plain_state();
    for _ in 0..13 {
        step(&mut s, &[], key(|p| p.r = true));
    }
    assert_eq!(s.pitch_deg, 25.0);
    step(&mut s, &[], key(|p| p.r = true));
    assert_eq!(s.pitch_deg, 25.0, "a 14th R");
    for _ in 0..13 {
        step(&mut s, &[], key(|p| p.f = true));
    }
    assert_eq!(s.pitch_deg, 90.0);

    // During an orbit the keys change nothing.
    let mut s = plain_state();
    right_drag(&mut s, &[], (640.0, 360.0), (740.0, 300.0));
    for f in [
        (|p: &mut Pressed| p.q = true) as fn(&mut Pressed),
        |p| p.e = true,
        |p| p.r = true,
        |p| p.f = true,
    ] {
        let mut i = at((740.0, 300.0));
        f(&mut i.pressed);
        step(&mut s, &[], i);
        assert_eq!(angles(&s), (335.0, 75.0), "a key during an orbit");
    }
}

#[test]
fn gate11_right_press_on_slider_and_box() {
    let mut s = plain_state();
    right_drag(&mut s, &[], (640.0, 706.0), (740.0, 600.0));
    assert_eq!(angles(&s), (0.0, 90.0), "a right press on the bar");
    assert_eq!(s.orbit, None);
    assert_eq!(s.scrub, None);

    let b = [vbox(1, 0.0, 0.0, 0.0, 4.5)];
    let mut s = plain_state();
    step(
        &mut s,
        &b,
        ViewInput {
            right_press: true,
            right_release: true,
            ..at((640.0, 360.0))
        },
    );
    assert_eq!(s.follow, None, "a right click picks nothing");
}

fn pick_plain(s: &mut ViewState, b: &[VehicleBox]) {
    step(
        s,
        b,
        ViewInput {
            press: true,
            release: true,
            ..at((640.0, 360.0))
        },
    );
    assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1), "a plain click picks");
}

#[test]
fn gate11_follow_survives_orbit_and_keys() {
    let b = [vbox(1, 0.0, 0.0, 0.0, 4.5)];
    let mut s = plain_state();
    pick_plain(&mut s, &b);
    right_drag(&mut s, &b, (640.0, 360.0), (740.0, 300.0));
    step(
        &mut s,
        &b,
        ViewInput {
            right_release: true,
            ..at((740.0, 300.0))
        },
    );
    assert!(s.follow.is_some(), "an orbit leaves the follow on");
    for f in [
        (|p: &mut Pressed| p.q = true) as fn(&mut Pressed),
        |p| p.e = true,
        |p| p.r = true,
        |p| p.f = true,
    ] {
        step(&mut s, &b, key(f));
        assert!(s.follow.is_some(), "a key step leaves the follow on");
    }
}

fn ctrl(i: ViewInput) -> ViewInput {
    ViewInput {
        held: Held { ctrl: true, ..i.held },
        ..i
    }
}

#[test]
fn gate11_ctrl_left_drag() {
    let mut s = plain_state();
    step(
        &mut s,
        &[],
        ctrl(ViewInput {
            press: true,
            ..at((640.0, 360.0))
        }),
    );
    assert_eq!(s.press, None, "no Press");
    step(&mut s, &[], ctrl(at((740.0, 300.0))));
    println!("gate11 ctrl drag: yaw {} pitch {}", s.yaw_deg, s.pitch_deg);
    assert_eq!(angles(&s), (335.0, 75.0));
    assert_eq!((s.cx, s.cy, s.k), (0.0, 0.0, 1.0));
    assert_eq!(s.press, None, "no Press");
    // Control released: the orbit goes on to the release.
    step(&mut s, &[], at((740.0, 300.0)));
    assert_eq!(angles(&s), (335.0, 75.0));
    assert_eq!(s.orbit.map(|o| o.button), Some(OrbitButton::Left));
    step(
        &mut s,
        &[],
        ViewInput {
            release: true,
            ..at((740.0, 300.0))
        },
    );
    assert_eq!(s.orbit, None);
    assert_eq!((s.cx, s.cy), (0.0, 0.0));

    // Control pressed only after the press: a pan.
    let mut s = plain_state();
    step(
        &mut s,
        &[],
        ViewInput {
            press: true,
            ..at((640.0, 360.0))
        },
    );
    step(&mut s, &[], ctrl(at((740.0, 300.0))));
    println!("gate11 ctrl after the press: centre ({}, {})", s.cx, s.cy);
    close2((s.cx, s.cy), (-100.0, -60.0), TOL, "pan");
    assert_eq!(angles(&s), (0.0, 90.0));
    assert_eq!(s.orbit, None);
}

#[test]
fn gate11_ctrl_left_over_a_box() {
    let b = [vbox(1, 0.0, 0.0, 0.0, 4.5)];
    let mut s = plain_state();
    step(
        &mut s,
        &b,
        ctrl(ViewInput {
            press: true,
            release: true,
            ..at((640.0, 360.0))
        }),
    );
    assert_eq!(s.follow, None, "a Ctrl click picks nothing");

    pick_plain(&mut s, &b);
    step(
        &mut s,
        &b,
        ctrl(ViewInput {
            press: true,
            ..at((640.0, 360.0))
        }),
    );
    step(&mut s, &b, ctrl(at((740.0, 300.0))));
    step(
        &mut s,
        &b,
        ctrl(ViewInput {
            release: true,
            ..at((740.0, 300.0))
        }),
    );
    assert_eq!(
        s.follow,
        Some(Follow {
            vehicle_id: 1,
            drawn: true
        })
    );
    assert_eq!((s.cx, s.cy), (0.0, 0.0), "the centre on the box");
    assert_eq!(angles(&s), (335.0, 75.0));
}

#[test]
fn gate11_ctrl_left_on_slider() {
    for playing in [false, true] {
        let mut s = plain_state();
        s.playing = playing;
        let t0 = s.t;
        step(
            &mut s,
            &[],
            ctrl(ViewInput {
                press: true,
                ..at((640.0, 706.0))
            }),
        );
        step(&mut s, &[], ctrl(at((900.0, 706.0))));
        step(
            &mut s,
            &[],
            ctrl(ViewInput {
                release: true,
                ..at((900.0, 706.0))
            }),
        );
        assert_eq!(s.scrub, None, "no scrub");
        assert_eq!(s.orbit, None);
        assert_eq!(s.press, None);
        assert_eq!(s.playing, playing, "the clock as it was");
        if !playing {
            assert_eq!(s.t, t0, "t stays");
        } else {
            close(s.t, t0 + 3.0 * DT, TOL, "played on");
        }
        assert_eq!(angles(&s), (0.0, 90.0));
    }
}

#[test]
fn gate11_ctrl_left_during_right_orbit() {
    let mut s = plain_state();
    right_drag(&mut s, &[], (640.0, 360.0), (740.0, 300.0));
    step(
        &mut s,
        &[],
        ctrl(ViewInput {
            press: true,
            ..at((740.0, 300.0))
        }),
    );
    assert_eq!(s.orbit.map(|o| o.button), Some(OrbitButton::Right));
    assert_eq!(s.press, None);
    step(
        &mut s,
        &[],
        ctrl(ViewInput {
            release: true,
            ..at((740.0, 300.0))
        }),
    );
    assert!(s.orbit.is_some(), "a left release does not end a right orbit");
    step(
        &mut s,
        &[],
        ViewInput {
            right_release: true,
            ..at((740.0, 300.0))
        },
    );
    assert_eq!(s.orbit, None);
    assert_eq!(angles(&s), (335.0, 75.0));
}

#[test]
fn gate11_pan_zoom_wasd_at_a_tilt() {
    let tilted = || {
        let mut s = plain_state();
        s.yaw_deg = 30.0;
        s.pitch_deg = 40.0;
        s
    };
    let mut s = tilted();
    let under = s.world((640.0, 360.0));
    step(
        &mut s,
        &[],
        ViewInput {
            press: true,
            ..at((640.0, 360.0))
        },
    );
    step(&mut s, &[], at((740.0, 300.0)));
    println!("gate11 pan at a tilt: centre ({}, {})", s.cx, s.cy);
    close2(
        (s.cx, s.cy),
        (-145.222180146317, -33.60236249663467),
        TOL,
        "pan centre",
    );
    close2(s.world((740.0, 300.0)), under, TOL, "the press's point");
    close2(under, (0.0, 0.0), TOL, "under the press");

    let mut s = tilted();
    step(
        &mut s,
        &[],
        ViewInput {
            scroll_lines: 5.0,
            ..at((800.0, 300.0))
        },
    );
    println!("gate11 zoom at a tilt: k {} centre ({}, {})", s.k, s.cx, s.cy);
    close(s.k, 1.1f64.powi(-5), TOL, "k");
    close2(
        (s.cx, s.cy),
        (76.5140026126445, 0.3460562657681663),
        TOL,
        "zoom centre",
    );
    close2(
        s.world((800.0, 300.0)),
        (201.84201134738174, 0.9128877112287341),
        TOL,
        "under the cursor",
    );

    let mut s = plain_state();
    s.yaw_deg = 90.0;
    for _ in 0..30 {
        step(
            &mut s,
            &[],
            ViewInput {
                held: Held {
                    w: true,
                    ..Default::default()
                },
                ..idle()
            },
        );
    }
    println!("gate11 W at yaw 90: centre ({}, {})", s.cx, s.cy);
    close(s.cx, 320.0, TOL, "W x");
    assert_eq!(s.cy, 0.0, "W y exactly 0");
}

#[test]
fn gate11_pick_at_a_tilt() {
    let fit = Fit {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        rect: (-1e4, -1e4, 1e4, 1e4),
        size: (W, H),
    };
    let mut s = ViewState::new(0.0, 300.0, vec![], fit);
    s.yaw_deg = 30.0;
    s.pitch_deg = 45.0;
    s.k = 0.05;
    assert_eq!(s.size.1 * s.k, 36.0);
    let b = [vbox(1, 0.0, 0.0, 120.0, 4.5)];
    let click = (640.0, 337.5125250710996);
    let roof = project(&s.pose(), W, H, [0.0, 0.0, 1.55]);
    println!("gate11 roof centre at ({}, {})", roof.0, roof.1);
    close2(roof, click, TOL, "the roof centre's pixel");
    let p = s.at_height(click, 0.8);
    let g = s.world(click);
    println!(
        "gate11 click at 0.80 m: ({:.3}, {:.3}); ground ({:.3}, {:.3})",
        p.0, p.1, g.0, g.1
    );
    close2(p, (0.395, 0.684), 5e-4, "at 0.80 m");
    close2(g, (0.816, 1.414), 5e-4, "ground");
    let d_ground = assimilator_video::view::state::footprint_distance(&b[0], g);
    assert!(d_ground > 8.0 * s.k, "a ground-plane pick would miss ({d_ground})");
    step(
        &mut s,
        &b,
        ViewInput {
            press: true,
            release: true,
            ..at(click)
        },
    );
    assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1), "picked at a tilt");
}

// ── Gate 12: the keyframe line ───────────────────────────────────────────────

#[test]
fn gate12_keyframe_line() {
    let mut s = plain_state();
    s.t = 64.1;
    s.cx = 512.3;
    s.cy = -133.2;
    s.k = 1.0 / 3.0;
    let a = s.keyframe_line();
    assert_eq!(
        a,
        "{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );
    check_line(&s);
    let mut s = plain_state();
    s.t = 200.0;
    s.k = 1.0 / 12.0;
    s.follow = Some(Follow {
        vehicle_id: 103,
        drawn: true,
    });
    let b = s.keyframe_line();
    assert_eq!(
        b,
        "{ t = 200.000, follow = 103, height_m = 60.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );
    check_line(&s);

    // A yaw that would print as 360.00 prints 0.00, and reads back to itself.
    s.follow = None;
    s.yaw_deg = 359.999;
    let line = s.keyframe_line();
    assert!(line.contains("yaw_deg = 0.00"), "{line}");
    assert_eq!(Keyframe::parse(&line).unwrap().format(), line);

    // The orbit line and gate 11's other states, as a file.
    let orbit =
        "{ t = 0.000, x = 0.00, y = 0.00, height_m = 720.00, yaw_deg = 335.00, pitch_deg = 75.00 }";
    let file = format!("keyframes = [\n  {orbit},\n  {a},\n  {b},\n]\n");
    let v: toml::Table = toml::from_str(&file).expect("the lines read as a file");
    assert_eq!(v["keyframes"].as_array().map(|a| a.len()), Some(3));
    let kfs = keyframes::parse_file(&file).expect("the file reader takes them");
    assert_eq!(kfs.len(), 3);
    assert_eq!((kfs[0].yaw_deg, kfs[0].pitch_deg), (335.0, 75.0));
    for (kf, line) in kfs.iter().zip([orbit, a.as_str(), b.as_str()]) {
        assert_eq!(kf.format(), line);
    }
}

// ── Gate 6: the flight, constructed ──────────────────────────────────────────

fn kf(t: f64, x: f64, y: f64, h: f64, yaw: f64, pitch: f64) -> Keyframe {
    Keyframe {
        t,
        at: At::Centre(x, y),
        height_m: h,
        yaw_deg: yaw,
        pitch_deg: pitch,
    }
}

fn pose_of(k: &Keyframe) -> Pose {
    let At::Centre(cx, cy) = k.at else {
        panic!("a fixed keyframe")
    };
    Pose {
        cx,
        cy,
        height_m: k.height_m,
        yaw_deg: k.yaw_deg,
        pitch_deg: k.pitch_deg,
    }
}

fn no_boxes(_: f64) -> Vec<VehicleBox> {
    vec![]
}

fn gate6_keyframes() -> Vec<Keyframe> {
    vec![
        kf(10.0, 0.0, 0.0, 200.0, 0.0, 90.0),
        kf(20.0, 100.0, 0.0, 200.0, 0.0, 90.0),
        kf(30.0, 100.0, 100.0, 100.0, 90.0, 45.0),
        kf(40.0, 100.0, 100.0, 100.0, 90.0, 45.0),
        kf(50.0, 0.0, 100.0, 300.0, 350.0, 60.0),
    ]
}

fn close_pose(p: &Pose, want: (f64, f64, f64, f64, f64), what: &str) {
    close(p.cx, want.0, TOL, &format!("{what} x"));
    close(p.cy, want.1, TOL, &format!("{what} y"));
    close(p.height_m, want.2, TOL, &format!("{what} height_m"));
    close(p.yaw_deg, want.3, TOL, &format!("{what} yaw"));
    close(p.pitch_deg, want.4, TOL, &format!("{what} pitch"));
}

/// `v` lies between `a` and `b`.
fn between(v: f64, a: f64, b: f64) -> bool {
    a.min(b) <= v && v <= a.max(b)
}

/// The distance from `p` to the segment `a`–`b`.
fn to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let u = if l2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0)
    };
    (p.0 - a.0 - u * dx).hypot(p.1 - a.1 - u * dy)
}

/// Each scalar of `p` lies between keyframes `a` and `b`'s (the yaw the short way).
fn scalars_between(p: &Pose, a: &Keyframe, b: &Keyframe) -> bool {
    let dyaw = camera::short_way(a.yaw_deg, b.yaw_deg);
    let off = camera::short_way(a.yaw_deg, p.yaw_deg);
    let off = if dyaw == 180.0 && off == -180.0 { 180.0 } else { off };
    between(p.height_m, a.height_m, b.height_m)
        && between(p.pitch_deg, a.pitch_deg, b.pitch_deg)
        && between(off, 0.0, dyaw)
}

#[test]
fn gate6_flight_constructed() {
    let k = gate6_keyframes();
    let f = Flight::with_drawn(k.clone(), vec![]);
    let at = |t: f64| f.pose_at_by(t, &no_boxes);
    let pos = |t: f64| {
        let p = at(t);
        (p.cx, p.cy)
    };

    // The table.
    for (t, j) in [(5.0, 0), (10.0, 0), (20.0, 1), (30.0, 2), (35.0, 2), (40.0, 2), (50.0, 4), (55.0, 4)] {
        assert_eq!(at(t), pose_of(&k[j]), "t = {t}: keyframe {} exactly", j + 1);
    }
    for (t, want) in [
        (15.0, (41.89453125, -4.39453125, 200.0, 0.0, 90.0)),
        (25.0, (104.39453125, 58.10546875, 141.42135623730945, 45.0, 67.5)),
        (45.0, (50.0, 100.0, 173.20508075688775, 40.0, 52.5)),
    ] {
        let p = at(t);
        println!(
            "gate6 t = {t}: ({}, {}, {}, {}, {})",
            p.cx, p.cy, p.height_m, p.yaw_deg, p.pitch_deg
        );
        close_pose(&p, want, &format!("t = {t}"));
    }

    // The hold: every frame time of [30, 40] at 30 fps.
    let mut drift = 0;
    for n in 0..=300 {
        let t = 30.0 + n as f64 / 30.0;
        if at(t) != pose_of(&k[2]) {
            drift += 1;
        }
    }
    println!("gate6 hold: 301 frames, {drift} drift");
    assert_eq!(drift, 0);

    // No overshoot, and τ never decreases nor passes K3's knot on K2 → K3.
    let frames: Vec<f64> = (0..=1200).map(|n| 10.0 + n as f64 / 30.0).collect();
    let mut violations = 0;
    // K3's knot: |K2 − K1|^½ + |K3 − K2|^½.
    let tau_k3 = 20.0;
    let mut last_tau = f64::NEG_INFINITY;
    for &t in &frames {
        let i = k.partition_point(|kf| kf.t <= t).saturating_sub(1).min(k.len() - 2);
        if !scalars_between(&at(t), &k[i], &k[i + 1]) {
            violations += 1;
        }
        if t < 30.0 {
            let tau = f.tau_at(t).expect("K1 → K3 is one run");
            assert!(tau >= last_tau, "τ decreases at {t}");
            assert!(tau <= tau_k3, "τ passes K3's knot at {t}");
            last_tau = tau;
        }
    }
    println!("gate6 overshoot: {violations} violations");
    assert_eq!(violations, 0);

    // No corner at K2; at rest at 10, 30, 40 and 50.
    let e = 1e-4;
    let v = |a: f64, b: f64| {
        let (p, q) = (pos(a), pos(b));
        ((q.0 - p.0) / (b - a), (q.1 - p.1) / (b - a))
    };
    for (what, got) in [
        ("central", v(20.0 - e, 20.0 + e)),
        ("left", v(20.0 - e, 20.0)),
        ("right", v(20.0, 20.0 + e)),
    ] {
        println!("gate6 velocity at K2, {what}: ({}, {})", got.0, got.1);
        close2(got, (5.0, 5.0), 1e-3, &format!("velocity at K2 {what}"));
    }
    for t in [10.0, 30.0, 40.0, 50.0] {
        for (what, got) in [("left", v(t - e, t)), ("right", v(t, t + e))] {
            close2(got, (0.0, 0.0), 1e-3, &format!("velocity at {t} {what}"));
        }
    }

    // The curve leaves the lines.
    let mut worst = [(0.0f64, 0.0f64); 4];
    let mut vmax = (0.0f64, 0.0);
    for &t in &frames {
        let i = k.partition_point(|kf| kf.t <= t).saturating_sub(1).min(k.len() - 2);
        let (a, b) = (pose_of(&k[i]), pose_of(&k[i + 1]));
        let d = to_segment(pos(t), (a.cx, a.cy), (b.cx, b.cy));
        if d > worst[i].0 {
            worst[i] = (d, t);
        }
        let h = 1e-6;
        let s = v(t - h, t + h);
        let sp = s.0.hypot(s.1);
        if sp > vmax.0 {
            vmax = (sp, t);
        }
    }
    println!(
        "gate6 off the lines: K1→K2 {} at {}, K2→K3 {} at {}; top speed {} at {}",
        worst[0].0, worst[0].1, worst[1].0, worst[1].1, vmax.0, vmax.1
    );
    close(worst[0].0, 7.4073228602604, TOL, "off the line K1→K2");
    close(worst[1].0, 7.4073228602604, TOL, "off the line K2→K3");
    close(worst[0].1, 17.2333, 1e-4, "K1→K2 where");
    close(worst[1].1, 22.7667, 1e-4, "K2→K3 where");
    assert!(worst[2].0 == 0.0 && worst[3].0 < TOL, "the hold and K4→K5 stay on their lines");
    close(vmax.0, 15.006, 1e-3, "top speed");

    // Yaw the short way.
    let yaws: Vec<f64> = (40..=50).map(|s| at(s as f64).yaw_deg).collect();
    println!("gate6 yaw K4→K5: {yaws:?}");
    for (y, want) in yaws.iter().zip([90.0, 87.2, 79.6, 68.4, 54.8, 40.0, 25.2, 11.6, 0.4, 352.8, 350.0]) {
        close(*y, want, TOL, "yaw");
    }
    for &t in frames.iter().filter(|&&t| t > 40.0) {
        let y = at(t).yaw_deg;
        assert!(!(90.0 < y && y < 350.0), "yaw {y} at {t}: the long way");
    }
    let tie = |a: f64, b: f64| {
        let f = Flight::with_drawn(
            vec![kf(0.0, 0.0, 0.0, 100.0, a, 90.0), kf(10.0, 0.0, 0.0, 100.0, b, 90.0)],
            vec![],
        );
        f.pose_at_by(5.0, &no_boxes).yaw_deg
    };
    close(tie(0.0, 180.0), 90.0, TOL, "0 → 180 turns clockwise");
    close(tie(180.0, 0.0), 270.0, TOL, "180 → 0 turns clockwise");
}

// ── The fixture: gates 5, 7, 8 and 9 ─────────────────────────────────────────

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_run() -> Run {
    run::load(&LoadOptions {
        project: root().join("scratch/urban_grid"),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: None,
        to: None,
    })
    .expect("the fixture loads (scripts/fixture.sh)")
}

/// A scratch file for a keyframe file's text.
fn write_camera(name: &str, text: &str) -> PathBuf {
    let dir = root().join("scratch/out/camera");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, text).unwrap();
    p
}

const LINE: &str = "{ t = 20.000, x = 0.00, y = 300.00, height_m = 1240.00 }";

fn one(line: &str) -> String {
    format!("keyframes = [\n  {line},\n]\n")
}

fn render_cmd(camera: &Path, out: &Path) -> (i32, String) {
    let o = Command::new(env!("CARGO_BIN_EXE_assimilator-video"))
        .args(["render", "--project"])
        .arg(root().join("scratch/urban_grid"))
        .args(["--scenario", "baseline", "--seed", "42", "--out"])
        .arg(out)
        .arg("--camera")
        .arg(camera)
        .output()
        .unwrap();
    (
        o.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&o.stderr).into_owned(),
    )
}

#[test]
#[ignore = "needs the fixture"]
fn gate5_keyframe_file_errors() {
    let kf = |body: &str| one(&format!("{{ {body} }}"));
    let cases: Vec<(&str, Option<String>, &str)> = vec![
        ("no-file", None, "cannot read"),
        ("not-toml", Some("keyframes = [ {".into()), "not TOML"),
        ("no-keyframes", Some("# no keys\n".into()), "no `keyframes`"),
        ("empty", Some("keyframes = []\n".into()), "empty"),
        ("second-key", Some(format!("{}speed = 2\n", one(LINE))), "unknown field"),
        ("unknown-key", Some(kf("t = 20.0, x = 0.0, y = 300.0, height_m = 1240.0, zoom = 2.0")), "unknown field"),
        ("no-t", Some(kf("x = 0.0, y = 300.0, height_m = 1240.0")), "no `t`"),
        ("no-height", Some(kf("t = 20.0, x = 0.0, y = 300.0")), "no `height_m`"),
        ("x-and-follow", Some(kf("t = 20.0, x = 0.0, y = 300.0, follow = 1, height_m = 1240.0")), "both"),
        ("neither", Some(kf("t = 20.0, height_m = 1240.0")), "neither"),
        ("x-without-y", Some(kf("t = 20.0, x = 0.0, height_m = 1240.0")), "without"),
        ("t-inf", Some(kf("t = inf, x = 0.0, y = 300.0, height_m = 1240.0")), "finite"),
        ("height-nan", Some(kf("t = 20.0, x = 0.0, y = 300.0, height_m = nan")), "finite"),
        ("follow-negative", Some(kf("t = 64.1, follow = -3, height_m = 120.0")), "≥ 0"),
        ("follow-fraction", Some(kf("t = 64.1, follow = 1.5, height_m = 120.0")), "invalid type"),
        ("height-zero", Some(kf("t = 20.0, x = 0.0, y = 300.0, height_m = 0")), "positive"),
        ("pitch-low", Some(kf("t = 20.0, x = 0.0, y = 300.0, height_m = 1240.0, pitch_deg = 24.99")), "pitch_deg"),
        ("pitch-high", Some(kf("t = 20.0, x = 0.0, y = 300.0, height_m = 1240.0, pitch_deg = 90.01")), "pitch_deg"),
        (
            "out-of-order",
            Some(format!(
                "keyframes = [\n  {LINE},\n  {{ t = 10.000, x = 0.00, y = 300.00, height_m = 1240.00 }},\n]\n"
            )),
            "out of order",
        ),
        ("duplicate", Some(format!("keyframes = [\n  {LINE},\n  {LINE},\n]\n")), "duplicate"),
        ("follow-gone", Some(kf("t = 200.0, follow = 1, height_m = 60.0")), "not drawn"),
        ("follow-unknown", Some(kf("t = 64.1, follow = 999999, height_m = 60.0")), "not in the FCD"),
    ];
    for (label, text, want) in cases {
        let path = match &text {
            Some(t) => write_camera(&format!("gate5_{label}.toml"), t),
            None => root().join("scratch/out/camera/does_not_exist.toml"),
        };
        let out = root().join(format!("scratch/out/camera/gate5_{label}.mp4"));
        let partial = root().join(format!("scratch/out/camera/gate5_{label}.mp4.partial"));
        let _ = std::fs::remove_file(&out);
        let (code, stderr) = render_cmd(&path, &out);
        println!("gate5 {label}: exit {code}: {}", stderr.trim_end());
        assert_eq!(code, 1, "{label}: exit");
        assert_eq!(stderr.lines().count(), 1, "{label}: one stderr line");
        assert!(stderr.starts_with("error: --camera"), "{label}: {stderr}");
        assert!(stderr.contains(want), "{label}: wants {want:?}: {stderr}");
        assert!(!stderr.contains("\"frame\""), "{label}: progress");
        assert!(!out.exists() && !partial.exists(), "{label}: an output file");
    }
}

#[test]
#[ignore = "needs the fixture"]
fn gate5_keyframe_file_accepted() {
    let run = fixture_run();
    let accept = |text: &str| {
        let kfs = keyframes::parse_file(text).unwrap_or_else(|e| panic!("{text}: {e}"));
        keyframes::check_follows(&kfs, &run.motion, &run.fcd, &run.placement)
            .unwrap_or_else(|e| panic!("{text}: {e}"));
        Flight::new(kfs, &run.fcd)
    };
    let at = |f: &Flight, t: f64| f.pose_at(t, &run.motion, &run.fcd, &run.placement);

    // Phase 3's lines, unchanged.
    let f = accept(&one("{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }"));
    assert_eq!(at(&f, 64.1), Pose { cx: 512.3, cy: -133.2, height_m: 240.0, yaw_deg: 0.0, pitch_deg: 90.0 });
    let kfs = keyframes::parse_file(&one("{ t = 200.000, follow = 103, height_m = 60.00 }")).unwrap();
    assert_eq!((kfs[0].yaw_deg, kfs[0].pitch_deg), (0.0, 90.0));
    let f = accept(&one("{ t = 64.100, follow = 1, height_m = 120.00 }"));
    let p = at(&f, 64.1);
    assert_eq!((p.yaw_deg, p.pitch_deg, p.height_m), (0.0, 90.0, 120.0));

    for (yaw, want) in [(370.0, 10.0), (-90.0, 270.0)] {
        let f = accept(&one(&format!(
            "{{ t = 20.000, x = 0.00, y = 300.00, height_m = 1240.00, yaw_deg = {yaw} }}"
        )));
        println!("gate5 yaw_deg = {yaw} reads as {}", at(&f, 20.0).yaw_deg);
        assert_eq!(at(&f, 20.0).yaw_deg, want);
    }

    let flight = std::fs::read_to_string(root().join("tests/flight.toml")).unwrap();
    let inline = keyframes::parse_file(&flight).unwrap();
    let mut tables = String::new();
    for k in &inline {
        tables.push_str(&format!("[[keyframes]]\n{}\n\n", {
            let l = k.format();
            l[2..l.len() - 2].replace(", ", "\n")
        }));
    }
    assert_eq!(keyframes::parse_file(&tables).unwrap(), inline, "[[keyframes]] tables");

    // A single keyframe holds for the whole run; one before `from` is read.
    let f = accept(&one(LINE));
    let want = Pose { cx: 0.0, cy: 300.0, height_m: 1240.0, yaw_deg: 0.0, pitch_deg: 90.0 };
    for t in [run.from, 20.0, 150.0, run.to] {
        assert_eq!(at(&f, t), want);
    }
    let f = accept(&one("{ t = 5.000, x = 10.00, y = 20.00, height_m = 300.00 }"));
    assert_eq!(at(&f, run.from), Pose { cx: 10.0, cy: 20.0, height_m: 300.0, yaw_deg: 0.0, pitch_deg: 90.0 });
}

// ── Gate 7: the flight on the fixture ────────────────────────────────────────

fn placed_of(run: &Run, t: f64, id: u64) -> Option<(f64, f64)> {
    run.boxes_at(t)
        .iter()
        .find(|b| b.vehicle_id == id)
        .map(|b| (b.at.x, b.at.y))
}

#[test]
#[ignore = "needs the fixture"]
fn gate7_flight_on_fixture() {
    let run = fixture_run();
    let k = keyframes::read(&root().join("tests/flight.toml")).expect("tests/flight.toml");
    keyframes::check_follows(&k, &run.motion, &run.fcd, &run.placement).expect("its follows");
    let f = Flight::new(k.clone(), &run.fcd);
    let d = run.to - run.from;
    let speedup = clock::default_speedup(d);
    let frames = clock::frame_count(d, 30, speedup);
    assert_eq!((run.from, speedup, frames), (9.099999999999984, 1.0, 8700));
    let times: Vec<f64> = (0..frames).map(|n| clock::frame_time(run.from, n, speedup, 30)).collect();
    let poses: Vec<Pose> = times
        .iter()
        .map(|&t| f.pose_at(t, &run.motion, &run.fcd, &run.placement))
        .collect();
    let first = Pose { cx: 0.0, cy: 300.0, height_m: 1240.0, yaw_deg: 0.0, pitch_deg: 90.0 };
    let sixth = Pose { cx: 300.0, cy: 600.0, height_m: 700.0, yaw_deg: 315.0, pitch_deg: 35.0 };
    let (mut held, mut follow, mut blend, mut violations) = (0, 0, 0, 0);
    let mut last_h = f64::INFINITY;
    let last_row = run.fcd.vehicles.iter().find(|v| v.vehicle_id == 1).unwrap().rows.last().unwrap().time;
    let last_seen = placed_of(&run, last_row, 1).expect("vehicle 1 at its last row");
    println!("gate7 vehicle 1 last row {last_row}, at {last_seen:?}");
    for (&t, p) in times.iter().zip(&poses) {
        if t <= 50.0 {
            assert_eq!(*p, first, "t = {t}");
            held += 1;
        } else if t >= 220.0 {
            assert_eq!(*p, sixth, "t = {t}");
            held += 1;
        }
        if (64.1..=140.0).contains(&t) {
            let v = placed_of(&run, t, 1).expect("vehicle 1 drawn");
            assert_eq!((p.cx, p.cy), v, "t = {t}: on vehicle 1");
            assert_eq!((p.yaw_deg, p.pitch_deg), (90.0, 45.0), "t = {t}");
            assert!(p.height_m < last_h, "t = {t}: height {} not decreasing", p.height_m);
            assert!(between(p.height_m, 60.0, 120.0));
            last_h = p.height_m;
            follow += 1;
        }
        if t > 147.1 && t < 160.0 {
            assert!(placed_of(&run, t, 1).is_none(), "t = {t}: vehicle 1 drawn");
            let dseg = to_segment((p.cx, p.cy), last_seen, (600.0, 300.0));
            assert!(dseg <= TOL, "t = {t}: {dseg} m off the segment");
            blend += 1;
        }
        let i = k.partition_point(|kf| kf.t <= t).saturating_sub(1).min(k.len() - 2);
        if t > k[0].t && t < k[k.len() - 1].t && !scalars_between(p, &k[i], &k[i + 1]) {
            violations += 1;
        }
    }
    println!("gate7: {held} held frames, {follow} following, {blend} blending, {violations} violations");
    assert_eq!(violations, 0);

    // One-frame steps just before and just after each keyframe in the window.
    let mut worst: f64 = 0.0;
    for kf in &k {
        let Some(n) = times.iter().rposition(|&t| t <= kf.t) else { continue };
        if n < 1 || n + 1 >= times.len() {
            continue;
        }
        let step = |a: usize, b: usize| (poses[b].cx - poses[a].cx, poses[b].cy - poses[a].cy);
        let (s0, s1) = (step(n - 1, n), step(n, n + 1));
        let dd = (s1.0 - s0.0).hypot(s1.1 - s0.1);
        println!("gate7 keyframe t = {}: steps differ by {dd:.5} m", kf.t);
        worst = worst.max(dd);
    }
    println!("gate7 largest step change at a keyframe: {worst:.5} m");
    assert!(worst <= 0.1);
}

// ── Gates 8 and 9: through the GPU ───────────────────────────────────────────

fn options(fcd: Option<PathBuf>, from: Option<f64>, to: Option<f64>, w: u32, h: u32) -> RenderOptions {
    RenderOptions {
        project: root().join("scratch/urban_grid"),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd,
        from,
        to,
        speedup: None,
        fps: 30,
        width: w,
        height: h,
    }
}

/// Phase 1 gate 3's method: the weighted centroid of the pixels that differ from the empty
/// frame.
fn centroid(frame: &[u8], empty: &[u8], w: usize, h: usize) -> (f64, f64) {
    let (mut sx, mut sy, mut sw) = (0.0, 0.0, 0.0);
    for j in 0..h {
        for i in 0..w {
            let o = (j * w + i) * 4;
            let wt: u32 = (0..3)
                .map(|c| (frame[o + c] as i32 - empty[o + c] as i32).unsigned_abs())
                .sum();
            if wt > 0 {
                sx += (i as f64 + 0.5) * wt as f64;
                sy += (j as f64 + 0.5) * wt as f64;
                sw += wt as f64;
            }
        }
    }
    assert!(sw > 0.0, "no vehicle pixels");
    (sx / sw, sy / sw)
}

#[test]
#[ignore = "needs the fixture and the GPU"]
fn gate8_placement() {
    let (w, h) = (3840u32, 2160u32);
    let subset = root().join("scratch/derived/subset_g3_1.parquet");
    assert!(subset.is_file(), "{} (scripts/fixture.sh and --test gates)", subset.display());
    for (yaw, pitch, want) in [
        (0.0, 90.0, (2727.83, 549.92)),
        (90.0, 90.0, (1389.92, 272.17)),
        (30.0, 40.0, (2266.51, 635.05)),
    ] {
        let file = write_camera(
            &format!("gate8_{yaw}_{pitch}.toml"),
            &one(&format!(
                "{{ t = 64.100, x = -150.00, y = 500.00, height_m = 400.00, yaw_deg = {yaw:.2}, pitch_deg = {pitch:.2} }}"
            )),
        );
        let mut job = Job::prepare_with_camera(&options(Some(subset.clone()), Some(64.1), Some(65.1), w, h), &file)
            .expect("prepare_with_camera");
        let pose = job.pose_at(64.1).expect("a keyframed job");
        let b = job.boxes_at(64.1);
        let v = b.iter().find(|b| b.vehicle_id == 1).expect("vehicle 1 at 64.1");
        let predicted = project(&pose, w as f64, h as f64, [v.at.x, v.at.y, 0.8]);
        let frame = job.render_at(64.1).unwrap();
        let empty = job.render_empty().unwrap();
        let got = centroid(&frame, &empty, w as usize, h as usize);
        let err = (got.0 - predicted.0).hypot(got.1 - predicted.1);
        println!(
            "gate8 yaw {yaw} pitch {pitch}: placed ({:.3}, {:.3}); project ({:.2}, {:.2}); centroid ({:.2}, {:.2}); error {err:.2} px",
            v.at.x, v.at.y, predicted.0, predicted.1, got.0, got.1
        );
        close2(predicted, want, 0.1, "the prediction");
        assert!(err <= 6.0, "centroid error {err:.2} px > 6");
    }
}

#[test]
#[ignore = "needs the fixture and the GPU"]
fn gate9_old_lines_frame_the_same_ground() {
    let (w, h) = (1920usize, 1080usize);
    let mut ortho = Job::prepare(&options(None, None, None, w as u32, h as u32)).unwrap();
    let fit = *ortho.camera();
    let a = ortho.render_empty().unwrap();
    let from = ortho.clock.from;
    drop(ortho);
    let line = Keyframe {
        t: from,
        at: At::Centre(fit.cx, fit.cy),
        height_m: h as f64 * fit.k,
        yaw_deg: 0.0,
        pitch_deg: 90.0,
    }
    .format();
    println!("gate9 line: {line}");
    let file = write_camera("gate9.toml", &one(&line));
    let mut persp = Job::prepare_with_camera(&options(None, None, None, w as u32, h as u32), &file).unwrap();
    let b = persp.render_empty().unwrap();
    let px = |f: &[u8], i: usize, j: usize| {
        let o = (j * w + i) * 4;
        [f[o], f[o + 1], f[o + 2]]
    };
    let (mut differ, mut off_edge) = (0usize, 0usize);
    for j in 0..h {
        for i in 0..w {
            if px(&a, i, j) == px(&b, i, j) {
                continue;
            }
            differ += 1;
            let (mut road, mut bg) = (false, false);
            for jj in j.saturating_sub(1)..=(j + 1).min(h - 1) {
                for ii in i.saturating_sub(1)..=(i + 1).min(w - 1) {
                    let c = px(&a, ii, jj);
                    road |= c == scene::ROAD;
                    bg |= c == scene::BACKGROUND;
                }
            }
            if !(road && bg) {
                off_edge += 1;
            }
        }
    }
    let frac = differ as f64 / (w * h) as f64;
    println!(
        "gate9: {differ} pixels differ ({:.3} % of the frame), {off_edge} not on a road edge",
        100.0 * frac
    );
    assert_eq!(off_edge, 0, "a differing pixel off a road edge");
    assert!(frac <= 0.02, "{:.3} % > 2 %", 100.0 * frac);
}
