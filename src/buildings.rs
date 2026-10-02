//! Buildings around a georeferenced network (vis-002): the `buildings.geojson` cache of
//! `scripts/fetch-buildings.sh`, read and checked (§2.4), projected with the engine's
//! import formula (§2.5), and the walls and roofs of each building as mesh data in world
//! metres (§2.6). No Bevy: `src/draw.rs` bakes the mesh.

use std::f64::consts::PI;
use std::path::Path;

use assimilator_config::network::NetworkConfig;
use earcut::Earcut;
use serde_json::Value;

/// A floor's height when a building has `num_floors` but no `height`, metres.
pub const FLOOR_HEIGHT_M: f64 = 3.5;
/// A building's height when it has neither, metres.
pub const DEFAULT_HEIGHT_M: f64 = 10.0;

/// lng/lat (degrees) to world metres about `origin`, which is `metadata.map_origin`
/// (`[lng, lat]`). The engine's `wgs84_to_metric`, copied with its operation order so the
/// result is bit for bit the engine's: `origin_lat * PI / 180.0` is not `to_radians()`.
pub fn lnglat_to_xy(origin: [f64; 2], lng: f64, lat: f64) -> (f64, f64) {
    let (origin_lng, origin_lat) = (origin[0], origin[1]);
    let x = (lng - origin_lng) * 111320.0 * (origin_lat * PI / 180.0).cos();
    let y = (lat - origin_lat) * 111320.0;
    (x, y)
}

/// The inverse of [`lnglat_to_xy`]: world metres to `(lng, lat)`.
pub fn xy_to_lnglat(origin: [f64; 2], x: f64, y: f64) -> (f64, f64) {
    let (origin_lng, origin_lat) = (origin[0], origin[1]);
    let lng = origin_lng + x / (111320.0 * (origin_lat * PI / 180.0).cos());
    let lat = origin_lat + y / 111320.0;
    (lng, lat)
}

/// The network's own extent, `[x0, y0, x1, y1]` metres: every node point and every link
/// `geometry` point (§2.3.1).
pub fn network_extent(network: &NetworkConfig) -> [f64; 4] {
    let mut e = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let nodes = network.nodes.iter().map(|n| [n.x, n.y]);
    let geometry = network
        .links
        .iter()
        .flat_map(|l| l.geometry.iter().copied());
    for [x, y] in nodes.chain(geometry) {
        e = [e[0].min(x), e[1].min(y), e[2].max(x), e[3].max(y)];
    }
    e
}

fn map_origin(network: &NetworkConfig) -> Result<[f64; 2], String> {
    network.metadata.map_origin.ok_or_else(|| {
        "the network has no metadata.map_origin: buildings need a georeferenced network".into()
    })
}

/// The box `scripts/fetch-buildings.sh` fetches: the network's extent plus `margin`
/// metres, in lng/lat, rounded outward to 7 decimals. `[W, S, E, N]` in units of 1e-7°:
/// west and south take the floor of value × 10⁷, east and north the ceiling.
pub fn fetch_bbox(network: &NetworkConfig, margin: f64) -> Result<[i64; 4], String> {
    let origin = map_origin(network)?;
    let [x0, y0, x1, y1] = network_extent(network);
    if !(x0.is_finite() && y0.is_finite()) {
        return Err("the network has no points".into());
    }
    let (w, s) = xy_to_lnglat(origin, x0 - margin, y0 - margin);
    let (e, n) = xy_to_lnglat(origin, x1 + margin, y1 + margin);
    Ok([
        (w * 1e7).floor() as i64,
        (s * 1e7).floor() as i64,
        (e * 1e7).ceil() as i64,
        (n * 1e7).ceil() as i64,
    ])
}

/// A value in units of 1e-7° as a decimal with 7 places, exactly.
pub fn format_e7(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let a = v.unsigned_abs();
    format!("{sign}{}.{:07}", a / 10_000_000, a % 10_000_000)
}

/// Which rule gave a building its height (§2.4.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeightRule {
    Height,
    NumFloors,
    Default,
}

/// One polygon in world metres, oriented: the exterior counter-clockwise and the holes
/// clockwise, seen from above. No ring repeats its first vertex at the end.
#[derive(Debug, Clone, PartialEq)]
pub struct Polygon {
    pub exterior: Vec<[f64; 2]>,
    pub holes: Vec<Vec<[f64; 2]>>,
}

impl Polygon {
    /// The exterior, then the holes.
    pub fn rings(&self) -> impl Iterator<Item = &Vec<[f64; 2]>> {
        std::iter::once(&self.exterior).chain(self.holes.iter())
    }
}

/// One feature of the file: a `Polygon` is one polygon, a `MultiPolygon` several.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub id: String,
    pub polygons: Vec<Polygon>,
    pub height: f64,
    pub rule: HeightRule,
}

/// How many buildings took each height rule.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Counts {
    pub height: usize,
    pub num_floors: usize,
    pub default: usize,
}

/// Every building of the file, in file order.
#[derive(Debug, Clone, PartialEq)]
pub struct Buildings {
    pub buildings: Vec<Building>,
    pub counts: Counts,
    /// The tallest height, metres.
    pub tallest: f64,
}

/// Twice the signed area of a ring (shoelace); positive when counter-clockwise.
pub fn ring_area2(ring: &[[f64; 2]]) -> f64 {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let (a, b) = (ring[i], ring[(i + 1) % n]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

/// One feature's checked geometry, still in lng/lat: polygons of rings, each with its
/// closing position.
type RawPolygons = Vec<Vec<Vec<[f64; 2]>>>;

/// Read and check `path` against `network` (§2.4.3), in order: the file, then each
/// feature in file order with its own checks in order, then `map_origin`, then the
/// extent. The first failure is the error.
pub fn read(path: &Path, network: &NetworkConfig) -> Result<Buildings, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read the file: {e}"))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("not JSON: {e}"))?;
    let features = match (doc.get("type"), doc.get("features")) {
        (Some(Value::String(t)), Some(Value::Array(f))) if t == "FeatureCollection" => f,
        _ => return Err("not a GeoJSON FeatureCollection with a `features` array".into()),
    };

    let mut raw: Vec<(String, RawPolygons, f64, HeightRule)> = Vec::with_capacity(features.len());
    for (i, f) in features.iter().enumerate() {
        let id = match f.get("properties").and_then(|p| p.get("id")) {
            Some(Value::String(id)) => id.clone(),
            _ => return Err(format!("feature {}: no string `properties.id`", i + 1)),
        };
        let err = |e: String| format!("feature {id}: {e}");
        let polygons = geometry(f.get("geometry")).map_err(err)?;
        for p in &polygons {
            for ring in p {
                for &[lng, lat] in ring {
                    if !(-180.0..=180.0).contains(&lng) {
                        return Err(err(format!("longitude {lng} is outside [-180, 180]")));
                    }
                    if !(-90.0..=90.0).contains(&lat) {
                        return Err(err(format!("latitude {lat} is outside [-90, 90]")));
                    }
                }
            }
        }
        let (height, rule) = height(f.get("properties")).map_err(err)?;
        raw.push((id, polygons, height, rule));
    }

    let origin = map_origin(network)?;
    let mut counts = Counts::default();
    let mut tallest: f64 = 0.0;
    let buildings: Vec<Building> = raw
        .into_iter()
        .map(|(id, polygons, height, rule)| {
            match rule {
                HeightRule::Height => counts.height += 1,
                HeightRule::NumFloors => counts.num_floors += 1,
                HeightRule::Default => counts.default += 1,
            }
            tallest = tallest.max(height);
            let polygons = polygons
                .into_iter()
                .map(|rings| project(origin, rings))
                .collect();
            Building {
                id,
                polygons,
                height,
                rule,
            }
        })
        .collect();

    let [x0, y0, x1, y1] = network_extent(network);
    let meets = |b: &Building| {
        let pts = || b.polygons.iter().flat_map(|p| p.exterior.iter());
        let bx0 = pts().map(|p| p[0]).fold(f64::INFINITY, f64::min);
        let bx1 = pts().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
        let by0 = pts().map(|p| p[1]).fold(f64::INFINITY, f64::min);
        let by1 = pts().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
        bx0 <= x1 && bx1 >= x0 && by0 <= y1 && by1 >= y0
    };
    if !buildings.iter().any(meets) {
        return Err(format!(
            "no building meets the network's extent (x {x0:.1}…{x1:.1} m, y {y0:.1}…{y1:.1} m)"
        ));
    }
    Ok(Buildings {
        buildings,
        counts,
        tallest,
    })
}

/// A feature's `geometry`: a `Polygon` or a `MultiPolygon` whose rings each have at least
/// 4 positions of two or more numbers, the first equal to the last.
fn geometry(g: Option<&Value>) -> Result<RawPolygons, String> {
    let Some(g) = g.filter(|g| !g.is_null()) else {
        return Err("no geometry".into());
    };
    let coords = g.get("coordinates");
    let polygons: Vec<&Value> = match g.get("type").and_then(Value::as_str) {
        Some("Polygon") => vec![coords.ok_or("a Polygon with no coordinates")?],
        Some("MultiPolygon") => coords
            .and_then(Value::as_array)
            .ok_or("a MultiPolygon whose coordinates are not an array")?
            .iter()
            .collect(),
        Some(t) => {
            return Err(format!(
                "the geometry is a {t}, not a Polygon or MultiPolygon"
            ));
        }
        None => return Err("the geometry has no type".into()),
    };
    polygons
        .into_iter()
        .map(|p| {
            let rings = p
                .as_array()
                .filter(|r| !r.is_empty())
                .ok_or("a polygon with no rings")?;
            rings.iter().map(ring).collect()
        })
        .collect()
}

fn ring(r: &Value) -> Result<Vec<[f64; 2]>, String> {
    let r = r
        .as_array()
        .ok_or("a ring that is not an array of positions")?;
    let ring = r
        .iter()
        .map(|p| match p.as_array().map(Vec::as_slice) {
            Some([lng, lat, rest @ ..])
                if lng.is_number() && lat.is_number() && rest.iter().all(Value::is_number) =>
            {
                Ok([lng.as_f64().unwrap(), lat.as_f64().unwrap()])
            }
            _ => Err(format!("a position {p} is not two or more numbers")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    if ring.len() < 4 {
        return Err(format!("a ring has {} positions, fewer than 4", ring.len()));
    }
    if ring.first() != ring.last() {
        return Err("a ring is not closed (its first position is not its last)".into());
    }
    Ok(ring)
}

/// §2.4.2: `height` if present and not null, else `num_floors` × [`FLOOR_HEIGHT_M`], else
/// [`DEFAULT_HEIGHT_M`]. A value that is used must be valid.
fn height(props: Option<&Value>) -> Result<(f64, HeightRule), String> {
    let get = |k: &str| props.and_then(|p| p.get(k)).filter(|v| !v.is_null());
    if let Some(h) = get("height") {
        return match h.as_f64() {
            Some(v) if v.is_finite() && v > 0.0 => Ok((v, HeightRule::Height)),
            _ => Err(format!("height {h} is not a positive finite number")),
        };
    }
    if let Some(n) = get("num_floors") {
        return match n.as_f64() {
            Some(v) if v >= 1.0 && v.fract() == 0.0 => {
                Ok((v * FLOOR_HEIGHT_M, HeightRule::NumFloors))
            }
            _ => Err(format!("num_floors {n} is not an integer of at least 1")),
        };
    }
    Ok((DEFAULT_HEIGHT_M, HeightRule::Default))
}

/// Project a polygon's rings, drop each closing position, and orient them: the exterior
/// counter-clockwise, the holes clockwise.
fn project(origin: [f64; 2], rings: Vec<Vec<[f64; 2]>>) -> Polygon {
    let mut rings = rings.into_iter().map(|r| {
        let mut out: Vec<[f64; 2]> = r[..r.len() - 1]
            .iter()
            .map(|&[lng, lat]| {
                let (x, y) = lnglat_to_xy(origin, lng, lat);
                [x, y]
            })
            .collect();
        if ring_area2(&out) < 0.0 {
            out.reverse();
        }
        out
    });
    let exterior = rings.next().expect("a polygon has rings");
    let holes = rings
        .map(|mut h| {
            h.reverse();
            h
        })
        .collect();
    Polygon { exterior, holes }
}

/// Mesh data in world metres (`x` east, `y` north, `z` up), triangles counter-clockwise
/// seen from outside the solid. One building's walls come first, `4 · wall_quads`
/// vertices: a quad per ring edge `a → b` as `a0 b0 b1 a1`, with one normal. Its roofs
/// follow.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeshData {
    pub positions: Vec<[f64; 3]>,
    pub normals: Vec<[f64; 3]>,
    pub indices: Vec<u32>,
    pub wall_quads: usize,
    pub roof_triangles: usize,
}

impl MeshData {
    fn append(&mut self, other: MeshData) {
        let base = self.positions.len() as u32;
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.indices.extend(other.indices.iter().map(|i| base + i));
        self.wall_quads += other.wall_quads;
        self.roof_triangles += other.roof_triangles;
    }
}

/// One building's walls and roofs (§2.6). Walls stand from `z` = 0 to the height on every
/// ring edge, holes included, each facing out: the right of the ring's direction. Each
/// polygon's roof is triangulated with `earcut` at `z` = height, facing up. No floor.
pub fn building_mesh(b: &Building) -> MeshData {
    let mut m = MeshData::default();
    let h = b.height;
    for ring in b.polygons.iter().flat_map(Polygon::rings) {
        let n = ring.len();
        for i in 0..n {
            let (a, c) = (ring[i], ring[(i + 1) % n]);
            let (dx, dy) = (c[0] - a[0], c[1] - a[1]);
            let len = dx.hypot(dy);
            let normal = if len > 0.0 {
                [dy / len, -dx / len, 0.0]
            } else {
                [0.0; 3]
            };
            let base = m.positions.len() as u32;
            m.positions.extend([
                [a[0], a[1], 0.0],
                [c[0], c[1], 0.0],
                [c[0], c[1], h],
                [a[0], a[1], h],
            ]);
            m.normals.extend([normal; 4]);
            m.indices
                .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
            m.wall_quads += 1;
        }
    }
    let mut earcut = Earcut::<f64>::new();
    let mut triangles: Vec<u32> = Vec::new();
    for p in &b.polygons {
        let mut vertices = p.exterior.clone();
        let mut holes: Vec<u32> = Vec::with_capacity(p.holes.len());
        for hole in &p.holes {
            holes.push(vertices.len() as u32);
            vertices.extend_from_slice(hole);
        }
        earcut.earcut(vertices.iter().copied(), &holes, &mut triangles);
        let base = m.positions.len() as u32;
        m.positions.extend(vertices.iter().map(|v| [v[0], v[1], h]));
        m.normals
            .extend(std::iter::repeat_n([0.0, 0.0, 1.0], vertices.len()));
        m.indices.extend(triangles.iter().map(|i| base + i));
        m.roof_triangles += triangles.len() / 3;
    }
    m
}

/// Every building's [`building_mesh`], in file order.
pub fn mesh_data(buildings: &Buildings) -> MeshData {
    let mut m = MeshData::default();
    for b in &buildings.buildings {
        m.append(building_mesh(b));
    }
    m
}
