//! vis-003 Phase 2 exit gates 5, 6, 7, 9 and 13's `H` (`specs/data_spec.md`): the HUD's
//! numbers and text on a synthetic FCD and on both fixtures, checked against the FCD read
//! straight, its line drawn through the GPU, and `H`. The headless ones without a fixture
//! run in plain `cargo test`. Those that need the fixtures of `scripts/fixture.sh` and
//! `scripts/fixture.sh midtown`, or the GPU, are ignored; run everything with
//! `cargo test --release --test hud -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 8 and 13's CLI cases are `scripts/gates-hud.sh`.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};

use arrow_array::{Float64Array, UInt64Array};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use assimilator_video::clock;
use assimilator_video::fcd::{Fcd, Row, Snapshot, VehicleRows};
use assimilator_video::hud;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::scene;
use assimilator_video::view::state::Follow;
use assimilator_video::view::{Fit, ViewInput, ViewState};
use assimilator_video::{Job, RenderOptions};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ── Gate 5: a synthetic run ──────────────────────────────────────────────────

/// Gate 5's FCD, with no file: snapshots at 1, 2, 3 and 5 s, none at 4 (a step with no
/// vehicle), as `(time, [(vehicle_id, speed)])`.
const SYNTHETIC: [(f64, &[(u64, f64)]); 4] = [
    (1.0, &[(1, 10.0), (2, 5.0)]),
    (2.0, &[(1, 0.0), (2, 5.0), (3, 2.5)]),
    (3.0, &[(1, 0.0), (3, 2.5)]),
    (5.0, &[(4, 13.8889)]),
];

fn synthetic() -> Fcd {
    let mut rows = Vec::new();
    let mut snapshots = Vec::new();
    for (time, snap) in SYNTHETIC {
        let start = rows.len();
        for &(vehicle_id, speed) in snap {
            rows.push(Row {
                time,
                vehicle_id,
                link: 0,
                lane: 0,
                position: 0.0,
                speed,
                length: 4.5,
            });
        }
        snapshots.push(Snapshot {
            time,
            start,
            end: rows.len(),
        });
    }
    let mut by_vehicle: BTreeMap<u64, Vec<Row>> = BTreeMap::new();
    for r in &rows {
        by_vehicle.entry(r.vehicle_id).or_default().push(*r);
    }
    let vehicles = by_vehicle
        .into_iter()
        .map(|(vehicle_id, rows)| VehicleRows { vehicle_id, rows })
        .collect();
    Fcd {
        links: vec!["L".into()],
        rows,
        snapshots,
        vehicles,
        file_span: (1.0, 5.0),
    }
}

/// Whether any vehicle is drawn at `t`, by the motion rule: `t_first − 1e-6 ≤ t ≤
/// t_last + 1e-6` (`rules/motion.md`).
fn drawn(spans: &[(f64, f64)], t: f64) -> bool {
    spans.iter().any(|&(a, b)| a - 1e-6 <= t && t <= b + 1e-6)
}

#[test]
fn gate5_synthetic_text() {
    let fcd = synthetic();
    let spans: Vec<(f64, f64)> = fcd
        .vehicles
        .iter()
        .map(|v| (v.rows[0].time, v.rows[v.rows.len() - 1].time))
        .collect();
    for (t, want) in [
        (0.5, "0:00 · 0 vehicles · – km/h"),
        (1.0, "0:01 · 2 vehicles · 27 km/h"),
        (1.5, "0:01 · 2 vehicles · 27 km/h"),
        (1.9999995, "0:02 · 3 vehicles · 9 km/h"),
        (2.5, "0:02 · 3 vehicles · 9 km/h"),
        (3.0, "0:03 · 2 vehicles · 5 km/h"),
        (3.5, "0:03 · 0 vehicles · – km/h"),
        (4.0, "0:04 · 0 vehicles · – km/h"),
        (5.0, "0:05 · 1 vehicle · 50 km/h"),
    ] {
        let (n, mean) = hud::numbers(&fcd, t, drawn(&spans, t));
        let text = hud::text(t, n, mean);
        println!("gate 5: {t}: {text:?}");
        assert_eq!(text, want, "at {t}");
    }
    // 1.25 m/s is 4.5 km/h exactly, and rounds up.
    assert_eq!(hud::numbers(&fcd, 3.0, true), (2, Some(1.25)));
    assert_eq!(1.25 * 3.6, 4.5);
    println!("gate 5: nine texts: PASS");
}

#[test]
fn gate5_clock() {
    for (t, want) in [
        (0.0, "0:00"),
        (0.9999995, "0:01"),
        (59.999, "0:59"),
        (60.0, "1:00"),
        (599.5, "9:59"),
        (3599.9999, "59:59"),
        (3600.0, "1:00:00"),
        (36000.0, "10:00:00"),
    ] {
        let c = hud::clock(t);
        println!("gate 5: clock({t}) = {c:?}");
        assert_eq!(c, want, "at {t}");
    }
    println!("gate 5: eight clocks: PASS");
}

// ── Gate 6: the fixtures, against the FCD read straight ──────────────────────

fn load(dir: &str, from: Option<f64>, to: Option<f64>) -> Run {
    run::load(&LoadOptions {
        project: root().join("scratch").join(dir),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from,
        to,
    })
    .unwrap()
}

/// The FCD read straight, with none of the crate's code: every time's row count and its
/// speeds summed in `vehicle_id` order, and every vehicle's first and last time.
struct Straight {
    /// `(time, rows, sum of speed)`, in time order.
    times: Vec<(f64, usize, f64)>,
    spans: Vec<(f64, f64)>,
}

fn straight(dir: &str) -> Straight {
    let path = root().join("scratch").join(dir).join("fcd/baseline_42.parquet");
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(&path).unwrap()).unwrap();
    let mask = ProjectionMask::columns(builder.parquet_schema(), ["time", "vehicle_id", "speed"]);
    let mut rows: Vec<(f64, u64, f64)> = Vec::new();
    for batch in builder.with_projection(mask).build().unwrap() {
        let batch = batch.unwrap();
        let col = |name: &str| batch.column_by_name(name).unwrap().clone();
        let (time, vid, speed) = (col("time"), col("vehicle_id"), col("speed"));
        let time = time.as_any().downcast_ref::<Float64Array>().unwrap();
        let vid = vid.as_any().downcast_ref::<UInt64Array>().unwrap();
        let speed = speed.as_any().downcast_ref::<Float64Array>().unwrap();
        for i in 0..batch.num_rows() {
            rows.push((time.value(i), vid.value(i), speed.value(i)));
        }
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut times: Vec<(f64, usize, f64)> = Vec::new();
    let mut spans: BTreeMap<u64, (f64, f64)> = BTreeMap::new();
    for &(t, v, s) in &rows {
        match times.last_mut() {
            Some(last) if last.0 == t => {
                last.1 += 1;
                last.2 += s;
            }
            _ => times.push((t, 1, s)),
        }
        let span = spans.entry(v).or_insert((t, t));
        span.1 = t;
    }
    Straight {
        times,
        spans: spans.into_values().collect(),
    }
}

/// §2.20.4–§2.20.8's text at `t`, from the FCD read straight.
fn expected(s: &Straight, t: f64) -> String {
    let secs = (t + 1e-6).floor().max(0.0) as u64;
    let clock = if secs >= 3600 {
        format!("{}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    };
    let k = s.times.partition_point(|x| x.0 <= t + 1e-6);
    let (n, speed) = match (drawn(&s.spans, t), k) {
        (true, k) if k > 0 => {
            let (_, n, sum) = s.times[k - 1];
            let kmh = (3.6 * (sum / n as f64) + 0.5).floor() as i64;
            (n, format!("{kmh} km/h"))
        }
        _ => (0, "– km/h".to_string()),
    };
    let noun = if n == 1 { "vehicle" } else { "vehicles" };
    format!("{clock} · {n} {noun} · {speed}")
}

/// Every frame of one render's window: the HUD's text through `Run::boxes_at` and
/// `Fcd::snapshot_at` against the FCD read straight.
fn gate6_every_frame(dir: &str, from: Option<f64>, to: Option<f64>, speedup: Option<f64>, frames: u64) {
    let run = load(dir, from, to);
    let s = straight(dir);
    let d = run.to - run.from;
    let speedup = speedup.unwrap_or_else(|| clock::default_speedup(d));
    let n_frames = clock::frame_count(d, 30, speedup);
    assert_eq!(n_frames, frames, "{dir}: frames");
    let (mut differ, mut empty_with_rows) = (0, 0);
    for n in 0..n_frames {
        let t = clock::frame_time(run.from, n, speedup, 30);
        let boxes = run.boxes_at(t);
        let (count, mean) = hud::numbers(&run.fcd, t, !boxes.is_empty());
        let got = hud::text(t, count, mean);
        let want = expected(&s, t);
        if got != want {
            if differ < 10 {
                println!("gate 6: {dir} frame {n} (t {t}): {got:?}, the FCD read straight {want:?}");
            }
            differ += 1;
        }
        if boxes.is_empty() && run.fcd.snapshot_at(t).is_some_and(|x| x.end > x.start) {
            empty_with_rows += 1;
        }
    }
    println!(
        "gate 6: {dir} from {:?} to {:?} at {speedup}x: {differ} of {n_frames} frames differ; \
         no box drawn with a non-empty snapshot: {empty_with_rows}",
        run.from, run.to
    );
    assert_eq!(differ, 0);
    assert_eq!(empty_with_rows, 0);
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate6_urban_grid_every_frame() {
    gate6_every_frame("urban_grid", None, None, None, 8700);
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_window_every_frame() {
    gate6_every_frame("midtown", Some(300.0), Some(360.0), Some(1.0), 1800);
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_whole_run_every_frame() {
    gate6_every_frame("midtown", None, None, None, 9000);
}

// ── Gate 13: `H` ─────────────────────────────────────────────────────────────

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

/// `input` with `h` pressed flips the flag and does exactly what `input` alone does.
fn step_h(s: &mut ViewState, input: ViewInput) {
    let mut without = s.clone();
    let line = without.frame(&input, no_boxes);
    let before = s.hud_shown;
    let mut with_h = input;
    with_h.pressed.h = true;
    assert_eq!(s.frame(&with_h, no_boxes), line, "the keyframe line");
    assert_eq!(s.hud_shown, !before, "h flips the flag");
    let mut flipped_back = s.clone();
    flipped_back.hud_shown = before;
    assert_eq!(flipped_back, without, "h changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.hud_shown;
    s.frame(&input, no_boxes);
    assert_eq!(s.hud_shown, before, "only h flips the flag");
}

#[test]
fn gate13_h() {
    let mut s = plain_state();
    assert!(!s.hud_shown, "off at new");
    let (buildings, see_through, streets, signals) = (
        s.buildings_shown,
        s.see_through,
        s.streets_shown,
        s.signals_shown,
    );
    step_h(&mut s, at((640.0, 360.0)));
    assert!(s.hud_shown);
    step_h(&mut s, at((640.0, 360.0)));
    assert!(!s.hud_shown);

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_h(&mut s, k);
    step_h(&mut s, at((640.0, 360.0)));
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
    step_h(&mut s, at((700.0, 330.0)));
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
    step_h(&mut s, at((460.0, 340.0)));
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
    step_h(&mut s, at((300.0, 706.0)));
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
        (s.buildings_shown, s.see_through, s.streets_shown, s.signals_shown),
        (buildings, see_through, streets, signals),
        "h never touches buildings_shown, see_through, streets_shown or signals_shown"
    );
    println!(
        "gate 13: h flips hud_shown and nothing else, through an orbit, a drag and a scrub: PASS"
    );
}

// ── Gates 6, 7 and 9: through the GPU ────────────────────────────────────────

fn opts(dir: &str, from: Option<f64>, to: Option<f64>, speedup: Option<f64>, w: u32, h: u32) -> RenderOptions {
    RenderOptions {
        project: root().join("scratch").join(dir),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from,
        to,
        speedup,
        fps: 30,
        width: w,
        height: h,
    }
}

/// A job set as `render` sets it by default (`src/main.rs`): streets, see-through,
/// signals, and the HUD when `hud`.
fn job(o: &RenderOptions, camera: Option<&Path>, buildings: Option<&Path>, hud: bool) -> Job {
    let mut j = Job::prepare_with(o, camera, buildings).unwrap();
    j.set_streets(true).unwrap();
    j.set_see_through(true);
    j.set_signals(true).unwrap();
    j.set_hud(hud).unwrap();
    j
}

fn count(f: &[u8], c: [u8; 3]) -> usize {
    f.chunks(4).filter(|p| p[..3] == c).count()
}

/// The text each listed frame drew, read back after it rendered.
fn gate6_sampled(o: &RenderOptions, want: &[(u64, &str)]) {
    let mut j = Job::prepare_with(o, None, None).unwrap();
    j.set_hud(true).unwrap();
    for &(n, text) in want {
        let t = j.clock.time_of(n);
        j.render_at(t).unwrap();
        let drew = j.hud_text();
        println!("gate 6: {} frame {n} (t {t:.4}): {drew:?}", o.project.display());
        assert_eq!(drew.as_deref(), Some(text), "frame {n}");
    }
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown); renders through the GPU"]
fn gate6_midtown_sampled_frames() {
    gate6_sampled(
        &opts("midtown", Some(300.0), Some(360.0), Some(1.0), 1920, 1080),
        &[
            (0, "5:00 · 184 vehicles · 17 km/h"),
            (3, "5:00 · 185 vehicles · 16 km/h"),
            (30, "5:01 · 185 vehicles · 16 km/h"),
            (450, "5:15 · 196 vehicles · 24 km/h"),
            (900, "5:30 · 201 vehicles · 20 km/h"),
            (1350, "5:45 · 206 vehicles · 23 km/h"),
            (1799, "5:59 · 203 vehicles · 18 km/h"),
        ],
    );
    println!("gate 6: Midtown's sampled frames: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); renders through the GPU"]
fn gate6_urban_grid_sampled_frames() {
    gate6_sampled(
        &opts("urban_grid", None, None, None, 1920, 1080),
        &[
            (0, "0:09 · 6 vehicles · 50 km/h"),
            (29, "0:10 · 6 vehicles · 50 km/h"),
            (600, "0:29 · 18 vehicles · 42 km/h"),
            (2700, "1:39 · 66 vehicles · 33 km/h"),
            (5400, "3:09 · 96 vehicles · 30 km/h"),
            (8699, "4:59 · 107 vehicles · 21 km/h"),
        ],
    );
    println!("gate 6: urban_grid's sampled frames: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); renders through the GPU"]
fn gate7_drawn_alone() {
    // (w, h, box, changed, exactly the fill, exactly black added)
    for (w, h, want_box, want_changed, want_fill, want_black) in [
        (1280, 720, [18, 18, 269, 46], 3915, 340, 941),
        (1920, 1080, [27, 27, 403, 69], 8384, 1264, 3087),
        (3840, 2160, [55, 55, 805, 137], 31453, 8163, 14795),
    ] {
        let o = opts("urban_grid", None, None, None, w, h);
        let mut j = job(&o, None, None, false);
        let off = j.render_at(150.0).unwrap();
        j.set_hud(true).unwrap();
        let on = j.render_at(150.0).unwrap();
        let text = j.hud_text();
        let b = j.hud_box().unwrap();
        let (mut changed, mut outside) = (0, 0);
        for (i, (p, q)) in off.chunks(4).zip(on.chunks(4)).enumerate() {
            if p != q {
                changed += 1;
                let (x, y) = ((i % w as usize) as u32, (i / w as usize) as u32);
                if !(b[0] <= x && x < b[2] && b[1] <= y && y < b[3]) {
                    outside += 1;
                }
            }
        }
        let (fill_on, fill_off) = (count(&on, hud::FILL), count(&off, hud::FILL));
        let (black_on, black_off) = (count(&on, hud::OUTLINE), count(&off, hud::OUTLINE));
        let size = hud::size(w as f64, h as f64);
        println!(
            "gate 7: {w}x{h}: S {size}, m {}, o {}; text {text:?}; box {b:?}; changed {changed}, \
             outside {outside}; exactly the fill {fill_on} (without {fill_off}); exactly black \
             {black_on} − {black_off} = {} added",
            hud::margin(size),
            hud::outline_offset(size),
            black_on - black_off
        );
        assert_eq!(text.as_deref(), Some("2:30 · 83 vehicles · 30 km/h"));
        assert_eq!(b, want_box, "{w}x{h}: box");
        assert_eq!((changed, outside), (want_changed, 0), "{w}x{h}: changed, outside");
        assert_eq!((fill_on, fill_off), (want_fill, 0), "{w}x{h}: the fill");
        assert_eq!(black_on - black_off, want_black, "{w}x{h}: black added");
    }
    println!("gate 7: drawn alone at 720p, 1080p and 2160p: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); renders through the GPU"]
fn gate7_tabular_widths() {
    let run = load("urban_grid", None, None);
    let cam = scene::Camera::fit(&run.strips, 1920, 1080);
    let mut r = Renderer::new(&run.strips, cam, 1, None, None).unwrap();
    r.set_hud(true).unwrap();
    let o = hud::outline_offset(hud::size(1920.0, 1080.0));
    let mut widths = Vec::new();
    for s in [
        "1:11 · 111 vehicles · 11 km/h",
        "8:88 · 888 vehicles · 88 km/h",
        "0:00 · 000 vehicles · 00 km/h",
        "4:07 · 205 vehicles · 23 km/h",
        "19:59 · 216 vehicles · 40 km/h",
    ] {
        r.set_hud_text(s);
        r.render(&[]).unwrap();
        r.set_hud_text(s);
        r.render(&[]).unwrap();
        let b = r.hud_box().unwrap();
        let width = b[2] - b[0] - 2 * o;
        println!("gate 7: {s:?}: box {b:?}, laid out {width} px wide");
        widths.push((b, width));
    }
    // The four with a four-character clock are gated; the fifth digit is recorded only.
    for &(b, width) in &widths[..4] {
        assert_eq!((b, width), ([27, 27, 420, 69], 387));
    }
    println!("gate 7: tabular figures, 387 px each: PASS");
}

/// Each listed frame rendered alone by a new job equals the same frame rendered in
/// sequence from frame 0.
fn gate9(o: &RenderOptions, camera: Option<&Path>, buildings: Option<&Path>, frames: &[u64]) {
    let mut seq = job(o, camera, buildings, true);
    let last = *frames.iter().max().unwrap();
    let mut kept = BTreeMap::new();
    for n in 0..=last {
        let f = seq.render_frame(n).unwrap();
        if frames.contains(&n) {
            kept.insert(n, (f, seq.hud_text()));
        }
    }
    for &n in frames {
        let mut fresh = job(o, camera, buildings, true);
        let f = fresh.render_frame(n).unwrap();
        let (g, text) = &kept[&n];
        let diff = f.chunks(4).zip(g.chunks(4)).filter(|(p, q)| p != q).count();
        println!(
            "gate 9: {} frame {n} (t {:.4}, {text:?}): alone against in sequence, {diff} pixels differ",
            o.project.display(),
            seq.clock.time_of(n)
        );
        assert_eq!(fresh.hud_text(), *text, "frame {n}: the text");
        assert!(f == *g, "frame {n}");
    }
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); renders through the GPU"]
fn gate9_urban_grid_alone_and_in_sequence() {
    gate9(
        &opts("urban_grid", None, None, None, 1920, 1080),
        None,
        None,
        &[0, 1, 30, 299, 300],
    );
    println!("gate 9: urban_grid, 5 of 5 equal: PASS");
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown); renders through the GPU"]
fn gate9_city_flight_alone_and_in_sequence() {
    let camera = root().join("tests/city-flight.toml");
    let buildings = root().join("scratch/midtown/buildings.geojson");
    gate9(
        &opts("midtown", Some(300.0), Some(310.0), Some(1.0), 1920, 1080),
        Some(&camera),
        Some(&buildings),
        &[0, 29, 30, 299],
    );
    println!("gate 9: the city flight, 4 of 4 equal: PASS");
}
