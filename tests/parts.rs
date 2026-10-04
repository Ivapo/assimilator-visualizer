//! vis-002 Phase 4 exit gates 6, 7, 8, 10, 11 and 12 (`specs/city_spec.md`): building
//! parts read, checked and meshed, drawn through the GPU, cut, credited, and Midtown's
//! traffic. The headless ones without a fixture run in plain `cargo test`. Those that
//! need the Midtown fixture of `scripts/fixture.sh midtown` (with its parts cache,
//! `buildings-parts.geojson`) or the GPU are ignored; run everything with
//! `cargo test --release --test parts -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 5, 9 and 11's CLI half are `scripts/gates-parts.sh`.

use std::collections::HashMap;
use std::path::PathBuf;

use assimilator_config::network::NetworkConfig;
use assimilator_video::buildings::{
    self, Buildings, HeightRule, building_mesh, mesh_data, xy_to_lnglat,
};
use serde_json::{Value, json};

/// Midtown's `metadata.map_origin`, `[lng, lat]`.
const MIDTOWN_ORIGIN: [f64; 2] = [-73.9775152177763, 40.76472024499192];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn midtown() -> PathBuf {
    root().join("scratch/midtown")
}

/// `tests/buildings.rs`'s made-up network: Midtown's origin and two endpoints, so its
/// extent is x −130…200, y −30…30.
fn test_network() -> NetworkConfig {
    serde_yaml::from_str(
        "metadata:\n  name: vis-002 test\n  map_origin: [-73.9775152177763, 40.76472024499192]\nnodes:\n  - id: A\n    type: endpoint\n    point: [-130, -30]\n  - id: B\n    type: endpoint\n    point: [200, 30]\nlinks: []\n",
    )
    .expect("the test network deserialises")
}

/// A scratch file for a GeoJSON case.
fn write_case(name: &str, text: &str) -> PathBuf {
    let dir = root().join("scratch/out/parts-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(name);
    std::fs::write(&p, text).unwrap();
    p
}

/// A `Polygon` around the rectangle `x0…x1, y0…y1` metres about Midtown's origin.
fn square(x0: f64, y0: f64, x1: f64, y1: f64) -> Value {
    let p = |x: f64, y: f64| {
        let (lng, lat) = xy_to_lnglat(MIDTOWN_ORIGIN, x, y);
        json!([lng, lat])
    };
    json!({"type": "Polygon", "coordinates": [[p(x0, y0), p(x1, y0), p(x1, y1), p(x0, y1), p(x0, y0)]]})
}

fn feature(props: Value) -> Value {
    json!({"type": "Feature", "properties": props, "geometry": square(120.0, -5.0, 130.0, 5.0)})
}

fn collection(features: Vec<Value>) -> String {
    json!({"type": "FeatureCollection", "features": features}).to_string()
}

fn load_midtown_network() -> NetworkConfig {
    assimilator_video::inputs::load_network(&midtown(), "baseline")
        .expect("the Midtown fixture (scripts/fixture.sh midtown)")
}

// ── Gate 6: reading and the mesh, Midtown ────────────────────────────────────

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate6_midtown_parts() {
    let net = load_midtown_network();
    let file = midtown().join("buildings-parts.geojson");
    let b = buildings::read(&file, &net).expect("buildings-parts.geojson");
    let with_parts = b.buildings.iter().filter(|x| !x.parts.is_empty()).count();
    let parts: Vec<_> = b.buildings.iter().flat_map(|x| x.parts.iter()).collect();
    println!(
        "gate6 parts cache: {} buildings, {with_parts} with parts, {} parts",
        b.buildings.len(),
        parts.len()
    );
    assert_eq!(
        (b.buildings.len(), with_parts, parts.len()),
        (4336, 487, 4209)
    );

    let rule = |r: HeightRule| parts.iter().filter(|p| p.rule == r).count();
    let tops = (
        rule(HeightRule::Height),
        rule(HeightRule::NumFloors),
        rule(HeightRule::Default),
    );
    println!(
        "gate6 parts' tops: {} by height, {} by num_floors, {} at 10 m",
        tops.0, tops.1, tops.2
    );
    assert_eq!(tops, (4152, 17, 40));
    assert!(
        parts
            .iter()
            .filter(|p| p.rule == HeightRule::Default)
            .all(|p| p.height == 10.0)
    );

    // Which rule gave each base, from the file's own properties.
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    let props: HashMap<&str, &Value> = doc["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["properties"]["id"].as_str().unwrap(), &f["properties"]))
        .collect();
    let base_rule = |id: &str| {
        let p = props[id];
        if !p["min_height"].is_null() {
            "min_height"
        } else if !p["min_floor"].is_null() {
            "min_floor"
        } else {
            "none"
        }
    };
    let raised: Vec<_> = parts.iter().filter(|p| p.base > 0.0).collect();
    let by_min_height = raised
        .iter()
        .filter(|p| base_rule(&p.id) == "min_height")
        .count();
    println!(
        "gate6 parts' bases: {} above 0, {by_min_height} by min_height",
        raised.len()
    );
    assert_eq!((raised.len(), by_min_height), (695, 695));
    for p in &parts {
        if base_rule(&p.id) == "min_height" {
            assert_eq!(p.base, props[p.id.as_str()]["min_height"].as_f64().unwrap());
        }
    }

    let mut based: Vec<(String, f64, &str, bool)> = b
        .buildings
        .iter()
        .filter(|x| x.base > 0.0)
        .map(|x| (x.id.clone(), x.base, base_rule(&x.id), !x.parts.is_empty()))
        .collect();
    based.sort_by(|a, b| a.0.cmp(&b.0));
    println!("gate6 buildings with a base: {based:?}");
    let short: Vec<(&str, f64, &str, bool)> = based
        .iter()
        .map(|(id, base, r, p)| (&id[..8], *base, *r, *p))
        .collect();
    assert_eq!(
        short,
        vec![
            ("2bd09890", 7.5, "min_height", false),
            ("c92d28b5", 7.0, "min_floor", false),
            ("de4ad6d6", 3.7, "min_height", true),
        ]
    );
    println!("gate6 tallest {}", b.tallest);
    assert_eq!(b.tallest, 472.0);

    let m = mesh_data(&b);
    println!(
        "gate6 parts mesh: {} vertices, {} indices, {} wall quads, {} roof triangles",
        m.positions.len(),
        m.indices.len(),
        m.wall_quads,
        m.roof_triangles
    );
    assert_eq!(
        (
            m.positions.len(),
            m.indices.len(),
            m.wall_quads,
            m.roof_triangles
        ),
        (336_835, 558_147, 67_367, 51_315)
    );

    // Today's cache through the same reader.
    let today =
        buildings::read(&midtown().join("buildings.geojson"), &net).expect("buildings.geojson");
    assert!(
        today
            .buildings
            .iter()
            .all(|x| x.base == 0.0 && x.parts.is_empty())
    );
    let t = mesh_data(&today);
    println!(
        "gate6 today's cache: {} buildings, every base 0 and no parts; {} vertices, {} indices, {} wall quads, {} roof triangles",
        today.buildings.len(),
        t.positions.len(),
        t.indices.len(),
        t.wall_quads,
        t.roof_triangles
    );
    assert_eq!(
        (
            t.positions.len(),
            t.indices.len(),
            t.wall_quads,
            t.roof_triangles
        ),
        (213_880, 359_190, 42_776, 34_178)
    );

    // Every building with no parts and base 0 meshes as the same building of today's cache.
    let by_id: HashMap<&str, &buildings::Building> =
        today.buildings.iter().map(|x| (x.id.as_str(), x)).collect();
    let mut same = 0;
    for x in b
        .buildings
        .iter()
        .filter(|x| x.parts.is_empty() && x.base == 0.0)
    {
        let old = by_id[x.id.as_str()];
        assert_eq!(
            (x.height, x.rule, &x.polygons),
            (old.height, old.rule, &old.polygons),
            "{}",
            x.id
        );
        assert!(building_mesh(x) == building_mesh(old), "{}: the mesh", x.id);
        same += 1;
    }
    println!("gate6 {same} buildings without parts and base 0 mesh as today");
    assert_eq!(same, 3847);
}

// ── Gate 7: the reader's new checks ──────────────────────────────────────────

#[test]
fn gate7_parts_errors() {
    let net = test_network();
    let cases: Vec<(&str, Vec<Value>, &str)> = vec![
        (
            "building_id_number",
            vec![feature(json!({"id": "p", "building_id": 7, "height": 10}))],
            "feature p: building_id 7 is not a string",
        ),
        (
            "min_height_negative",
            vec![feature(json!({"id": "b", "height": 10, "min_height": -1}))],
            "feature b: min_height -1 is not a finite number of at least 0",
        ),
        (
            "min_floor_fraction",
            vec![feature(json!({"id": "b", "height": 10, "min_floor": 1.5}))],
            "feature b: min_floor 1.5 is not an integer of at least 0",
        ),
        (
            "min_height_negative_with_parts",
            vec![
                feature(json!({"id": "b", "height": 10, "min_height": -1})),
                feature(json!({"id": "p", "building_id": "b", "height": 5})),
            ],
            "feature b: min_height -1 is not a finite number of at least 0",
        ),
        (
            "building_base_at_top",
            vec![feature(json!({"id": "b", "height": 20, "min_height": 20}))],
            "feature b: base 20 m is not below its top 20 m",
        ),
        (
            "part_base_above_top",
            vec![
                feature(json!({"id": "b", "height": 30})),
                feature(json!({"id": "p", "building_id": "b", "height": 10, "min_height": 12})),
            ],
            "feature p: base 12 m is not below its top 10 m",
        ),
        (
            "orphan",
            vec![
                feature(json!({"id": "b", "height": 30})),
                feature(json!({"id": "p", "building_id": "nobody", "height": 10})),
            ],
            "feature p: part of nobody, which is not a building of the file",
        ),
        (
            "part_of_a_part",
            vec![
                feature(json!({"id": "b", "height": 30})),
                feature(json!({"id": "p", "building_id": "b", "height": 10})),
                feature(json!({"id": "q", "building_id": "p", "height": 10})),
            ],
            "feature q: part of p, which is not a building of the file",
        ),
        // The first in file order wins, and an orphan is reported only after every
        // feature's own checks pass.
        (
            "first_wins",
            vec![
                feature(json!({"id": "b1", "height": 10, "min_floor": 1.5})),
                feature(json!({"id": "b2", "height": 10, "min_height": -1})),
            ],
            "feature b1: min_floor 1.5 is not an integer of at least 0",
        ),
        (
            "orphan_after_feature_checks",
            vec![
                feature(json!({"id": "p", "building_id": "nobody", "height": 10})),
                feature(json!({"id": "b", "height": 10, "min_height": -1})),
            ],
            "feature b: min_height -1 is not a finite number of at least 0",
        ),
    ];
    for (label, features, want) in cases {
        let e = buildings::read(
            &write_case(&format!("gate7_{label}.geojson"), &collection(features)),
            &net,
        )
        .expect_err(label);
        println!("gate7 {label}: {e}");
        assert_eq!(e, want, "{label}");
    }
}

#[test]
fn gate7_parts_accepted() {
    let net = test_network();
    let text = collection(vec![
        // Its own min_height is above its own height: it reads, since it is not drawn.
        feature(json!({"id": "b", "height": 10, "min_height": 30})),
        feature(json!({"id": "p2", "building_id": "b", "height": 20, "min_floor": 2})),
        feature(
            json!({"id": "p1", "building_id": "b", "height": 40, "min_height": null, "min_floor": null}),
        ),
        feature(json!({"id": "c", "height": 12, "building_id": null, "min_height": 0})),
    ]);
    let b = buildings::read(&write_case("gate7_ok_parts.geojson", &text), &net).expect("accepted");
    let x = &b.buildings[0];
    let p: Vec<(&str, f64, f64)> = x
        .parts
        .iter()
        .map(|p| (p.id.as_str(), p.base, p.height))
        .collect();
    println!(
        "gate7 accepted: {} buildings; b base {} top {} parts {p:?}; c base {} top {}",
        b.buildings.len(),
        x.base,
        x.top(),
        b.buildings[1].base,
        b.buildings[1].top()
    );
    assert_eq!(b.buildings.len(), 2);
    assert_eq!(
        p,
        vec![("p2", 7.0, 20.0), ("p1", 0.0, 40.0)],
        "file order; min_floor 2 is 7 m"
    );
    assert_eq!((x.base, x.height, x.top()), (30.0, 10.0, 40.0));
    assert_eq!((b.buildings[1].base, b.buildings[1].top()), (0.0, 12.0));
    assert_eq!(b.tallest, 40.0);
    assert_eq!(
        (b.counts.height, b.counts.num_floors, b.counts.default),
        (2, 0, 0)
    );

    // tests/shapes.geojson reads exactly as in Phase 1's gate 8.
    let s: Buildings = buildings::read(&root().join("tests/shapes.geojson"), &net).unwrap();
    let got: Vec<(&str, f64, HeightRule, usize)> = s
        .buildings
        .iter()
        .map(|x| (x.id.as_str(), x.height, x.rule, x.polygons.len()))
        .collect();
    println!("gate7 shapes: {got:?}");
    assert_eq!(
        got,
        vec![
            ("shape-cube", 30.0, HeightRule::Height, 1),
            ("shape-courtyard", 20.0, HeightRule::Height, 1),
            ("shape-l", 14.0, HeightRule::NumFloors, 1),
            ("shape-default", 10.0, HeightRule::Default, 1),
            ("shape-both", 12.0, HeightRule::Height, 1),
            ("shape-multi", 8.0, HeightRule::Height, 2),
        ]
    );
    assert!(
        s.buildings
            .iter()
            .all(|x| x.base == 0.0 && x.parts.is_empty())
    );
    assert_eq!(
        (s.counts.height, s.counts.num_floors, s.counts.default),
        (4, 1, 1)
    );
    assert_eq!(s.tallest, 30.0);
}
