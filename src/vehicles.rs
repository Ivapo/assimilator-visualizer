//! Vehicles (vis-004 Phase 1): in the perspective views each vehicle is drawn as a shape
//! for its FCD `vehicle_class`, its brake lamps lit while its row at or before the frame's
//! time brakes or stands, its headlamps always lit. No Bevy types: the table read from the
//! FCD's `vehicle_class` and `acceleration` (§2.12), the kinds (§2.6.6), the brake rule
//! (§2.8.1), each kind's parts (§2.6) and the colours (§2.7); `src/draw.rs` draws them.

use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result, bail};
use arrow_array::types::Int32Type;
use arrow_array::{Array, DictionaryArray, Float64Array, RecordBatch, StringArray, UInt64Array};
use arrow_schema::DataType;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::clock::EPS;
use crate::render::VehicleBox;

/// Under this speed a vehicle stands, and its brake lamps are lit, m/s (the engine
/// dashboard's, §2.8.1).
pub const STANDING: f64 = 0.01;
/// A lamp's or a pane's thickness, and how far it stands proud of its face, m (§2.6.1).
pub const T: f64 = 0.02;
/// Every shape's floor, m: today's box's.
pub const FLOOR: f64 = 0.05;

/// The colours of §2.7, sRGB.
pub const GLASS: [u8; 3] = [34, 42, 56];
pub const DARK: [u8; 3] = [38, 38, 42];
pub const BRAKE_OFF: [u8; 3] = [92, 22, 26];
/// The dashboard's brake red, `#ef4444`.
pub const BRAKE_ON: [u8; 3] = [239, 68, 68];
pub const HEAD_ON: [u8; 3] = [255, 240, 196];

/// A shape (§2.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Car,
    Truck,
    Bus,
    Bike,
    /// Any other class name, and a vehicle with none: today's box with the car's lamps.
    Plain,
}

/// The shape of a class name, compared exactly (§2.6.6).
pub fn kind(name: &str) -> Kind {
    match name {
        "car" => Kind::Car,
        "truck" => Kind::Truck,
        "bus" => Kind::Bus,
        "bike" => Kind::Bike,
        _ => Kind::Plain,
    }
}

/// The dashboard's brake rule (§2.8.1): braking, or standing still.
pub fn brake(speed: f64, acceleration: f64) -> bool {
    acceleration < 0.0 || speed < STANDING
}

/// One vehicle's shape and its rows' times and brake states, in `time` order.
#[derive(Debug, Clone, PartialEq)]
pub struct Vehicle {
    pub vehicle_id: u64,
    pub kind: Kind,
    pub times: Vec<f64>,
    pub brakes: Vec<bool>,
}

/// Every vehicle of a run, in `vehicle_id` order.
#[derive(Debug, Clone, PartialEq)]
pub struct Fleet {
    pub vehicles: Vec<Vehicle>,
}

/// How one box is drawn at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    pub kind: Kind,
    pub brake: bool,
}

impl Fleet {
    pub fn new(mut vehicles: Vec<Vehicle>) -> Fleet {
        vehicles.sort_by_key(|v| v.vehicle_id);
        Fleet { vehicles }
    }

    /// Each box's look at `t`, in the boxes' order: its vehicle's kind, and the brake state
    /// of its row at or before `t` (within [`EPS`]), its first row before that. A box whose
    /// vehicle is not in the table is plain and unlit.
    pub fn looks(&self, t: f64, boxes: &[VehicleBox]) -> Vec<Look> {
        boxes
            .iter()
            .map(|b| {
                match self
                    .vehicles
                    .binary_search_by_key(&b.vehicle_id, |v| v.vehicle_id)
                {
                    Ok(i) => {
                        let v = &self.vehicles[i];
                        let row = v.times.partition_point(|&x| x <= t + EPS).max(1) - 1;
                        Look {
                            kind: v.kind,
                            brake: v.brakes[row],
                        }
                    }
                    Err(_) => Look {
                        kind: Kind::Plain,
                        brake: false,
                    },
                }
            })
            .collect()
    }
}

/// How the file holds `vehicle_class`.
enum Classes {
    Dictionary,
    Utf8,
}

/// The table of §2.12 from the FCD file at `path`: every row's `time`, `vehicle_id`,
/// `speed`, `acceleration` and `vehicle_class`, by vehicle. A vehicle's kind is its first
/// row's. With no `vehicle_class` every vehicle is plain; with no `acceleration` a row
/// brakes only standing; a null reads as the column missing for its row. Either column of
/// another type is the schema-mismatch error.
pub fn read(path: &Path) -> Result<Fleet> {
    let file = File::open(path).with_context(|| format!("missing file: {}", path.display()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| format!("schema mismatch: {} is not a Parquet file", path.display()))?;
    let schema = builder.schema().clone();
    let classes = match schema.field_with_name("vehicle_class") {
        Err(_) => None,
        Ok(f) => match f.data_type() {
            DataType::Dictionary(k, v) if **k == DataType::Int32 && **v == DataType::Utf8 => {
                Some(Classes::Dictionary)
            }
            DataType::Utf8 => Some(Classes::Utf8),
            other => bail!(
                "schema mismatch: {} column \"vehicle_class\" is {other}, expected \
                 Dictionary(Int32, Utf8) or Utf8",
                path.display()
            ),
        },
    };
    let has_acceleration = match schema.field_with_name("acceleration") {
        Err(_) => false,
        Ok(f) if f.data_type() == &DataType::Float64 => true,
        Ok(f) => bail!(
            "schema mismatch: {} column \"acceleration\" is {}, expected Float64",
            path.display(),
            f.data_type()
        ),
    };
    let mut wanted = vec!["time", "vehicle_id", "speed"];
    if has_acceleration {
        wanted.push("acceleration");
    }
    if classes.is_some() {
        wanted.push("vehicle_class");
    }
    let mask = ProjectionMask::columns(builder.parquet_schema(), wanted.iter().copied());
    let reader = builder.with_projection(mask).build()?;

    // (vehicle_id, time, brake, kind)
    let mut rows: Vec<(u64, f64, bool, Kind)> = Vec::new();
    for batch in reader {
        let batch = batch?;
        let time = col::<Float64Array>(&batch, "time", path)?;
        let vid = col::<UInt64Array>(&batch, "vehicle_id", path)?;
        let speed = col::<Float64Array>(&batch, "speed", path)?;
        let acc = if has_acceleration {
            Some(col::<Float64Array>(&batch, "acceleration", path)?)
        } else {
            None
        };
        // Each row's kind: a dictionary's values are named once, and its keys index them.
        let kinds: Vec<Kind> = match classes {
            None => vec![Kind::Plain; batch.num_rows()],
            Some(Classes::Dictionary) => {
                let d = col::<DictionaryArray<Int32Type>>(&batch, "vehicle_class", path)?;
                let names = d
                    .values()
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .with_context(|| {
                        format!(
                            "schema mismatch: {} column \"vehicle_class\"",
                            path.display()
                        )
                    })?;
                let by_key: Vec<Kind> = (0..names.len())
                    .map(|k| {
                        if names.is_null(k) {
                            Kind::Plain
                        } else {
                            kind(names.value(k))
                        }
                    })
                    .collect();
                (0..d.len())
                    .map(|i| {
                        if d.is_null(i) {
                            Kind::Plain
                        } else {
                            by_key[d.keys().value(i) as usize]
                        }
                    })
                    .collect()
            }
            Some(Classes::Utf8) => {
                let s = col::<StringArray>(&batch, "vehicle_class", path)?;
                (0..s.len())
                    .map(|i| {
                        if s.is_null(i) {
                            Kind::Plain
                        } else {
                            kind(s.value(i))
                        }
                    })
                    .collect()
            }
        };
        for (i, &k) in kinds.iter().enumerate() {
            if time.is_null(i) || vid.is_null(i) || speed.is_null(i) {
                bail!(
                    "schema mismatch: {} has a null in a required column",
                    path.display()
                );
            }
            let v = speed.value(i);
            let lit = match acc {
                Some(a) if !a.is_null(i) => brake(v, a.value(i)),
                _ => v < STANDING,
            };
            rows.push((vid.value(i), time.value(i), lit, k));
        }
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    let mut vehicles: Vec<Vehicle> = Vec::new();
    for (vehicle_id, time, lit, k) in rows {
        match vehicles.last_mut() {
            Some(v) if v.vehicle_id == vehicle_id => {
                v.times.push(time);
                v.brakes.push(lit);
            }
            _ => vehicles.push(Vehicle {
                vehicle_id,
                kind: k,
                times: vec![time],
                brakes: vec![lit],
            }),
        }
    }
    Ok(Fleet { vehicles })
}

fn col<'a, A: 'static>(batch: &'a RecordBatch, name: &str, path: &Path) -> Result<&'a A> {
    batch
        .column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<A>())
        .with_context(|| format!("schema mismatch: {} column {name:?}", path.display()))
}

/// What a part is painted with (§2.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    /// The speed colour, or the link colours' grey; shaded by face.
    Body,
    /// The car's cabin: its top is [`Paint::Body`], its sides and ends [`GLASS`].
    Cabin,
    Glass,
    Dark,
    /// [`BRAKE_ON`] while braking, [`BRAKE_OFF`] otherwise; unshaded.
    Brake,
    /// [`HEAD_ON`]; unshaded.
    Head,
}

/// A cuboid in the vehicle's own frame (§2.6.1): `x` forward from the placed point along
/// the heading, `y` up from the ground, `z` to the right; metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Part {
    pub lo: [f64; 3],
    pub hi: [f64; 3],
    pub paint: Paint,
}

fn part(lo: [f64; 3], hi: [f64; 3], paint: Paint) -> Part {
    Part { lo, hi, paint }
}

/// A pair of plates at `x`, heights `y`, one at each side at `|z|` in `z`: the left first.
fn pair(x: [f64; 2], y: [f64; 2], z: [f64; 2], paint: Paint, out: &mut Vec<Part>) {
    out.push(part([x[0], y[0], -z[1]], [x[1], y[1], -z[0]], paint));
    out.push(part([x[0], y[0], z[0]], [x[1], y[1], z[1]], paint));
}

/// The rear's dark tail panel and its brake lamps, and the front's headlamps (OQ-4 (c)):
/// the panel 0.08 m beyond the brake lamps' heights and 0.05 m inside the half-width `w2`.
fn lamps(h: f64, rear_y: [f64; 2], head_y: [f64; 2], z: [f64; 2], w2: f64, out: &mut Vec<Part>) {
    out.push(part(
        [-h - T, rear_y[0] - 0.08, -(w2 - 0.05)],
        [-h, rear_y[1] + 0.08, w2 - 0.05],
        Paint::Dark,
    ));
    pair([-h - 2.0 * T, -h - T], rear_y, z, Paint::Brake, out);
    pair([h, h + T], head_y, z, Paint::Head, out);
}

/// The parts of a `kind` of length `length`, §2.6's tables row by row in their order.
pub fn parts(kind: Kind, length: f64) -> Vec<Part> {
    let (l, h) = (length, length / 2.0);
    let mut v = Vec::new();
    match kind {
        Kind::Car => {
            v.push(part([-h, FLOOR, -0.9], [h, 0.85, 0.9], Paint::Body));
            v.push(part(
                [-0.32 * l, 0.85, -0.78],
                [0.16 * l, 1.5, 0.78],
                Paint::Cabin,
            ));
            lamps(h, [0.55, 0.75], [0.5, 0.65], [0.5, 0.85], 0.9, &mut v);
        }
        Kind::Truck => {
            let w2 = 1.25;
            let c = (0.35 * l).min(2.3);
            let g = (0.03 * l).min(0.25);
            v.push(part([-h, FLOOR, -1.1], [h, 0.95, 1.1], Paint::Dark));
            v.push(part([h - c, 0.95, -w2], [h, 3.1, w2], Paint::Body));
            v.push(part([-h, 0.95, -w2], [h - c - g, 3.6, w2], Paint::Body));
            v.push(part([h, 1.9, -1.1], [h + T, 2.9, 1.1], Paint::Glass));
            pair(
                [h - c + 0.3, h - 0.15],
                [1.9, 2.8],
                [w2, w2 + T],
                Paint::Glass,
                &mut v,
            );
            lamps(h, [1.05, 1.3], [1.05, 1.3], [0.75, 1.15], w2, &mut v);
        }
        Kind::Bus => {
            let w2 = 1.275;
            v.push(part([-h, FLOOR, -w2], [h, 3.1, w2], Paint::Body));
            pair(
                [-h + 0.6, h - 0.4],
                [1.25, 2.6],
                [w2, w2 + T],
                Paint::Glass,
                &mut v,
            );
            v.push(part([h, 0.9, -1.15], [h + T, 2.85, 1.15], Paint::Glass));
            v.push(part([-h - T, 1.8, -0.9], [-h, 2.7, 0.9], Paint::Glass));
            lamps(h, [0.5, 0.8], [0.45, 0.7], [0.85, 1.2], w2, &mut v);
        }
        Kind::Bike => {
            v.push(part([-h, FLOOR, -0.04], [h, 0.75, 0.04], Paint::Dark));
            v.push(part([-0.35, 0.75, -0.22], [0.15, 1.65, 0.22], Paint::Body));
        }
        Kind::Plain => {
            v.push(part([-h, FLOOR, -0.9], [h, 1.55, 0.9], Paint::Body));
            lamps(h, [0.55, 0.75], [0.5, 0.65], [0.5, 0.85], 0.9, &mut v);
        }
    }
    v
}
