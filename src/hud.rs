//! The HUD (vis-003 §2.20): one line in the top-left corner of every frame, the frame's
//! sim clock, the vehicles on the road and their mean speed, `5:00 · 184 vehicles · 17
//! km/h`. This module builds the numbers and the text; `draw::spawn_hud` draws them. No
//! Bevy.
//!
//! The numbers are the FCD's: the rows of the snapshot shown at `t` (`Fcd::snapshot_at`),
//! held between rows, counted, and their `speed` averaged, stopped vehicles included
//! (§2.20.4, §2.20.5). The clock is `t` in whole seconds (§2.20.6).

use crate::fcd::Fcd;

/// `S = round(min(H, 9W/16) / RATIO)` px (§2.20.9, OQ-6 (b)).
pub const RATIO: f64 = 36.0;
/// The fill, sRGB: the credit line's (§2.20.9, OQ-6 (a)).
pub const FILL: [u8; 3] = [240, 240, 240];
/// The outline, sRGB.
pub const OUTLINE: [u8; 3] = [0, 0, 0];

/// Every glyph [`text`] can write, and the `p` of mph, laid out once at spawn so that no
/// frame depends on the glyphs an earlier one laid out (§2.20.9).
pub const GLYPHS: &str = "0123456789:·– vehiclsmp/hk";

/// Between items: a middle dot between two spaces, as the credit line's.
const SEPARATOR: &str = " · ";

/// The font size for a `w`×`h` frame, px, at least 1.
pub fn size(w: f64, h: f64) -> u32 {
    ((h.min(9.0 * w / 16.0) / RATIO).round() as u32).max(1)
}

/// The margin from the top and left edges, px: `S`.
pub fn margin(size: u32) -> u32 {
    size
}

/// How far each of the eight outline copies sits from the fill, px.
pub fn outline_offset(size: u32) -> u32 {
    ((size as f64 / 12.0).round() as u32).max(1)
}

/// The numbers at sim time `t`: the vehicles on the road, and their mean speed in m/s.
/// `drawn` says whether any box is drawn at `t`; without one the network is empty, and
/// so is the FCD with no snapshot at or before `t`: `(0, None)` (§2.20.4).
pub fn numbers(fcd: &Fcd, t: f64, drawn: bool) -> (usize, Option<f64>) {
    if !drawn {
        return (0, None);
    }
    let Some(s) = fcd.snapshot_at(t) else {
        return (0, None);
    };
    let rows = fcd.rows_of(s);
    if rows.is_empty() {
        return (0, None);
    }
    let sum: f64 = rows.iter().map(|r| r.speed).sum();
    (rows.len(), Some(sum / rows.len() as f64))
}

/// `t` floored to whole seconds within vis-001's `1e-6`, `m:ss`, or `h:mm:ss` from an
/// hour; `0:00` before 0 (§2.20.6).
pub fn clock(t: f64) -> String {
    let s = (t + 1e-6).floor().max(0.0) as u64;
    let (h, m, sec) = (s / 3600, (s / 60) % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{sec:02}")
    } else {
        format!("{m}:{sec:02}")
    }
}

/// The line (§2.20.8): the clock, the count with "vehicle" or "vehicles", and the mean
/// speed in whole km/h, halves up (OQ-5 (a)), or `–` with none.
pub fn text(t: f64, n: usize, mean: Option<f64>) -> String {
    let speed = match mean {
        Some(v) => format!("{} km/h", (3.6 * v + 0.5).floor() as i64),
        None => "– km/h".to_string(),
    };
    let noun = if n == 1 { "vehicle" } else { "vehicles" };
    [clock(t), format!("{n} {noun}"), speed].join(SEPARATOR)
}
