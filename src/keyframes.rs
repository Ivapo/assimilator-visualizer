//! Keyframes (vis-001 §2.11.4): the line `view`'s `K` prints, read back with TOML's
//! reader. Plain Rust with no Bevy types.

use serde::Deserialize;

use crate::camera::{PITCH_MAX, PITCH_MIN, wrap360};

/// Where a keyframe's camera is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    Centre(f64, f64),
    Follow(u64),
}

/// One keyframe line (§2.9.5, §2.11.4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub t: f64,
    pub at: At,
    pub height_m: f64,
    /// In [0, 360); 0 when the line has none.
    pub yaw_deg: f64,
    /// In [`PITCH_MIN`, `PITCH_MAX`]; 90 when the line has none.
    pub pitch_deg: f64,
}

/// One keyframe as TOML gives it, before any check.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    t: Option<f64>,
    x: Option<f64>,
    y: Option<f64>,
    follow: Option<i64>,
    height_m: Option<f64>,
    yaw_deg: Option<f64>,
    pitch_deg: Option<f64>,
}

/// A line read as the value of one key.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct One {
    k: Raw,
}

/// A TOML error as one line, with no source excerpt.
fn toml_error(e: &toml::de::Error) -> String {
    e.message().trim().replace('\n', " ")
}

impl Keyframe {
    pub fn format(&self) -> String {
        let mut yaw = format!("{:.2}", self.yaw_deg);
        if yaw == "360.00" {
            yaw = "0.00".into();
        }
        let tail = format!(
            "height_m = {:.2}, yaw_deg = {yaw}, pitch_deg = {:.2}",
            self.height_m, self.pitch_deg
        );
        match self.at {
            At::Centre(x, y) => format!(
                "{{ t = {:.3}, x = {x:.2}, y = {y:.2}, {tail} }}",
                self.t
            ),
            At::Follow(id) => format!("{{ t = {:.3}, follow = {id}, {tail} }}", self.t),
        }
    }

    /// Read one line with the file's reader (`k = <line>`), and apply every check that
    /// needs no run and no other keyframe.
    pub fn parse(line: &str) -> Result<Keyframe, String> {
        let one: One = toml::from_str(&format!("k = {line}")).map_err(|e| toml_error(&e))?;
        check(one.k)
    }
}

/// The checks of §2.11.4 on one keyframe that need no run, in the order listed there.
fn check(r: Raw) -> Result<Keyframe, String> {
    let t = r.t.ok_or("no `t`")?;
    let height_m = r.height_m.ok_or("no `height_m`")?;
    let at = match (r.x, r.y, r.follow) {
        (Some(_), _, Some(_)) | (_, Some(_), Some(_)) => {
            return Err("both `x`/`y` and `follow`".into());
        }
        (None, None, None) => return Err("neither `x`/`y` nor `follow`".into()),
        (Some(_), None, None) => return Err("`x` without `y`".into()),
        (None, Some(_), None) => return Err("`y` without `x`".into()),
        (Some(x), Some(y), None) => At::Centre(x, y),
        (None, None, Some(id)) => At::Follow(id.max(0) as u64),
    };
    let finite = |name: &str, v: f64| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err(format!("`{name}` must be finite (got {v})"))
        }
    };
    finite("t", t)?;
    finite("height_m", height_m)?;
    if let At::Centre(x, y) = at {
        finite("x", x)?;
        finite("y", y)?;
    }
    let yaw = finite("yaw_deg", r.yaw_deg.unwrap_or(0.0))?;
    let pitch = finite("pitch_deg", r.pitch_deg.unwrap_or(PITCH_MAX))?;
    if let Some(id) = r.follow
        && id < 0
    {
        return Err(format!("`follow` must be a vehicle id ≥ 0 (got {id})"));
    }
    if height_m <= 0.0 {
        return Err(format!("`height_m` must be positive (got {height_m})"));
    }
    if !(PITCH_MIN..=PITCH_MAX).contains(&pitch) {
        return Err(format!(
            "`pitch_deg` must be in [{PITCH_MIN}, {PITCH_MAX}] (got {pitch})"
        ));
    }
    Ok(Keyframe {
        t,
        at,
        height_m,
        yaw_deg: wrap360(yaw),
        pitch_deg: pitch,
    })
}
