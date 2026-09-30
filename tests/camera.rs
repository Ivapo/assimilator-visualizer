//! vis-001 Phase 5 exit gates 4–12: the pose, the keyframe line and file, the flight and
//! `view`'s orbit and tilt. Gates 4, 6, 11 and 12 are headless and run in plain
//! `cargo test`. Those that need the fixture of `scripts/fixture.sh` or the GPU are
//! ignored; run everything with
//! `cargo test --release --test camera -- --include-ignored --test-threads=1 --nocapture`.

use assimilator_video::camera::{self, Pose, distance, project, ray_to_plane};
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::VehicleBox;
use assimilator_video::scene::Camera;
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
}
