//! The fixed render clock (vis-001 §2.3, §2.4): frame `n` shows sim time
//! `from + n · speedup / fps`, and the frame count follows from the window alone.

/// Guards against a window accumulated from 0.1 s steps landing a hair above a whole
/// number of frames (§2.4), and against snapshot times a hair above a frame's time.
pub const EPS: f64 = 1e-6;

/// The default speed-up: the video lasts the window clamped to 30 s – 5 min.
pub fn default_speedup(duration: f64) -> f64 {
    duration / duration.clamp(30.0, 300.0)
}

/// `N = ceil(D · fps / speedup − 1e-6)`.
pub fn frame_count(duration: f64, fps: u32, speedup: f64) -> u64 {
    let n = (duration * fps as f64 / speedup - EPS).ceil();
    if n < 1.0 { 1 } else { n as u64 }
}

/// Sim time shown by frame `n` (from 0).
pub fn frame_time(from: f64, n: u64, speedup: f64, fps: u32) -> f64 {
    from + n as f64 * speedup / fps as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_defaults() {
        // The fixture's FCD: 9.1 s … 299.1 s, accumulated in 0.1 s steps.
        let mut from = 0.1_f64;
        for _ in 0..90 {
            from += 0.1;
        }
        let mut to = from;
        for _ in 0..2900 {
            to += 0.1;
        }
        let d = to - from;
        let s = default_speedup(d);
        assert!((s - 1.0).abs() < 1e-12);
        assert_eq!(frame_count(d, 30, s), 8700);
    }

    #[test]
    fn clamps() {
        assert!((default_speedup(3600.0) - 12.0).abs() < 1e-12);
        assert!((default_speedup(10.0) - 1.0 / 3.0).abs() < 1e-12);
        assert_eq!(frame_count(10.0, 30, default_speedup(10.0)), 900);
        assert_eq!(frame_count(60.0, 24, 2.0), 720);
    }
}
