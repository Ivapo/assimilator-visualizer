//! vis-004 Phase 1 exit gates 5, 6, 7, 9 and 13's `V` (`specs/vehicles_spec.md`): the kinds,
//! the parts, the brake rule, the looks and the table's read on files the test writes; the
//! looks of all three fixtures against the FCD read straight; the line-up's lamps drawn
//! through the GPU; a frame alone against the same frame in sequence; and `V`. The
//! headless ones without a fixture run in plain `cargo test`. Those that need the fixtures
//! of `scripts/fixture.sh`, `scripts/fixture.sh midtown` and `scripts/fixture.sh mix`, or
//! the GPU, are ignored; run everything with
//! `cargo test --release --test vehicles -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 1, 2, 8 and 13's CLI cases are `scripts/gates-vehicles.sh`.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use arrow_array::types::Int32Type;
use arrow_array::{
    Array, ArrayRef, DictionaryArray, Float32Array, Float64Array, Int32Array, LargeStringArray,
    RecordBatch, StringArray, UInt64Array,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::{ArrowWriter, ProjectionMask};

use assimilator_video::camera::{self, Pose};
use assimilator_video::draw;
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::scene::{self, Strip};
use assimilator_video::vehicles::{self, Fleet, Kind, Look, Paint, T, Vehicle};
use assimilator_video::view::state::{Fit, Follow, ViewInput, ViewState};
use assimilator_video::{Job, RenderOptions};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ── Gate 5: synthetic ────────────────────────────────────────────────────────

#[test]
fn gate5_kind() {
    let cases = [
        ("car", Kind::Car),
        ("truck", Kind::Truck),
        ("bus", Kind::Bus),
        ("bike", Kind::Bike),
        ("Car", Kind::Plain),
        ("van", Kind::Plain),
        ("lorry", Kind::Plain),
        ("", Kind::Plain),
    ];
    for (name, want) in cases {
        println!("gate 5: kind({name:?}) = {:?}", vehicles::kind(name));
        assert_eq!(vehicles::kind(name), want, "{name:?}");
    }
    println!("gate 5: kind: PASS");
}

#[test]
fn gate5_parts() {
    const KINDS: [Kind; 5] = [Kind::Car, Kind::Truck, Kind::Bus, Kind::Bike, Kind::Plain];
    for kind in KINDS {
        // (parts, height, |z| at most)
        let (n, top, z) = match kind {
            Kind::Car => (7, 1.5, 0.9),
            Kind::Truck => (11, 3.6, 1.25 + T),
            Kind::Bus => (10, 3.1, 1.275 + T),
            Kind::Bike => (2, 1.65, 0.22),
            Kind::Plain => (6, 1.55, 0.9),
        };
        for l in [1.8, 4.5, 8.0, 12.0, 16.5] {
            let p = vehicles::parts(kind, l);
            let min = |k: usize| p.iter().map(|q| q.lo[k]).fold(f64::INFINITY, f64::min);
            let max = |k: usize| p.iter().map(|q| q.hi[k]).fold(f64::NEG_INFINITY, f64::max);
            let (x0, x1) = match kind {
                Kind::Bike => (-l / 2.0, l / 2.0),
                _ => (-l / 2.0 - 2.0 * T, l / 2.0 + T),
            };
            let zmax = p
                .iter()
                .map(|q| q.lo[2].abs().max(q.hi[2].abs()))
                .fold(0.0, f64::max);
            println!(
                "gate 5: {kind:?} at L {l}: {} parts, x {} … {}, y {} … {}, |z| ≤ {zmax}",
                p.len(),
                min(0),
                max(0),
                min(1),
                max(1)
            );
            assert_eq!(p.len(), n, "{kind:?} at {l}: parts");
            assert_eq!((min(0), max(0)), (x0, x1), "{kind:?} at {l}: x");
            assert_eq!((min(1), max(1)), (0.05, top), "{kind:?} at {l}: y");
            assert_eq!(zmax, z, "{kind:?} at {l}: |z|");
            assert!(
                p.iter().all(|q| q.lo.iter().zip(&q.hi).all(|(a, b)| a < b)),
                "{kind:?} at {l}: lo < hi"
            );
        }
    }
    // The truck's cab and box (§2.6.5), its parts 1 and 2.
    let len = |l: f64, i: usize| {
        let q = vehicles::parts(Kind::Truck, l)[i];
        assert_eq!(q.paint, Paint::Body);
        q.hi[0] - q.lo[0]
    };
    for (l, i, want) in [
        (8.0, 2, 5.46),
        (16.5, 2, 13.95),
        (8.0, 1, 2.3),
        (16.5, 1, 2.3),
        (4.0, 1, 1.4),
    ] {
        let got = len(l, i);
        println!(
            "gate 5: the truck's {} at L {l}: {got} m",
            ["", "cab", "box"][i]
        );
        assert!(
            (got - want).abs() < 1e-9,
            "truck at {l}, part {i}: {got}, want {want}"
        );
    }
    println!("gate 5: parts: PASS");
}

#[test]
fn gate5_brake() {
    for (speed, acc, want) in [
        (5.0, 0.0, false),
        (5.0, -1e-12, true),
        (0.01, 0.0, false),
        (0.00999, 0.0, true),
        (0.0, 1.5, true),
    ] {
        println!(
            "gate 5: brake({speed}, {acc}) = {}",
            vehicles::brake(speed, acc)
        );
        assert_eq!(vehicles::brake(speed, acc), want, "({speed}, {acc})");
    }
    println!("gate 5: brake: PASS");
}

fn vbox(vehicle_id: u64) -> VehicleBox {
    VehicleBox {
        vehicle_id,
        at: Placed {
            x: 0.0,
            y: 0.0,
            heading: 90.0,
        },
        length: 4.5,
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
fn gate5_looks() {
    let fleet = Fleet::new(vec![Vehicle {
        vehicle_id: 7,
        kind: vehicles::kind("truck"),
        times: vec![1.0, 2.0, 3.0],
        brakes: vec![false, true, false],
    }]);
    let boxes = [vbox(7), vbox(8)];
    for (t, want) in [
        (0.5, false),
        (1.0, false),
        (1.9999995, true),
        (2.5, true),
        (3.0, false),
        (10.0, false),
    ] {
        let looks = fleet.looks(t, &boxes);
        println!("gate 5: looks at {t}: {looks:?}");
        assert_eq!(
            looks,
            [
                Look {
                    kind: Kind::Truck,
                    brake: want
                },
                Look {
                    kind: Kind::Plain,
                    brake: false
                }
            ],
            "at {t}"
        );
    }
    println!("gate 5: looks: PASS");
}

/// Two vehicles of three rows each, in (`time`, `vehicle_id`) order as the engine writes
/// them: vehicle 1 a `car` braking at its second row only, vehicle 2 a `bus` standing at
/// its second row and braking at its third.
const TIMES: [f64; 6] = [1.0, 1.0, 2.0, 2.0, 3.0, 3.0];
const IDS: [u64; 6] = [1, 2, 1, 2, 1, 2];
const SPEEDS: [f64; 6] = [5.0, 5.0, 5.0, 0.005, 5.0, 5.0];
const ACCS: [f64; 6] = [0.0, 1.0, -1.0, 1.0, 0.5, -0.5];
const CLASSES: [&str; 6] = ["car", "bus", "car", "bus", "car", "bus"];

/// Columns beside `time`, `vehicle_id` and `speed`: each name and its values.
type Extra = Vec<(&'static str, ArrayRef)>;

/// A parquet file in `scratch/out/vehicles-tests/` with `time`, `vehicle_id` and `speed`,
/// and `extra`.
fn write(name: &str, extra: Extra) -> PathBuf {
    let dir = root().join("scratch/out/vehicles-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let mut cols: Vec<(&str, ArrayRef, bool)> = vec![
        ("time", Arc::new(Float64Array::from(TIMES.to_vec())), false),
        (
            "vehicle_id",
            Arc::new(UInt64Array::from(IDS.to_vec())),
            false,
        ),
        (
            "speed",
            Arc::new(Float64Array::from(SPEEDS.to_vec())),
            false,
        ),
    ];
    cols.extend(extra.into_iter().map(|(n, a)| (n, a, true)));
    let batch = RecordBatch::try_from_iter_with_nullable(cols).unwrap();
    let mut w = ArrowWriter::try_new(File::create(&path).unwrap(), batch.schema(), None).unwrap();
    w.write(&batch).unwrap();
    w.close().unwrap();
    path
}

fn acc() -> (&'static str, ArrayRef) {
    ("acceleration", Arc::new(Float64Array::from(ACCS.to_vec())))
}

fn dict(classes: &[Option<&'static str>]) -> (&'static str, ArrayRef) {
    let d: DictionaryArray<Int32Type> = classes.iter().copied().collect();
    ("vehicle_class", Arc::new(d))
}

fn fleet(k1: Kind, b1: [bool; 3], k2: Kind, b2: [bool; 3]) -> Fleet {
    let v = |id, kind, b: [bool; 3]| Vehicle {
        vehicle_id: id,
        kind,
        times: vec![1.0, 2.0, 3.0],
        brakes: b.to_vec(),
    };
    Fleet::new(vec![v(1, k1, b1), v(2, k2, b2)])
}

#[test]
fn gate5_read() {
    let all = CLASSES.map(Some);
    let full = fleet(
        Kind::Car,
        [false, true, false],
        Kind::Bus,
        [false, true, true],
    );
    let cases: [(&str, Extra, Fleet); 5] = [
        ("all.parquet", vec![acc(), dict(&all)], full.clone()),
        (
            "no_class.parquet",
            vec![acc()],
            fleet(
                Kind::Plain,
                [false, true, false],
                Kind::Plain,
                [false, true, true],
            ),
        ),
        (
            "no_acceleration.parquet",
            vec![dict(&all)],
            fleet(
                Kind::Car,
                [false, false, false],
                Kind::Bus,
                [false, true, false],
            ),
        ),
        (
            "utf8.parquet",
            vec![
                acc(),
                (
                    "vehicle_class",
                    Arc::new(StringArray::from(CLASSES.to_vec())),
                ),
            ],
            full.clone(),
        ),
        (
            "nulls.parquet",
            vec![
                (
                    "acceleration",
                    Arc::new(Float64Array::from(vec![
                        Some(0.0),
                        Some(1.0),
                        None,
                        Some(1.0),
                        Some(0.5),
                        Some(-0.5),
                    ])),
                ),
                dict(&[
                    Some("car"),
                    None,
                    Some("car"),
                    Some("bus"),
                    Some("car"),
                    Some("bus"),
                ]),
            ],
            fleet(
                Kind::Car,
                [false, false, false],
                Kind::Plain,
                [false, true, true],
            ),
        ),
    ];
    for (name, extra, want) in cases {
        let got = vehicles::read(&write(name, extra)).unwrap();
        println!("gate 5: read {name}: {:?}", got.vehicles);
        assert_eq!(got, want, "{name}");
    }
    let errors: [(&str, Extra, &str); 3] = [
        (
            "class_int.parquet",
            vec![
                acc(),
                (
                    "vehicle_class",
                    Arc::new(Int32Array::from(vec![0, 1, 0, 1, 0, 1])),
                ),
            ],
            "column \"vehicle_class\" is Int32, expected Dictionary(Int32, Utf8) or Utf8",
        ),
        (
            "class_large.parquet",
            vec![
                acc(),
                (
                    "vehicle_class",
                    Arc::new(LargeStringArray::from(CLASSES.to_vec())),
                ),
            ],
            "column \"vehicle_class\" is LargeUtf8, expected Dictionary(Int32, Utf8) or Utf8",
        ),
        (
            "acc_f32.parquet",
            vec![
                (
                    "acceleration",
                    Arc::new(Float32Array::from(ACCS.map(|a| a as f32).to_vec())),
                ),
                dict(&all),
            ],
            "column \"acceleration\" is Float32, expected Float64",
        ),
    ];
    for (name, extra, tail) in errors {
        let path = write(name, extra);
        let got = vehicles::read(&path).unwrap_err().to_string();
        println!("gate 5: read {name}: {got}");
        assert_eq!(
            got,
            format!("schema mismatch: {} {tail}", path.display()),
            "{name}"
        );
    }
    println!("gate 5: read: PASS");
}

// ── Gate 6: the fixtures, against the FCD read straight ──────────────────────

/// Every vehicle's rows read straight, with none of the crate's code: `(time, brake,
/// class)`, each vehicle's in `time` order.
fn straight(dir: &str) -> BTreeMap<u64, Vec<(f64, bool, String)>> {
    let path = root()
        .join("scratch")
        .join(dir)
        .join("fcd/baseline_42.parquet");
    let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(&path).unwrap()).unwrap();
    let mask = ProjectionMask::columns(
        builder.parquet_schema(),
        [
            "time",
            "vehicle_id",
            "speed",
            "acceleration",
            "vehicle_class",
        ],
    );
    let mut by: BTreeMap<u64, Vec<(f64, bool, String)>> = BTreeMap::new();
    for batch in builder.with_projection(mask).build().unwrap() {
        let batch = batch.unwrap();
        let f = |name: &str| {
            batch
                .column_by_name(name)
                .unwrap()
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .clone()
        };
        let (time, speed, acc) = (f("time"), f("speed"), f("acceleration"));
        let vid = batch.column_by_name("vehicle_id").unwrap();
        let vid = vid.as_any().downcast_ref::<UInt64Array>().unwrap();
        let class = batch.column_by_name("vehicle_class").unwrap();
        let class = class
            .as_any()
            .downcast_ref::<DictionaryArray<Int32Type>>()
            .unwrap();
        let names = class
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        for i in 0..batch.num_rows() {
            assert!(!acc.is_null(i) && !class.is_null(i));
            let name = names.value(class.keys().value(i) as usize).to_string();
            let lit = acc.value(i) < 0.0 || speed.value(i) < 0.01;
            by.entry(vid.value(i))
                .or_default()
                .push((time.value(i), lit, name));
        }
    }
    for rows in by.values_mut() {
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    by
}

/// The column of a class name in the counts: car, truck, bus, bike, plain.
fn column(name: &str) -> usize {
    match name {
        "car" => 0,
        "truck" => 1,
        "bus" => 2,
        "bike" => 3,
        _ => 4,
    }
}

fn opts(dir: &str, from: Option<f64>, to: Option<f64>, speedup: Option<f64>) -> RenderOptions {
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
        width: 1920,
        height: 1080,
    }
}

/// At every frame, each drawn vehicle's kind and state against its row at or before the
/// frame's time, found with the test's own loops; then the counts at `samples`, `(frame,
/// drawn, lit, by kind)`.
fn gate6(
    o: &RenderOptions,
    camera: &str,
    frames: u64,
    samples: &[(u64, usize, usize, [usize; 5])],
) {
    let dir = o.project.file_name().unwrap().to_str().unwrap().to_string();
    let rows = straight(&dir);
    let mut job = Job::prepare_with(o, Some(&root().join(camera)), None).unwrap();
    assert_eq!(job.looks_at(job.clock.from), None, "off at prepare");
    job.set_shapes(true).unwrap();
    assert_eq!(job.clock.frames, frames, "{dir}: frames");
    let (mut differ, mut frames_differ, mut drawn_all) = (0usize, 0usize, 0usize);
    for n in 0..frames {
        let t = job.clock.time_of(n);
        let boxes = job.boxes_at(t);
        let looks = job.looks_at(t).unwrap();
        assert_eq!(looks.len(), boxes.len());
        let mut counts = (0usize, [0usize; 5]);
        let mut any = false;
        for (b, look) in boxes.iter().zip(&looks) {
            let r = &rows[&b.vehicle_id];
            let mut k = 0;
            for (j, row) in r.iter().enumerate() {
                if row.0 <= t + 1e-6 {
                    k = j;
                } else {
                    break;
                }
            }
            let (lit, name) = (r[k].1, r[0].2.as_str());
            let kind = match name {
                "car" => Kind::Car,
                "truck" => Kind::Truck,
                "bus" => Kind::Bus,
                "bike" => Kind::Bike,
                _ => Kind::Plain,
            };
            if *look != (Look { kind, brake: lit }) {
                if differ < 5 {
                    println!(
                        "gate 6: {dir} frame {n} (t {t}) vehicle {}: {look:?}, straight {kind:?} {lit}",
                        b.vehicle_id
                    );
                }
                differ += 1;
                any = true;
            }
            counts.0 += lit as usize;
            counts.1[column(name)] += 1;
        }
        frames_differ += any as usize;
        drawn_all += boxes.len();
        if let Some(&(_, drawn, lit, by)) = samples.iter().find(|s| s.0 == n) {
            println!(
                "gate 6: {dir} frame {n} (t {t}): drawn {}, lit {}, car/truck/bus/bike/plain {:?}",
                boxes.len(),
                counts.0,
                counts.1
            );
            assert_eq!(
                (boxes.len(), counts.0, counts.1),
                (drawn, lit, by),
                "{dir} at frame {n}"
            );
        }
    }
    println!(
        "gate 6: {dir}: {frames_differ} of {frames} frames differ ({differ} of {drawn_all} \
         vehicle-frames)"
    );
    assert_eq!(frames_differ, 0);
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown); builds a perspective renderer"]
fn gate6_midtown_every_frame() {
    gate6(
        &opts("midtown", Some(300.0), Some(360.0), Some(1.0)),
        "tests/city-flight.toml",
        1800,
        &[
            (0, 184, 137, [184, 0, 0, 0, 0]),
            (900, 201, 132, [201, 0, 0, 0, 0]),
            (1799, 202, 142, [202, 0, 0, 0, 0]),
        ],
    );
    println!("gate 6: Midtown 300–360 s: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); builds a perspective renderer"]
fn gate6_urban_grid_every_frame() {
    gate6(
        &opts("urban_grid", None, None, None),
        "tests/flight.toml",
        8700,
        &[
            (0, 6, 0, [6, 0, 0, 0, 0]),
            (4227, 83, 56, [83, 0, 0, 0, 0]),
            (8699, 107, 78, [107, 0, 0, 0, 0]),
        ],
    );
    println!("gate 6: urban_grid's whole run: PASS");
}

#[test]
#[ignore = "needs scratch/mix (scripts/fixture.sh mix); builds a perspective renderer"]
fn gate6_mix_every_frame() {
    gate6(
        &opts("mix", Some(300.0), Some(360.0), Some(1.0)),
        "tests/city-flight.toml",
        1800,
        &[
            (0, 178, 126, [138, 20, 7, 6, 7]),
            (900, 193, 141, [145, 20, 12, 8, 8]),
            (1799, 195, 136, [144, 21, 12, 11, 7]),
        ],
    );
    println!("gate 6: the mix 300–360 s: PASS");
}

// ── Gate 13: `V` ─────────────────────────────────────────────────────────────

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

/// `input` with `v` pressed flips the shapes, and leaves every other field as a twin
/// stepped through `input` alone does.
fn step_v(s: &mut ViewState, input: ViewInput) {
    let mut twin = s.clone();
    let line = twin.frame(&input, no_boxes);
    let before = s.shapes_shown;
    let mut with_v = input;
    with_v.pressed.v = true;
    assert_eq!(s.frame(&with_v, no_boxes), line, "the keyframe line");
    assert_eq!(s.shapes_shown, !before, "v flips the shapes");
    twin.shapes_shown = s.shapes_shown;
    assert_eq!(*s, twin, "v changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.shapes_shown;
    s.frame(&input, no_boxes);
    assert_eq!(s.shapes_shown, before, "only v flips the shapes");
}

#[test]
fn gate13_v() {
    let mut s = plain_state();
    assert!(!s.shapes_shown, "off at new");
    let mut seen = Vec::new();
    for _ in 0..4 {
        step_v(&mut s, at((640.0, 360.0)));
        seen.push(s.shapes_shown);
    }
    println!("gate 13: four frames with v: {seen:?}");
    assert_eq!(seen, [true, false, true, false]);

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_v(&mut s, k);
    step_v(&mut s, at((640.0, 360.0)));
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
    step_v(&mut s, at((700.0, 330.0)));
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
    step_v(&mut s, at((460.0, 340.0)));
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
    step_v(&mut s, at((300.0, 706.0)));
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
        "gate 13: v flips shapes and boxes and changes nothing else, through an orbit, a drag \
         and a scrub: PASS"
    );
}

// ── Gates 7 and 9: through the GPU ───────────────────────────────────────────

fn count_differ(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(p, q)| p != q).count()
}

/// `frame` (RGBA, 1920×1080) as a binary PPM at `path`.
fn ppm(path: &Path, frame: &[u8]) {
    let mut out = b"P6\n1920 1080\n255\n".to_vec();
    out.extend(frame.chunks(4).flat_map(|p| [p[0], p[1], p[2]]));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, out).unwrap();
}

/// The probe's line-up (§2.19): one strip 80 m long and 16 m wide along +x, and two rows
/// heading 90°, the one at `y` = +4 coasting and the one at −4 braking, each a car 4.5 m,
/// a truck 12, a truck 8, a bus 12, a bike 1.8 and a plain shape 5.5 from `x` = −30 with
/// 3 m between, ids 1–12, speeds cycling 1, 4, 7, 11, 20, 7 m/s (the second row one on).
fn lineup() -> (Vec<Strip>, Vec<VehicleBox>, Vec<Look>) {
    let mut s = Strip {
        left: vec![],
        right: vec![],
    };
    for i in 0..=80 {
        let x = i as f64 - 40.0;
        s.left.push([x, 8.0]);
        s.right.push([x, -8.0]);
    }
    let kinds = [
        (Kind::Car, 4.5),
        (Kind::Truck, 12.0),
        (Kind::Truck, 8.0),
        (Kind::Bus, 12.0),
        (Kind::Bike, 1.8),
        (Kind::Plain, 5.5),
    ];
    let speeds = [1.0, 4.0, 7.0, 11.0, 20.0, 7.0];
    let (mut boxes, mut looks) = (vec![], vec![]);
    let mut id = 1;
    for (row, y, brake) in [(0, 4.0, false), (1, -4.0, true)] {
        let mut x = -30.0;
        for (k, &(kind, l)) in kinds.iter().enumerate() {
            x += l / 2.0;
            boxes.push(VehicleBox {
                vehicle_id: id,
                at: Placed {
                    x,
                    y,
                    heading: 90.0,
                },
                length: l,
                speed: speeds[(k + row) % 6],
                track: TrackPos {
                    piece: Piece::Link(0),
                    along: 0.0,
                    lateral: 0.0,
                    odo: 0.0,
                },
            });
            looks.push(Look { kind, brake });
            x += l / 2.0 + 3.0;
            id += 1;
        }
    }
    (vec![s], boxes, looks)
}

/// Every lamp of `paint` in `frame`: the pixel holding the centre of its outer face, and
/// whether it is exactly its colour.
fn lamps(
    frame: &[u8],
    pose: &Pose,
    boxes: &[VehicleBox],
    looks: &[Look],
    paint: Paint,
) -> (usize, usize) {
    let (mut exact, mut n) = (0, 0);
    for (rank, (b, l)) in boxes.iter().zip(looks).enumerate() {
        let h = b.at.heading.to_radians();
        let (f, r) = ((h.sin(), h.cos()), (h.cos(), -h.sin()));
        for part in vehicles::parts(l.kind, b.length)
            .into_iter()
            .filter(|p| p.paint == paint)
        {
            let want = match paint {
                Paint::Brake if l.brake => vehicles::BRAKE_ON,
                Paint::Brake => vehicles::BRAKE_OFF,
                _ => vehicles::HEAD_ON,
            };
            let lx = if part.lo[0] < 0.0 {
                part.lo[0]
            } else {
                part.hi[0]
            };
            let ly = (part.lo[1] + part.hi[1]) / 2.0 + rank as f64 * draw::RANK_LIFT_3D;
            let lz = (part.lo[2] + part.hi[2]) / 2.0;
            let w = [
                b.at.x + lx * f.0 + lz * r.0,
                b.at.y + lx * f.1 + lz * r.1,
                ly,
            ];
            let (x, y) = camera::project(pose, 1920.0, 1080.0, w);
            let got = if (0.0..1920.0).contains(&x) && (0.0..1080.0).contains(&y) {
                let k = 4 * (y.floor() as usize * 1920 + x.floor() as usize);
                [frame[k], frame[k + 1], frame[k + 2]]
            } else {
                [0, 0, 0]
            };
            n += 1;
            if got == want {
                exact += 1;
            } else {
                println!(
                    "gate 7: vehicle {} {:?} {paint:?} at ({x:.1}, {y:.1}): {got:?}, want {want:?}",
                    b.vehicle_id, l.kind
                );
            }
        }
    }
    (exact, n)
}

#[test]
#[ignore = "renders through the GPU"]
fn gate7_lineup() {
    let (strips, boxes, looks) = lineup();
    let cam = scene::Camera::fit(&strips, 1920, 1080);
    let out = root().join("scratch/out/vehicles");
    // (pose, the lamps counted, the exact count, the PPM)
    for (pose, paint, want, name) in [
        (
            Pose {
                cx: 0.0,
                cy: -2.0,
                height_m: 58.0,
                yaw_deg: 45.0,
                pitch_deg: 35.0,
            },
            Paint::Brake,
            20,
            "vehicles-lineup-rear.ppm",
        ),
        (
            Pose {
                cx: 0.0,
                cy: 2.0,
                height_m: 58.0,
                yaw_deg: 225.0,
                pitch_deg: 35.0,
            },
            Paint::Head,
            19,
            "vehicles-lineup-front.ppm",
        ),
    ] {
        let mut r = Renderer::new_perspective(&strips, cam, 64, &pose, None, None).unwrap();
        let shapes = r.render_looks(&boxes, Some(&looks)).unwrap();
        let (exact, n) = lamps(&shapes, &pose, &boxes, &looks, paint);
        println!("gate 7: {name}: {exact} of {n} {paint:?} lamp centres exact");
        assert_eq!((exact, n), (want, 20), "{name}");
        let plain = r.render(&boxes).unwrap();
        let d = count_differ(&shapes, &plain);
        println!("gate 7: {name}: shapes against boxes, {d} pixels differ");
        assert!(d > 0, "{name}: shapes against boxes");
        let (e1, e2) = (
            r.render(&[]).unwrap(),
            r.render_looks(&[], Some(&[])).unwrap(),
        );
        println!(
            "gate 7: {name}: no vehicle, {} pixels differ",
            count_differ(&e1, &e2)
        );
        assert!(e1 == e2, "{name}: no vehicle");
        ppm(&out.join(name), &shapes);
    }
    println!("gate 7: the line-up's lamps, 20 of 20 brake and 19 of 20 head: PASS");
}

/// A job set as `render` sets it by default (`src/main.rs`): streets, see-through,
/// signals, the HUD, then the shapes.
fn job(o: &RenderOptions, camera: &Path, buildings: Option<&Path>) -> Job {
    let mut j = Job::prepare_with(o, Some(camera), buildings).unwrap();
    j.set_streets(true).unwrap();
    j.set_see_through(true);
    j.set_signals(true).unwrap();
    j.set_hud(true).unwrap();
    j.set_shapes(true).unwrap();
    j
}

/// Each listed frame rendered alone by a new job equals the same frame rendered in
/// sequence from frame 0; at each, in sequence and alone, the job's looks are
/// `Fleet::looks` of a table read with `vehicles::read`.
fn gate9(o: &RenderOptions, camera: &Path, buildings: Option<&Path>, frames: &[u64]) {
    let fleet = vehicles::read(&o.project.join("fcd/baseline_42.parquet")).unwrap();
    let mut seq = job(o, camera, buildings);
    let last = *frames.iter().max().unwrap();
    let mut kept = BTreeMap::new();
    for n in 0..=last {
        let f = seq.render_frame(n).unwrap();
        if frames.contains(&n) {
            let t = seq.clock.time_of(n);
            assert_eq!(
                seq.looks_at(t),
                Some(fleet.looks(t, &seq.boxes_at(t))),
                "frame {n} in sequence: the looks"
            );
            kept.insert(n, f);
        }
    }
    for &n in frames {
        let mut fresh = job(o, camera, buildings);
        let f = fresh.render_frame(n).unwrap();
        let t = fresh.clock.time_of(n);
        let looks = fresh.looks_at(t).unwrap();
        let lit = looks.iter().filter(|l| l.brake).count();
        let diff = count_differ(&f, &kept[&n]);
        println!(
            "gate 9: {} frame {n} (t {t:.4}, {} vehicles, {lit} lit): alone against in \
             sequence, {diff} pixels differ",
            o.project.display(),
            looks.len()
        );
        assert_eq!(
            looks,
            fleet.looks(t, &fresh.boxes_at(t)),
            "frame {n} alone: the looks"
        );
        assert!(f == kept[&n], "frame {n}");
    }
}

#[test]
#[ignore = "needs scratch/mix (scripts/fixture.sh mix) and scratch/midtown's buildings; renders through the GPU"]
fn gate9_mix_city_flight_alone_and_in_sequence() {
    let buildings = root().join("scratch/midtown/buildings.geojson");
    gate9(
        &opts("mix", Some(300.0), Some(310.0), Some(1.0)),
        &root().join("tests/city-flight.toml"),
        Some(&buildings),
        &[0, 1, 29, 30, 299],
    );
    println!("gate 9: the mix's city flight, 5 of 5 equal: PASS");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh); renders through the GPU"]
fn gate9_urban_grid_alone_and_in_sequence() {
    gate9(
        &opts("urban_grid", Some(9.1), Some(19.1), Some(1.0)),
        &root().join("tests/flight.toml"),
        None,
        &[0, 1, 30, 299],
    );
    println!("gate 9: urban_grid's flight, 4 of 4 equal: PASS");
}
