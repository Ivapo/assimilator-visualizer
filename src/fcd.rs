//! FCD Parquet, read by column name (vis-001 §2.1, Phase 1 "Reading FCD").

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result, bail};
use arrow_array::{
    Array, Float64Array, LargeStringArray, RecordBatch, StringArray, UInt32Array, UInt64Array,
};
use arrow_schema::DataType;
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::clock::EPS;

/// Box length when the file has no `vehicle_length` column (OQ-3).
pub const DEFAULT_LENGTH: f64 = 4.5;

/// One FCD row. `link` indexes [`Fcd::links`].
#[derive(Debug, Clone, Copy)]
pub struct Row {
    pub time: f64,
    pub vehicle_id: u64,
    pub link: u32,
    pub lane: u32,
    pub position: f64,
    pub speed: f64,
    pub length: f64,
}

/// All rows sharing one `time`, sorted by `vehicle_id`.
#[derive(Debug, Clone, Copy)]
pub struct Snapshot {
    pub time: f64,
    pub start: usize,
    pub end: usize,
}

/// The rows of a render window: every snapshot in `[from, to]` plus the last one at or
/// before `from`.
#[derive(Debug, Clone)]
pub struct Fcd {
    pub links: Vec<String>,
    pub rows: Vec<Row>,
    pub snapshots: Vec<Snapshot>,
    /// First and last `time` in the whole file.
    pub file_span: (f64, f64),
}

const REQUIRED: [(&str, DataType); 6] = [
    ("time", DataType::Float64),
    ("vehicle_id", DataType::UInt64),
    ("link_id", DataType::Utf8),
    ("lane", DataType::UInt32),
    ("position", DataType::Float64),
    ("speed", DataType::Float64),
];

/// First and last `time` in the file, for the default window.
pub fn time_span(path: &Path) -> Result<(f64, f64)> {
    let (_, _, span) = read_all(path)?;
    Ok(span)
}

/// Read the rows of the window `[from, to]`, plus the snapshot at or before `from`.
pub fn read_window(path: &Path, from: f64, to: f64) -> Result<Fcd> {
    let (links, mut rows, file_span) = read_all(path)?;
    rows.sort_by(|a, b| {
        a.time
            .total_cmp(&b.time)
            .then(a.vehicle_id.cmp(&b.vehicle_id))
    });
    let snapshots = group(&rows);
    // The last snapshot at or before `from`, else the first one.
    let first = snapshots
        .iter()
        .rposition(|s| s.time <= from + EPS)
        .unwrap_or(0);
    let keep: Vec<Snapshot> = snapshots[first..]
        .iter()
        .copied()
        .take_while(|s| s.time <= to + EPS)
        .collect();
    let (lo, hi) = match (keep.first(), keep.last()) {
        (Some(a), Some(b)) => (a.start, b.end),
        _ => (0, 0),
    };
    let rows: Vec<Row> = rows[lo..hi].to_vec();
    let snapshots = keep
        .into_iter()
        .map(|s| Snapshot {
            time: s.time,
            start: s.start - lo,
            end: s.end - lo,
        })
        .collect();
    Ok(Fcd {
        links,
        rows,
        snapshots,
        file_span,
    })
}

impl Fcd {
    /// The snapshot at the latest `time ≤ t` (within [`EPS`]), if any.
    pub fn snapshot_at(&self, t: f64) -> Option<&Snapshot> {
        let i = self.snapshots.partition_point(|s| s.time <= t + EPS);
        if i == 0 {
            None
        } else {
            Some(&self.snapshots[i - 1])
        }
    }

    pub fn rows_of(&self, s: &Snapshot) -> &[Row] {
        &self.rows[s.start..s.end]
    }
}

fn group(rows: &[Row]) -> Vec<Snapshot> {
    let mut out: Vec<Snapshot> = Vec::new();
    for (i, r) in rows.iter().enumerate() {
        match out.last_mut() {
            Some(s) if s.time == r.time => s.end = i + 1,
            _ => out.push(Snapshot {
                time: r.time,
                start: i,
                end: i + 1,
            }),
        }
    }
    out
}

/// Link names, rows (links indexed into the names) and the file's time span.
type AllRows = (Vec<String>, Vec<Row>, (f64, f64));

fn read_all(path: &Path) -> Result<AllRows> {
    let file = File::open(path).with_context(|| format!("missing file: {}", path.display()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| format!("schema mismatch: {} is not a Parquet file", path.display()))?;
    let schema = builder.schema().clone();
    let mut wanted: Vec<&str> = REQUIRED.iter().map(|(n, _)| *n).collect();
    for (name, ty) in REQUIRED.iter() {
        let field = schema.field_with_name(name).map_err(|_| {
            anyhow::anyhow!("schema mismatch: {} has no column {name:?}", path.display())
        })?;
        let ok = field.data_type() == ty
            || (*name == "link_id" && field.data_type() == &DataType::LargeUtf8);
        if !ok {
            bail!(
                "schema mismatch: {} column {name:?} is {}, expected {ty}",
                path.display(),
                field.data_type()
            );
        }
    }
    let has_length = match schema.field_with_name("vehicle_length") {
        Ok(f) if f.data_type() == &DataType::Float64 => true,
        Ok(f) => bail!(
            "schema mismatch: {} column \"vehicle_length\" is {}, expected Float64",
            path.display(),
            f.data_type()
        ),
        Err(_) => false,
    };
    if has_length {
        wanted.push("vehicle_length");
    }
    let mask = ProjectionMask::columns(builder.parquet_schema(), wanted.iter().copied());
    let reader = builder.with_projection(mask).build()?;

    let mut links: Vec<String> = Vec::new();
    let mut link_ix: HashMap<String, u32> = HashMap::new();
    let mut rows = Vec::new();
    let mut span = (f64::INFINITY, f64::NEG_INFINITY);
    for batch in reader {
        let batch = batch.with_context(|| format!("cannot read {}", path.display()))?;
        let time = f64_col(&batch, "time")?;
        let vid = col::<UInt64Array>(&batch, "vehicle_id")?;
        let lane = col::<UInt32Array>(&batch, "lane")?;
        let pos = f64_col(&batch, "position")?;
        let speed = f64_col(&batch, "speed")?;
        let length = if has_length {
            Some(f64_col(&batch, "vehicle_length")?)
        } else {
            None
        };
        let link_any = batch.column_by_name("link_id").expect("projected");
        let link_str = |i: usize| -> &str {
            if let Some(a) = link_any.as_any().downcast_ref::<StringArray>() {
                a.value(i)
            } else {
                link_any
                    .as_any()
                    .downcast_ref::<LargeStringArray>()
                    .expect("checked")
                    .value(i)
            }
        };
        for i in 0..batch.num_rows() {
            if time.is_null(i) || vid.is_null(i) || link_any.is_null(i) {
                bail!(
                    "schema mismatch: {} has a null in a required column",
                    path.display()
                );
            }
            let name = link_str(i);
            let link = match link_ix.get(name) {
                Some(&k) => k,
                None => {
                    let k = links.len() as u32;
                    links.push(name.to_string());
                    link_ix.insert(name.to_string(), k);
                    k
                }
            };
            let t = time.value(i);
            span = (span.0.min(t), span.1.max(t));
            rows.push(Row {
                time: t,
                vehicle_id: vid.value(i),
                link,
                lane: lane.value(i),
                position: pos.value(i),
                speed: speed.value(i),
                length: match length {
                    Some(l) if !l.is_null(i) => l.value(i),
                    _ => DEFAULT_LENGTH,
                },
            });
        }
    }
    if rows.is_empty() {
        bail!("{} has no FCD rows", path.display());
    }
    Ok((links, rows, span))
}

fn col<'a, T: 'static>(batch: &'a RecordBatch, name: &str) -> Result<&'a T> {
    batch
        .column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| anyhow::anyhow!("schema mismatch: column {name:?}"))
}

fn f64_col<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Float64Array> {
    col::<Float64Array>(batch, name)
}
