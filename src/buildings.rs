//! Buildings around a georeferenced network (vis-002): the `buildings.geojson` cache of
//! `scripts/fetch-buildings.sh`, read and checked (§2.4), projected with the engine's
//! import formula (§2.5), and the walls and roofs of each building as mesh data in world
//! metres (§2.6). No Bevy: `src/draw.rs` bakes the mesh.

use std::f64::consts::PI;

use assimilator_config::network::NetworkConfig;

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
