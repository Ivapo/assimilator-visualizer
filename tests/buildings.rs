//! vis-002 Phase 1 exit gates 6–12 and 14 (`specs/city_spec.md`): the projection, reading,
//! heights, the mesh, alignment, top-down coverage, the shapes in perspective and `B`.
//! The headless ones run in plain `cargo test`. Those that need the Midtown fixture of
//! `scripts/fixture.sh midtown` or the GPU are ignored; run everything with
//! `cargo test --release --test buildings -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 3–5, 7's CLI cases and 13 are `scripts/gates-city.sh`.

use std::path::PathBuf;

use assimilator_config::network::NetworkConfig;
use assimilator_video::buildings::{
    self, Building, Buildings, HeightRule, building_mesh, lnglat_to_xy, xy_to_lnglat,
};
use serde_json::{Value, json};

/// Midtown's `metadata.map_origin`, `[lng, lat]`.
const MIDTOWN_ORIGIN: [f64; 2] = [-73.9775152177763, 40.76472024499192];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shapes() -> PathBuf {
    root().join("tests/shapes.geojson")
}

fn midtown() -> PathBuf {
    root().join("scratch/midtown")
}

/// A georeferenced network with no links: Midtown's origin and two endpoints, so its
/// extent is x −130…200, y −30…30 and holds every shape.
fn test_network(map_origin: bool) -> NetworkConfig {
    let origin = if map_origin {
        "  map_origin: [-73.9775152177763, 40.76472024499192]\n"
    } else {
        ""
    };
    serde_yaml::from_str(&format!(
        "metadata:\n  name: vis-002 test\n{origin}nodes:\n  - id: A\n    type: endpoint\n    point: [-130, -30]\n  - id: B\n    type: endpoint\n    point: [200, 30]\nlinks: []\n"
    ))
    .expect("the test network deserialises")
}

/// A scratch file for a GeoJSON case.
fn write_case(name: &str, text: &str) -> PathBuf {
    let dir = root().join("scratch/out/buildings");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, text).unwrap();
    p
}

/// A closed lng/lat ring around the rectangle `x0…x1, y0…y1` metres about Midtown's
/// origin, counter-clockwise.
fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
    let p = |x: f64, y: f64| {
        let (lng, lat) = xy_to_lnglat(MIDTOWN_ORIGIN, x, y);
        json!([lng, lat])
    };
    json!([p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1), p(x0, y0)])
}

fn feature(props: Value, geometry: Value) -> Value {
    json!({"type": "Feature", "properties": props, "geometry": geometry})
}

fn collection(features: Vec<Value>) -> String {
    json!({"type": "FeatureCollection", "features": features}).to_string()
}

fn polygon(rings: Vec<Value>) -> Value {
    json!({"type": "Polygon", "coordinates": rings})
}

// ── Gate 6: projection ───────────────────────────────────────────────────────

#[test]
fn gate6_projection() {
    let o = MIDTOWN_ORIGIN;
    let cases: [([f64; 2], f64, f64, f64, f64); 5] = [
        (o, o[0], o[1], 0.0, 0.0),
        (
            o,
            -73.9899513167745,
            40.75608090194475,
            -1048.5305648910369,
            -961.7316680111448,
        ),
        (
            o,
            -73.96507911877812,
            40.773359588039085,
            1048.5305648898386,
            961.7316680103539,
        ),
        (o, -73.974, 40.7625, 296.3801816977122, -247.1576725002842),
        (
            [-0.1, 51.5],
            -0.09,
            51.51,
            692.9832935049988,
            1113.1999999997786,
        ),
    ];
    for (origin, lng, lat, x, y) in cases {
        let got = lnglat_to_xy(origin, lng, lat);
        println!(
            "gate6 ({lng}, {lat}) about {origin:?} → ({}, {})",
            got.0, got.1
        );
        assert_eq!(got, (x, y), "({lng}, {lat}) about {origin:?}");
        let (lng2, lat2) = xy_to_lnglat(origin, x, y);
        assert!(
            (lng2 - lng).abs() <= 1e-12 && (lat2 - lat).abs() <= 1e-12,
            "inverse of ({x}, {y}): ({lng2}, {lat2})"
        );
    }
    // What the operation order rules out: the regrouped product, and `to_radians`.
    let pi = std::f64::consts::PI;
    let regrouped = (-73.974 - o[0]) * (111320.0 * (o[1] * pi / 180.0).cos());
    assert_eq!(regrouped, 296.38018169771215);
    let to_radians = (-0.09 - -0.1) * 111320.0 * 51.5f64.to_radians().cos();
    assert_eq!(to_radians, 692.9832935049986);
}

// ── Gate 7: reading errors and accepted files (the library's cases) ──────────

#[test]
fn gate7_reading_errors() {
    let net = test_network(true);
    let sq = || square(120.0, -5.0, 130.0, 5.0);
    let with = |props: Value, geometry: Value| collection(vec![feature(props, geometry)]);
    let ring_of = |pts: &[[f64; 2]]| json!(pts);
    let corners: Vec<[f64; 2]> = sq()
        .as_array()
        .unwrap()
        .iter()
        .map(|p| [p[0].as_f64().unwrap(), p[1].as_f64().unwrap()])
        .collect();
    let mut bad_pos = sq();
    bad_pos[1] = json!(["a", 40.7]);
    let mut lat91 = sq();
    lat91[2] = json!([corners[2][0], 91.0]);
    lat91[3] = json!([corners[3][0], 91.0]);
    let metres = json!([
        [294.2, -240.2],
        [304.2, -240.2],
        [304.2, -230.2],
        [294.2, -240.2]
    ]);
    let at_zero = json!([[0.0, 0.0], [0.0001, 0.0], [0.0001, 0.0001], [0.0, 0.0]]);

    let cases: Vec<(&str, Option<String>, &str)> = vec![
        ("missing", None, "cannot read the file"),
        ("not_json", Some("{ not json".into()), "not JSON"),
        ("array", Some("[]".into()), "FeatureCollection"),
        (
            "top_feature",
            Some(feature(json!({"id": "x", "height": 10}), polygon(vec![sq()])).to_string()),
            "FeatureCollection",
        ),
        (
            "no_id",
            Some(with(json!({"height": 10}), polygon(vec![sq()]))),
            "feature 1: no string `properties.id`",
        ),
        (
            "point",
            Some(with(
                json!({"id": "p"}),
                json!({"type": "Point", "coordinates": corners[0]}),
            )),
            "Point, not a Polygon",
        ),
        (
            "ring3",
            Some(with(
                json!({"id": "r3"}),
                polygon(vec![ring_of(&[corners[0], corners[1], corners[0]])]),
            )),
            "3 positions, fewer than 4",
        ),
        (
            "unclosed",
            Some(with(
                json!({"id": "open"}),
                polygon(vec![ring_of(&corners[..4])]),
            )),
            "not closed",
        ),
        (
            "position_a",
            Some(with(json!({"id": "pa"}), polygon(vec![bad_pos]))),
            "not two or more numbers",
        ),
        (
            "lat91",
            Some(with(json!({"id": "l91"}), polygon(vec![lat91]))),
            "latitude 91",
        ),
        (
            "metres",
            Some(with(json!({"id": "m"}), polygon(vec![metres]))),
            "longitude 294.2",
        ),
        (
            "height0",
            Some(with(json!({"id": "h0", "height": 0}), polygon(vec![sq()]))),
            "height 0 is not",
        ),
        (
            "height_neg",
            Some(with(
                json!({"id": "h-5", "height": -5}),
                polygon(vec![sq()]),
            )),
            "height -5 is not",
        ),
        (
            "floors0",
            Some(with(
                json!({"id": "f0", "num_floors": 0}),
                polygon(vec![sq()]),
            )),
            "num_floors 0 is not",
        ),
        (
            "floors2.5",
            Some(with(
                json!({"id": "f2.5", "num_floors": 2.5}),
                polygon(vec![sq()]),
            )),
            "num_floors 2.5 is not",
        ),
        ("empty", Some(collection(vec![])), "no building meets"),
        (
            "far",
            Some(with(json!({"id": "far"}), polygon(vec![at_zero]))),
            "no building meets",
        ),
    ];
    for (label, text, want) in cases {
        let path = match &text {
            Some(t) => write_case(&format!("gate7_{label}.geojson"), t),
            None => root().join("scratch/out/buildings/does_not_exist.geojson"),
        };
        let e = buildings::read(&path, &net).expect_err(label);
        println!("gate7 {label}: {e}");
        assert!(e.contains(want), "{label}: {e:?} lacks {want:?}");
    }

    // A network with no map_origin (urban_grid's case), after every feature check.
    let e = buildings::read(&shapes(), &test_network(false)).expect_err("no map_origin");
    println!("gate7 no map_origin: {e}");
    assert!(e.contains("metadata.map_origin"), "{e}");
}

#[test]
fn gate7_accepted() {
    let net = test_network(true);
    buildings::read(&shapes(), &net).expect("tests/shapes.geojson");

    let mut ring3d = square(120.0, -5.0, 130.0, 5.0);
    for p in ring3d.as_array_mut().unwrap() {
        p.as_array_mut().unwrap().push(json!(0));
    }
    let cases = [
        (
            "position_3d",
            collection(vec![feature(json!({"id": "z"}), polygon(vec![ring3d]))]),
        ),
        (
            "extra_and_crs",
            json!({
                "type": "FeatureCollection",
                "name": "buildings",
                "crs": {"type": "name", "properties": {"name": "urn:ogc:def:crs:OGC:1.3:CRS84"}},
                "features": [feature(
                    json!({"id": "extra", "height": 12.5, "class": "office", "names": {"primary": "x"}}),
                    polygon(vec![square(120.0, -5.0, 130.0, 5.0)]),
                )],
            })
            .to_string(),
        ),
        (
            "height_beside_bad_floors",
            collection(vec![feature(
                json!({"id": "hb", "height": 12, "num_floors": 2.5}),
                polygon(vec![square(120.0, -5.0, 130.0, 5.0)]),
            )]),
        ),
    ];
    for (label, text) in cases {
        let b = buildings::read(
            &write_case(&format!("gate7_ok_{label}.geojson"), &text),
            &net,
        )
        .unwrap_or_else(|e| panic!("{label}: {e}"));
        println!("gate7 accepted {label}: {} building(s)", b.buildings.len());
        assert_eq!(b.buildings.len(), 1, "{label}");
    }
}

// ── Gate 8: heights ──────────────────────────────────────────────────────────

#[test]
fn gate8_heights_shapes() {
    let b = buildings::read(&shapes(), &test_network(true)).unwrap();
    let want = [
        ("shape-cube", 30.0, HeightRule::Height, 1),
        ("shape-courtyard", 20.0, HeightRule::Height, 1),
        ("shape-l", 14.0, HeightRule::NumFloors, 1),
        ("shape-default", 10.0, HeightRule::Default, 1),
        ("shape-both", 12.0, HeightRule::Height, 1),
        ("shape-multi", 8.0, HeightRule::Height, 2),
    ];
    assert_eq!(b.buildings.len(), want.len());
    for (got, (id, h, rule, polygons)) in b.buildings.iter().zip(want) {
        println!(
            "gate8 {}: {} m by {:?}, {} polygon(s)",
            got.id,
            got.height,
            got.rule,
            got.polygons.len()
        );
        assert_eq!(
            (got.id.as_str(), got.height, got.rule, got.polygons.len()),
            (id, h, rule, polygons)
        );
    }
    assert_eq!(
        (b.counts.height, b.counts.num_floors, b.counts.default),
        (4, 1, 1)
    );
    assert_eq!(b.tallest, 30.0);
}

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate8_heights_midtown() {
    let b = read_midtown();
    let log = std::fs::read_to_string(midtown().join("fetch.log")).expect("fetch.log");
    let report: Value = serde_json::from_str(log.trim()).expect("one JSON line");
    println!(
        "gate8 Midtown: {} by height, {} by num_floors, {} at the default; report {report}",
        b.counts.height, b.counts.num_floors, b.counts.default
    );
    assert_eq!(
        (b.counts.height, b.counts.num_floors, b.counts.default),
        (4277, 8, 51)
    );
    assert_eq!(
        (
            report["height"].as_u64(),
            report["num_floors"].as_u64(),
            report["default"].as_u64(),
            report["buildings"].as_u64(),
        ),
        (Some(4277), Some(8), Some(51), Some(4336))
    );
}

// ── Gate 9: the mesh ─────────────────────────────────────────────────────────

/// Even-odd: is `p` inside the ring?
fn in_ring(ring: &[[f64; 2]], p: [f64; 2]) -> bool {
    let mut inside = false;
    let n = ring.len();
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            inside = !inside;
        }
    }
    inside
}

fn in_building(b: &Building, p: [f64; 2]) -> bool {
    b.polygons
        .iter()
        .any(|poly| in_ring(&poly.exterior, p) && !poly.holes.iter().any(|h| in_ring(h, p)))
}

#[derive(Debug, Default, PartialEq)]
struct MeshCheck {
    buildings: usize,
    wall_quads: usize,
    roof_triangles: usize,
    area_failures: usize,
    inward: usize,
    wound_inward: usize,
    roof_off_height: usize,
    worst_area: f64,
}

fn check_mesh(all: &Buildings) -> MeshCheck {
    let mut c = MeshCheck::default();
    for b in &all.buildings {
        let m = building_mesh(b);
        c.buildings += 1;
        c.wall_quads += m.wall_quads;
        c.roof_triangles += m.roof_triangles;
        let footprint: f64 = b
            .polygons
            .iter()
            .flat_map(|p| p.rings())
            .map(|r| buildings::ring_area2(r) / 2.0)
            .sum();
        let walls_v = 4 * m.wall_quads;
        let walls_i = 6 * m.wall_quads;
        let roof: f64 = m.indices[walls_i..]
            .chunks(3)
            .map(|t| {
                let [a, b, c] = [0, 1, 2].map(|k| m.positions[t[k] as usize]);
                ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])) / 2.0
            })
            .sum();
        let rel = (roof - footprint).abs() / footprint.abs();
        c.worst_area = c.worst_area.max(rel);
        if rel.is_nan() || rel > 1e-9 {
            c.area_failures += 1;
            println!("gate9 {}: roof {roof} m² vs footprint {footprint} m²", b.id);
        }
        c.roof_off_height += m.positions[walls_v..]
            .iter()
            .filter(|p| p[2] != b.height)
            .count();
        for q in 0..m.wall_quads {
            let [a0, b0, b1] = [0, 1, 2].map(|k| m.positions[4 * q + k]);
            let n = m.normals[4 * q];
            let mid = [(a0[0] + b0[0]) / 2.0, (a0[1] + b0[1]) / 2.0];
            if in_building(b, [mid[0] + 0.001 * n[0], mid[1] + 0.001 * n[1]]) {
                c.inward += 1;
            }
            // The first triangle's winding faces the same way as the normal.
            let (u, v) = (
                [b0[0] - a0[0], b0[1] - a0[1], b0[2] - a0[2]],
                [b1[0] - a0[0], b1[1] - a0[1], b1[2] - a0[2]],
            );
            let cross = [
                u[1] * v[2] - u[2] * v[1],
                u[2] * v[0] - u[0] * v[2],
                u[0] * v[1] - u[1] * v[0],
            ];
            if cross[0] * n[0] + cross[1] * n[1] + cross[2] * n[2] <= 0.0 {
                c.wound_inward += 1;
            }
        }
    }
    c
}

#[test]
fn gate9_mesh_shapes() {
    let c = check_mesh(&buildings::read(&shapes(), &test_network(true)).unwrap());
    println!("gate9 shapes: {c:?}");
    // Edges: cube 4, courtyard 4 + 4, L 6, default 4, both 4, multi 4 + 4.
    // Roofs: Σ (n + 2h − 2) = 2 + 8 + 4 + 2 + 2 + 2·2.
    assert_eq!(
        (
            c.buildings,
            c.wall_quads,
            c.roof_triangles,
            c.area_failures,
            c.inward,
            c.wound_inward,
            c.roof_off_height
        ),
        (6, 34, 22, 0, 0, 0, 0)
    );
}

fn read_midtown() -> Buildings {
    let net = assimilator_video::inputs::load_network(&midtown(), "baseline")
        .expect("the Midtown fixture (scripts/fixture.sh midtown)");
    buildings::read(&midtown().join("buildings.geojson"), &net).expect("the Midtown cache")
}

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate9_mesh_midtown() {
    let c = check_mesh(&read_midtown());
    println!("gate9 Midtown: {c:?}");
    assert_eq!(c.buildings, 4336, "buildings meshed");
    assert_eq!(c.area_failures, 0, "roof-area failures");
    assert_eq!(c.wall_quads, 42776, "wall quads");
    assert_eq!(c.inward, 0, "inward walls");
    assert_eq!(c.wound_inward, 0, "walls wound inward");
    assert_eq!(c.roof_off_height, 0, "roof vertices off their height");
    assert_eq!(c.roof_triangles, 34178, "roof triangles");
}

// ── Gate 10: alignment ───────────────────────────────────────────────────────

/// The parameters in [0, 1] of segment `p → q` that lie inside the union of `polys`.
fn inside_intervals(p: [f64; 2], q: [f64; 2], polys: &[&buildings::Polygon]) -> Vec<(f64, f64)> {
    let r = [q[0] - p[0], q[1] - p[1]];
    let mut out: Vec<(f64, f64)> = Vec::new();
    for poly in polys {
        let mut ts = vec![0.0, 1.0];
        for ring in poly.rings() {
            let n = ring.len();
            for i in 0..n {
                let (a, b) = (ring[i], ring[(i + 1) % n]);
                let s = [b[0] - a[0], b[1] - a[1]];
                let den = r[0] * s[1] - r[1] * s[0];
                if den == 0.0 {
                    continue;
                }
                let w = [a[0] - p[0], a[1] - p[1]];
                let t = (w[0] * s[1] - w[1] * s[0]) / den;
                let u = (w[0] * r[1] - w[1] * r[0]) / den;
                if (0.0..=1.0).contains(&u) && t > 0.0 && t < 1.0 {
                    ts.push(t);
                }
            }
        }
        ts.sort_by(f64::total_cmp);
        for w in ts.windows(2) {
            let tm = (w[0] + w[1]) / 2.0;
            let m = [p[0] + tm * r[0], p[1] + tm * r[1]];
            if w[1] > w[0]
                && in_ring(&poly.exterior, m)
                && !poly.holes.iter().any(|h| in_ring(h, m))
            {
                out.push((w[0], w[1]));
            }
        }
    }
    out.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (a, b) in out {
        match merged.last_mut() {
            Some(last) if a <= last.1 => last.1 = last.1.max(b),
            _ => merged.push((a, b)),
        }
    }
    merged
}

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate10_alignment() {
    let net = assimilator_video::inputs::load_network(&midtown(), "baseline").unwrap();
    let b = read_midtown();
    let polys: Vec<(&buildings::Polygon, [f64; 4])> = b
        .buildings
        .iter()
        .flat_map(|b| b.polygons.iter())
        .map(|p| {
            let xs = p.exterior.iter().map(|v| v[0]);
            let ys = p.exterior.iter().map(|v| v[1]);
            let bbox = [
                xs.clone().fold(f64::INFINITY, f64::min),
                ys.clone().fold(f64::INFINITY, f64::min),
                xs.fold(f64::NEG_INFINITY, f64::max),
                ys.fold(f64::NEG_INFINITY, f64::max),
            ];
            (p, bbox)
        })
        .collect();
    let (mut total, mut inside, mut links_over_1m) = (0.0, 0.0, 0);
    for link in &net.links {
        let mut link_inside = 0.0;
        for w in link.geometry.windows(2) {
            let (p, q) = (w[0], w[1]);
            let len = (q[0] - p[0]).hypot(q[1] - p[1]);
            total += len;
            let near: Vec<&buildings::Polygon> = polys
                .iter()
                .filter(|(_, bb)| {
                    p[0].min(q[0]) <= bb[2]
                        && p[0].max(q[0]) >= bb[0]
                        && p[1].min(q[1]) <= bb[3]
                        && p[1].max(q[1]) >= bb[1]
                })
                .map(|(poly, _)| *poly)
                .collect();
            link_inside += inside_intervals(p, q, &near)
                .iter()
                .map(|(a, b)| (b - a) * len)
                .sum::<f64>();
        }
        inside += link_inside;
        if link_inside > 1.0 {
            links_over_1m += 1;
        }
    }
    let share = 100.0 * inside / total;
    println!(
        "gate10: {} links, {total:.1} m of centreline, {inside:.1} m inside footprints = {share:.3} %, {links_over_1m} links with more than 1 m inside",
        net.links.len()
    );
    assert!((total - 33530.0).abs() < 0.05, "centreline length {total}");
    assert!(share <= 3.0, "{share} % inside footprints");
}
