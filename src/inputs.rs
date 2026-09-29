//! The run's files, read in place (vis-001 §2.1). Nothing here writes to the project.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use assimilator_config::network::NetworkConfig;
use assimilator_config::project::{ProjectConfig, resolve_scenario};

/// Where a run's files are, after the defaults of §2.1 and the CLI overrides.
#[derive(Debug, Clone)]
pub struct RunPaths {
    pub project_dir: PathBuf,
    pub scenario: String,
    pub seed: u64,
    pub results: PathBuf,
    pub fcd: PathBuf,
}

impl RunPaths {
    /// `results.db` defaults to `<project>/results.db`; FCD to
    /// `fcd/<scenario>_<seed>.parquet` beside the database.
    pub fn new(
        project_dir: &Path,
        scenario: &str,
        seed: u64,
        results: Option<PathBuf>,
        fcd: Option<PathBuf>,
    ) -> Self {
        let results = results.unwrap_or_else(|| project_dir.join("results.db"));
        let fcd = fcd.unwrap_or_else(|| {
            results
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .join("fcd")
                .join(format!("{scenario}_{seed}.parquet"))
        });
        RunPaths {
            project_dir: project_dir.to_path_buf(),
            scenario: scenario.to_string(),
            seed,
            results,
            fcd,
        }
    }
}

/// The scenario's resolved network, loaded as the engine CLI's `load_project_scenario`
/// does, without `--set` overrides.
pub fn load_network(project_dir: &Path, scenario: &str) -> Result<NetworkConfig> {
    let file = project_dir.join("project.yaml");
    let text = std::fs::read_to_string(&file)
        .with_context(|| format!("missing file: cannot read {}", file.display()))?;
    let value = assimilator_config::parse_and_check(&text, &file.display().to_string())?;
    let project: ProjectConfig =
        serde_yaml::from_value(value).context("failed to parse project.yaml")?;
    Ok(resolve_scenario(&project, scenario, project_dir)?.network)
}

/// Require a `runs` row for scenario and seed whose status is `completed`. The database
/// is opened read-only and `immutable=1`, so no `-wal`/`-shm` file appears beside it.
pub fn require_completed_run(results: &Path, scenario: &str, seed: u64) -> Result<()> {
    if !results.is_file() {
        bail!("missing file: {}", results.display());
    }
    let abs = results
        .canonicalize()
        .with_context(|| format!("missing file: {}", results.display()))?;
    let uri = format!(
        "file:{}?mode=ro&immutable=1",
        uri_escape(&abs.to_string_lossy())
    );
    let flags = rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
        | rusqlite::OpenFlags::SQLITE_OPEN_URI
        | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let conn = rusqlite::Connection::open_with_flags(&uri, flags)
        .with_context(|| format!("cannot open {}", results.display()))?;
    let statuses: Vec<String> = conn
        .prepare("SELECT status FROM runs WHERE scenario = ?1 AND seed = ?2")
        .and_then(|mut st| {
            st.query_map(rusqlite::params![scenario, seed as i64], |r| r.get(0))?
                .collect::<rusqlite::Result<Vec<String>>>()
        })
        .with_context(|| format!("schema mismatch: cannot read runs in {}", results.display()))?;
    if !statuses.iter().any(|s| s == "completed") {
        bail!(
            "no completed run for scenario {scenario:?} seed {seed} in {} (found: {:?})",
            results.display(),
            statuses
        );
    }
    Ok(())
}

/// Percent-escape the characters SQLite's URI parser treats specially.
fn uri_escape(path: &str) -> String {
    let mut out = String::with_capacity(path.len());
    for c in path.chars() {
        match c {
            '%' => out.push_str("%25"),
            '?' => out.push_str("%3f"),
            '#' => out.push_str("%23"),
            _ => out.push(c),
        }
    }
    out
}
