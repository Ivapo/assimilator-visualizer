//! `view`'s state (vis-001 §2.9): the clock, the camera, picking, following and the
//! keyframe line. Plain Rust with no Bevy types: one frame's input goes in, the new state
//! comes out, so the gates drive it with scripted input and no window.

use crate::render::VehicleBox;
use crate::scene::{self, Camera, Strip};

/// The finest zoom, metres per logical pixel (§2.9.2).
pub const K_MIN: f64 = 0.02;
/// Real time counted per frame at most, seconds (§2.9.1).
pub const DT_CAP: f64 = 0.1;
/// A frame step, sim seconds: one frame of `render` at its defaults (§2.9.1).
pub const FRAME_STEP: f64 = 1.0 / 30.0;
/// §2.4's epsilon, sim seconds.
pub const EPS: f64 = 1e-6;
/// A press becomes a drag this far from where it was pressed, logical pixels.
pub const DRAG_PX: f64 = 4.0;
/// The pick radius, logical pixels (§2.9.3).
pub const PICK_PX: f64 = 8.0;
/// One scroll line of zoom.
pub const ZOOM_STEP: f64 = 1.1;
/// Trackpad pixels per scroll line.
pub const PX_PER_LINE: f64 = 20.0;
/// The speed ladder is `2^(i − 3)` for `i` in `0..=9`: 1/8× to 64×.
pub const SPEED_STEPS: usize = 10;
const SPEED_ONE: usize = 3;

/// Whether a key's logical character doubles the speed: `+` or `=`, whatever the keyboard
/// layout (the character, not the key's position, vis-001 §2.9.1).
pub fn is_speed_up(c: &str) -> bool {
    matches!(c, "+" | "=")
}

/// Whether a key's logical character halves the speed: `-`.
pub fn is_speed_down(c: &str) -> bool {
    c == "-"
}

/// Keys pressed this frame (one action per press; held keys do not repeat).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Pressed {
    pub space: bool,
    pub plus: bool,
    pub minus: bool,
    pub left: bool,
    pub right: bool,
    pub esc: bool,
    pub k: bool,
}

/// Keys held this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Held {
    pub shift: bool,
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
}

/// One frame of input.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ViewInput {
    pub pressed: Pressed,
    pub held: Held,
    /// The cursor in logical pixels, origin top left, `y` down; `None` outside the window.
    pub cursor: Option<(f64, f64)>,
    /// The left button went down this frame.
    pub press: bool,
    /// The left button went up this frame.
    pub release: bool,
    /// Scroll in lines, positive up (zoom in).
    pub scroll_lines: f64,
    /// Scroll in trackpad pixels, positive up.
    pub scroll_pixels: f64,
    /// The window's size in logical pixels.
    pub size: (f64, f64),
    /// Real seconds since the last frame.
    pub dt: f64,
}

/// The launch fit (§2.9.2): `render`'s camera at the window's logical size, and the
/// fitted rectangle (the strips' bounding box plus `scene::MARGIN`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub cx: f64,
    pub cy: f64,
    pub k: f64,
    /// `(x0, y0, x1, y1)`, world metres.
    pub rect: (f64, f64, f64, f64),
    /// The window's logical size at launch.
    pub size: (f64, f64),
}

impl Fit {
    pub fn new(strips: &[Strip], width: u32, height: u32) -> Fit {
        let cam = Camera::fit(strips, width, height);
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
        let m = scene::MARGIN;
        Fit {
            cx: cam.cx,
            cy: cam.cy,
            k: cam.k,
            rect: (x0 - m, y0 - m, x1 + m, y1 + m),
            size: (width as f64, height as f64),
        }
    }

    pub fn k_max(&self) -> f64 {
        2.0 * self.k
    }
}

/// A left-button press not yet released.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Press {
    /// Where it was pressed, logical pixels (moved when a zoom re-anchors a drag).
    pub at: (f64, f64),
    /// The centre at the press.
    pub centre: (f64, f64),
    pub dragging: bool,
}

/// The vehicle followed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Follow {
    pub vehicle_id: u64,
    /// Whether it is drawn at `t`; if not, the camera holds (§2.9.3).
    pub drawn: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewState {
    pub from: f64,
    pub to: f64,
    /// Snapshot times in increasing order, including the one before `from`.
    pub snaps: Vec<f64>,
    pub fit: Fit,
    pub t: f64,
    pub playing: bool,
    /// Index into the speed ladder.
    pub speed: usize,
    pub cx: f64,
    pub cy: f64,
    /// Metres per logical pixel.
    pub k: f64,
    /// The window's logical size, as of the last frame.
    pub size: (f64, f64),
    pub press: Option<Press>,
    pub follow: Option<Follow>,
}

impl ViewState {
    pub fn new(from: f64, to: f64, snaps: Vec<f64>, fit: Fit) -> ViewState {
        ViewState {
            from,
            to,
            snaps,
            fit,
            t: from,
            playing: false,
            speed: SPEED_ONE,
            cx: fit.cx,
            cy: fit.cy,
            k: fit.k,
            size: fit.size,
            press: None,
            follow: None,
        }
    }

    /// Sim seconds per real second.
    pub fn speed(&self) -> f64 {
        2f64.powi(self.speed as i32 - SPEED_ONE as i32)
    }

    /// The world point under a cursor at `(px, py)` logical pixels.
    pub fn world(&self, (px, py): (f64, f64)) -> (f64, f64) {
        let (w, h) = self.size;
        (
            self.cx + (px - w / 2.0) * self.k,
            self.cy - (py - h / 2.0) * self.k,
        )
    }

    /// Apply one frame of input: clock; `Esc`; pan and zoom; click; follow; then `K`,
    /// whose line describes the state after the frame.
    pub fn frame(
        &mut self,
        input: &ViewInput,
        boxes_at: impl Fn(f64) -> Vec<VehicleBox>,
    ) -> Option<String> {
        self.size = input.size;
        let dt = input.dt.clamp(0.0, DT_CAP);
        self.clock(input, dt);

        if input.pressed.esc {
            self.follow = None;
        }

        // Pan and zoom.
        if input.press
            && let Some(c) = input.cursor
        {
            self.press = Some(Press {
                at: c,
                centre: (self.cx, self.cy),
                dragging: false,
            });
        }
        if let (Some(p), Some(c)) = (self.press.as_mut(), input.cursor) {
            let (dx, dy) = (c.0 - p.at.0, c.1 - p.at.1);
            if !p.dragging && dx.hypot(dy) >= DRAG_PX {
                p.dragging = true;
                self.follow = None;
            }
            if p.dragging {
                self.cx = p.centre.0 - dx * self.k;
                self.cy = p.centre.1 + dy * self.k;
            }
        }
        let held = input.held;
        if held.w || held.a || held.s || held.d {
            self.follow = None;
            let v = 0.5 * self.size.0 * self.k * dt;
            if held.w {
                self.cy += v;
            }
            if held.s {
                self.cy -= v;
            }
            if held.a {
                self.cx -= v;
            }
            if held.d {
                self.cx += v;
            }
        }
        let n = input.scroll_lines + input.scroll_pixels / PX_PER_LINE;
        if n != 0.0 {
            let k2 = (self.k * ZOOM_STEP.powf(-n)).clamp(K_MIN, self.fit.k_max());
            let p = match (self.follow, input.cursor) {
                (None, Some(c)) => self.world(c),
                _ => (self.cx, self.cy),
            };
            let r = k2 / self.k;
            self.cx = p.0 + (self.cx - p.0) * r;
            self.cy = p.1 + (self.cy - p.1) * r;
            self.k = k2;
        }
        self.clamp_centre();
        // A zoom during a drag re-anchors it, so the drag continues from here.
        if n != 0.0
            && let (Some(p), Some(c)) = (self.press.as_mut(), input.cursor)
            && p.dragging
        {
            p.at = c;
            p.centre = (self.cx, self.cy);
        }

        // Click.
        let mut boxes: Option<Vec<VehicleBox>> = None;
        // A release always ends the press; it is a click only if it never became a drag.
        if input.release
            && let Some(p) = self.press.take()
            && !p.dragging
        {
            let at = self.world(input.cursor.unwrap_or(p.at));
            let bs = boxes.get_or_insert_with(|| boxes_at(self.t));
            if let Some(id) = pick(bs, at, self.k) {
                self.follow = Some(Follow {
                    vehicle_id: id,
                    drawn: false,
                });
            }
        }

        // Follow.
        if let Some(f) = self.follow.as_mut() {
            let bs = boxes.get_or_insert_with(|| boxes_at(self.t));
            match bs.iter().find(|b| b.vehicle_id == f.vehicle_id) {
                Some(b) => {
                    f.drawn = true;
                    self.cx = b.at.x;
                    self.cy = b.at.y;
                }
                None => f.drawn = false,
            }
        }

        input.pressed.k.then(|| self.keyframe_line())
    }

    fn clock(&mut self, input: &ViewInput, dt: f64) {
        let p = input.pressed;
        if p.plus {
            self.speed = (self.speed + 1).min(SPEED_STEPS - 1);
        }
        if p.minus {
            self.speed = self.speed.saturating_sub(1);
        }
        if p.space {
            if self.playing {
                self.playing = false;
            } else {
                if self.t >= self.to {
                    self.t = self.from;
                }
                self.playing = true;
            }
        }
        if p.left || p.right {
            self.playing = false;
            let t = self.t;
            self.t = match (p.right, input.held.shift) {
                (true, false) => t + FRAME_STEP,
                (false, false) => t - FRAME_STEP,
                (true, true) => self
                    .snaps
                    .iter()
                    .copied()
                    .find(|&s| s > t + EPS)
                    .unwrap_or(t),
                (false, true) => self
                    .snaps
                    .iter()
                    .rev()
                    .copied()
                    .find(|&s| s < t - EPS)
                    .unwrap_or(t),
            };
        }
        if self.playing {
            self.t += self.speed() * dt;
            if self.t >= self.to {
                self.t = self.to;
                self.playing = false;
            }
        }
        self.t = self.t.clamp(self.from, self.to);
    }

    fn clamp_centre(&mut self) {
        let (x0, y0, x1, y1) = self.fit.rect;
        self.cx = self.cx.clamp(x0, x1);
        self.cy = self.cy.clamp(y0, y1);
    }

    /// The camera on screen as a keyframe (§2.9.5).
    pub fn keyframe(&self) -> Keyframe {
        let at = match self.follow {
            Some(Follow {
                vehicle_id,
                drawn: true,
            }) => At::Follow(vehicle_id),
            _ => At::Centre(self.cx, self.cy),
        };
        Keyframe {
            t: self.t,
            at,
            height_m: self.size.1 * self.k,
        }
    }

    pub fn keyframe_line(&self) -> String {
        self.keyframe().format()
    }

    /// The readout (§2.9.4).
    pub fn readout(&self) -> String {
        let s = self.speed();
        let speed = if s >= 1.0 {
            format!("{s}")
        } else {
            format!("1/{}", 1.0 / s)
        };
        let mut line = format!(
            "t {:.2} s [{:.1}–{:.1}]  ×{speed}  {}",
            self.t,
            self.from,
            self.to,
            if self.playing { "playing" } else { "paused" }
        );
        if let Some(f) = self.follow {
            line.push_str(&format!("  following {}", f.vehicle_id));
            if !f.drawn {
                line.push_str(" (not drawn)");
            }
            line.push_str(" — Esc to stop");
        }
        line
    }
}

/// The distance from `p` to box `b`'s footprint, the `length × BOX_WIDTH` rectangle at
/// its heading; 0 inside it.
pub fn footprint_distance(b: &VehicleBox, p: (f64, f64)) -> f64 {
    let h = b.at.heading.to_radians();
    let (dx, dy) = (p.0 - b.at.x, p.1 - b.at.y);
    // Forward (sin h, cos h) and right (cos h, −sin h), heading clockwise from north.
    let u = dx * h.sin() + dy * h.cos();
    let v = dx * h.cos() - dy * h.sin();
    let eu = (u.abs() - b.length / 2.0).max(0.0);
    let ev = (v.abs() - scene::BOX_WIDTH / 2.0).max(0.0);
    eu.hypot(ev)
}

/// The box nearest world point `p` within `PICK_PX · k` metres; on a tie, the higher
/// `vehicle_id` (the one drawn on top).
pub fn pick(boxes: &[VehicleBox], p: (f64, f64), k: f64) -> Option<u64> {
    let mut best: Option<(f64, u64)> = None;
    for b in boxes {
        let d = footprint_distance(b, p);
        if d > PICK_PX * k {
            continue;
        }
        best = match best {
            Some((bd, bid)) if bd < d || (bd == d && bid > b.vehicle_id) => Some((bd, bid)),
            _ => Some((d, b.vehicle_id)),
        };
    }
    best.map(|(_, id)| id)
}

/// Where a keyframe's camera is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum At {
    Centre(f64, f64),
    Follow(u64),
}

/// One keyframe line (§2.9.5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    pub t: f64,
    pub at: At,
    pub height_m: f64,
}

impl Keyframe {
    pub fn format(&self) -> String {
        match self.at {
            At::Centre(x, y) => format!(
                "{{ t = {:.3}, x = {:.2}, y = {:.2}, height_m = {:.2} }}",
                self.t, x, y, self.height_m
            ),
            At::Follow(id) => format!(
                "{{ t = {:.3}, follow = {id}, height_m = {:.2} }}",
                self.t, self.height_m
            ),
        }
    }

    /// Parse exactly the two forms [`Keyframe::format`] writes; reject anything else.
    pub fn parse(line: &str) -> Result<Keyframe, String> {
        let inner = line
            .strip_prefix("{ ")
            .and_then(|s| s.strip_suffix(" }"))
            .ok_or_else(|| format!("not an inline table: {line:?}"))?;
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        for part in inner.split(", ") {
            let (k, v) = part
                .split_once(" = ")
                .ok_or_else(|| format!("not `key = value`: {part:?}"))?;
            keys.push(k);
            vals.push(v);
        }
        let num = |v: &str| -> Result<f64, String> {
            let ok = !v.is_empty()
                && v.chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '.')
                && v.chars().any(|c| c.is_ascii_digit());
            match (ok, v.parse::<f64>()) {
                (true, Ok(x)) => Ok(x),
                _ => Err(format!("not a number: {v:?}")),
            }
        };
        match keys.as_slice() {
            ["t", "x", "y", "height_m"] => Ok(Keyframe {
                t: num(vals[0])?,
                at: At::Centre(num(vals[1])?, num(vals[2])?),
                height_m: num(vals[3])?,
            }),
            ["t", "follow", "height_m"] => {
                let id = vals[1];
                if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
                    return Err(format!("not a vehicle id: {id:?}"));
                }
                Ok(Keyframe {
                    t: num(vals[0])?,
                    at: At::Follow(id.parse().map_err(|e| format!("{id:?}: {e}"))?),
                    height_m: num(vals[2])?,
                })
            }
            _ => Err(format!(
                "keys must be t, x, y, height_m or t, follow, height_m: {line:?}"
            )),
        }
    }
}
