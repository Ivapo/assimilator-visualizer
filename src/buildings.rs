//! Buildings around a georeferenced network (vis-002): the `buildings.geojson` cache of
//! `scripts/fetch-buildings.sh`, read and checked (§2.4), projected with the engine's
//! import formula (§2.5), and the walls and roofs of each building as mesh data in world
//! metres (§2.6). A building with Overture `building_part`s is drawn from its parts, each
//! from its own base (§2.16). No Bevy: `src/draw.rs` bakes the mesh.

use std::collections::{HashMap, HashSet};
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

/// Which rule gave a building or a part its height (§2.4.2).
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

/// One building of the file, a feature without a `building_id`: a `Polygon` is one
/// polygon, a `MultiPolygon` several.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub id: String,
    pub polygons: Vec<Polygon>,
    pub height: f64,
    pub rule: HeightRule,
    /// The base, metres: `min_height`, else `min_floor` × [`FLOOR_HEIGHT_M`], else 0
    /// (§2.16.5). Not drawn when the building has parts.
    pub base: f64,
    /// Its `building_part`s, in file order. With any, the building is drawn from them only.
    pub parts: Vec<Part>,
}

impl Building {
    /// The drawn top (§2.16.5): its highest part's `height`, else its own `height`.
    pub fn top(&self) -> f64 {
        if self.parts.is_empty() {
            self.height
        } else {
            self.parts
                .iter()
                .map(|p| p.height)
                .fold(f64::NEG_INFINITY, f64::max)
        }
    }
}

/// One `building_part` of a building: a feature whose `building_id` names it. Its top and
/// base follow the building's rules (§2.16.5); the building's own are not used for it.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub id: String,
    pub polygons: Vec<Polygon>,
    pub base: f64,
    pub height: f64,
    pub rule: HeightRule,
}

/// How many buildings (not parts) took each height rule.
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
    /// The highest drawn top ([`Building::top`]), metres.
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

/// One feature, checked, still in lng/lat.
struct RawFeature {
    id: String,
    polygons: RawPolygons,
    height: f64,
    rule: HeightRule,
    building_id: Option<String>,
    base: f64,
}

/// Read and check `path` against `network` (§2.4.3, §2.16.5), in order: the file, then
/// each feature in file order with its own checks in order, then that every part's
/// `building_id` names a building of the file, then `map_origin`, then the extent. The
/// first failure is the error.
pub fn read(path: &Path, network: &NetworkConfig) -> Result<Buildings, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read the file: {e}"))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("not JSON: {e}"))?;
    let features = match (doc.get("type"), doc.get("features")) {
        (Some(Value::String(t)), Some(Value::Array(f))) if t == "FeatureCollection" => f,
        _ => return Err("not a GeoJSON FeatureCollection with a `features` array".into()),
    };

    // The buildings with parts: every string `building_id` of the file.
    let with_parts: HashSet<&str> = features
        .iter()
        .filter_map(|f| f.get("properties")?.get("building_id")?.as_str())
        .collect();

    let mut raw: Vec<RawFeature> = Vec::with_capacity(features.len());
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
        let props = f.get("properties");
        let (height, rule) = height(props).map_err(err)?;
        let building_id = match props.and_then(|p| p.get("building_id")) {
            None | Some(Value::Null) => None,
            Some(Value::String(b)) => Some(b.clone()),
            Some(v) => return Err(err(format!("building_id {v} is not a string"))),
        };
        let base = base(props).map_err(err)?;
        // A building with parts is not drawn, so its own base and top are not compared.
        if (building_id.is_some() || !with_parts.contains(id.as_str())) && base >= height {
            return Err(err(format!(
                "base {base} m is not below its top {height} m"
            )));
        }
        raw.push(RawFeature {
            id,
            polygons,
            height,
            rule,
            building_id,
            base,
        });
    }

    let is_building: HashSet<&str> = raw
        .iter()
        .filter(|r| r.building_id.is_none())
        .map(|r| r.id.as_str())
        .collect();
    for r in &raw {
        if let Some(b) = &r.building_id
            && !is_building.contains(b.as_str())
        {
            return Err(format!(
                "feature {}: part of {b}, which is not a building of the file",
                r.id
            ));
        }
    }

    let origin = map_origin(network)?;
    let mut counts = Counts::default();
    let mut buildings: Vec<Building> = Vec::with_capacity(is_building.len());
    let mut index: HashMap<String, usize> = HashMap::with_capacity(is_building.len());
    let mut parts: Vec<(String, Part)> = Vec::new();
    for r in raw {
        let polygons = r
            .polygons
            .into_iter()
            .map(|rings| project(origin, rings))
            .collect();
        match r.building_id {
            None => {
                match r.rule {
                    HeightRule::Height => counts.height += 1,
                    HeightRule::NumFloors => counts.num_floors += 1,
                    HeightRule::Default => counts.default += 1,
                }
                index.entry(r.id.clone()).or_insert(buildings.len());
                buildings.push(Building {
                    id: r.id,
                    polygons,
                    height: r.height,
                    rule: r.rule,
                    base: r.base,
                    parts: Vec::new(),
                });
            }
            Some(b) => parts.push((
                b,
                Part {
                    id: r.id,
                    polygons,
                    base: r.base,
                    height: r.height,
                    rule: r.rule,
                },
            )),
        }
    }
    for (b, part) in parts {
        buildings[index[&b]].parts.push(part);
    }
    let tallest = buildings.iter().map(Building::top).fold(0.0, f64::max);

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

/// §2.16.5: `min_height` if present and not null, else `min_floor` × [`FLOOR_HEIGHT_M`],
/// else 0. A value that is used must be valid.
fn base(props: Option<&Value>) -> Result<f64, String> {
    let get = |k: &str| props.and_then(|p| p.get(k)).filter(|v| !v.is_null());
    if let Some(h) = get("min_height") {
        return match h.as_f64() {
            Some(v) if v.is_finite() && v >= 0.0 => Ok(v),
            _ => Err(format!(
                "min_height {h} is not a finite number of at least 0"
            )),
        };
    }
    if let Some(n) = get("min_floor") {
        return match n.as_f64() {
            Some(v) if v >= 0.0 && v.fract() == 0.0 => Ok(v * FLOOR_HEIGHT_M),
            _ => Err(format!("min_floor {n} is not an integer of at least 0")),
        };
    }
    Ok(0.0)
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
/// seen from outside the solid. A building is one volume, or one per part in file order;
/// each volume's walls come first, a quad per ring edge `a → b` as `a0 b0 b1 a1` with one
/// normal, and its roofs follow.
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

/// One building's walls and roofs (§2.6, §2.16.6): without parts, its own polygons from
/// its base to its height; with parts, each part's from the part's base to its height, in
/// file order, each part's walls then its roof. With base 0 and no parts, the walls of
/// §2.6 from `z` = 0.
pub fn building_mesh(b: &Building) -> MeshData {
    let mut m = MeshData::default();
    if b.parts.is_empty() {
        volume(&mut m, &b.polygons, b.base, b.height);
    } else {
        for p in &b.parts {
            volume(&mut m, &p.polygons, p.base, p.height);
        }
    }
    m
}

/// One volume's walls and roofs. Walls stand from `z` = `z0` to `h` on every ring edge,
/// holes included, each facing out: the right of the ring's direction. Each polygon's roof
/// is triangulated with `earcut` at `z` = `h`, facing up. No floor and no underside.
fn volume(m: &mut MeshData, polygons: &[Polygon], z0: f64, h: f64) {
    for ring in polygons.iter().flat_map(Polygon::rings) {
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
                [a[0], a[1], z0],
                [c[0], c[1], z0],
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
    for p in polygons {
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
}

/// Every building's [`building_mesh`], in file order.
pub fn mesh_data(buildings: &Buildings) -> MeshData {
    let mut m = MeshData::default();
    for b in &buildings.buildings {
        m.append(building_mesh(b));
    }
    m
}
