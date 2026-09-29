//! What is drawn (vis-001 Phase 1 "Scene"): road strips on the placed polylines, a box
//! per vehicle, and the top-down camera fitted to the strips.

use crate::place::Placement;

/// Vehicle box width and height, metres, for every vehicle (the engine has no width).
pub const BOX_WIDTH: f64 = 1.8;
pub const BOX_HEIGHT: f64 = 1.5;
/// Vertical lift per rank in `vehicle_id` order among a frame's vehicles, metres. Where
/// boxes overlap, the depth test, not submission order, decides which shows: Bevy's opaque
/// pass bins draws and does not keep their order stable, so equal-height boxes would
/// resolve differently from frame to frame. The camera looks straight down
/// orthographically and nothing is lit, so the lift moves no pixel; the higher
/// `vehicle_id` wins an overlap.
pub const RANK_LIFT: f64 = 0.01;
/// Camera margin around the strips' bounding box, metres.
pub const MARGIN: f64 = 20.0;
/// Longest step between strip samples, metres.
pub const STRIP_STEP: f64 = 1.0;

/// sRGB colours. The speed colours are never the road grey or the background.
pub const BACKGROUND: [u8; 3] = [18, 22, 30];
pub const ROAD: [u8; 3] = [92, 96, 104];
/// Upper bounds (m/s) of the speed bins, and each bin's colour, slow to fast.
pub const SPEED_BINS: [(f64, [u8; 3]); 5] = [
    (2.0, [230, 57, 70]),
    (5.0, [244, 132, 45]),
    (9.0, [247, 201, 72]),
    (13.0, [144, 214, 94]),
    (f64::INFINITY, [72, 191, 227]),
];

pub fn speed_bin(speed: f64) -> usize {
    SPEED_BINS
        .iter()
        .position(|(hi, _)| speed < *hi)
        .unwrap_or(SPEED_BINS.len() - 1)
}

/// One link's road strip: left and right edge points, in sample order.
#[derive(Debug, Clone)]
pub struct Strip {
    pub left: Vec<[f64; 2]>,
    pub right: Vec<[f64; 2]>,
}

/// Strips in the network's link order, each `total_width` wide and centred on the same
/// trimmed, offset polyline placement uses, sampled through
/// `interpolate_with_lateral(link, s, 0.0)` at most [`STRIP_STEP`] apart plus the end.
pub fn strips(placement: &Placement) -> Vec<Strip> {
    let mut out = Vec::with_capacity(placement.network.links.len());
    for link in &placement.network.links {
        let id = link.id.0.as_str();
        let len = placement.link_length(id);
        let half = link.total_width() / 2.0;
        let n = ((len / STRIP_STEP).ceil() as usize).max(1);
        let mut strip = Strip {
            left: Vec::with_capacity(n + 1),
            right: Vec::with_capacity(n + 1),
        };
        for i in 0..=n {
            let s = len * i as f64 / n as f64;
            let Some(p) = placement.place_lateral(id, s, 0.0) else {
                break;
            };
            let h = p.heading.to_radians();
            // Right of travel: (cos h, −sin h) for a heading clockwise from north.
            let (rx, ry) = (h.cos(), -h.sin());
            strip.right.push([p.x + rx * half, p.y + ry * half]);
            strip.left.push([p.x - rx * half, p.y - ry * half]);
        }
        if strip.left.len() >= 2 {
            out.push(strip);
        }
    }
    out
}

/// The top-down orthographic camera. Pixel `(i, j)` covers `[i, i+1) × [j, j+1)`, with
/// `j` down and north up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub cx: f64,
    pub cy: f64,
    /// Metres per pixel.
    pub k: f64,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    /// Fit the strips' bounding box plus [`MARGIN`]: `k = max(bw / width, bh / height)`.
    pub fn fit(strips: &[Strip], width: u32, height: u32) -> Self {
        let (mut x0, mut y0, mut x1, mut y1) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for p in strips
            .iter()
            .flat_map(|s| s.left.iter().chain(s.right.iter()))
        {
            x0 = x0.min(p[0]);
            y0 = y0.min(p[1]);
            x1 = x1.max(p[0]);
            y1 = y1.max(p[1]);
        }
        let (bw, bh) = (x1 - x0 + 2.0 * MARGIN, y1 - y0 + 2.0 * MARGIN);
        let k = (bw / width as f64).max(bh / height as f64);
        Camera {
            cx: (x0 + x1) / 2.0,
            cy: (y0 + y1) / 2.0,
            k,
            width,
            height,
        }
    }

    /// World metres to continuous pixel coordinates.
    pub fn world_to_pixel(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.width as f64 / 2.0 + (x - self.cx) / self.k,
            self.height as f64 / 2.0 - (y - self.cy) / self.k,
        )
    }
}
