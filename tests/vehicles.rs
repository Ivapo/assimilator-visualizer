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
use std::path::PathBuf;
use std::sync::Arc;

use arrow_array::types::Int32Type;
use arrow_array::{
    Array, ArrayRef, DictionaryArray, Float32Array, Float64Array, Int32Array, LargeStringArray,
    RecordBatch, StringArray, UInt64Array,
};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::{ArrowWriter, ProjectionMask};

use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::VehicleBox;
use assimilator_video::vehicles::{self, Fleet, Kind, Look, Paint, T, Vehicle};
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
