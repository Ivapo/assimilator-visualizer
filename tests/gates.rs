//! vis-001 Phase 1 exit gates 3 and 7, measured on the lossless readback at 3840×2160.
//!
//! They need the fixture (`scripts/fixture.sh`), so they are ignored by default:
//! `cargo test --test gates -- --ignored --test-threads=1 --nocapture`.
//!
//! A vehicle's pixels are those that differ from the same frame with no vehicles, each
//! weighted by its difference (sum of |ΔR|+|ΔG|+|ΔB|), so MSAA edges count by coverage
//! and neither yuv420p nor x264 enters the measurement.

use std::path::{Path, PathBuf};
use std::process::Command;

use assimilator_video::fcd::{self, Row};
use assimilator_video::place::Placement;
use assimilator_video::{Job, RenderOptions, inputs};

const W: u32 = 3840;
const H: u32 = 2160;
const TOL_PX: f64 = 2.0;
const K_MAX: f64 = 0.6;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn project() -> PathBuf {
    root().join("scratch/urban_grid")
}
fn run_fcd() -> PathBuf {
    project().join("fcd/baseline_42.parquet")
}
fn lengths_fcd() -> PathBuf {
    root().join("scratch/derived/lengths.parquet")
}

/// A single-vehicle subset of `src`, written by the fixture's derivation binary.
fn subset(src: &Path, vehicle: u64, tag: &str) -> PathBuf {
    let out = root().join(format!("scratch/derived/subset_{tag}_{vehicle}.parquet"));
    let st = Command::new(env!("CARGO_BIN_EXE_fcd-derive"))
        .args(["subset", "--input"])
        .arg(src)
        .arg("--output")
        .arg(&out)
        .args(["--vehicle", &vehicle.to_string()])
        .status()
        .expect("run fcd-derive");
    assert!(st.success(), "fcd-derive subset failed");
    out
}

/// Frame 0 of a render whose window starts at `from`, from `fcd_path`.
fn job(fcd_path: &Path, from: f64) -> Job {
    Job::prepare(&RenderOptions {
        project: project(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: Some(fcd_path.to_path_buf()),
        from: Some(from),
        to: Some(from + 1.0),
        speedup: None,
        fps: 30,
        width: W,
        height: H,
    })
    .expect("prepare")
}

struct Diff {
    /// (pixel centre x, pixel centre y, weight)
    px: Vec<(f64, f64, f64)>,
}

fn diff(frame: &[u8], empty: &[u8]) -> Diff {
    let mut px = Vec::new();
    for j in 0..H as usize {
        for i in 0..W as usize {
            let o = (j * W as usize + i) * 4;
            let w: u32 = (0..3)
                .map(|c| (frame[o + c] as i32 - empty[o + c] as i32).unsigned_abs())
                .sum();
            if w > 0 {
                px.push((i as f64 + 0.5, j as f64 + 0.5, w as f64));
            }
        }
    }
    Diff { px }
}

impl Diff {
    fn centroid(&self) -> (f64, f64) {
        let s: f64 = self.px.iter().map(|p| p.2).sum();
        assert!(s > 0.0, "no vehicle pixels");
        (
            self.px.iter().map(|p| p.0 * p.2).sum::<f64>() / s,
            self.px.iter().map(|p| p.1 * p.2).sum::<f64>() / s,
        )
    }

    /// Extent along the heading, in pixels: project each pixel centre onto the heading,
    /// sum weights in 1 px bins, normalise by the fullest bin (a full cross-section), and
    /// add the bins up. Partially covered end bins count by their coverage.
    fn extent_along(&self, heading_deg: f64) -> f64 {
        let h = heading_deg.to_radians();
        // World (sin h, cos h) is (sin h, −cos h) in pixels (y down).
        let (ux, uy) = (h.sin(), -h.cos());
        let (cx, cy) = self.centroid();
        let mut bins = std::collections::BTreeMap::<i64, f64>::new();
        for &(x, y, w) in &self.px {
            let t = (x - cx) * ux + (y - cy) * uy;
            *bins.entry(t.floor() as i64).or_default() += w;
        }
        let max = bins.values().cloned().fold(0.0, f64::max);
        bins.values().map(|v| v / max).sum()
    }
}

struct Fixture {
    rows: Vec<Row>,
    links: Vec<String>,
    placement: Placement,
}

fn fixture() -> Fixture {
    assert!(run_fcd().is_file(), "run scripts/fixture.sh first");
    let f = fcd::read_window(&run_fcd(), f64::NEG_INFINITY, f64::INFINITY).unwrap();
    let placement = Placement::new(inputs::load_network(&project(), "baseline").unwrap());
    Fixture {
        rows: f.rows,
        links: f.links,
        placement,
    }
}

impl Fixture {
    fn link(&self, r: &Row) -> &str {
        &self.links[r.link as usize]
    }
    fn len(&self, r: &Row) -> f64 {
        self.placement.link_length(self.link(r))
    }
    /// A row at 30–70 % of its link, after warm-up, matching `pred`.
    fn mid_link(&self, pred: impl Fn(&Row) -> bool) -> Row {
        *self
            .rows
            .iter()
            .find(|r| {
                let l = self.len(r);
                r.time > 60.0 && r.position > 0.3 * l && r.position < 0.7 * l && pred(r)
            })
            .expect("no matching mid-link row")
    }
    /// A row frozen at the end of its approach link during a junction transit (OQ-2):
    /// the next row of the same vehicle has the same link and position, the position is
    /// at least `L − length/2 − 0.1`, and a later row is on another link.
    fn junction_hold(&self) -> Row {
        let mut by_vehicle: Vec<&Row> = self.rows.iter().collect();
        by_vehicle.sort_by(|a, b| {
            a.vehicle_id
                .cmp(&b.vehicle_id)
                .then(a.time.total_cmp(&b.time))
        });
        for w in by_vehicle.windows(3) {
            let (a, b, c) = (w[0], w[1], w[2]);
            if a.vehicle_id != b.vehicle_id || b.vehicle_id != c.vehicle_id || a.time < 60.0 {
                continue;
            }
            if a.link == b.link
                && a.position == b.position
                && a.speed > 0.5
                && a.position >= self.len(a) - a.length / 2.0 - 0.1
                && c.link != a.link
            {
                return *a;
            }
        }
        panic!("no junction-hold row in the fixture");
    }
}

fn gate3_one(fx: &Fixture, label: &str, r: Row) -> f64 {
    let sub = subset(&run_fcd(), r.vehicle_id, "g3");
    let mut j = job(&sub, r.time);
    assert!(j.k() <= K_MAX, "k = {} > {K_MAX}", j.k());
    let frame = j.render_frame(0).unwrap();
    let empty = j.render_empty().unwrap();
    let got = diff(&frame, &empty).centroid();
    let link = fx.link(&r);
    let p = fx.placement.place(link, r.position, r.lane).unwrap();
    let want = j.camera().world_to_pixel(p.x, p.y);
    let err = ((got.0 - want.0).powi(2) + (got.1 - want.1).powi(2)).sqrt();
    let l = fx.len(&r);
    eprintln!(
        "gate3 {label}: vehicle {} t={} link {link} lane {} s={:.3} (L={:.3}, front overhang {:+.3} m) \
         centroid ({:.3},{:.3}) expected ({:.3},{:.3}) error {:.3} px  k={:.4}",
        r.vehicle_id,
        r.time,
        r.lane,
        r.position,
        l,
        r.position + r.length / 2.0 - l,
        got.0,
        got.1,
        want.0,
        want.1,
        err,
        j.k()
    );
    err
}

#[test]
#[ignore = "needs scripts/fixture.sh"]
fn gate3_position() {
    let fx = fixture();
    let lane0 = fx.mid_link(|r| r.lane == 0);
    let lane1 = fx.mid_link(|r| r.lane == 1);
    let hold = fx.junction_hold();
    let errs = [
        gate3_one(&fx, "lane0", lane0),
        gate3_one(&fx, "lane1", lane1),
        gate3_one(&fx, "junction-hold", hold),
    ];
    for e in errs {
        assert!(e <= TOL_PX, "centroid error {e:.3} px > {TOL_PX}");
    }
}

fn gate7_one(src: &Path, r: Row, want_m: f64, label: &str) -> f64 {
    let tag = if src == lengths_fcd() {
        "g7len"
    } else {
        "g7run"
    };
    let sub = subset(src, r.vehicle_id, tag);
    let mut j = job(&sub, r.time);
    assert!(j.k() <= K_MAX, "k = {} > {K_MAX}", j.k());
    let frame = j.render_frame(0).unwrap();
    let empty = j.render_empty().unwrap();
    let heading = j.boxes_at(r.time)[0].at.heading;
    let got = diff(&frame, &empty).extent_along(heading);
    let want = want_m / j.k();
    let err = (got - want).abs();
    eprintln!(
        "gate7 {label}: vehicle {} t={} heading {heading:.1} extent {got:.3} px, expected {want:.3} px \
         ({want_m} m), error {err:.3} px",
        r.vehicle_id, r.time
    );
    err
}

#[test]
#[ignore = "needs scripts/fixture.sh"]
fn gate7_box_length() {
    let fx = fixture();
    let a = fx.mid_link(|_| true);
    let b = fx.mid_link(|r| r.vehicle_id != a.vehicle_id);
    let short = fx.mid_link(|r| r.vehicle_id % 3 == 0);
    let long = fx.mid_link(|r| r.vehicle_id % 3 == 2);
    let errs = [
        gate7_one(&run_fcd(), a, fcd::DEFAULT_LENGTH, "run-fcd A"),
        gate7_one(&run_fcd(), b, fcd::DEFAULT_LENGTH, "run-fcd B"),
        gate7_one(&lengths_fcd(), short, 2.0, "derived 2.0 m"),
        gate7_one(&lengths_fcd(), long, 12.0, "derived 12.0 m"),
    ];
    for e in errs {
        assert!(e <= TOL_PX, "extent error {e:.3} px > {TOL_PX}");
    }
}

/// Regression check for gate 2 (§2.3 determinism). Every frame that shows a snapshot
/// with overlapping boxes (same link and lane, centres closer than their mean length) is
/// rendered twice in a row in one process, at the gate-2 settings (defaults,
/// 1920×1080), and the two readbacks must be identical. Overlapping boxes are where an
/// unstable draw order shows: they occur at approach ends, where a box frozen during its
/// junction transit (OQ-2) is reached by the next vehicle.
#[test]
#[ignore = "needs scripts/fixture.sh"]
fn determinism_overlap_frames() {
    let _ = fixture();
    let mut j = Job::prepare(&RenderOptions {
        project: project(),
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
    })
    .expect("prepare");
    let overlapping: Vec<bool> = j
        .fcd
        .snapshots
        .iter()
        .map(|s| {
            let rows = &j.fcd.rows[s.start..s.end];
            rows.iter().enumerate().any(|(i, a)| {
                rows[i + 1..].iter().any(|b| {
                    a.link == b.link
                        && a.lane == b.lane
                        && (a.position - b.position).abs() < (a.length + b.length) / 2.0
                })
            })
        })
        .collect();
    let frames: Vec<u64> = (0..j.clock.frames)
        .filter(|&n| {
            let t = j.clock.time_of(n);
            let i = j.fcd.snapshots.partition_point(|s| s.time <= t + 1e-6);
            i > 0 && overlapping[i - 1]
        })
        .collect();
    let snaps = overlapping.iter().filter(|&&o| o).count();
    let mut bad = Vec::new();
    for &n in &frames {
        let a = j.render_frame(n).unwrap();
        let b = j.render_frame(n).unwrap();
        if a != b {
            bad.push(n);
        }
    }
    eprintln!(
        "determinism: {snaps} overlap snapshots, {} frames rendered twice, {} differ {:?}",
        frames.len(),
        bad.len(),
        &bad[..bad.len().min(10)]
    );
    assert!(
        bad.is_empty(),
        "{} frames differ between two renders",
        bad.len()
    );
}
