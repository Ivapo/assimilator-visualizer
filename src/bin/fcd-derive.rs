//! Derives the vis-001 Phase 1 test FCD files from an engine run's FCD Parquet
//! (Phase 1, fixture step 5). Columns are found by name; every other column is copied.
//!
//! ```text
//! fcd-derive lengths      --input <fcd> --output <file>   # vehicle_length 2/8/12 m by vehicle_id mod 3 (set or overwritten)
//! fcd-derive unknown-link --input <fcd> --output <file>   # first row's link_id → an id absent from the network
//! fcd-derive subset       --input <fcd> --output <file> --vehicle <id>
//! fcd-derive drop-columns --input <fcd> --output <file> --column <name>...   # e.g. a pre-asm-020 file
//! ```

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use arrow_array::{ArrayRef, BooleanArray, Float64Array, RecordBatch, StringArray, UInt64Array};
use arrow_schema::{DataType, Field, Schema};
use arrow_select::filter::filter_record_batch;
use clap::{Parser, Subcommand};
use parquet::arrow::ArrowWriter;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;

/// A link id no network has.
const UNKNOWN_LINK: &str = "__vis_unknown_link__";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    Lengths {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    UnknownLink {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    Subset {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        vehicle: u64,
    },
    DropColumns {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long = "column", required = true)]
        columns: Vec<String>,
    },
}

fn main() -> Result<()> {
    match Cli::parse().mode {
        Mode::Lengths { input, output } => {
            let batches = read(&input)?;
            let out: Vec<RecordBatch> = batches
                .iter()
                .map(|b| {
                    let vid = u64_col(b, "vehicle_id")?;
                    let len: Float64Array = vid
                        .iter()
                        .map(|v| v.map(|v| [2.0, 8.0, 12.0][(v % 3) as usize]))
                        .collect();
                    let mut fields: Vec<Field> = b
                        .schema()
                        .fields()
                        .iter()
                        .map(|f| f.as_ref().clone())
                        .collect();
                    let mut cols: Vec<ArrayRef> = b.columns().to_vec();
                    // An engine FCD from asm-020 Phase 3 on already has the column.
                    match b.schema().index_of("vehicle_length") {
                        Ok(i) => cols[i] = Arc::new(len),
                        Err(_) => {
                            fields.push(Field::new("vehicle_length", DataType::Float64, true));
                            cols.push(Arc::new(len));
                        }
                    }
                    Ok(RecordBatch::try_new(Arc::new(Schema::new(fields)), cols)?)
                })
                .collect::<Result<_>>()?;
            write(&output, &out)
        }
        Mode::UnknownLink { input, output } => {
            let mut batches = read(&input)?;
            let Some(first) = batches.iter().position(|b| b.num_rows() > 0) else {
                bail!("{} has no rows", input.display());
            };
            let b = &batches[first];
            let idx = b.schema().index_of("link_id")?;
            let links = b
                .column(idx)
                .as_any()
                .downcast_ref::<StringArray>()
                .context("link_id is not Utf8")?;
            let replaced: StringArray = links
                .iter()
                .enumerate()
                .map(|(i, v)| if i == 0 { Some(UNKNOWN_LINK) } else { v })
                .collect();
            let mut cols = b.columns().to_vec();
            cols[idx] = Arc::new(replaced);
            batches[first] = RecordBatch::try_new(b.schema(), cols)?;
            write(&output, &batches)
        }
        Mode::Subset {
            input,
            output,
            vehicle,
        } => {
            let batches = read(&input)?;
            let out: Vec<RecordBatch> = batches
                .iter()
                .map(|b| {
                    let keep: BooleanArray = u64_col(b, "vehicle_id")?
                        .iter()
                        .map(|v| Some(v == Some(vehicle)))
                        .collect();
                    Ok(filter_record_batch(b, &keep)?)
                })
                .collect::<Result<_>>()?;
            if out.iter().all(|b| b.num_rows() == 0) {
                bail!("vehicle {vehicle} is not in {}", input.display());
            }
            write(&output, &out)
        }
        Mode::DropColumns {
            input,
            output,
            columns,
        } => {
            let batches = read(&input)?;
            let out: Vec<RecordBatch> = batches
                .iter()
                .map(|b| {
                    let mut b = b.clone();
                    for name in &columns {
                        let i = b.schema().index_of(name).with_context(|| {
                            format!("{} has no column {name:?}", input.display())
                        })?;
                        b.remove_column(i);
                    }
                    Ok(b)
                })
                .collect::<Result<_>>()?;
            write(&output, &out)
        }
    }
}

fn u64_col<'a>(b: &'a RecordBatch, name: &str) -> Result<&'a UInt64Array> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<UInt64Array>())
        .with_context(|| format!("no UInt64 column {name:?}"))
}

fn read(path: &PathBuf) -> Result<Vec<RecordBatch>> {
    let f = File::open(path).with_context(|| format!("cannot open {}", path.display()))?;
    let reader = ParquetRecordBatchReaderBuilder::try_new(f)?.build()?;
    Ok(reader.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn write(path: &PathBuf, batches: &[RecordBatch]) -> Result<()> {
    let schema = batches.first().context("no batches")?.schema();
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let props = WriterProperties::builder()
        .set_compression(Compression::ZSTD(ZstdLevel::default()))
        .build();
    let mut w = ArrowWriter::try_new(File::create(path)?, schema, Some(props))?;
    for b in batches {
        w.write(b)?;
    }
    w.close()?;
    Ok(())
}
