//! The camera pose and its perspective projection (vis-001 §2.11.1). Plain Rust with no
//! Bevy types: `view`'s state, the keyframed flight and the gates all use it.
//!
//! World coordinates are `x` east, `y` north, `z` up, in metres. Angles are degrees.

/// The vertical field of view, degrees (Bevy's default).
pub const FOV_DEG: f64 = 45.0;
/// The pitch range, degrees: 90 is straight down. The floor keeps the horizon out of
/// every frame, since it is above `FOV_DEG / 2`.
pub const PITCH_MIN: f64 = 25.0;
pub const PITCH_MAX: f64 = 90.0;
/// The far plane, in units of the camera's distance from the look-at point.
pub const FAR_PER_D: f64 = 20.0;

/// A camera pose (§2.11.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// The look-at point on the ground, world metres.
    pub cx: f64,
    pub cy: f64,
    /// The world height visible at the look-at point, metres.
    pub height_m: f64,
    /// The compass direction the camera faces: 0 = north, clockwise. North is up at 0.
    pub yaw_deg: f64,
    /// The angle between the view axis and the ground, in [`PITCH_MIN`, `PITCH_MAX`].
    pub pitch_deg: f64,
}

impl Pose {
    /// Straight down and north up, at the look-at point.
    pub fn top_down(cx: f64, cy: f64, height_m: f64) -> Pose {
        Pose {
            cx,
            cy,
            height_m,
            yaw_deg: 0.0,
            pitch_deg: PITCH_MAX,
        }
    }

    /// The camera's distance from the look-at point, metres.
    pub fn distance(&self) -> f64 {
        distance(self.height_m)
    }

    /// The view axis `a`, image up `u` and image right `r`.
    pub fn axes(&self) -> ([f64; 3], [f64; 3], [f64; 3]) {
        let (sy, cy) = (sin_deg(self.yaw_deg), cos_deg(self.yaw_deg));
        let (sp, cp) = (sin_deg(self.pitch_deg), cos_deg(self.pitch_deg));
        (
            [sy * cp, cy * cp, -sp],
            [sy * sp, cy * sp, cp],
            [cy, -sy, 0.0],
        )
    }

    /// The eye, `(cx, cy, 0) − d·a`.
    pub fn eye(&self) -> [f64; 3] {
        let (a, _, _) = self.axes();
        let d = self.distance();
        [self.cx - d * a[0], self.cy - d * a[1], -d * a[2]]
    }
}

/// `sin` of degrees, exactly 0 or ±1 at multiples of 90°.
pub fn sin_deg(d: f64) -> f64 {
    match d.rem_euclid(360.0) {
        0.0 | 180.0 => 0.0,
        90.0 => 1.0,
        270.0 => -1.0,
        _ => d.to_radians().sin(),
    }
}

/// `cos` of degrees, exactly 0 or ±1 at multiples of 90°.
pub fn cos_deg(d: f64) -> f64 {
    match d.rem_euclid(360.0) {
        0.0 => 1.0,
        90.0 | 270.0 => 0.0,
        180.0 => -1.0,
        _ => d.to_radians().cos(),
    }
}

/// An angle wrapped to [0, 360): `rem_euclid`, with a result of 360 (from a tiny negative)
/// and −0 taken as 0. Every wrap in Phase 5 is this one.
pub fn wrap360(d: f64) -> f64 {
    let r = d.rem_euclid(360.0);
    if r >= 360.0 || r == 0.0 { 0.0 } else { r }
}

/// The turn from `y0` to `y1` the short way round, in (−180, 180]: exactly 180° apart
/// turns clockwise.
pub fn short_way(y0: f64, y1: f64) -> f64 {
    180.0 - (180.0 - (y1 - y0)).rem_euclid(360.0)
}

/// `d = height_m / (2·tan(φ/2))`.
pub fn distance(height_m: f64) -> f64 {
    height_m / (2.0 * (FOV_DEG / 2.0).to_radians().tan())
}

/// The focal length in pixels for an image `h` pixels tall.
fn focal(h: f64) -> f64 {
    (h / 2.0) / (FOV_DEG / 2.0).to_radians().tan()
}

fn dot(p: [f64; 3], q: [f64; 3]) -> f64 {
    p[0] * q[0] + p[1] * q[1] + p[2] * q[2]
}

/// Where world point `x` is in a `w × h` image: pixels, origin top left, `y` down.
pub fn project(pose: &Pose, w: f64, h: f64, x: [f64; 3]) -> (f64, f64) {
    let (a, u, r) = pose.axes();
    let e = pose.eye();
    let v = [x[0] - e[0], x[1] - e[1], x[2] - e[2]];
    let (xc, yc, zc) = (dot(v, r), dot(v, u), dot(v, a));
    let f = focal(h);
    (w / 2.0 + f * xc / zc, h / 2.0 - f * yc / zc)
}

/// The point on the horizontal plane at height `z` seen at `pixel` in a `w × h` image.
pub fn ray_to_plane(pose: &Pose, w: f64, h: f64, (px, py): (f64, f64), z: f64) -> (f64, f64) {
    let (a, u, r) = pose.axes();
    let e = pose.eye();
    let f = focal(h);
    let (xc, yc) = ((px - w / 2.0) / f, -(py - h / 2.0) / f);
    let dir = [
        xc * r[0] + yc * u[0] + a[0],
        xc * r[1] + yc * u[1] + a[1],
        xc * r[2] + yc * u[2] + a[2],
    ];
    let l = (z - e[2]) / dir[2];
    (e[0] + l * dir[0], e[1] + l * dir[1])
}
