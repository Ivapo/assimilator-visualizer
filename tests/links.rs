//! vis-003 Phase 3 exit gates 5, 6, 7, 9 and 13's `C` (`specs/data_spec.md`): the link
//! colours' measures and steps on a synthetic FCD and on both fixtures, checked against the
//! FCD read straight, their tint and legend drawn through the GPU, a frame alone against
//! the same frame in sequence, and `C`. The headless ones without a fixture run in plain
//! `cargo test`. Those that need the fixtures of `scripts/fixture.sh` and
//! `scripts/fixture.sh midtown`, or the GPU, are ignored; run everything with
//! `cargo test --release --test links -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 8 and 13's CLI cases are `scripts/gates-links.sh`.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::PathBuf;

use arrow_array::{Array, Float64Array, LargeStringArray, StringArray, UInt32Array, UInt64Array};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use assimilator_video::clock;
use assimilator_video::fcd::{Fcd, Row, Snapshot, VehicleRows};
use assimilator_video::links::{self, Links, Measure, Unit};
use assimilator_video::render::VehicleBox;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::view::state::Follow;
use assimilator_video::view::{Fit, ViewInput, ViewState};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ── Gate 5: a synthetic run ──────────────────────────────────────────────────

/// Gate 5's units: `(name, length, lanes, limit)`.
const UNITS: [(&str, f64, u32, f64); 4] = [
    ("A", 100.0, 2, 10.0),
    ("B", 50.0, 1, 20.0),
    ("C", 100.0, 1, 8.333333333333334),
    ("D", 100.0, 1, 10.0),
];

/// Gate 5's rows, `(time, [(vehicle_id, unit, lane, speed)])`: none at 4.0 (a step with
/// no vehicle).
const SYNTHETIC: [(f64, &[(u64, u32, u32, f64)]); 4] = [
    (
        1.0,
        &[
            (1, 0, 0, 10.0),
            (2, 0, 1, 0.0),
            (3, 1, 0, 5.0),
            (5, 2, 0, 8.3333333330),
            (6, 3, 0, 7.4999999),
        ],
    ),
    (2.0, &[(1, 0, 0, 5.0), (2, 0, 1, 0.0), (3, 1, 0, 20.0)]),
    (3.0, &[(2, 0, 1, 2.0)]),
    (5.0, &[(4, 1, 0, 0.05)]),
];

/// An `Fcd` of these snapshots over links `A`–`D`, with no file.
fn fcd_of(snaps: &[(f64, &[(u64, u32, u32, f64)])]) -> Fcd {
    let mut rows = Vec::new();
    let mut snapshots = Vec::new();
    for &(time, snap) in snaps {
        let start = rows.len();
        for &(vehicle_id, link, lane, speed) in snap {
            rows.push(Row {
                time,
                vehicle_id,
                link,
                lane,
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
    let span = (snaps[0].0, snaps[snaps.len() - 1].0);
    Fcd {
        links: UNITS.iter().map(|u| u.0.to_string()).collect(),
        rows,
        snapshots,
        vehicles,
        file_span: span,
    }
}

fn link_unit(i: usize) -> Unit {
    let (name, length, lanes, limit) = UNITS[i];
    Unit {
        link: name.into(),
        lane: None,
        length,
        lanes,
        limits: vec![limit; lanes as usize],
        span: [-1.0, 1.0],
    }
}

fn lane_unit(i: usize, k: u32) -> Unit {
    let (name, length, _, limit) = UNITS[i];
    Unit {
        link: name.into(),
        lane: Some(k),
        length,
        lanes: 1,
        limits: vec![limit],
        span: [-1.0, 1.0],
    }
}

/// Each unit's four steps at `(t, w)`, `relative speed density stopped`, `–` for no
/// value; and the window's FCD times as `n`.
fn steps(l: &Links, t: f64, w: f64) -> Vec<String> {
    let per: Vec<Vec<Option<f64>>> = Measure::ALL.iter().map(|&m| l.values_at(t, w, m)).collect();
    (0..l.units.len())
        .map(|u| {
            Measure::ALL
                .iter()
                .zip(&per)
                .map(|(&m, v)| v[u].map_or("–".to_string(), |v| links::step(m, v).to_string()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

#[test]
fn gate5_synthetic_steps() {
    let fcd = fcd_of(&SYNTHETIC);
    let l = Links::with_units((0..4).map(link_unit).collect(), &fcd);
    assert_eq!(l.interval(), Some(1.0), "the interval: gaps 1, 1, 2");
    let none = "– – 0 0";
    for (t, w, want) in [
        (0.5, 60.0, [none, none, none, none]),
        (1.0, 60.0, ["1 2 2 3", "2 2 3 0", "0 0 2 0", "0 1 2 0"]),
        (3.0, 60.0, ["2 2 2 2", "1 0 3 0", "0 0 1 0", "0 1 1 0"]),
        (5.0, 60.0, ["2 2 2 2", "2 0 3 1", "0 0 0 0", "0 1 0 0"]),
        (2.0, 1.0, ["2 3 2 3", "0 0 3 0", none, none]),
        (1.9999995, 1.0, ["2 3 2 3", "0 0 3 0", none, none]),
        (4.0, 1.0, [none, none, none, none]),
        (4.5, 2.0, ["3 3 1 0", none, none, none]),
    ] {
        let got = steps(&l, t, w);
        println!("gate 5: t {t}, W {w}: A {:?} B {:?} C {:?} D {:?}", got[0], got[1], got[2], got[3]);
        assert_eq!(got, want, "t {t}, W {w}");
    }
    // The rounding cases: C's speed is 29.9999999988 km/h and D's relative 74.999999 %,
    // binned on the whole number shown.
    let speed = l.values_at(1.0, 60.0, Measure::Speed);
    let relative = l.values_at(1.0, 60.0, Measure::Relative);
    println!(
        "gate 5: C's speed {} km/h, step {}; D's relative {} %, step {}",
        speed[2].unwrap() * 3.6,
        links::step(Measure::Speed, speed[2].unwrap()),
        relative[3].unwrap() * 100.0,
        links::step(Measure::Relative, relative[3].unwrap())
    );
    // The counts on their edges: at 5 s A's 2 stopped rows over 4 times, B's 1 over 4.
    let stopped = l.values_at(5.0, 60.0, Measure::Stopped);
    assert_eq!((stopped[0], stopped[1]), (Some(0.5), Some(0.25)));
    println!(
        "gate 5: at 5 s A's stopped {:?}, B's {:?}; at 1 s A's density {:?}",
        stopped[0],
        stopped[1],
        l.values_at(1.0, 60.0, Measure::Density)[0]
    );

    // Per lane: A's two lanes, B, C, D.
    let units = vec![lane_unit(0, 0), lane_unit(0, 1), link_unit(1), link_unit(2), link_unit(3)];
    let lanes = Links::with_units(units, &fcd);
    for (t, a0, a1) in [(1.0, "0 0 2 0", "3 3 2 3"), (3.0, "0 1 2 0", "3 3 2 2")] {
        let got = steps(&lanes, t, 60.0);
        println!("gate 5: per lane, t {t}, W 60: A lane 0 {:?}, A lane 1 {:?}", got[0], got[1]);
        assert_eq!((got[0].as_str(), got[1].as_str()), (a0, a1), "per lane at {t}");
        assert_eq!(&got[2..], &steps(&l, t, 60.0)[1..], "B, C, D as per link at {t}");
    }
    println!("gate 5: eight (t, W) and per lane: PASS");
}

#[test]
fn gate5_check_window() {
    let l = Links::with_units((0..4).map(link_unit).collect(), &fcd_of(&SYNTHETIC));
    let i = l.interval();
    for w in [0.5, 0.999] {
        let e = links::check_window(w, i).unwrap_err().to_string();
        println!("gate 5: --link-window {w}: {e}");
        assert_eq!(
            e,
            format!(
                "--link-window {w} s is shorter than the run's FCD interval (1.000 s); \
                 --link-window 1.000 colours by each frame's own FCD row"
            )
        );
    }
    for w in [0.9999995, 1.0, 60.0] {
        println!("gate 5: --link-window {w}: taken");
        links::check_window(w, i).unwrap();
    }
    for (w, shown) in [(0.0, "0"), (-1.0, "-1"), (f64::NAN, "NaN"), (f64::INFINITY, "inf")] {
        let e = links::check_window(w, i).unwrap_err().to_string();
        println!("gate 5: --link-window {w}: {e}");
        assert_eq!(e, format!("--link-window must be a positive number of seconds (got {shown})"));
    }
    let one = Links::with_units(vec![link_unit(0)], &fcd_of(&SYNTHETIC[..1]));
    assert_eq!(one.interval(), None);
    links::check_window(0.01, one.interval()).unwrap();
    println!("gate 5: one FCD time, --link-window 0.01: taken");
    println!("gate 5: check_window: PASS");
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

/// The FCD read straight, with none of the crate's code: every row `(time, vehicle_id,
/// link_id, lane, speed)` in (`time`, `vehicle_id`) order, and its distinct times.
struct Straight {
    rows: Vec<(f64, u64, String, u32, f64)>,
    /// `(time, first row)`, ascending.
    times: Vec<(f64, usize)>,
}

fn straight(dir: &str) -> Straight {
    let path = root().join("scratch").join(dir).join("fcd/baseline_42.parquet");
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(&path).unwrap()).unwrap();
    let mask = ProjectionMask::columns(
        builder.parquet_schema(),
        ["time", "vehicle_id", "link_id", "lane", "speed"],
    );
    let mut rows = Vec::new();
    for batch in builder.with_projection(mask).build().unwrap() {
        let batch = batch.unwrap();
        let col = |name: &str| batch.column_by_name(name).unwrap().clone();
        let (time, vid, link, lane, speed) =
            (col("time"), col("vehicle_id"), col("link_id"), col("lane"), col("speed"));
        let time = time.as_any().downcast_ref::<Float64Array>().unwrap();
        let vid = vid.as_any().downcast_ref::<UInt64Array>().unwrap();
        let lane = lane.as_any().downcast_ref::<UInt32Array>().unwrap();
        let speed = speed.as_any().downcast_ref::<Float64Array>().unwrap();
        for i in 0..batch.num_rows() {
            let name = match link.as_any().downcast_ref::<StringArray>() {
                Some(a) => a.value(i).to_string(),
                None => link
                    .as_any()
                    .downcast_ref::<LargeStringArray>()
                    .unwrap()
                    .value(i)
                    .to_string(),
            };
            assert!(!link.is_null(i));
            rows.push((time.value(i), vid.value(i), name, lane.value(i), speed.value(i)));
        }
    }
    rows.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let mut times: Vec<(f64, usize)> = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        if times.last().is_none_or(|x| x.0 != r.0) {
            times.push((r.0, i));
        }
    }
    Straight { rows, times }
}

/// The network's links as the straight computation sees them: each name's index, its
/// trimmed length, its lane count and its lanes' limits.
struct Net {
    index: BTreeMap<String, usize>,
    length: Vec<f64>,
    limits: Vec<Vec<f64>>,
}

fn net(r: &Run) -> Net {
    let mut n = Net {
        index: BTreeMap::new(),
        length: Vec::new(),
        limits: Vec::new(),
    };
    for (i, l) in r.placement.network.links.iter().enumerate() {
        n.index.insert(l.id.0.clone(), i);
        n.length.push(r.placement.link_length(&l.id.0));
        n.limits.push(l.lanes.iter().map(|x| x.speed_limit).collect());
    }
    n
}

/// §2.21.5's step, written again: the speed measures on the rounded whole number.
fn step_straight(m: Measure, v: f64) -> u8 {
    let (v, edges, worse_up) = match m {
        Measure::Relative => ((100.0 * v + 0.5).floor(), [25.0, 50.0, 75.0], false),
        Measure::Speed => ((3.6 * v + 0.5).floor(), [10.0, 20.0, 30.0], false),
        Measure::Density => (v, [3.0, 6.0, 12.0], true),
        Measure::Stopped => (v, [0.25, 0.5, 1.0], true),
    };
    let i = edges.iter().filter(|&&e| v >= e).count() as u8;
    if worse_up { i } else { 3 - i }
}

/// Every unit's step of every measure at `t` over `w` (§2.21.3–§2.21.5), from the rows
/// read straight: units in network order, per lane each lane in order.
fn straight_steps(s: &Straight, n: &Net, per_lane: bool, t: f64, w: f64) -> [Vec<Option<u8>>; 4] {
    let first: Vec<usize> = n
        .limits
        .iter()
        .scan(0, |acc, l| {
            let f = *acc;
            *acc += if per_lane { l.len() } else { 1 };
            Some(f)
        })
        .collect();
    let units = if per_lane { n.limits.iter().map(Vec::len).sum() } else { n.limits.len() };
    let lo = s.times.iter().position(|x| x.0 > t - w + 1e-6).unwrap_or(s.times.len());
    let hi = s.times.iter().position(|x| x.0 > t + 1e-6).unwrap_or(s.times.len()).max(lo);
    let times = (hi - lo) as f64;
    let row_of = |k: usize| s.times.get(k).map_or(s.rows.len(), |x| x.1);
    let (mut count, mut rel, mut spd, mut stop) =
        (vec![0usize; units], vec![0.0; units], vec![0.0; units], vec![0usize; units]);
    let mut owner = vec![0usize; units];
    for (link, &f) in first.iter().enumerate() {
        let k = if per_lane { n.limits[link].len() } else { 1 };
        for j in 0..k {
            owner[f + j] = link;
        }
    }
    for (_, _, name, lane, speed) in &s.rows[row_of(lo)..row_of(hi)] {
        let Some(&link) = n.index.get(name) else { continue };
        let lanes = n.limits[link].len();
        let lane = (*lane as usize).min(lanes - 1);
        let u = first[link] + if per_lane { lane } else { 0 };
        count[u] += 1;
        rel[u] += speed / n.limits[link][lane];
        spd[u] += speed;
        if *speed < 0.1 {
            stop[u] += 1;
        }
    }
    let lanes_of = |u: usize| if per_lane { 1.0 } else { n.limits[owner[u]].len() as f64 };
    let mean = |sum: &[f64], u: usize| (count[u] > 0).then(|| sum[u] / count[u] as f64);
    let per_time = |c: usize| if times > 0.0 { c as f64 / times } else { 0.0 };
    [
        (0..units).map(|u| mean(&rel, u).map(|v| step_straight(Measure::Relative, v))).collect(),
        (0..units).map(|u| mean(&spd, u).map(|v| step_straight(Measure::Speed, v))).collect(),
        (0..units)
            .map(|u| {
                let d = per_time(count[u]) / (n.length[owner[u]] * lanes_of(u) / 1000.0);
                Some(step_straight(Measure::Density, d))
            })
            .collect(),
        (0..units).map(|u| Some(step_straight(Measure::Stopped, per_time(stop[u])))).collect(),
    ]
}

fn crate_steps(l: &Links, t: f64, w: f64) -> [Vec<Option<u8>>; 4] {
    Measure::ALL.map(|m| {
        l.values_at(t, w, m)
            .into_iter()
            .map(|v| v.map(|v| links::step(m, v)))
            .collect()
    })
}

/// `[no value, best, …, worst]` of one measure's steps.
fn tally(steps: &[Option<u8>]) -> [usize; 5] {
    let mut c = [0; 5];
    for s in steps {
        c[s.map_or(0, |s| s as usize + 1)] += 1;
    }
    c
}

/// Every frame of one render's window, every measure, `W` = 60, per link and per lane:
/// `Links` against the FCD read straight; the interval; the step counts at `samples`
/// (frame, per lane, the four tallies).
fn gate6(
    dir: &str,
    from: Option<f64>,
    to: Option<f64>,
    speedup: Option<f64>,
    frames: u64,
    samples: &[(Option<u64>, f64, bool, [[usize; 5]; 4])],
) {
    let r = load(dir, from, to);
    let s = straight(dir);
    let n = net(&r);
    let d = r.to - r.from;
    let speedup = speedup.unwrap_or_else(|| clock::default_speedup(d));
    let n_frames = clock::frame_count(d, 30, speedup);
    assert_eq!(n_frames, frames, "{dir}: frames");
    for per_lane in [false, true] {
        let l = Links::new(&r.placement, &r.fcd, per_lane);
        let i = l.interval().unwrap();
        println!("gate 6: {dir} per lane {per_lane}: {} units, interval {i:?}", l.units.len());
        assert!((i - 1.0).abs() < 1e-9, "the interval");
        let mut differ = [0usize; 4];
        let mut frames_differ = 0;
        for f in 0..n_frames {
            let t = clock::frame_time(r.from, f, speedup, 30);
            let (got, want) = (crate_steps(&l, t, 60.0), straight_steps(&s, &n, per_lane, t, 60.0));
            let mut any = false;
            for m in 0..4 {
                let k = got[m].iter().zip(&want[m]).filter(|(a, b)| a != b).count();
                if k > 0 {
                    any = true;
                    if differ[m] < 5 {
                        println!("gate 6: {dir} frame {f} (t {t}) {:?}: {k} units differ", Measure::ALL[m]);
                    }
                }
                differ[m] += k;
            }
            frames_differ += any as usize;
        }
        println!(
            "gate 6: {dir} per lane {per_lane}: {frames_differ} of {n_frames} frames differ \
             (unit-frames relative {}, speed {}, density {}, stopped {})",
            differ[0], differ[1], differ[2], differ[3]
        );
        assert_eq!(frames_differ, 0);
        for &(frame, t, lanes, want) in samples.iter().filter(|x| x.2 == per_lane) {
            let t = frame.map_or(t, |f| clock::frame_time(r.from, f, speedup, 30));
            let got = crate_steps(&l, t, 60.0).map(|v| tally(&v));
            println!("gate 6: {dir} t {t} per lane {lanes}: {got:?}");
            assert_eq!(got, want, "{dir} at {t}, per lane {lanes}");
        }
    }
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_every_frame() {
    gate6(
        "midtown",
        Some(300.0),
        Some(360.0),
        Some(1.0),
        1800,
        &[
            (Some(0), 0.0, false, [[67, 60, 40, 62, 26], [67, 70, 39, 60, 19], [0, 143, 59, 42, 11], [0, 175, 34, 37, 9]]),
            (Some(900), 0.0, false, [[57, 62, 51, 56, 29], [57, 73, 50, 57, 18], [0, 141, 59, 41, 14], [0, 178, 33, 33, 11]]),
            (Some(1799), 0.0, false, [[60, 57, 51, 61, 26], [60, 64, 55, 60, 16], [0, 134, 65, 44, 12], [0, 169, 43, 33, 10]]),
            (Some(0), 0.0, true, [[186, 115, 87, 79, 66], [186, 143, 75, 77, 52], [0, 325, 86, 84, 38], [0, 431, 77, 23, 2]]),
        ],
    );
    println!("gate 6: Midtown 300–360 s: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate6_urban_grid_every_frame() {
    gate6(
        "urban_grid",
        None,
        None,
        None,
        8700,
        &[
            (Some(0), 0.0, false, [[42, 6, 0, 0, 0], [42, 6, 0, 0, 0], [0, 48, 0, 0, 0], [0, 48, 0, 0, 0]]),
            (None, 150.0, false, [[26, 9, 9, 4, 0], [26, 15, 7, 0, 0], [0, 30, 5, 13, 0], [0, 34, 4, 6, 4]]),
            (Some(8699), 0.0, false, [[23, 9, 9, 7, 0], [23, 13, 10, 2, 0], [0, 26, 7, 12, 3], [0, 33, 0, 5, 10]]),
            (None, 150.0, true, [[52, 17, 18, 7, 1], [52, 29, 11, 3, 0], [0, 60, 11, 22, 2], [0, 75, 10, 9, 1]]),
        ],
    );
    println!("gate 6: urban_grid's whole run: PASS");
}

// ── Gate 13: `C` ─────────────────────────────────────────────────────────────

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

/// `input` with `c` pressed steps the measure, and leaves every other field as a twin
/// stepped through `input` alone does.
fn step_c(s: &mut ViewState, input: ViewInput) {
    let mut twin = s.clone();
    let line = twin.frame(&input, no_boxes);
    let before = s.link_measure;
    let mut with_c = input;
    with_c.pressed.c = true;
    assert_eq!(s.frame(&with_c, no_boxes), line, "the keyframe line");
    assert_eq!(s.link_measure, Measure::next(before), "c steps the measure");
    twin.link_measure = s.link_measure;
    assert_eq!(*s, twin, "c changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.link_measure;
    s.frame(&input, no_boxes);
    assert_eq!(s.link_measure, before, "only c steps the measure");
}

#[test]
fn gate13_c() {
    let mut s = plain_state();
    assert_eq!(s.link_measure, None, "off at new");
    let mut seen = Vec::new();
    for _ in 0..5 {
        step_c(&mut s, at((640.0, 360.0)));
        seen.push(s.link_measure);
    }
    println!("gate 13: five frames with c: {seen:?}");
    assert_eq!(
        seen,
        [
            Some(Measure::Relative),
            Some(Measure::Speed),
            Some(Measure::Density),
            Some(Measure::Stopped),
            None
        ]
    );
    s.link_measure = Some(Measure::Speed);
    step_c(&mut s, at((640.0, 360.0)));
    assert_eq!(s.link_measure, Some(Measure::Density), "from speed");

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_c(&mut s, k);
    step_c(&mut s, at((640.0, 360.0)));
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
    step_c(&mut s, at((700.0, 330.0)));
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
    step_c(&mut s, at((460.0, 340.0)));
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
    step_c(&mut s, at((300.0, 706.0)));
    assert!(s.scrub.is_some(), "the scrub goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((300.0, 706.0))
        },
    );
    assert!(s.scrub.is_none());
    println!(
        "gate 13: c cycles off → relative → speed → density → stopped → off and changes \
         nothing else, through an orbit, a drag and a scrub: PASS"
    );
}
