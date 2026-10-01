//! vis-001 Phase 3 exit gates 2 (for `view`) and 4–9: the view state driven by scripted
//! input, with no window and no GPU. The gates that need the fixture of
//! `scripts/fixture.sh` are ignored; run them with
//! `cargo test --release --test view -- --ignored --test-threads=1 --nocapture`.
//! Constructed picks and the keyframe line run in plain `cargo test`.

use std::path::PathBuf;
use std::process::Command;

use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::VehicleBox;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::view::state::{
    At, Fit, Follow, K_MIN, Keyframe, ViewInput, ViewState, is_speed_down, is_speed_up, pick,
};

/// Distances in metres and times in seconds: the state is set by formulas.
const TOL: f64 = 1e-9;
const W: f64 = 1280.0;
const H: f64 = 720.0;
const DT: f64 = 1.0 / 60.0;

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
    (run, s)
}

fn idle() -> ViewInput {
    ViewInput {
        size: (W, H),
        ..Default::default()
    }
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} vs {b} (tol {tol})");
}

/// Gate 9's round trip on the state's current line.
fn check_line(s: &ViewState) {
    let line = s.keyframe_line();
    let kf = Keyframe::parse(&line).unwrap_or_else(|e| panic!("{line}: {e}"));
    assert_eq!(kf.format(), line);
    close(kf.t, s.t, 0.0005 + TOL, "line t");
    close(kf.height_m, s.size.1 * s.k, 0.005 + TOL, "line height_m");
    match (kf.at, s.follow) {
        (At::Follow(id), Some(f)) => {
            assert!(f.drawn, "{line}: follow printed on hold");
            assert_eq!(id, f.vehicle_id);
        }
        (At::Centre(x, y), f) => {
            assert!(!matches!(f, Some(Follow { drawn: true, .. })), "{line}");
            close(x, s.cx, 0.005 + TOL, "line x");
            close(y, s.cy, 0.005 + TOL, "line y");
        }
        (At::Follow(_), None) => panic!("{line}: follow without a follow"),
    }
}

/// One frame, then gate 9's check.
fn step(s: &mut ViewState, run: &Run, input: ViewInput) -> Option<String> {
    let line = s.frame(&input, |t| run.boxes_at(t));
    check_line(s);
    line
}

fn pixel_of(s: &ViewState, (x, y): (f64, f64)) -> (f64, f64) {
    (
        s.size.0 / 2.0 + (x - s.cx) / s.k,
        s.size.1 / 2.0 - (y - s.cy) / s.k,
    )
}

fn click(s: &mut ViewState, run: &Run, at: (f64, f64)) {
    step(
        s,
        run,
        ViewInput {
            cursor: Some(at),
            press: true,
            release: true,
            ..idle()
        },
    );
}

fn placed_of(run: &Run, t: f64, id: u64) -> Option<(f64, f64)> {
    run.boxes_at(t)
        .iter()
        .find(|b| b.vehicle_id == id)
        .map(|b| (b.at.x, b.at.y))
}

// ── Gate 4: pan ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "needs the fixture"]
fn gate4_pan() {
    let (run, s0) = fixture();
    let kf = s0.fit.k;
    close(kf, 1240.0 / 720.0, TOL, "k_fit");
    println!(
        "gate4 k_fit {kf}, fit centre ({}, {}), rect {:?}",
        s0.cx, s0.cy, s0.fit.rect
    );

    // A drag of (+100, −40) logical pixels.
    let mut s = s0.clone();
    let p0 = (400.0, 300.0);
    let w0 = s.world(p0);
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some(p0),
            press: true,
            ..idle()
        },
    );
    for i in 1..=10 {
        let f = i as f64 / 10.0;
        let c = (p0.0 + 100.0 * f, p0.1 - 40.0 * f);
        step(
            &mut s,
            &run,
            ViewInput {
                cursor: Some(c),
                ..idle()
            },
        );
    }
    let p1 = (p0.0 + 100.0, p0.1 - 40.0);
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some(p1),
            release: true,
            ..idle()
        },
    );
    let (dx, dy) = (s.cx - s0.cx, s.cy - s0.cy);
    println!("gate4 drag: centre moved ({dx:.6}, {dy:.6}) m");
    close(dx, -100.0 * kf, TOL, "drag dx");
    close(dy, -40.0 * kf, TOL, "drag dy");
    close(dx, -172.22, 0.005, "drag dx (spec)");
    close(dy, -68.89, 0.005, "drag dy (spec)");
    let w1 = s.world(p1);
    close(w1.0, w0.0, TOL, "world x under cursor");
    close(w1.1, w0.1, TOL, "world y under cursor");
    assert!(s.follow.is_none() && s.press.is_none());

    // Holding D for 0.5 s of real time.
    let mut s = s0.clone();
    let mut sum = 0.0;
    for _ in 0..30 {
        let mut i = ViewInput { dt: DT, ..idle() };
        i.held.d = true;
        step(&mut s, &run, i);
        sum += DT;
    }
    let dx = s.cx - s0.cx;
    println!("gate4 D for {sum} s: cx moved {dx:.6} m");
    close(dx, 0.5 * sum * W * kf, TOL, "D pan");
    close(dx, 551.11, 0.005, "D pan (spec)");
    close(s.cy, s0.cy, TOL, "D pan cy");

    // A drag that would pass the fitted rectangle stops at its edge.
    let mut s = s0.clone();
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some((100.0, 360.0)),
            press: true,
            ..idle()
        },
    );
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some((1200.0, 360.0)),
            ..idle()
        },
    );
    println!("gate4 far drag: cx {} (rect x0 {})", s.cx, s.fit.rect.0);
    assert_eq!(s.cx, s.fit.rect.0, "the far drag stops at the west edge");
    close(s.cy, s0.cy, TOL, "far drag cy");
}

// ── Gate 5: zoom ─────────────────────────────────────────────────────────────

#[test]
#[ignore = "needs the fixture"]
fn gate5_zoom() {
    let (run, s0) = fixture();
    let kf = s0.fit.k;
    let c = (800.0, 300.0);
    let p = s0.world(c);
    println!("gate5 cursor {c:?} over world ({:.6}, {:.6})", p.0, p.1);
    close(p.0, 575.56, 0.005, "world x (spec)");
    close(p.1, 403.33, 0.005, "world y (spec)");
    let scroll = |s: &mut ViewState, lines: f64| {
        step(
            s,
            &run,
            ViewInput {
                cursor: Some(c),
                scroll_lines: lines,
                ..idle()
            },
        );
    };

    let mut s = s0.clone();
    scroll(&mut s, 5.0);
    close(s.k, kf * 1.1f64.powf(-5.0), TOL, "k after 5 lines");
    let q = s.world(c);
    close(q.0, p.0, TOL, "fixed point x, 5 lines");
    close(q.1, p.1, TOL, "fixed point y, 5 lines");
    println!("gate5 5 lines: k {} centre ({:.6}, {:.6})", s.k, s.cx, s.cy);

    let mut s = s0.clone();
    scroll(&mut s, 100.0);
    assert_eq!(s.k, K_MIN, "100 lines up give exactly k_min");
    let q = s.world(c);
    close(q.0, p.0, TOL, "fixed point x at k_min");
    close(q.1, p.1, TOL, "fixed point y at k_min");
    println!("gate5 k_min: centre ({:.6}, {:.6})", s.cx, s.cy);
    scroll(&mut s, -100.0);
    assert_eq!(s.k, 2.0 * kf, "then 100 lines down give exactly k_max");
    close(s.k, 3.4444, 0.00005, "k_max (spec)");
    let q = s.world(c);
    close(q.0, p.0, TOL, "fixed point x at k_max");
    close(q.1, p.1, TOL, "fixed point y at k_max");
    println!("gate5 k_max: centre ({:.10}, {:.10})", s.cx, s.cy);
    close(s.cx, 24.44, 0.005, "k_max centre x (spec)");
    close(s.cy, 196.67, 0.005, "k_max centre y (spec)");
    let (x0, y0, x1, y1) = s.fit.rect;
    assert!(
        x0 < s.cx && s.cx < x1 && y0 < s.cy && s.cy < y1,
        "inside the rectangle"
    );

    let mut a = s0.clone();
    let mut b = s0.clone();
    step(
        &mut a,
        &run,
        ViewInput {
            cursor: Some(c),
            scroll_pixels: 40.0,
            ..idle()
        },
    );
    step(
        &mut b,
        &run,
        ViewInput {
            cursor: Some(c),
            scroll_lines: 2.0,
            ..idle()
        },
    );
    close(a.k, b.k, TOL, "40 px = 2 lines: k");
    close(a.cx, b.cx, TOL, "40 px = 2 lines: cx");
    close(a.cy, b.cy, TOL, "40 px = 2 lines: cy");
}

// ── Gate 6: clock ────────────────────────────────────────────────────────────

fn press(f: impl FnOnce(&mut ViewInput)) -> ViewInput {
    let mut i = idle();
    f(&mut i);
    i
}

#[test]
#[ignore = "needs the fixture"]
fn gate6_clock() {
    let (run, s0) = fixture();
    assert_eq!(s0.t, s0.from);
    assert!(!s0.playing);
    assert_eq!(s0.speed(), 1.0);

    let mut s = s0.clone();
    step(&mut s, &run, press(|i| i.pressed.space = true));
    assert!(s.playing);
    for _ in 0..600 {
        step(&mut s, &run, ViewInput { dt: DT, ..idle() });
    }
    println!(
        "gate6 600 frames of 1/60 s: t moved {:e} from 10 s",
        s.t - s0.t - 10.0
    );
    close(s.t - s0.t, 10.0, TOL, "600 frames at 1×");
    let t = s.t;
    step(&mut s, &run, ViewInput { dt: 0.5, ..idle() });
    close(s.t - t, 0.1, TOL, "one frame of 0.5 s (the cap)");

    let mut s = s0.clone();
    for _ in 0..6 {
        step(&mut s, &run, press(|i| i.pressed.plus = true));
    }
    assert_eq!(s.speed(), 64.0, "6 presses of + reach 64×");
    step(&mut s, &run, press(|i| i.pressed.plus = true));
    assert_eq!(s.speed(), 64.0, "+ stays at 64×");
    let mut s = s0.clone();
    for _ in 0..3 {
        step(&mut s, &run, press(|i| i.pressed.minus = true));
    }
    assert_eq!(s.speed(), 0.125, "3 presses of − reach 1/8×");
    step(&mut s, &run, press(|i| i.pressed.minus = true));
    assert_eq!(s.speed(), 0.125, "− stays at 1/8×");
    assert!(s.readout().contains("×1/8"), "{}", s.readout());

    let mut s = s0.clone();
    step(&mut s, &run, press(|i| i.pressed.plus = true));
    step(&mut s, &run, press(|i| i.pressed.plus = true));
    assert_eq!(s.speed(), 4.0);
    step(&mut s, &run, press(|i| i.pressed.space = true));
    let t = s.t;
    step(&mut s, &run, ViewInput { dt: DT, ..idle() });
    close(s.t - t, 1.0 / 15.0, TOL, "4× for 1/60 s");

    // Frame steps pause.
    let t = s.t;
    assert!(s.playing);
    step(&mut s, &run, press(|i| i.pressed.right = true));
    assert!(!s.playing, "→ pauses");
    close(s.t - t, 1.0 / 30.0, TOL, "→");
    step(&mut s, &run, press(|i| i.pressed.left = true));
    close(s.t, t, TOL, "←");

    // Sample steps.
    let mut s = s0.clone();
    s.t = 60.100000000000584;
    let shift = |right: bool| {
        press(|i| {
            i.held.shift = true;
            if right {
                i.pressed.right = true
            } else {
                i.pressed.left = true
            }
        })
    };
    step(&mut s, &run, shift(true));
    assert_eq!(s.t, 61.1000000000006, "Shift+→");
    s.t = 60.100000000000584;
    step(&mut s, &run, shift(false));
    assert_eq!(s.t, 59.10000000000057, "Shift+←");
    let last = *s.snaps.last().unwrap();
    s.t = last;
    step(&mut s, &run, shift(true));
    assert_eq!(s.t, last, "Shift+→ at the last snapshot");
    println!(
        "gate6 sample steps: 60.100000000000584 → 61.1000000000006 / 59.10000000000057; last {last}"
    );

    // Playing into `to` pauses there; space restarts from `from`.
    let mut s = s0.clone();
    s.t = s.to - 0.05;
    step(&mut s, &run, press(|i| i.pressed.space = true));
    let mut n = 0;
    while s.playing {
        step(&mut s, &run, ViewInput { dt: DT, ..idle() });
        n += 1;
        assert!(n < 100);
    }
    assert_eq!(s.t, s.to, "pauses at to exactly");
    step(&mut s, &run, press(|i| i.pressed.space = true));
    assert!(s.playing);
    assert_eq!(s.t, s.from, "space at to restarts from from");
}

// ── Gate 7: pick ─────────────────────────────────────────────────────────────

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

#[test]
fn gate7_pick_constructed() {
    // k = 0.5 m per logical pixel: the radius is 4 m.
    let k = 0.5;
    // North-facing 4.5 m box: x in ±0.9, y in ±2.25.
    let a = vbox(1, 0.0, 0.0, 0.0, 4.5);
    assert_eq!(pick(&[a], (0.3, 1.0), k), Some(1), "inside");
    assert_eq!(
        pick(&[a], (0.9 + 7.9 * k, 0.0), k),
        Some(1),
        "7.9 px from the side"
    );
    assert_eq!(
        pick(&[a], (0.9 + 8.1 * k, 0.0), k),
        None,
        "8.1 px from the side"
    );
    assert_eq!(
        pick(&[a], (0.0, 2.25 + 7.9 * k), k),
        Some(1),
        "7.9 px from the end"
    );
    assert_eq!(
        pick(&[a], (0.0, -2.25 - 8.1 * k), k),
        None,
        "8.1 px from the end"
    );
    // East-facing: the length lies along x.
    let e = vbox(3, 100.0, 0.0, 90.0, 4.5);
    assert_eq!(
        pick(&[e], (100.0 + 2.25 + 7.9 * k, 0.0), k),
        Some(3),
        "east, 7.9 px"
    );
    assert_eq!(
        pick(&[e], (100.0 + 2.25 + 8.1 * k, 0.0), k),
        None,
        "east, 8.1 px"
    );
    assert_eq!(
        pick(&[e], (100.0, 0.9 + 7.9 * k), k),
        Some(3),
        "east, side 7.9 px"
    );
    // Overlapping: inside both, the higher id.
    let b = vbox(2, 0.5, 0.5, 30.0, 4.5);
    assert_eq!(pick(&[a, b], (0.2, 0.2), k), Some(2), "overlap");
    assert_eq!(
        pick(&[b, a], (0.2, 0.2), k),
        Some(2),
        "overlap, other order"
    );
    // Two at equal distance: the higher id.
    let l = vbox(5, -3.0, 50.0, 0.0, 4.5);
    let r = vbox(4, 3.0, 50.0, 0.0, 4.5);
    assert_eq!(pick(&[l, r], (0.0, 50.0), k), Some(5), "equal distance");
    assert_eq!(
        pick(&[r, l], (0.0, 50.0), k),
        Some(5),
        "equal distance, other order"
    );
    // Nearest wins over a higher id.
    assert_eq!(pick(&[l, r], (1.0, 50.0), k), Some(4), "nearest");
}

#[test]
#[ignore = "needs the fixture"]
fn gate7_pick_fixture() {
    let (run, s0) = fixture();
    let t = 64.1;
    let p = placed_of(&run, t, 1).expect("vehicle 1 drawn at 64.1 s");
    println!("gate7 vehicle 1 at t = {t}: ({:.3}, {:.3})", p.0, p.1);

    let mut s = s0.clone();
    s.t = t;
    let c = pixel_of(&s, p);
    click(&mut s, &run, c);
    assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1), "picked at k_fit");

    let mut s = s0.clone();
    s.t = t;
    s.k = K_MIN;
    s.cx = p.0;
    s.cy = p.1;
    click(&mut s, &run, (W / 2.0, H / 2.0));
    assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1), "picked at k_min");

    // A click with no box near changes nothing, including the follow.
    let before = s.follow;
    let far = pixel_of(&s, (p.0 + 1000.0 * K_MIN * 8.0, p.1));
    click(&mut s, &run, far);
    assert_eq!(s.follow, before);
}

// ── Gate 8: follow ───────────────────────────────────────────────────────────

#[test]
#[ignore = "needs the fixture"]
fn gate8_follow() {
    let (run, s0) = fixture();
    let (x0, y0, x1, y1) = s0.fit.rect;
    let mut s = s0.clone();
    s.t = 64.1;
    let p = placed_of(&run, s.t, 1).unwrap();
    let c = pixel_of(&s, p);
    click(&mut s, &run, c);
    assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1));
    close(s.cx, p.0, TOL, "centred x");
    close(s.cy, p.1, TOL, "centred y");

    step(&mut s, &run, press(|i| i.pressed.space = true));
    let (mut drawn, mut held, mut zooms) = (0, 0, 0);
    let mut last: Option<(f64, f64)> = None;
    let mut n = 0u32;
    while s.t < 150.0 {
        n += 1;
        let lines = if n.is_multiple_of(300) && zooms < 10 {
            zooms += 1;
            1.0
        } else {
            0.0
        };
        step(
            &mut s,
            &run,
            ViewInput {
                dt: DT,
                scroll_lines: lines,
                cursor: Some((100.0, 100.0)),
                ..idle()
            },
        );
        let f = s.follow.expect("still following");
        match placed_of(&run, s.t, 1) {
            Some(q) => {
                assert!(f.drawn);
                assert!(
                    last.is_none() || held == 0,
                    "drawn again after the hold while playing"
                );
                close(s.cx, q.0, TOL, "follow x");
                close(s.cy, q.1, TOL, "follow y");
                assert!(
                    x0 <= q.0 && q.0 <= x1 && y0 <= q.1 && q.1 <= y1,
                    "placed point in rect"
                );
                assert!(
                    s.readout().ends_with("  following 1 — Esc to stop"),
                    "{}",
                    s.readout()
                );
                last = Some(q);
                drawn += 1;
            }
            None => {
                assert!(!f.drawn);
                let l = last.expect("drawn before the hold");
                assert_eq!((s.cx, s.cy), l, "the camera holds the last centre");
                assert!(
                    s.readout()
                        .ends_with("  following 1 (not drawn) — Esc to stop"),
                    "{}",
                    s.readout()
                );
                assert!(s.keyframe_line().contains(" x = "), "{}", s.keyframe_line());
                held += 1;
            }
        }
    }
    assert_eq!(zooms, 10);
    close(
        s.k,
        s0.fit.k * 1.1f64.powf(-1.0).powi(10),
        1e-12,
        "k after 10 lines",
    );
    println!(
        "gate8 {drawn} frames drawn and centred, {held} frames held; t = {:.3}",
        s.t
    );
    println!("gate8 hold line: {}", s.keyframe_line());
    println!("gate8 readout: {}", s.readout());
    assert!(drawn > 0 && held > 0);

    // K on hold prints the held centre.
    let line = step(&mut s, &run, press(|i| i.pressed.k = true)).expect("K prints");
    assert_eq!(line, s.keyframe_line());
    assert!(line.contains(" x = ") && !line.contains("follow"), "{line}");

    // Stepping back, sample by sample, into its interval re-centres on it (its last row
    // is at 147.1 s; the camera holds until then).
    let held_at = (s.cx, s.cy);
    let mut steps = 0;
    loop {
        step(
            &mut s,
            &run,
            press(|i| {
                i.held.shift = true;
                i.pressed.left = true;
            }),
        );
        steps += 1;
        if placed_of(&run, s.t, 1).is_some() {
            break;
        }
        assert!(!s.follow.unwrap().drawn);
        assert_eq!((s.cx, s.cy), held_at, "still holding at t = {}", s.t);
        assert!(steps < 10);
    }
    let f = s.follow.unwrap();
    assert!(f.drawn, "t = {} is back in vehicle 1's interval", s.t);
    println!("gate8 {steps} Shift+← steps back to t = {}", s.t);
    let q = placed_of(&run, s.t, 1).unwrap();
    close(s.cx, q.0, TOL, "re-centred x");
    close(s.cy, q.1, TOL, "re-centred y");
    let line = step(&mut s, &run, press(|i| i.pressed.k = true)).unwrap();
    assert!(line.contains("follow = 1,"), "{line}");
    println!("gate8 back at t = {}: {line}", s.t);

    // A drag, a WASD pan and Esc each stop following.
    let centre = (W / 2.0, H / 2.0);
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some(centre),
            press: true,
            ..idle()
        },
    );
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some((centre.0 + 10.0, centre.1)),
            ..idle()
        },
    );
    assert!(s.follow.is_none(), "a drag stops following");
    step(
        &mut s,
        &run,
        ViewInput {
            cursor: Some((centre.0 + 10.0, centre.1)),
            release: true,
            ..idle()
        },
    );

    let refollow = |s: &mut ViewState| {
        let q = placed_of(&run, s.t, 1).unwrap();
        let c = pixel_of(s, q);
        click(s, &run, c);
        assert_eq!(s.follow.map(|f| f.vehicle_id), Some(1));
    };
    refollow(&mut s);
    step(
        &mut s,
        &run,
        press(|i| {
            i.held.w = true;
            i.dt = DT;
        }),
    );
    assert!(s.follow.is_none(), "W stops following");
    refollow(&mut s);
    step(&mut s, &run, press(|i| i.pressed.esc = true));
    assert!(s.follow.is_none(), "Esc stops following");
}

// ── Gate 11 fix: the speed keys by logical character ────────────────────────

/// `+` (Shift+`=` on a US layout, its own key elsewhere) and `=` both double the speed,
/// `-` halves it, by the character a key types rather than its position. The window maps
/// `Key::Character` through these; the keypad keys are matched by position.
#[test]
fn speed_keys_by_character() {
    for c in ["+", "="] {
        assert!(is_speed_up(c), "{c:?} speeds up");
        assert!(!is_speed_down(c), "{c:?} does not slow down");
    }
    assert!(is_speed_down("-"));
    assert!(!is_speed_up("-"));
    for c in ["", "k", "K", "0", "*", "_", "−", "++"] {
        assert!(
            !is_speed_up(c) && !is_speed_down(c),
            "{c:?} is not a speed key"
        );
    }

    // Pressed `+` goes back up the ladder from 1/8× (the finding: `−` reached 1/8× and
    // `+` could not return).
    let mut s = plain_state();
    let key = |plus: bool, minus: bool| ViewInput {
        pressed: assimilator_video::view::state::Pressed {
            plus,
            minus,
            ..Default::default()
        },
        ..idle()
    };
    for _ in 0..3 {
        s.frame(&key(false, true), |_| vec![]);
    }
    assert_eq!(s.speed(), 0.125);
    for _ in 0..3 {
        s.frame(&key(true, false), |_| vec![]);
    }
    assert_eq!(s.speed(), 1.0);
}

#[test]
fn readout_while_following() {
    let mut s = plain_state();
    s.t = 64.1;
    assert_eq!(s.readout(), "t 64.10 s [0.0–300.0]  ×1  paused");
    s.follow = Some(Follow {
        vehicle_id: 103,
        drawn: true,
    });
    assert_eq!(
        s.readout(),
        "t 64.10 s [0.0–300.0]  ×1  paused  following 103 — Esc to stop"
    );
    s.follow = Some(Follow {
        vehicle_id: 103,
        drawn: false,
    });
    assert_eq!(
        s.readout(),
        "t 64.10 s [0.0–300.0]  ×1  paused  following 103 (not drawn) — Esc to stop"
    );
}

// ── Gate 9: the keyframe line ────────────────────────────────────────────────

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

#[test]
fn gate9_keyframe_line() {
    let mut s = plain_state();
    s.t = 64.1;
    s.cx = 512.3;
    s.cy = -133.2;
    s.k = 1.0 / 3.0;
    assert_eq!(
        s.keyframe_line(),
        "{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );
    let line = s
        .frame(
            &ViewInput {
                pressed: assimilator_video::view::state::Pressed {
                    k: true,
                    ..Default::default()
                },
                ..idle()
            },
            |_| vec![],
        )
        .expect("K prints");
    assert_eq!(
        line,
        "{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );

    let mut s = plain_state();
    s.t = 200.0;
    s.k = 1.0 / 12.0;
    s.follow = Some(Follow {
        vehicle_id: 103,
        drawn: true,
    });
    assert_eq!(
        s.keyframe_line(),
        "{ t = 200.000, follow = 103, height_m = 60.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );
    s.follow = Some(Follow {
        vehicle_id: 103,
        drawn: false,
    });
    assert_eq!(
        s.keyframe_line(),
        "{ t = 200.000, x = 0.00, y = 0.00, height_m = 60.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );

    // Phase 3's lines still read, straight down and north up (vis-001 §2.11.4).
    for line in [
        "{ t = 64.100, x = 512.30, y = -133.20, height_m = 240.00 }",
        "{ t = 200.000, follow = 103, height_m = 60.00 }",
    ] {
        let kf = Keyframe::parse(line).unwrap();
        assert_eq!((kf.yaw_deg, kf.pitch_deg), (0.0, 90.0), "{line}");
    }
    let kf = Keyframe::parse("{ t = 200.000, follow = 103, height_m = 60.00 }").unwrap();
    assert_eq!(kf.at, At::Follow(103));

    // -0.00 is written and read back.
    let neg = Keyframe {
        t: 1.0,
        at: At::Centre(-0.001, 0.0),
        height_m: 10.0,
        yaw_deg: 0.0,
        pitch_deg: 90.0,
    };
    let line = neg.format();
    assert_eq!(
        line,
        "{ t = 1.000, x = -0.00, y = 0.00, height_m = 10.00, yaw_deg = 0.00, pitch_deg = 90.00 }"
    );
    assert_eq!(Keyframe::parse(&line).unwrap().format(), line);

    for bad in [
        // a missing key
        "{ t = 1.000, x = 1.00, height_m = 2.00 }",
        "{ t = 1.000, follow = 3 }",
        // an extra key
        "{ t = 1.000, x = 1.00, y = 2.00, height_m = 2.00, z = 1.00 }",
        "{ t = 1.000, follow = 3, height_m = 2.00, x = 1.00 }",
        // both x and follow
        "{ t = 1.000, x = 1.00, y = 2.00, follow = 3, height_m = 2.00 }",
        "{ t = 1.000, follow = 3, x = 1.00, y = 2.00, height_m = 2.00 }",
        // not the written shape
        "{ t = 1.000, x = 1.00, y = 2.00, height_m = 2.00 },",
        "{ t = inf, x = 1.00, y = 2.00, height_m = 2.00 }",
        "{ t = 1.000, follow = -3, height_m = 2.00 }",
        "",
    ] {
        assert!(Keyframe::parse(bad).is_err(), "accepted {bad:?}");
    }
}

// ── Gate 2 (view): input errors, before any window ───────────────────────────

fn view_cmd(fcd: &str, path: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_assimilator-video"));
    cmd.args(["view", "--project"])
        .arg(root().join("scratch/urban_grid"))
        .args(["--scenario", "baseline", "--seed", "42", "--fcd"])
        .arg(root().join(fcd));
    if let Some(p) = path {
        cmd.env("PATH", p);
    }
    let out = cmd.output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
#[ignore = "needs the fixture"]
fn gate2_view_input_errors() {
    for (label, fcd, path, want) in [
        (
            "missing-fcd",
            "scratch/derived/does_not_exist.parquet",
            None,
            "missing file",
        ),
        (
            "unknown-link",
            "scratch/derived/unknown_link.parquet",
            None,
            "",
        ),
        (
            "missing-fcd, no ffmpeg",
            "scratch/derived/does_not_exist.parquet",
            Some("/usr/bin:/bin"),
            "missing file",
        ),
    ] {
        let (code, out, err) = view_cmd(fcd, path);
        println!(
            "gate2 view {label}: exit {code}, stderr {:?}, stdout {:?}",
            err.trim_end(),
            out
        );
        assert_eq!(code, 1, "{label}");
        assert!(out.is_empty(), "{label}: stdout {out:?}");
        assert_eq!(err.lines().count(), 1, "{label}: stderr {err:?}");
        assert!(err.starts_with("error: "), "{label}: {err:?}");
        assert!(err.contains(want), "{label}: {err:?}");
        assert!(!err.contains("ffmpeg"), "{label}: {err:?}");
    }
}
