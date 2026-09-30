//! vis-001 Phase 4 exit gates 4–10: the time slider, driven headless through
//! `ViewState::frame` and `view::slider`. Gates 4–8 run on constructed windows in plain
//! `cargo test`; gate 5's round trip over the fixture's snapshot times and gates 9–10 need
//! the fixture of `scripts/fixture.sh` and are ignored; run everything with
//! `cargo test --release --test slider -- --include-ignored --test-threads=1 --nocapture`.

use std::path::PathBuf;

use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::VehicleBox;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::view::slider::Bar;
use assimilator_video::view::state::{Fit, ViewInput, ViewState};

/// Distances in pixels or metres and times in seconds: the state is set by formulas.
const TOL: f64 = 1e-9;
const W: f64 = 1280.0;
const H: f64 = 720.0;
const DT: f64 = 1.0 / 60.0;
/// The fixture's window, as Phase 3 recorded it.
const FROM: f64 = 9.099999999999984;
const TO: f64 = 299.0999999999995;
/// `t(400)` on the constructed window `[0, 300]` at 1280 wide.
const T400: f64 = 92.3076923076923;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture() -> (Run, ViewState) {
    let run = run::load(&LoadOptions {
        project: root().join("scratch/urban_grid"),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: None,
        to: None,
    })
    .expect("the fixture loads (scripts/fixture.sh)");
    let fit = Fit::new(&run.strips, W as u32, H as u32);
    let snaps = run.fcd.snapshots.iter().map(|s| s.time).collect();
    let s = ViewState::new(run.from, run.to, snaps, fit);
    assert_eq!((s.from, s.to), (FROM, TO), "the fixture's window");
    (run, s)
}

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

/// The cursor (and the pointer) at `at`.
fn at(at: (f64, f64)) -> ViewInput {
    ViewInput {
        cursor: Some(at),
        pointer: Some(at),
        ..idle()
    }
}

fn pressed(p: (f64, f64)) -> ViewInput {
    ViewInput {
        press: true,
        ..at(p)
    }
}

fn released(p: (f64, f64)) -> ViewInput {
    ViewInput {
        release: true,
        ..at(p)
    }
}

fn clicked(p: (f64, f64)) -> ViewInput {
    ViewInput {
        press: true,
        release: true,
        ..at(p)
    }
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b} (tol {tol})");
}

fn no_boxes(_: f64) -> Vec<VehicleBox> {
    Vec::new()
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

// ── Gate 4: geometry ─────────────────────────────────────────────────────────

#[test]
fn gate4_geometry() {
    let b = Bar::new((W, H)).expect("a bar at 1280×720");
    assert_eq!((b.x0, b.x1, b.y_bar), (16.0, 1264.0, 706.0));
    assert!(b.hit((640.0, 692.0)) && b.hit((8.0, 720.0)));
    assert!(b.hit((1272.0, 720.0)) && !b.hit((1272.1, 706.0)));
    assert!(!b.hit((640.0, 691.9)) && !b.hit((7.9, 706.0)));
    println!("gate4 1280×720: x0 16, x1 1264, y_bar 706, hit [8, 1272] × [692, 720]");

    let b = Bar::new((64.0, 64.0)).expect("a bar at 64×64");
    assert_eq!(b.len(), 32.0);
    assert_eq!(Bar::new((63.0, 720.0)), None);
    assert_eq!(Bar::new((1280.0, 63.0)), None);

    let mut s = plain_state();
    s.t = T400;
    s.frame(
        &ViewInput {
            size: (1600.0, 900.0),
            ..idle()
        },
        no_boxes,
    );
    assert_eq!(s.t, T400, "a resize leaves t");
    let b = s.bar().expect("a bar at 1600×900");
    close(b.x1, 1584.0, TOL, "x1 at 1600");
    close(b.y_bar, 886.0, TOL, "y_bar at 900");
    let x = b.x(s.t, s.from, s.to);
    close(x, 498.4615384615385, TOL, "x(t) at 1600");
    println!("gate4 1600×900: x1 {}, y_bar {}, x(t) {x}", b.x1, b.y_bar);
}

// ── Gate 5: x ↔ t ────────────────────────────────────────────────────────────

#[test]
fn gate5_x_t() {
    let b = Bar::new((W, H)).unwrap();
    assert_eq!(b.t(16.0, FROM, TO), FROM);
    assert_eq!(b.t(1264.0, FROM, TO), TO);
    assert_eq!(b.x(FROM, FROM, TO), 16.0);
    assert_eq!(b.x(TO, FROM, TO), 1264.0);
    let mid = b.t(640.0, FROM, TO);
    close(mid, 154.09999999999974, TOL, "fixture t(640)");
    println!("gate5 fixture window: ends exact both ways, t(640) = {mid}");
    for t in [0.0, 150.0, 300.0] {
        close(
            b.t(b.x(t, 0.0, 300.0), 0.0, 300.0),
            t,
            TOL,
            "constructed round trip",
        );
    }
    close(b.t(640.0, 0.0, 300.0), 150.0, TOL, "constructed t(640)");
}

#[test]
#[ignore = "needs the fixture"]
fn gate5_round_trip_fixture() {
    let (_run, s) = fixture();
    let b = Bar::new((W, H)).unwrap();
    let times: Vec<f64> = s
        .snaps
        .iter()
        .copied()
        .filter(|&t| (s.from..=s.to).contains(&t))
        .collect();
    assert_eq!(times.len(), 291, "snapshot times in the window");
    let mut worst: f64 = 0.0;
    for &t in &times {
        let e = (b.t(b.x(t, s.from, s.to), s.from, s.to) - t).abs();
        worst = worst.max(e);
        assert!(e <= TOL, "t(x({t})) off by {e}");
    }
    println!(
        "gate5 round trip: {} snapshot times, worst {worst:e} s",
        times.len()
    );
}

// ── Gate 6: clamping and the pointer ─────────────────────────────────────────

#[test]
fn gate6_clamp_pointer() {
    let mut s = plain_state();
    s.frame(&pressed((640.0, 706.0)), no_boxes);
    assert!(s.scrub.is_some());
    s.frame(&at((0.0, 706.0)), no_boxes);
    assert_eq!(s.t, 0.0, "x = 0");
    s.frame(&at((1280.0, 706.0)), no_boxes);
    assert_eq!(s.t, 300.0, "x = 1280");
    let off = |x: f64| ViewInput {
        pointer: Some((x, 706.0)),
        ..idle()
    };
    s.frame(&off(-50.0), no_boxes);
    assert_eq!(s.t, 0.0, "x = −50, off the window");
    s.frame(&off(2000.0), no_boxes);
    assert_eq!(s.t, 300.0, "x = 2000, off the window");
    s.frame(
        &ViewInput {
            cursor: Some((400.0, 706.0)),
            ..idle()
        },
        no_boxes,
    );
    close(s.t, T400, TOL, "cursor only");
    let held = s.t;
    s.frame(&idle(), no_boxes);
    assert_eq!(s.t, held, "neither: t holds");
    println!(
        "gate6: 0, 300, 0, 300 exactly; cursor only {}; neither holds",
        s.t
    );
}

// ── Gate 7: the bar owns its input ───────────────────────────────────────────

#[test]
fn gate7_bar_owns_input() {
    // One box under (640, 706).
    let on_bar = |_: f64| vec![vbox(1, 0.0, -346.0, 90.0, 4.5)];
    let mut s = plain_state();
    s.frame(&clicked((640.0, 706.0)), on_bar);
    assert_eq!(s.follow, None, "nothing picked");
    close(s.t, 150.0, TOL, "click on the bar");
    assert_eq!(s.press, None);

    let mut s = plain_state();
    s.frame(&pressed((640.0, 706.0)), on_bar);
    s.frame(&at((740.0, 706.0)), on_bar);
    assert_eq!((s.cx, s.cy), (0.0, 0.0), "no pan along the bar");
    close(s.t, 174.03846153846155, TOL, "t(740)");
    s.frame(&at((740.0, 606.0)), on_bar);
    assert_eq!((s.cx, s.cy), (0.0, 0.0), "no pan off the bar");
    close(s.t, 174.03846153846155, TOL, "t follows x only");
    s.frame(&released((740.0, 606.0)), on_bar);
    assert_eq!(s.follow, None);

    // One box under (640, 691), just above the hit area.
    let above = |_: f64| vec![vbox(1, 0.0, -331.0, 90.0, 4.5)];
    let mut s = plain_state();
    s.frame(&clicked((640.0, 691.0)), above);
    assert_eq!(
        s.follow.map(|f| f.vehicle_id),
        Some(1),
        "picked above the bar"
    );

    let mut s = plain_state();
    s.frame(&pressed((640.0, 691.0)), above);
    s.frame(&at((740.0, 691.0)), above);
    close(s.cx, -100.0, TOL, "pan x");
    close(s.cy, 0.0, TOL, "pan y");
    s.frame(&at((740.0, 706.0)), above);
    close(s.cx, -100.0, TOL, "pan onto the bar x");
    close(s.cy, 15.0, TOL, "pan onto the bar y");
    assert_eq!(s.t, 0.0, "a pan never scrubs");
    println!("gate7 pan onto the bar: centre ({}, {})", s.cx, s.cy);

    let scroll = |p: (f64, f64)| ViewInput {
        scroll_lines: 3.0,
        ..at(p)
    };
    let mut s = plain_state();
    s.frame(&scroll((640.0, 706.0)), no_boxes);
    assert_eq!(s.k, 1.0, "scroll over the bar");
    s.frame(&pressed((640.0, 706.0)), no_boxes);
    s.frame(&scroll((640.0, 300.0)), no_boxes);
    assert_eq!(s.k, 1.0, "scroll during a scrub");
    let mut s = plain_state();
    s.frame(&scroll((640.0, 691.0)), no_boxes);
    close(s.k, 1.1f64.powi(-3), TOL, "scroll above the bar");
}

// ── Gate 8: playback across a drag ───────────────────────────────────────────

#[test]
fn gate8_playback() {
    let mut s = plain_state();
    s.playing = true;
    s.frame(&pressed((400.0, 706.0)), no_boxes);
    for i in 0..30 {
        s.frame(&at((400.0, 706.0)), no_boxes);
        close(s.t, T400, TOL, &format!("held frame {i}"));
        assert!(!s.playing, "paused while held");
        assert!(s.readout().contains("paused"), "{}", s.readout());
    }
    s.frame(&released((400.0, 706.0)), no_boxes);
    assert!(s.playing, "resumes on release");
    s.frame(&at((400.0, 706.0)), no_boxes);
    close(s.t, 92.32435897435897, TOL, "the frame after release");
    println!("gate8: held at {T400}, then {}", s.t);

    let mut s = plain_state();
    s.frame(&pressed((400.0, 706.0)), no_boxes);
    s.frame(&released((400.0, 706.0)), no_boxes);
    assert!(!s.playing, "paused before, paused after");

    let mut s = plain_state();
    s.playing = true;
    s.frame(&pressed((640.0, 706.0)), no_boxes);
    s.frame(&at((1264.0, 706.0)), no_boxes);
    s.frame(&released((1264.0, 706.0)), no_boxes);
    assert_eq!(s.t, 300.0, "released at the end");
    assert!(!s.playing, "paused at to");

    let mut s = plain_state();
    s.playing = true;
    s.frame(&clicked((640.0, 706.0)), no_boxes);
    close(s.t, 150.0, TOL, "a click");
    assert!(s.playing, "a click keeps playing");

    let mut s = plain_state();
    s.playing = true;
    s.frame(&pressed((400.0, 706.0)), no_boxes);
    let mut keys = at((400.0, 706.0));
    keys.pressed.space = true;
    keys.pressed.left = true;
    keys.pressed.right = true;
    s.frame(&keys, no_boxes);
    close(s.t, T400, TOL, "space, ← and → ignored");
    assert_eq!(s.scrub, Some(true), "the resume unchanged");
    let mut plus = at((400.0, 706.0));
    plus.pressed.plus = true;
    s.frame(&plus, no_boxes);
    assert_eq!(s.speed(), 2.0, "+ during a scrub");
    s.frame(&released((400.0, 706.0)), no_boxes);
    assert!(s.playing, "resumes after the ignored keys");

    let mut s = plain_state();
    s.playing = true;
    s.frame(&pressed((400.0, 706.0)), no_boxes);
    s.frame(
        &ViewInput {
            size: (63.0, 720.0),
            ..at((400.0, 706.0))
        },
        no_boxes,
    );
    assert_eq!(s.scrub, None, "no bar ends the scrub");
    close(s.t, T400, TOL, "t held when the bar goes");
    assert!(s.playing, "playing resumes when the bar goes");
}
