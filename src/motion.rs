//! Motion between samples (vis-001 §2.8). Before the first frame, every vehicle's rows
//! from the whole file become a track: monotone cubic Hermite intervals along links
//! (§2.8.1), junction spans on the engine's turn path timed by the live speed (§2.8.2),
//! smoothstep lane slides (§2.8.4), and the interval it is drawn in (§2.8.5).

use assimilator_core::network_data::RoutePair;

use crate::clock::EPS;
use crate::fcd::{Fcd, Row};
use crate::place::{Placed, Placement};

/// `r′` outside this band is out of band (§2.8.2).
pub const BAND: (f64, f64) = (0.8, 1.25);
/// Below this integral of speed (m) a span is timed by time instead (§2.8.2).
const I_MIN: f64 = 1e-3;

/// The piece of the drawn path a vehicle is on. Links index [`Fcd::links`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    Link(u32),
    /// A turn path, keyed by approach and departure link, entry lane and the path's own
    /// departure lane. Both lanes are `None` on a movement's centreline (§2.8.2 step 3).
    Turn {
        a: u32,
        b: u32,
        entry: Option<u32>,
        dep: Option<u32>,
    },
    /// The straight chord of a transparent node or a span with no matching movement.
    Chord {
        a: u32,
        b: u32,
    },
}

/// Where a vehicle is on its track.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackPos {
    pub piece: Piece,
    /// Distance along the piece: link `s` (centreline), or arc length on a path.
    pub along: f64,
    /// Lateral offset: from the link centreline on a link, from the path on a path.
    pub lateral: f64,
    /// Distance along the drawn path since the vehicle's first row, lateral motion
    /// excluded.
    pub odo: f64,
}

/// A vehicle at one time.
#[derive(Debug, Clone, Copy)]
pub struct Track {
    pub at: Placed,
    pub length: f64,
    /// FCD speed, linear between rows.
    pub speed: f64,
    pub pos: TrackPos,
}

/// How a span's path was found (§2.8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// Step 1: the per-lane path for the FCD lanes.
    LanePath,
    /// Step 2: the per-lane path to the engine's departure lane.
    EngineLane,
    /// Step 3: the movement's centreline.
    Centreline,
    /// No `JunctionConfig`: the chord.
    Transparent,
    /// No movement matches, or no path in steps 1–3: the chord.
    NoMovement,
}

#[derive(Debug, Clone)]
pub struct IntervalRecord {
    pub vehicle_id: u64,
    pub t0: f64,
    pub t1: f64,
    pub v0: f64,
    pub v1: f64,
    pub clamped: bool,
    pub backward: bool,
}

#[derive(Debug, Clone)]
pub struct SpanRecord {
    pub vehicle_id: u64,
    pub a: String,
    pub b: String,
    pub entry_lane: u32,
    pub departure_lane: u32,
    /// The path's own departure lane: `departure_lane` at step 1 and on a chord, the
    /// engine's at step 2, `None` on a centreline.
    pub path_departure_lane: Option<u32>,
    pub how: How,
    pub t_a: f64,
    pub t_d: f64,
    pub g: f64,
    pub i: f64,
    pub r: f64,
    pub r_prime: f64,
    /// `L_a − p_f`, the frozen shortfall (OQ-8).
    pub shortfall: f64,
    pub out_of_band: bool,
}

/// What motion found in the file (§2.8.6's evidence). Never printed by `render`.
#[derive(Debug, Clone, Default)]
pub struct MotionReport {
    pub intervals: Vec<IntervalRecord>,
    pub clamped: usize,
    pub backward: usize,
    pub spans: Vec<SpanRecord>,
    pub lane_path: usize,
    pub engine_lane: usize,
    pub centreline: usize,
    pub transparent: usize,
    pub no_movement: usize,
    pub out_of_band: usize,
    /// Lane slides along links, on a span's approach, and to a span's departure lane.
    pub slides_along: usize,
    pub slides_entry: usize,
    pub slides_departure: usize,
}

struct Along {
    t0: f64,
    t1: f64,
    link: u32,
    s0: f64,
    s1: f64,
    m0: f64,
    m1: f64,
    v0: f64,
    v1: f64,
    lat0: f64,
    lat1: f64,
    backward: bool,
    odo0: f64,
    length: f64,
}

struct Span {
    a: u32,
    b: u32,
    /// Row times and speeds A…D, and the speed integral at each.
    times: Vec<f64>,
    speeds: Vec<f64>,
    cum: Vec<f64>,
    s_a: f64,
    /// Lengths of the approach piece and the path.
    l1: f64,
    lt: f64,
    g: f64,
    i: f64,
    path: Vec<(f64, f64)>,
    piece: Piece,
    base_a: f64,
    base_b: f64,
    /// Slide from row A's lane to the path's entry lane over `[t_A, t_{A+1}]`, and from
    /// the path's departure lane to row D's over `[t_{D−1}, t_D]`.
    d_in: f64,
    d_out: f64,
    odo0: f64,
    length: f64,
}

enum Seg {
    Along(Along),
    Span(Box<Span>),
}

impl Seg {
    fn t0(&self) -> f64 {
        match self {
            Seg::Along(a) => a.t0,
            Seg::Span(s) => s.times[0],
        }
    }
}

struct VehicleTrack {
    vehicle_id: u64,
    t_first: f64,
    t_last: f64,
    /// Used only when the vehicle has a single row.
    first: Row,
    segs: Vec<Seg>,
}

pub struct Motion {
    vehicles: Vec<VehicleTrack>,
    report: MotionReport,
}

fn smooth(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn lerp(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a) * u
}

impl Motion {
    pub fn build(fcd: &Fcd, pl: &Placement) -> Motion {
        let mut report = MotionReport::default();
        let vehicles = fcd
            .vehicles
            .iter()
            .filter(|v| !v.rows.is_empty())
            .map(|v| build_vehicle(v.vehicle_id, &v.rows, fcd, pl, &mut report))
            .collect();
        Motion { vehicles, report }
    }

    pub fn report(&self) -> &MotionReport {
        &self.report
    }

    /// The largest number of vehicles drawn at once.
    pub fn max_drawn(&self) -> usize {
        let mut ev: Vec<(f64, i32)> = self
            .vehicles
            .iter()
            .flat_map(|v| [(v.t_first - EPS, 1), (v.t_last + EPS, -1)])
            .collect();
        ev.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)));
        let (mut n, mut max) = (0i32, 0i32);
        for (_, d) in ev {
            n += d;
            max = max.max(n);
        }
        max as usize
    }

    /// Every vehicle drawn at `t` (§2.8.5), in `vehicle_id` order.
    pub fn at(&self, t: f64, fcd: &Fcd, pl: &Placement) -> Vec<(u64, Track)> {
        self.vehicles
            .iter()
            .filter(|v| v.t_first - EPS <= t && t <= v.t_last + EPS)
            .map(|v| (v.vehicle_id, eval_vehicle(v, t, fcd, pl)))
            .collect()
    }
}

fn build_vehicle(
    vehicle_id: u64,
    rows: &[Row],
    fcd: &Fcd,
    pl: &Placement,
    report: &mut MotionReport,
) -> VehicleTrack {
    let name = |l: u32| fcd.links[l as usize].as_str();
    let mut segs = Vec::new();
    let mut odo = 0.0;
    // Junction spans first: (A, D) for each link change.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for d in 1..rows.len() {
        if rows[d].link != rows[d - 1].link {
            let p_f = rows[d - 1].position;
            let mut vs = d - 1;
            while vs > 0 && rows[vs - 1].link == rows[d - 1].link {
                vs -= 1;
            }
            let a = (vs..d - 1)
                .rev()
                .find(|&j| rows[j].position < p_f)
                .unwrap_or(vs);
            spans.push((a, d));
        }
    }
    let mut i = 0;
    let mut next_span = spans.iter().peekable();
    while i + 1 < rows.len() {
        if let Some(&&(a, d)) = next_span.peek()
            && i == a
        {
            let span = build_span(vehicle_id, &rows[a..=d], odo, fcd, pl, report);
            odo += span.g;
            segs.push(Seg::Span(Box::new(span)));
            next_span.next();
            i = d;
            continue;
        }
        let (r0, r1) = (&rows[i], &rows[i + 1]);
        let dt = r1.time - r0.time;
        let ds = r1.position - r0.position;
        let (mut m0, mut m1) = (r0.speed * dt, r1.speed * dt);
        let mut clamped = false;
        if ds <= 0.0 {
            m0 = 0.0;
            m1 = 0.0;
        } else {
            let (al, be) = (m0 / ds, m1 / ds);
            let q = al * al + be * be;
            if q > 9.0 {
                let k = 3.0 / q.sqrt();
                m0 *= k;
                m1 *= k;
                clamped = true;
            }
        }
        let backward = ds < 0.0;
        report.clamped += clamped as usize;
        report.backward += backward as usize;
        report.intervals.push(IntervalRecord {
            vehicle_id,
            t0: r0.time,
            t1: r1.time,
            v0: r0.speed,
            v1: r1.speed,
            clamped,
            backward,
        });
        if r0.lane != r1.lane {
            report.slides_along += 1;
        }
        let link = name(r0.link);
        segs.push(Seg::Along(Along {
            t0: r0.time,
            t1: r1.time,
            link: r0.link,
            s0: r0.position,
            s1: r1.position,
            m0,
            m1,
            v0: r0.speed,
            v1: r1.speed,
            lat0: pl.lane_offset(link, r0.lane),
            lat1: pl.lane_offset(link, r1.lane),
            backward,
            odo0: odo,
            length: r0.length,
        }));
        odo += ds;
        i += 1;
    }
    VehicleTrack {
        vehicle_id,
        t_first: rows[0].time,
        t_last: rows[rows.len() - 1].time,
        first: rows[0],
        segs,
    }
}

/// A span's path: how it was found, its polyline and length, its piece, and the lateral
/// bases on the approach and departure links.
type Found = (How, Vec<(f64, f64)>, f64, Piece, f64, f64);

/// One junction span, rows `A…D` (§2.8.2).
fn build_span(
    vehicle_id: u64,
    rows: &[Row],
    odo0: f64,
    fcd: &Fcd,
    pl: &Placement,
    report: &mut MotionReport,
) -> Span {
    let n = rows.len();
    let (ra, rf, rd) = (&rows[0], &rows[n - 2], &rows[n - 1]);
    let (a, b) = (ra.link, rd.link);
    let (an, bn) = (
        fcd.links[a as usize].as_str(),
        fcd.links[b as usize].as_str(),
    );
    let entry = rf.lane;
    let dep = rd.lane;
    let l_a = pl.link_length(an);
    let p_f = rf.position;

    // The movement and its path.
    let mut found: Option<Found> = None;
    let mut transparent = false;
    if let Some(node) = pl.to_node(an) {
        match pl.route_pair(&node, an, bn) {
            Some(RoutePair::Junction(m)) | Some(RoutePair::Waypoint(m)) => {
                let off_a = pl.lane_offset(an, entry);
                if let Some(p) = pl.lane_turn_path(&node, m, entry, dep) {
                    let piece = Piece::Turn {
                        a,
                        b,
                        entry: Some(entry),
                        dep: Some(dep),
                    };
                    let base_b = pl.lane_offset(bn, dep);
                    found = Some((
                        How::LanePath,
                        p.path.clone(),
                        p.length,
                        piece,
                        off_a,
                        base_b,
                    ));
                } else {
                    let d2 = pl.resolve_departure_lane(m, entry);
                    if let Some(p) = pl.lane_turn_path(&node, m, entry, d2) {
                        let piece = Piece::Turn {
                            a,
                            b,
                            entry: Some(entry),
                            dep: Some(d2),
                        };
                        let base_b = pl.lane_offset(bn, d2);
                        found = Some((
                            How::EngineLane,
                            p.path.clone(),
                            p.length,
                            piece,
                            off_a,
                            base_b,
                        ));
                    } else if let Some(p) = pl.turn_path(&node, m) {
                        let piece = Piece::Turn {
                            a,
                            b,
                            entry: None,
                            dep: None,
                        };
                        found = Some((How::Centreline, p.path.clone(), p.length, piece, 0.0, 0.0));
                    }
                }
            }
            Some(RoutePair::Transparent) => transparent = true,
            None => {}
        }
    }
    let (how, path, lt, piece, base_a, base_b) = found.unwrap_or_else(|| {
        let p0 = pl.place(an, l_a, entry).expect("placed link");
        let p1 = pl.place(bn, 0.0, dep).expect("placed link");
        let len = ((p1.x - p0.x).powi(2) + (p1.y - p0.y).powi(2)).sqrt();
        (
            if transparent {
                How::Transparent
            } else {
                How::NoMovement
            },
            vec![(p0.x, p0.y), (p1.x, p1.y)],
            len,
            Piece::Chord { a, b },
            pl.lane_offset(an, entry),
            pl.lane_offset(bn, dep),
        )
    });
    match how {
        How::LanePath => report.lane_path += 1,
        How::EngineLane => report.engine_lane += 1,
        How::Centreline => report.centreline += 1,
        How::Transparent => report.transparent += 1,
        How::NoMovement => report.no_movement += 1,
    }
    let d_in = pl.lane_offset(an, ra.lane) - base_a;
    let d_out = pl.lane_offset(bn, dep) - base_b;
    report.slides_entry += (d_in.abs() > 1e-9) as usize;
    report.slides_departure += (d_out.abs() > 1e-9) as usize;

    let times: Vec<f64> = rows.iter().map(|r| r.time).collect();
    let speeds: Vec<f64> = rows.iter().map(|r| r.speed).collect();
    let mut cum = vec![0.0];
    for k in 1..n {
        let c = cum[k - 1] + (speeds[k - 1] + speeds[k]) / 2.0 * (times[k] - times[k - 1]);
        cum.push(c);
    }
    let i = cum[n - 1];
    let l1 = (l_a - ra.position).max(0.0);
    let g = l1 + lt + rd.position;
    let shortfall = l_a - p_f;
    let r = g / i;
    let r_prime = (g - shortfall) / i;
    let out_of_band = !(BAND.0..=BAND.1).contains(&r_prime);
    report.out_of_band += out_of_band as usize;
    report.spans.push(SpanRecord {
        vehicle_id,
        a: an.to_string(),
        b: bn.to_string(),
        entry_lane: entry,
        departure_lane: dep,
        path_departure_lane: match piece {
            Piece::Turn { dep, .. } => dep,
            _ => Some(dep),
        },
        how,
        t_a: times[0],
        t_d: times[n - 1],
        g,
        i,
        r,
        r_prime,
        shortfall,
        out_of_band,
    });
    Span {
        a,
        b,
        times,
        speeds,
        cum,
        s_a: ra.position,
        l1,
        lt,
        g,
        i,
        path,
        piece,
        base_a,
        base_b,
        d_in,
        d_out,
        odo0,
        length: ra.length,
    }
}

fn eval_vehicle(v: &VehicleTrack, t: f64, fcd: &Fcd, pl: &Placement) -> Track {
    let t = t.clamp(v.t_first, v.t_last);
    if v.segs.is_empty() {
        let r = &v.first;
        let link = fcd.links[r.link as usize].as_str();
        return Track {
            at: pl.place(link, r.position, r.lane).expect("placed row"),
            length: r.length,
            speed: r.speed,
            pos: TrackPos {
                piece: Piece::Link(r.link),
                along: r.position,
                lateral: pl.lane_offset(link, r.lane),
                odo: 0.0,
            },
        };
    }
    let k = v.segs.partition_point(|s| s.t0() <= t).max(1) - 1;
    match &v.segs[k] {
        Seg::Along(s) => eval_along(s, t, fcd, pl),
        Seg::Span(s) => eval_span(s, t, fcd, pl),
    }
}

fn eval_along(s: &Along, t: f64, fcd: &Fcd, pl: &Placement) -> Track {
    let u = ((t - s.t0) / (s.t1 - s.t0)).clamp(0.0, 1.0);
    let pos = if s.backward {
        if u < 1.0 { s.s0 } else { s.s1 }
    } else {
        let (u2, u3) = (u * u, u * u * u);
        (2.0 * u3 - 3.0 * u2 + 1.0) * s.s0
            + (u3 - 2.0 * u2 + u) * s.m0
            + (-2.0 * u3 + 3.0 * u2) * s.s1
            + (u3 - u2) * s.m1
    };
    let lateral = lerp(s.lat0, s.lat1, smooth(u));
    let link = fcd.links[s.link as usize].as_str();
    Track {
        at: pl.place_lateral(link, pos, lateral).expect("placed link"),
        length: s.length,
        speed: lerp(s.v0, s.v1, u),
        pos: TrackPos {
            piece: Piece::Link(s.link),
            along: pos,
            lateral,
            odo: s.odo0 + (pos - s.s0),
        },
    }
}

fn eval_span(s: &Span, t: f64, fcd: &Fcd, pl: &Placement) -> Track {
    let n = s.times.len();
    let k = (s.times.partition_point(|&x| x <= t).max(1) - 1).min(n - 2);
    let (t0, t1) = (s.times[k], s.times[k + 1]);
    let u = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
    let speed = lerp(s.speeds[k], s.speeds[k + 1], u);
    let dt = u * (t1 - t0);
    let integral = s.cum[k] + (s.speeds[k] + speed) / 2.0 * dt;
    let d = if s.i < I_MIN {
        let (ta, td) = (s.times[0], s.times[n - 1]);
        s.g * ((t - ta) / (td - ta)).clamp(0.0, 1.0)
    } else {
        s.g * integral / s.i
    };
    // Lane slides: into the path's entry lane over the first interval, out of its
    // departure lane over the last (§2.8.4).
    let u_in = (t - s.times[0]) / (s.times[1] - s.times[0]);
    let u_out = (t - s.times[n - 2]) / (s.times[n - 1] - s.times[n - 2]);
    let delta = s.d_in * (1.0 - smooth(u_in)) + s.d_out * smooth(u_out);
    let (at, piece, along, lateral) = if d <= s.l1 {
        let lat = s.base_a + delta;
        let a = fcd.links[s.a as usize].as_str();
        let x = s.s_a + d;
        (
            pl.place_lateral(a, x, lat).expect("placed link"),
            Piece::Link(s.a),
            x,
            lat,
        )
    } else if d < s.l1 + s.lt {
        let x = d - s.l1;
        (Placement::turn_point(&s.path, x, delta), s.piece, x, delta)
    } else {
        let lat = s.base_b + delta;
        let b = fcd.links[s.b as usize].as_str();
        let x = d - s.l1 - s.lt;
        (
            pl.place_lateral(b, x, lat).expect("placed link"),
            Piece::Link(s.b),
            x,
            lat,
        )
    };
    Track {
        at,
        length: s.length,
        speed,
        pos: TrackPos {
            piece,
            along,
            lateral,
            odo: s.odo0 + d,
        },
    }
}
