//! vis-001 Phase 6 exit gates 6–8: boxes stacked at one point and heading, drawn through
//! the `--camera` path. `synthetic` (gates 6 and 7) builds its scene here and needs neither
//! a fixture nor the engine (§2.12.1). `urban_grid_sample` (gate 8) needs the fixture of
//! `scripts/fixture.sh`. Both render through the GPU and are ignored; run them with
//! `cargo test --release --test ties -- --include-ignored --test-threads=1 --nocapture`.
//!
//! `urban_grid_sample` writes its raw frames to `TIES_WRITE=<dir>`, or compares against
//! those in `TIES_BASELINE=<dir>`; with neither set it prints one hash per frame.

use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use assimilator_video::camera::Pose;
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::scene::{Camera, Strip};
use assimilator_video::{Job, RenderOptions};

/// The stack's heading, degrees: Midtown frame 363's.
const HEADING: f64 = 209.23;
/// The stack's ids (Midtown frame 363's) and speeds, one speed bin each.
const STACK_IDS: [u64; 4] = [72, 126, 130, 180];
const STACK_SPEEDS: [f64; 4] = [1.0, 4.0, 7.0, 11.0];
/// One speed per bin (`scene::SPEED_BINS`).
const SPEEDS: [f64; 5] = [1.0, 4.0, 7.0, 11.0, 20.0];
const FILLERS: usize = 500;
/// Tie frames rendered per case, each after one churn frame.
const REPS: usize = 24;
const POOL: usize = 1024;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

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

/// One road strip, 60 m long along the stack's heading through (0, 0), sampled every 1 m,
/// 3.5 m each side.
fn strips() -> Vec<Strip> {
    let h = HEADING.to_radians();
    let (dx, dy) = (h.sin(), h.cos());
    let (rx, ry) = (h.cos(), -h.sin());
    let mut s = Strip {
        left: vec![],
        right: vec![],
    };
    for i in 0..=60 {
        let t = i as f64 - 30.0;
        let (x, y) = (t * dx, t * dy);
        s.right.push([x + rx * 3.5, y + ry * 3.5]);
        s.left.push([x - rx * 3.5, y - ry * 3.5]);
    }
    vec![s]
}

/// The four-box stack at (0, 0) with these lengths, among [`FILLERS`] other boxes, in
/// `vehicle_id` order. Filler `k` (ids from 1, skipping the stack's) sits at
/// (8·(k mod 25) − 92, 8·⌊k/25⌋ − 76), heading 37·k mod 360, 4.5 m, speeds cycling the bins.
fn stack(lengths: [f64; 4]) -> Vec<VehicleBox> {
    let mut out: Vec<VehicleBox> = (0..4)
        .map(|i| vbox(STACK_IDS[i], 0.0, 0.0, HEADING, lengths[i], STACK_SPEEDS[i]))
        .collect();
    out.extend(
        (1u64..)
            .filter(|id| !STACK_IDS.contains(id))
            .take(FILLERS)
            .enumerate()
            .map(|(k, id)| {
                let x = 8.0 * (k % 25) as f64 - 92.0;
                let y = 8.0 * (k / 25) as f64 - 76.0;
                vbox(id, x, y, (37 * k % 360) as f64, 4.5, SPEEDS[k % 5])
            }),
    );
    out.sort_by_key(|b| b.vehicle_id);
    out
}

/// The same frame with the stack's lower three moved off screen, ranks kept: the top box
/// alone.
fn top_alone(tie: &[VehicleBox]) -> Vec<VehicleBox> {
    tie.iter()
        .map(|b| match b.vehicle_id {
            72 | 126 | 130 => VehicleBox {
                at: Placed {
                    x: 9000.0,
                    y: 9000.0,
                    ..b.at
                },
                ..*b
            },
            _ => *b,
        })
        .collect()
}

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// A churn frame: each box in slice order is dropped when a draw `% 5 == 0`, and otherwise
/// kept with the speed of the next draw, so the draws change bins between tie frames.
fn churn(rng: &mut Lcg, base: &[VehicleBox]) -> Vec<VehicleBox> {
    let mut out = vec![];
    for b in base {
        if rng.next() % 5 != 0 {
            let speed = SPEEDS[(rng.next() % 5) as usize];
            out.push(VehicleBox { speed, ..*b });
        }
    }
    out
}

fn hash(v: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// Pixels (RGBA) that differ between two frames of one size.
fn pixels_apart(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

/// The three poses, all looking at (0, 0) with yaw 30, and their image sizes.
fn poses() -> [(&'static str, Camera, Pose); 3] {
    let cam = |width, height| Camera {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        width,
        height,
    };
    let pose = |height_m, pitch_deg| Pose {
        cx: 0.0,
        cy: 0.0,
        height_m,
        yaw_deg: 30.0,
        pitch_deg,
    };
    [
        ("close", cam(1280, 720), pose(12.0, 40.0)),
        ("far", cam(1920, 1080), pose(400.0, 55.0)),
        ("down", cam(1280, 720), pose(12.0, 90.0)),
    ]
}

// ── Gates 6 and 7: the synthetic stack ───────────────────────────────────────

#[test]
#[ignore = "renders through the GPU"]
fn synthetic() {
    let strips = strips();
    let stacks = [("equal", [4.5; 4]), ("mixed", [4.5, 4.1, 5.2, 4.8])];
    let mut failures = vec![];
    for (sname, lengths) in stacks {
        let tie = stack(lengths);
        for (pname, cam, pose) in poses() {
            let mut r = Renderer::new_perspective(&strips, cam, POOL, &pose, None, None)
                .expect("the synthetic scene builds");
            let mut rng = Lcg(42);
            let mut seen = BTreeSet::new();
            let mut first = None;
            for _ in 0..REPS {
                r.render(&churn(&mut rng, &tie)).expect("a churn frame");
                let f = r.render(&tie).expect("a tie frame");
                seen.insert(hash(&f));
                first.get_or_insert(f);
            }
            let first = first.unwrap();
            let case = format!("{sname}-{pname}");
            let mut line = format!(
                "TIES {case}: {} distinct of {REPS}; first {:016x}",
                seen.len(),
                hash(&first)
            );
            if seen.len() != 1 {
                failures.push(format!("{case}: {} distinct tie frames", seen.len()));
            }
            // The mixed stack is checked for determinism only (OQ-14).
            if sname == "equal" {
                let alone = r.render(&top_alone(&tie)).expect("the top box alone");
                let apart = pixels_apart(&first, &alone);
                line += &format!(
                    "; apart from the top box alone {apart} px (alone {:016x})",
                    hash(&alone)
                );
                if apart != 0 {
                    failures.push(format!("{case}: {apart} px apart from the top box alone"));
                }
            }
            println!("{line}");
        }
    }
    assert!(failures.is_empty(), "gate 7: {}", failures.join("; "));
}

// ── Gate 8: urban_grid's flight, sampled ─────────────────────────────────────

/// Differing pixels that are not edge pixels: a pixel is an edge pixel when its 3×3
/// neighbourhood in `base`, clipped to the frame, is not all one colour.
fn not_on_edges(base: &[u8], new: &[u8], w: usize, h: usize) -> (usize, usize) {
    let px = |f: &[u8], x: usize, y: usize| {
        let i = 4 * (y * w + x);
        [f[i], f[i + 1], f[i + 2], f[i + 3]]
    };
    let (mut differ, mut interior) = (0, 0);
    for y in 0..h {
        for x in 0..w {
            let c = px(base, x, y);
            if c == px(new, x, y) {
                continue;
            }
            differ += 1;
            let ys = y.saturating_sub(1)..=(y + 1).min(h - 1);
            let one = ys
                .flat_map(|yy| (x.saturating_sub(1)..=(x + 1).min(w - 1)).map(move |xx| (xx, yy)))
                .all(|(xx, yy)| px(base, xx, yy) == c);
            interior += one as usize;
        }
    }
    (differ, interior)
}

#[test]
#[ignore = "needs the fixture of scripts/fixture.sh and renders through the GPU"]
fn urban_grid_sample() {
    let write = std::env::var_os("TIES_WRITE").map(PathBuf::from);
    let baseline = std::env::var_os("TIES_BASELINE").map(PathBuf::from);
    assert!(
        write.is_none() || baseline.is_none(),
        "set TIES_WRITE or TIES_BASELINE, not both"
    );
    if let Some(d) = &write {
        std::fs::create_dir_all(d).expect("TIES_WRITE is a directory");
    }
    let o = RenderOptions {
        project: root().join("scratch/urban_grid"),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: None,
        to: None,
        speedup: None,
        fps: 30,
        width: 1920,
        height: 1080,
    };
    let mut job = Job::prepare_with_camera(&o, &root().join("tests/flight.toml"))
        .expect("the fixture loads (scripts/fixture.sh)");
    assert_eq!(job.clock.frames, 8700, "the flight's frame count");
    let (w, h) = (o.width as usize, o.height as usize);
    let (mut frames_changed, mut differ, mut interior) = (0, 0, 0);
    for n in (0..=8600).step_by(100) {
        let f = job.render_frame(n).expect("a frame");
        let name = format!("frame-{n:04}.rgba");
        let mut line = format!("SAMPLE frame {n}: {:016x}", hash(&f));
        if let Some(d) = &write {
            std::fs::write(d.join(&name), &f).expect("the raw frame is written");
        }
        if let Some(d) = &baseline {
            let base = std::fs::read(d.join(&name)).expect("the baseline frame is there");
            let (dn, inn) = not_on_edges(&base, &f, w, h);
            line += &format!("; {dn} px differ, {inn} not on an edge");
            frames_changed += (dn > 0) as usize;
            differ += dn;
            interior += inn;
        }
        println!("{line}");
    }
    if baseline.is_some() {
        println!(
            "SAMPLE total: {frames_changed} of 87 frames differ, {differ} px, {interior} not on an edge"
        );
        assert_eq!(interior, 0, "gate 8: differing pixels off the edges");
    }
}
