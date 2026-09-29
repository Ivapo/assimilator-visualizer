//! vis-001 exit gates measured through the library. Phase 2 gate 3 (placement) and gate
//! 11 (Phase 1 gate 7, box length) are measured on the lossless readback at 3840×2160;
//! gates 6–10 (motion) through `Job::boxes_at` and `Job::motion_report` at every frame.
//!
//! They need the fixture (`scripts/fixture.sh`), so they are ignored by default:
//! `cargo test --test gates -- --ignored --test-threads=1 --nocapture`.
//!
//! A vehicle's pixels are those that differ from the same frame with no vehicles, each
//! weighted by its difference (sum of |ΔR|+|ΔG|+|ΔB|), so MSAA edges count by coverage
//! and neither yuv420p nor x264 enters the measurement.

use std::path::{Path, PathBuf};
use std::process::Command;

use std::collections::HashMap;

use assimilator_config::types::{LaneIdx, LinkId};
use assimilator_core::network_data::RoutePair;
use assimilator_core::spatial_conflict::interpolate_pos;
use assimilator_video::fcd::{self, Row};
use assimilator_video::motion::{How, Piece};
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
/// The run's FCD without `vehicle_class` and `vehicle_length`, as a pre-asm-020 file.
fn no_length_fcd() -> PathBuf {
    root().join("scratch/derived/no_class_length.parquet")
}

/// Whether the Parquet file at `path` has a column `name`.
fn has_column(path: &Path, name: &str) -> bool {
    let f = std::fs::File::open(path).expect("open FCD");
    parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(f)
        .expect("read FCD")
        .schema()
        .field_with_name(name)
        .is_ok()
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

/// The gate-3 junction case, computed from the FCD rows and `NetworkData` with the
/// §2.8.2 formula and `spatial_conflict::interpolate_pos` called directly, not through
/// `src/motion.rs`: vehicle 1 at the row nearest t = 64.1 s. Returns the row time, the
/// expected world point, and the distance along the turn path and its length.
fn junction_expected(fx: &Fixture) -> (f64, (f64, f64), f64, f64) {
    let mut rows: Vec<Row> = fx
        .rows
        .iter()
        .filter(|r| r.vehicle_id == 1)
        .copied()
        .collect();
    rows.sort_by(|a, b| a.time.total_cmp(&b.time));
    let at = rows
        .iter()
        .position(|r| (r.time - 64.1).abs() < 1e-3)
        .expect("vehicle 1 has a row at 64.1 s");
    // D: the first link change after `at`; A: the last row before D below the frozen value.
    let d = (at + 1..rows.len())
        .find(|&k| rows[k].link != rows[k - 1].link)
        .expect("a link change after 64.1 s");
    let p_f = rows[d - 1].position;
    let a = (0..d - 1)
        .rev()
        .take_while(|&j| rows[j].link == rows[d - 1].link)
        .find(|&j| rows[j].position < p_f)
        .expect("an anchor");
    let (ra, rd) = (rows[a], rows[d]);
    let (la, lb) = (fx.link(&ra).to_string(), fx.link(&rd).to_string());
    let (entry, dep) = (rows[d - 1].lane, rd.lane);
    eprintln!(
        "gate3 junction span: vehicle 1 rows {:.1} s on {la} … {:.1} s on {lb}, lanes {entry} → {dep}",
        ra.time, rd.time
    );
    let nd = &fx.placement.data;
    let n = nd.link_to_node(&LinkId(la.clone())).unwrap().clone();
    let m = match nd.resolve_route_pair(&n, &LinkId(la.clone()), &LinkId(lb.clone())) {
        Some(RoutePair::Junction(m)) => m,
        other => panic!("expected a junction movement, got {other:?}"),
    };
    let path = nd
        .lane_turn_path(&n, &m.id, LaneIdx(entry), LaneIdx(dep))
        .expect("per-lane turn path");
    let l_a = nd.link_length(&LinkId(la.clone()));
    let l1 = l_a - ra.position;
    let g = l1 + path.length + rd.position;
    let span = &rows[a..=d];
    let integral = |upto: usize| -> f64 {
        (1..=upto)
            .map(|k| (span[k - 1].speed + span[k].speed) / 2.0 * (span[k].time - span[k - 1].time))
            .sum()
    };
    let i_total = integral(span.len() - 1);
    let i_t = integral(at - a);
    let dist = g * i_t / i_total;
    eprintln!(
        "gate3 junction span: G = {g:.3} m, I = {i_total:.3} m, r = {:.3}; at t = {:.1}: d = {dist:.3} m",
        g / i_total,
        rows[at].time
    );
    let x = dist - l1;
    assert!(
        x > 0.0 && x < path.length,
        "t = 64.1 s is not on the turn path (x = {x})"
    );
    (
        rows[at].time,
        interpolate_pos(&path.path, x),
        x,
        path.length,
    )
}

#[test]
#[ignore = "needs scripts/fixture.sh"]
fn gate3_position() {
    let fx = fixture();
    let lane0 = fx.mid_link(|r| r.lane == 0);
    let lane1 = fx.mid_link(|r| r.lane == 1);
    let mut errs = vec![
        gate3_one(&fx, "lane0", lane0),
        gate3_one(&fx, "lane1", lane1),
    ];
    // In a junction: the rendered centroid against the independently computed point.
    let (t, want_w, x, len) = junction_expected(&fx);
    let sub = subset(&run_fcd(), 1, "g3");
    let mut j = job(&sub, t);
    assert!(j.k() <= K_MAX, "k = {} > {K_MAX}", j.k());
    let frame = j.render_frame(0).unwrap();
    let empty = j.render_empty().unwrap();
    let got = diff(&frame, &empty).centroid();
    let want = j.camera().world_to_pixel(want_w.0, want_w.1);
    let err = ((got.0 - want.0).powi(2) + (got.1 - want.1).powi(2)).sqrt();
    let r = fx
        .rows
        .iter()
        .find(|r| r.vehicle_id == 1 && r.time == t)
        .unwrap();
    let p1 = fx.placement.place(fx.link(r), r.position, r.lane).unwrap();
    eprintln!(
        "gate3 junction: vehicle 1 t={t} on the turn path {x:.2} m of {len:.2} m, {:.2} m from \
         Phase 1's frozen point; centroid ({:.3},{:.3}) expected ({:.3},{:.3}) error {err:.3} px",
        ((want_w.0 - p1.x).powi(2) + (want_w.1 - p1.y).powi(2)).sqrt(),
        got.0,
        got.1,
        want.0,
        want.1
    );
    errs.push(err);
    for e in errs {
        assert!(e <= TOL_PX, "centroid error {e:.3} px > {TOL_PX}");
    }
}

fn gate7_one(src: &Path, r: Row, want_m: f64, label: &str) -> f64 {
    let tag = if src == lengths_fcd() {
        "g7len"
    } else if src == no_length_fcd() {
        "g7nolen"
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
    // (a) the engine's own column; (b) the derived lengths; (c) no column, the fallback.
    assert!(
        has_column(&run_fcd(), "vehicle_length"),
        "run's FCD has no vehicle_length"
    );
    assert!(has_column(&lengths_fcd(), "vehicle_length"));
    assert!(!has_column(&no_length_fcd(), "vehicle_length"));
    assert!(!has_column(&no_length_fcd(), "vehicle_class"));
    let errs = [
        gate7_one(&run_fcd(), a, 4.5, "run-fcd column A"),
        gate7_one(&run_fcd(), b, 4.5, "run-fcd column B"),
        gate7_one(&lengths_fcd(), short, 2.0, "derived 2.0 m"),
        gate7_one(&lengths_fcd(), long, 12.0, "derived 12.0 m"),
        gate7_one(
            &no_length_fcd(),
            a,
            fcd::DEFAULT_LENGTH,
            "no column A (fallback)",
        ),
        gate7_one(
            &no_length_fcd(),
            b,
            fcd::DEFAULT_LENGTH,
            "no column B (fallback)",
        ),
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

fn default_job() -> Job {
    Job::prepare(&RenderOptions {
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
    .expect("prepare")
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Gates 6–10 (Phase 2), through the library at every frame of the default render.
#[test]
#[ignore = "needs scripts/fixture.sh"]
fn gate6_10_motion() {
    let _ = fixture();
    let j = default_job();
    let rep = j.motion_report();
    let pl = &j.placement;
    let links = &j.fcd.links;
    let h = j.clock.speedup / j.clock.fps as f64;
    let rows_of: HashMap<u64, &[Row]> = j
        .fcd
        .vehicles
        .iter()
        .map(|v| (v.vehicle_id, v.rows.as_slice()))
        .collect();
    let mut spans_of: HashMap<u64, Vec<usize>> = HashMap::new();
    for (k, s) in rep.spans.iter().enumerate() {
        spans_of.entry(s.vehicle_id).or_default().push(k);
    }
    let off = |link: u32, lane: u32| pl.lane_offset(&links[link as usize], lane);
    let lanes_of: HashMap<&str, usize> = pl
        .network
        .links
        .iter()
        .map(|l| (l.id.0.as_str(), l.lanes.len()))
        .collect();
    let frames: Vec<f64> = (0..j.clock.frames).map(|n| j.clock.time_of(n)).collect();

    // ---- Gates 6, 9 and 10: every frame, and every consecutive pair.
    let mut prev: HashMap<u64, assimilator_video::render::VehicleBox> = HashMap::new();
    let (mut worst_step, mut worst_step_at) = (0.0_f64, (0u64, 0.0_f64, false));
    let (mut exc_link, mut exc_span, mut lat_step) = (0.0_f64, 0.0_f64, 0.0_f64);
    let (mut over_bound, mut odo_back, mut pairs_checked) = (Vec::new(), 0usize, 0usize);
    let (mut overlaps, mut overlap_frames, mut drawn, mut drawn_mismatch) =
        (0usize, 0usize, 0usize, 0usize);
    for (fi, &t) in frames.iter().enumerate() {
        let boxes = j.boxes_at(t);
        // Gate 10.
        let want: Vec<u64> = j
            .fcd
            .vehicles
            .iter()
            .filter(|v| v.rows[0].time - 1e-6 <= t && t <= v.rows[v.rows.len() - 1].time + 1e-6)
            .map(|v| v.vehicle_id)
            .collect();
        let got: Vec<u64> = boxes.iter().map(|b| b.vehicle_id).collect();
        if got != want {
            drawn_mismatch += 1;
        }
        drawn += boxes.len();
        // Gate 9.
        // (piece, lane on a link) → (distance along, length) of each box on it.
        type Key = (Piece, Option<usize>);
        let mut groups: HashMap<Key, Vec<(f64, f64)>> = HashMap::new();
        for b in &boxes {
            let lane = match b.track.piece {
                Piece::Link(l) => {
                    let name = links[l as usize].as_str();
                    let n = lanes_of[name];
                    (0..n as u32)
                        .min_by(|&x, &y| {
                            (pl.lane_offset(name, x) - b.track.lateral)
                                .abs()
                                .total_cmp(&(pl.lane_offset(name, y) - b.track.lateral).abs())
                        })
                        .map(|x| x as usize)
                }
                _ => None,
            };
            groups
                .entry((b.track.piece, lane))
                .or_default()
                .push((b.track.along, b.length));
        }
        let mut here = 0;
        for g in groups.values() {
            for (i, a) in g.iter().enumerate() {
                for b in &g[i + 1..] {
                    if (a.0 - b.0).abs() < (a.1 + b.1) / 2.0 {
                        here += 1;
                    }
                }
            }
        }
        overlaps += here;
        overlap_frames += (here > 0) as usize;
        // Gate 6, against the previous frame.
        if fi > 0 {
            let t0 = frames[fi - 1];
            for b in &boxes {
                let Some(a) = prev.get(&b.vehicle_id) else {
                    continue;
                };
                pairs_checked += 1;
                let rows = rows_of[&b.vehicle_id];
                let lo = rows.iter().rposition(|r| r.time <= t0 + 1e-9).unwrap_or(0);
                let hi = rows
                    .iter()
                    .position(|r| r.time >= t - 1e-9)
                    .unwrap_or(rows.len() - 1);
                let around = &rows[lo..=hi.max(lo)];
                let span = spans_of.get(&b.vehicle_id).and_then(|ks| {
                    ks.iter()
                        .map(|&k| &rep.spans[k])
                        .find(|s| s.t_a < t - 1e-9 && t0 + 1e-9 < s.t_d)
                });
                let v_ref = around
                    .iter()
                    .map(|r| match span {
                        Some(s) if r.time >= s.t_a - 1e-9 && r.time <= s.t_d + 1e-9 => {
                            r.speed * s.r
                        }
                        _ => r.speed,
                    })
                    .fold(0.0, f64::max);
                let mut d_off = 0.0_f64;
                let mut dt = f64::INFINITY;
                for w in around.windows(2) {
                    dt = dt.min(w[1].time - w[0].time);
                    if w[0].link == w[1].link && w[0].lane != w[1].lane {
                        d_off = d_off
                            .max((off(w[0].link, w[0].lane) - off(w[1].link, w[1].lane)).abs());
                    }
                }
                if let Some(s) = span {
                    let a_id = links.iter().position(|l| *l == s.a).unwrap() as u32;
                    let b_id = links.iter().position(|l| *l == s.b).unwrap() as u32;
                    match s.path_departure_lane {
                        Some(pd) => {
                            d_off = d_off.max((off(b_id, s.departure_lane) - off(b_id, pd)).abs())
                        }
                        None => {
                            d_off = d_off
                                .max(off(a_id, s.entry_lane).abs())
                                .max(off(b_id, s.departure_lane).abs())
                        }
                    }
                }
                if !dt.is_finite() {
                    dt = 1.0;
                }
                let lambda = 1.5 * d_off * h / dt;
                let step = ((b.at.x - a.at.x).powi(2) + (b.at.y - a.at.y).powi(2)).sqrt();
                let bound = v_ref * h + lambda + 0.05;
                if step > bound {
                    over_bound.push((b.vehicle_id, t, step, bound));
                }
                if step > worst_step {
                    worst_step = step;
                    worst_step_at = (b.vehicle_id, t, span.is_some());
                }
                let dodo = b.track.odo - a.track.odo;
                if dodo < -1e-9 {
                    odo_back += 1;
                }
                let exc = dodo - v_ref * h;
                if span.is_some() {
                    exc_span = exc_span.max(exc);
                } else {
                    exc_link = exc_link.max(exc);
                }
                if a.track.piece == b.track.piece {
                    lat_step = lat_step.max((b.track.lateral - a.track.lateral).abs());
                }
            }
        }
        prev = boxes.into_iter().map(|b| (b.vehicle_id, b)).collect();
    }
    eprintln!(
        "gate6: {pairs_checked} frame pairs; largest step {worst_step:.3} m (vehicle {}, t = {:.3}, in span: {}); \
         largest longitudinal excess over v_ref·h {exc_link:.4} m along links, {exc_span:.4} m in spans; \
         largest lateral step {lat_step:.4} m; odo decreases {odo_back}; Δs<0 intervals {}; over the bound {}",
        worst_step_at.0,
        worst_step_at.1,
        worst_step_at.2,
        rep.backward,
        over_bound.len()
    );
    for o in over_bound.iter().take(10) {
        eprintln!(
            "gate6 over: vehicle {} t={:.3} step {:.4} bound {:.4}",
            o.0, o.1, o.2, o.3
        );
    }
    eprintln!(
        "gate9: {overlaps} overlapping pairs in {overlap_frames} of {} frames",
        frames.len()
    );
    eprintln!(
        "gate10: {drawn} vehicle-frames; {drawn_mismatch} frames whose drawn set differs from §2.8.5"
    );

    // ---- Gate 7: one-frame speed at each sample of an unclamped along-link interval.
    let odo_at = |v: u64, t: f64| {
        j.boxes_at(t)
            .into_iter()
            .find(|b| b.vehicle_id == v)
            .expect("drawn")
            .track
            .odo
    };
    let (mut e_start, mut e_end, mut measured, mut over7) = (0.0_f64, 0.0_f64, 0usize, 0usize);
    for iv in rep.intervals.iter().filter(|i| !i.clamped && !i.backward) {
        measured += 1;
        let s =
            ((odo_at(iv.vehicle_id, iv.t0 + h) - odo_at(iv.vehicle_id, iv.t0)) / h - iv.v0).abs();
        let e =
            ((odo_at(iv.vehicle_id, iv.t1) - odo_at(iv.vehicle_id, iv.t1 - h)) / h - iv.v1).abs();
        e_start = e_start.max(s);
        e_end = e_end.max(e);
        over7 += (s > 0.25 || e > 0.25) as usize;
    }
    eprintln!(
        "gate7: {} along-link intervals, {} clamped, {measured} measured; largest error {e_start:.3} m/s \
         after the start sample, {e_end:.3} m/s before the end sample; {over7} over 0.25 m/s",
        rep.intervals.len(),
        rep.clamped
    );

    // ---- Gate 8: spans.
    let mut r: Vec<f64> = rep.spans.iter().map(|s| s.r).collect();
    let mut rp: Vec<f64> = rep.spans.iter().map(|s| s.r_prime).collect();
    let r_med = median(&mut r);
    let rp_med = median(&mut rp);
    let mut cont = 0.0_f64;
    for s in &rep.spans {
        let rows = rows_of[&s.vehicle_id];
        for (t, probe) in [(s.t_a, s.t_a), (s.t_d, s.t_d - 1e-9)] {
            let row = rows.iter().find(|r| r.time == t).unwrap();
            let p = pl
                .place(&links[row.link as usize], row.position, row.lane)
                .unwrap();
            let b = j
                .boxes_at(probe)
                .into_iter()
                .find(|b| b.vehicle_id == s.vehicle_id)
                .unwrap();
            cont = cont.max(((b.at.x - p.x).powi(2) + (b.at.y - p.y).powi(2)).sqrt());
        }
    }
    for s in rep.spans.iter().filter(|s| s.how != How::LanePath) {
        eprintln!(
            "gate8 span {:?}: vehicle {} {} → {} entry lane {} FCD departure lane {} path departure lane {:?}",
            s.how, s.vehicle_id, s.a, s.b, s.entry_lane, s.departure_lane, s.path_departure_lane
        );
    }
    let e_resid: Vec<f64> = rep.spans.iter().map(|s| s.g - s.shortfall - s.i).collect();
    eprintln!(
        "gate8: {} spans — step 1 {}, step 2 {}, step 3 {}, transparent {}, no movement {}; \
         r {:.3}–{:.3} median {r_med:.3}; r′ {:.3}–{:.3} median {rp_med:.3}; out of band {}; \
         residual e {:.2}–{:.2} m; largest end discontinuity {:.2e} m; lane slides along {} entry {} departure {}",
        rep.spans.len(),
        rep.lane_path,
        rep.engine_lane,
        rep.centreline,
        rep.transparent,
        rep.no_movement,
        r[0],
        r[r.len() - 1],
        rp[0],
        rp[rp.len() - 1],
        rep.out_of_band,
        e_resid.iter().cloned().fold(f64::INFINITY, f64::min),
        e_resid.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        cont,
        rep.slides_along,
        rep.slides_entry,
        rep.slides_departure,
    );

    assert!(
        over_bound.is_empty(),
        "gate6: {} steps over the bound",
        over_bound.len()
    );
    assert_eq!(odo_back, 0, "gate6: odo decreased");
    assert_eq!(over7, 0, "gate7: {over7} intervals over 0.25 m/s");
    assert_eq!(rep.no_movement, 0, "gate8: spans with no movement");
    assert_eq!(rep.out_of_band, 0, "gate8: spans out of band");
    assert!(cont <= 1e-3, "gate8: span end discontinuity {cont} m");
    assert_eq!(overlaps, 0, "gate9: overlapping pairs");
    assert_eq!(drawn_mismatch, 0, "gate10: drawn set differs");
}

/// For gate 12's close-up: the pixel of the junction nearest the centre of the network,
/// at 3840×2160 with the default camera fit.
#[test]
#[ignore = "needs scripts/fixture.sh"]
fn gate12_centre_junction_pixel() {
    let _ = fixture();
    let j = Job::prepare(&RenderOptions {
        project: project(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: Some(150.0),
        to: Some(151.0),
        speedup: None,
        fps: 30,
        width: W,
        height: H,
    })
    .expect("prepare");
    let net = &j.placement.network;
    let ids: Vec<_> = net.junctions.iter().map(|jc| jc.node_id.clone()).collect();
    let nodes: Vec<_> = net.nodes.iter().filter(|n| ids.contains(&n.id)).collect();
    let cx = nodes.iter().map(|n| n.x).sum::<f64>() / nodes.len() as f64;
    let cy = nodes.iter().map(|n| n.y).sum::<f64>() / nodes.len() as f64;
    let c = nodes
        .iter()
        .min_by(|a, b| {
            ((a.x - cx).powi(2) + (a.y - cy).powi(2))
                .total_cmp(&((b.x - cx).powi(2) + (b.y - cy).powi(2)))
        })
        .unwrap();
    let (px, py) = j.camera().world_to_pixel(c.x, c.y);
    eprintln!(
        "gate12: centre junction {} at ({:.1}, {:.1}) px of {W}×{H}",
        c.id.0, px, py
    );
}
