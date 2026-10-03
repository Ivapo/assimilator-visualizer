//! The credit line of an imported network's render (vis-002 §2.14): built from the data,
//! drawn in the bottom-right corner of every frame. This module builds the text and its
//! numbers; `draw::spawn_credit` draws it and `render::Renderer` fits it. No Bevy.
//!
//! Its inputs, read before the first frame and only when the network has
//! `metadata.map_origin`: the project's `import_report.json` (its `source`), and the
//! `--buildings` file's release and each building's datasets (§2.14.3).

use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use anyhow::{Result, anyhow, bail};
use assimilator_config::network::NetworkConfig;
use serde_json::Value;

/// Item 1, always (§2.14.2).
pub const OSM: &str = "© OpenStreetMap contributors (ODbL)";

/// Between items: a middle dot between two spaces.
pub const SEPARATOR: &str = " · ";

/// The buildings file's `description`, before the release (§2.14.3).
pub const DESCRIPTION_PREFIX: &str = "Overture Maps buildings, release ";

/// Item 3's known datasets, `sources.dataset` → the text in the line, in the line's order
/// (§2.14.2). OpenStreetMap is item 1; any other dataset follows these, verbatim.
pub const DATASETS: [(&str, &str); 4] = [
    (
        "Esri Community Maps",
        "Esri Community Maps contributors (CC BY 4.0)",
    ),
    ("Google Open Buildings", "Google Open Buildings (CC BY 4.0)"),
    ("Microsoft ML Buildings", "Microsoft ML Buildings (ODbL)"),
    ("USGS Lidar", "USGS Lidar"),
];

/// The fill, sRGB (§2.14.5).
pub const FILL: [u8; 3] = [240, 240, 240];
/// The outline, sRGB.
pub const OUTLINE: [u8; 3] = [0, 0, 0];
/// A final font size under this is an error (§2.14.7).
pub const MIN_SIZE: u32 = 10;

/// Where the roads came from, per `import_report.json`'s `source` (§2.14.4).
enum Roads {
    /// No report, or `"source": null`.
    Unknown,
    Osm,
    Overture(String),
}

/// The buildings file's provenance: its release and the datasets its buildings name.
struct Provenance {
    release: String,
    datasets: BTreeSet<String>,
}

/// §2.14.2's line for a render of `network` in `project_dir`, with the `--buildings` file
/// at `buildings` if any, which `buildings::read` has already accepted. `None` without
/// `metadata.map_origin`, with nothing read. Otherwise the report is checked, then the
/// buildings file's two members (§2.14.7); the first failure is the error.
pub fn line(
    network: &NetworkConfig,
    project_dir: &Path,
    buildings: Option<&Path>,
) -> Result<Option<String>> {
    if network.metadata.map_origin.is_none() {
        return Ok(None);
    }
    let roads = roads(&project_dir.join("import_report.json"))?;
    let provenance = buildings.map(provenance).transpose()?;

    let mut items = vec![OSM.to_string()];
    let release = match (&provenance, &roads) {
        (Some(p), _) => Some(p.release.as_str()),
        (None, Roads::Overture(r)) => Some(r.as_str()),
        (None, Roads::Unknown | Roads::Osm) => None,
    };
    if let Some(r) = release {
        items.push(format!("Overture Maps Foundation, release {r}"));
    }
    if let Some(p) = provenance {
        let mut rest = p.datasets;
        rest.remove("OpenStreetMap");
        for (dataset, text) in DATASETS {
            if rest.remove(dataset) {
                items.push(text.to_string());
            }
        }
        items.extend(rest);
    }
    Ok(Some(items.join(SEPARATOR)))
}

/// `import_report.json` (§2.14.7, check 1). An absent file is not an error.
fn roads(path: &Path) -> Result<Roads> {
    let err = |e: String| anyhow!("{}: {e}", path.display());
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Roads::Unknown),
        Err(e) => return Err(err(format!("cannot read the file: {e}"))),
    };
    let doc: Value = serde_json::from_str(&text).map_err(|e| err(format!("not JSON: {e}")))?;
    let Value::Object(doc) = doc else {
        return Err(err("not a JSON object".into()));
    };
    // The engine's `Option<ImportSource>`: a missing key reads as null.
    let source = match doc.get("source") {
        None | Some(Value::Null) => return Ok(Roads::Unknown),
        Some(Value::Object(s)) => s,
        Some(_) => {
            return Err(err(
                "source is neither null nor an object with a string type".into(),
            ));
        }
    };
    match source.get("type") {
        Some(Value::String(t)) if t == "Overture" => match source.get("release") {
            Some(Value::String(r)) => Ok(Roads::Overture(r.clone())),
            _ => Err(err("source type \"Overture\" has no string release".into())),
        },
        Some(Value::String(t)) if t == "Osm" => Ok(Roads::Osm),
        Some(Value::String(t)) => Err(err(format!(
            "source type \"{t}\" has no credit (vis-002 OQ-11)"
        ))),
        _ => Err(err(
            "source is neither null nor an object with a string type".into(),
        )),
    }
}

/// The buildings file's two members (§2.14.3; §2.14.7, check 2).
fn provenance(path: &Path) -> Result<Provenance> {
    let err = |e: String| anyhow!("--buildings {}: {e}", path.display());
    let text =
        std::fs::read_to_string(path).map_err(|e| err(format!("cannot read the file: {e}")))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| err(format!("not JSON: {e}")))?;
    let release = doc
        .get("description")
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix(DESCRIPTION_PREFIX))
        .filter(|r| !r.is_empty() && !r.contains(char::is_whitespace))
        .ok_or_else(|| {
            err(
                "no release: fetched before vis-002 Phase 2; fetch it again with \
                 scripts/fetch-buildings.sh"
                    .into(),
            )
        })?
        .to_string();
    let mut datasets = BTreeSet::new();
    let features = doc
        .get("features")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    for (i, f) in features.iter().enumerate() {
        let props = f.get("properties");
        let name = match props.and_then(|p| p.get("id")) {
            Some(Value::String(id)) => id.clone(),
            _ => (i + 1).to_string(),
        };
        match props.and_then(|p| p.get("sources")) {
            None | Some(Value::Null) => {}
            Some(Value::Array(a)) if a.iter().all(Value::is_string) => {
                datasets.extend(a.iter().filter_map(Value::as_str).map(str::to_string));
            }
            Some(s) => bail!(err(format!(
                "feature {name}: sources {s} is not an array of strings"
            ))),
        }
    }
    Ok(Provenance { release, datasets })
}

/// The font size for a `width` × `height` frame, px: `round(min(H, 9W/16) / 54)`
/// (§2.14.5). The `9W/16` term keeps a portrait frame's line inside its width.
pub fn font_size(width: u32, height: u32) -> u32 {
    let h = (height as f64).min(9.0 * width as f64 / 16.0);
    (h / 54.0).round() as u32
}

/// From the right and bottom edges to the fill text's box, px.
pub fn margin(size: u32) -> u32 {
    size
}

/// The outline copies' offset, px: `max(1, round(S/12))`.
pub fn outline_offset(size: u32) -> u32 {
    ((size as f64 / 12.0).round() as u32).max(1)
}

/// A final font size under [`MIN_SIZE`]: the size it would be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooSmall(pub u32);

impl fmt::Display for TooSmall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the credit line would be {} px", self.0)
    }
}

impl std::error::Error for TooSmall {}

/// One pass of the fit (§2.14.5): the line laid out at `size` is `width_px` wide, and
/// `avail_px` is `W − 2·margin(S)` for the first size `S`. It keeps `size` when it fits,
/// else becomes `floor(size · avail_px / width_px)`. A result under [`MIN_SIZE`] is an
/// error, including a `size` already under it.
pub fn fit_size(size: u32, width_px: f64, avail_px: f64) -> Result<u32, TooSmall> {
    let s = if width_px <= avail_px {
        size
    } else {
        (size as f64 * avail_px / width_px).floor() as u32
    };
    if s < MIN_SIZE {
        Err(TooSmall(s))
    } else {
        Ok(s)
    }
}
