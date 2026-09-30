//! `view`'s time slider (vis-001 §2.10): the bar's geometry, x ↔ t, the hit test and the
//! ticks. Plain Rust with no Bevy types; the window only draws what this computes.

/// The track's inset from each side of the window, logical pixels (§2.10.1).
pub const INSET: f64 = 16.0;
/// The track's thickness.
pub const TRACK_H: f64 = 4.0;
/// The bar's centre line above the window's bottom edge.
pub const BAR_FROM_BOTTOM: f64 = 14.0;
/// The handle's side.
pub const HANDLE: f64 = 12.0;
/// A tick's width and height.
pub const TICK_W: f64 = 1.0;
pub const TICK_H: f64 = 10.0;
/// The hit area: the bottom `HIT_H` pixels, `HIT_PAD` wider than the track at each end.
pub const HIT_H: f64 = 28.0;
pub const HIT_PAD: f64 = 8.0;
/// The readout's bottom edge above the window's bottom edge.
pub const READOUT_BOTTOM: f64 = 30.0;
/// Below this width or height there is no bar.
pub const MIN_SIZE: f64 = 64.0;
/// Ticks closer than this are thinned to a longer period (§2.10.4).
pub const TICK_MIN_PX: f64 = 8.0;
/// Tick periods in sim seconds: 1, 5, 10, 15, 30 and 60 min, then 2, 3, 6, 12 and 24 h.
pub const PERIODS: [f64; 11] = [
    60.0, 300.0, 600.0, 900.0, 1800.0, 3600.0, 7200.0, 10800.0, 21600.0, 43200.0, 86400.0,
];

/// The bar in a window of `w × h` logical pixels (origin top left, `y` down).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bar {
    pub x0: f64,
    pub x1: f64,
    pub y_bar: f64,
    /// The window's logical size.
    pub w: f64,
    pub h: f64,
}

impl Bar {
    /// The bar for a window size; `None` when `w < 64` or `h < 64`.
    pub fn new((w, h): (f64, f64)) -> Option<Bar> {
        (w >= MIN_SIZE && h >= MIN_SIZE).then(|| Bar {
            x0: INSET,
            x1: w - INSET,
            y_bar: h - BAR_FROM_BOTTOM,
            w,
            h,
        })
    }

    /// The track's length, `x1 − x0`.
    pub fn len(&self) -> f64 {
        self.x1 - self.x0
    }

    /// The handle's `x` at `t`, a two-sided blend so both ends are exact (§2.10.2).
    pub fn x(&self, t: f64, from: f64, to: f64) -> f64 {
        let u = (t - from) / (to - from);
        (1.0 - u) * self.x0 + u * self.x1
    }

    /// The time at `x`, clamped to `[from, to]`; a two-sided blend (§2.10.2).
    pub fn t(&self, x: f64, from: f64, to: f64) -> f64 {
        let u = ((x - self.x0) / (self.x1 - self.x0)).clamp(0.0, 1.0);
        (1.0 - u) * from + u * to
    }

    /// Whether `(x, y)` is in the hit area.
    pub fn hit(&self, (x, y): (f64, f64)) -> bool {
        (self.x0 - HIT_PAD..=self.x1 + HIT_PAD).contains(&x)
            && (self.h - HIT_H..=self.h).contains(&y)
    }

    /// The tick period: the first of [`PERIODS`] spacing ticks `TICK_MIN_PX` or more apart;
    /// `None` if none does.
    pub fn period(&self, from: f64, to: f64) -> Option<f64> {
        PERIODS
            .iter()
            .copied()
            .find(|p| p * self.len() / (to - from) >= TICK_MIN_PX)
    }

    /// The tick times: every `m·P` in `[from, to]`, in increasing order.
    pub fn ticks(&self, from: f64, to: f64) -> Vec<f64> {
        let Some(p) = self.period(from, to) else {
            return Vec::new();
        };
        let (m0, m1) = ((from / p).ceil() as i64, (to / p).floor() as i64);
        (m0..=m1)
            .map(|m| m as f64 * p)
            .filter(|t| (from..=to).contains(t))
            .collect()
    }
}
