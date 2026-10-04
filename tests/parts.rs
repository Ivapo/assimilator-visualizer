//! vis-002 Phase 4 exit gates 6, 7, 8, 10, 11 and 12 (`specs/city_spec.md`): building
//! parts read, checked and meshed, drawn through the GPU, cut, credited, and Midtown's
//! traffic. The headless ones without a fixture run in plain `cargo test`. Those that
//! need the Midtown fixture of `scripts/fixture.sh midtown` (with its parts cache,
//! `buildings-parts.geojson`) or the GPU are ignored; run everything with
//! `cargo test --release --test parts -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 5, 9 and 11's CLI half are `scripts/gates-parts.sh`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use assimilator_config::network::NetworkConfig;
use assimilator_video::buildings::{
    self, Building, Buildings, Counts, HeightRule, Part, Polygon, building_mesh, mesh_data,
    xy_to_lnglat,
};
use assimilator_video::camera::{Pose, project};
use assimilator_video::clock;
use assimilator_video::draw::{self, BuildingsCut};
use assimilator_video::keyframes::{self, Flight};
use assimilator_video::motion::{Piece, TrackPos};
use assimilator_video::place::Placed;
use assimilator_video::render::{Renderer, VehicleBox};
use assimilator_video::run::{self, LoadOptions};
use assimilator_video::scene::{Camera, Strip};
use assimilator_video::see_through;
use assimilator_video::{Job, RenderOptions};
use bevy::mesh::{Mesh, VertexAttributeValues};
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

// ── Gate 8: the synthetic scenes, through the GPU ────────────────────────────

fn rect_polygon(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<Polygon> {
    vec![Polygon {
        exterior: vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        holes: vec![],
    }]
}

fn building(id: &str, poly: Vec<Polygon>, height: f64, base: f64, parts: Vec<Part>) -> Building {
    Building {
        id: id.into(),
        polygons: poly,
        height,
        rule: HeightRule::Height,
        base,
        parts,
    }
}

fn part(id: &str, poly: Vec<Polygon>, base: f64, height: f64) -> Part {
    Part {
        id: id.into(),
        polygons: poly,
        base,
        height,
        rule: HeightRule::Height,
    }
}

fn city(bs: Vec<Building>) -> Buildings {
    let tallest = bs.iter().map(Building::top).fold(0.0, f64::max);
    Buildings {
        counts: Counts {
            height: bs.len(),
            num_floors: 0,
            default: 0,
        },
        buildings: bs,
        tallest,
    }
}

/// A, a tower on a podium: the building x −30…30, y −40…−20, `height` 80; the podium the
/// same footprint, 0 to 12 m; the tower x 15…30, y −40…−30, 12 to 80 m. Without parts,
/// as today's cache draws it.
fn tower_on_podium(with_parts: bool) -> Building {
    let parts = if with_parts {
        vec![
            part("podium", rect_polygon(-30.0, -40.0, 30.0, -20.0), 0.0, 12.0),
            part("tower", rect_polygon(15.0, -40.0, 30.0, -30.0), 12.0, 80.0),
        ]
    } else {
        vec![]
    };
    building(
        "A",
        rect_polygon(-30.0, -40.0, 30.0, -20.0),
        80.0,
        0.0,
        parts,
    )
}

/// B, over the box: x −8…8, y −10…10, `height` 20, standing from `base`.
fn raised(base: f64) -> Building {
    building(
        "B",
        rect_polygon(-8.0, -10.0, 8.0, 10.0),
        20.0,
        base,
        vec![],
    )
}

/// Phase 3 gate 6's pose.
fn tilted() -> Pose {
    Pose {
        cx: 0.0,
        cy: 0.0,
        height_m: 60.0,
        yaw_deg: 0.0,
        pitch_deg: 35.0,
    }
}

fn vbox(id: u64, x: f64, y: f64, heading: f64, length: f64, speed: f64) -> VehicleBox {
    VehicleBox {
        vehicle_id: id,
        at: Placed { x, y, heading },
        length,
        speed,
        track: TrackPos {
            piece: Piece::Link(0),
            along: 0.0,
            lateral: 0.0,
            odo: 0.0,
        },
    }
}

/// Pixels that differ between two RGBA frames.
fn differ(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

#[test]
fn gate8_meshes() {
    let a = building_mesh(&tower_on_podium(true));
    let b = building_mesh(&raised(8.0));
    // Each volume: 4 wall quads (16 vertices, 24 indices) and a 2-triangle roof (4, 6).
    let tower_bottoms: Vec<f64> = a.positions[20..36]
        .chunks(4)
        .flat_map(|q| [q[0][2], q[1][2]])
        .collect();
    let b_bottoms: Vec<f64> = b.positions[..16]
        .chunks(4)
        .flat_map(|q| [q[0][2], q[1][2]])
        .collect();
    println!(
        "gate8 A: {} vertices, {} indices, tower wall bottoms {tower_bottoms:?}; B: {} vertices, {} indices, wall bottoms {b_bottoms:?}",
        a.positions.len(),
        a.indices.len(),
        b.positions.len(),
        b.indices.len()
    );
    assert_eq!((a.positions.len(), a.indices.len()), (40, 60));
    assert!(tower_bottoms.iter().all(|&z| z == 12.0));
    assert!(
        a.positions[0..16]
            .chunks(4)
            .all(|q| q[0][2] == 0.0 && q[2][2] == 12.0),
        "the podium"
    );
    assert!(
        a.positions[16..20].iter().all(|p| p[2] == 12.0),
        "the podium's roof"
    );
    assert!(
        a.positions[36..40].iter().all(|p| p[2] == 80.0),
        "the tower's roof"
    );
    assert_eq!((b.positions.len(), b.indices.len()), (20, 30));
    assert!(b_bottoms.iter().all(|&z| z == 8.0));
    assert_eq!(building_mesh(&tower_on_podium(true)).wall_quads, 8);
}

#[test]
#[ignore = "needs the GPU"]
fn gate8_synthetic() {
    let strips = vec![Strip {
        left: (0..=120).map(|i| [-60.0 + i as f64, 3.5]).collect(),
        right: (0..=120).map(|i| [-60.0 + i as f64, -3.5]).collect(),
    }];
    let cam = Camera {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        width: 1280,
        height: 720,
    };
    let pose = tilted();
    let bx = [vbox(7, 0.0, 0.0, 90.0, 4.5, 10.0)];
    let new = |bs: Option<&Buildings>| {
        Renderer::new_perspective(&strips, cam, 4, &pose, bs, None).unwrap()
    };
    // Box pixels, and the frame without the box; cut with see-through on.
    let shot = |bs: Option<&Buildings>, cut: bool| {
        let mut r = new(bs);
        if cut {
            r.set_see_through(bs);
            r.set_pose(&pose);
        }
        let with = r.render(&bx).unwrap();
        let empty = r.render(&[]).unwrap();
        (differ(&with, &empty), empty)
    };

    let a = city(vec![tower_on_podium(true)]);
    let a_flat = city(vec![tower_on_podium(false)]);
    let b = city(vec![raised(8.0)]);
    let b_ground = city(vec![raised(0.0)]);
    let slab = city(vec![building(
        "slab",
        rect_polygon(-30.0, -40.0, 30.0, -20.0),
        3.0,
        0.0,
        vec![],
    )]);

    let (none, _) = shot(None, false);
    let (a_flat_px, _) = shot(Some(&a_flat), false);
    let (a_px, _) = shot(Some(&a), false);
    let (a_cut_px, a_cut_empty) = shot(Some(&a), true);
    let (_, slab_empty) = shot(Some(&slab), false);
    let a_vs_slab = differ(&a_cut_empty, &slab_empty);
    let (b_ground_px, _) = shot(Some(&b_ground), false);
    let (b_px, _) = shot(Some(&b), false);
    let (b_cut_px, _) = shot(Some(&b), true);
    println!(
        "gate8 box pixels: no buildings {none}; A without parts {a_flat_px}, A {a_px}, A cut {a_cut_px} \
         (its empty frame {a_vs_slab} pixels apart from a 3 m slab of the podium); \
         B on the ground {b_ground_px}, B {b_px}, B cut {b_cut_px}"
    );
    assert_eq!(none, 1564);
    assert_eq!((a_flat_px, a_px, a_cut_px), (0, 1564, 1564));
    assert_eq!(a_vs_slab, 0, "A cut is a 3 m slab of the podium");
    assert_eq!((b_ground_px, b_px, b_cut_px), (0, 1008, 0));
}

// ── Gate 10: continuity, and buildings the cut leaves alone ──────────────────

fn positions(m: &Mesh) -> Vec<[u32; 3]> {
    match m.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(v)) => {
            v.iter().map(|p| p.map(f32::to_bits)).collect()
        }
        _ => panic!("positions"),
    }
}

/// One flight over Midtown with the parts cache, headless (Phase 3 gate 9's method):
/// returns the largest in-frame step and the mean number of buildings lowered a frame.
fn untouched(flight_file: &Path) -> (f64, f64) {
    let run = run::load(&LoadOptions {
        project: midtown(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: Some(300.0),
        to: Some(360.0),
    })
    .unwrap();
    let flight = Flight::new(keyframes::read(flight_file).unwrap(), &run.fcd);
    let b = buildings::read(
        &midtown().join("buildings-parts.geojson"),
        &run.placement.network,
    )
    .unwrap();
    let fit = Camera::fit(&run.strips, 1920, 1080);
    let (fx, fy) = (fit.cx, fit.cy);

    let cut = BuildingsCut::new(&b, (fx, fy));
    let full = draw::buildings_mesh(&mesh_data(&b), fx, fy);
    let own: Vec<f64> = b.buildings.iter().map(Building::top).collect();
    assert!(
        cut.mesh(&own) == full,
        "drawn tops: the mesh, attribute for attribute"
    );
    let full_pos = positions(&full);
    let mut ranges = Vec::with_capacity(b.buildings.len());
    let mut at = 0;
    for x in &b.buildings {
        let n = building_mesh(x).positions.len();
        ranges.push(at..at + n);
        at += n;
    }
    assert_eq!(at, full_pos.len());

    let centroids: Vec<[f64; 2]> = b
        .buildings
        .iter()
        .map(|x| {
            let pts: Vec<&[f64; 2]> = x.polygons.iter().flat_map(|p| p.exterior.iter()).collect();
            let n = pts.len() as f64;
            [
                pts.iter().map(|p| p[0]).sum::<f64>() / n,
                pts.iter().map(|p| p[1]).sum::<f64>() / n,
            ]
        })
        .collect();
    let in_frame = |pose: &Pose, c: [f64; 2]| {
        let (a, _, _) = pose.axes();
        let e = pose.eye();
        let depth = (c[0] - e[0]) * a[0] + (c[1] - e[1]) * a[1] + (0.0 - e[2]) * a[2];
        if depth <= 0.1 {
            return false;
        }
        let (px, py) = project(pose, 1920.0, 1080.0, [c[0], c[1], 0.0]);
        (0.0..1920.0).contains(&px) && (0.0..1080.0).contains(&py)
    };

    let mut prev: Option<Vec<f64>> = None;
    let (mut worst, mut lowered) = (0.0f64, 0u64);
    for n in 0..1800u64 {
        let pose = flight.pose_at(
            clock::frame_time(300.0, n, 1.0, 30),
            &run.motion,
            &run.fcd,
            &run.placement,
        );
        let hs = see_through::heights(&b, &pose);
        lowered += hs.iter().zip(&own).filter(|(a, b)| a < b).count() as u64;
        if n % 30 == 0 {
            let w = see_through::wedge(&pose);
            let pos = positions(&cut.mesh(&hs));
            for (i, x) in b.buildings.iter().enumerate() {
                if see_through::distance(&w, x) >= w.ease {
                    assert_eq!(
                        hs[i].to_bits(),
                        x.top().to_bits(),
                        "frame {n} building {i}: h′ == top()"
                    );
                    let r = ranges[i].clone();
                    assert_eq!(
                        pos[r.clone()],
                        full_pos[r],
                        "frame {n} building {i}: vertices"
                    );
                }
            }
        }
        if let Some(p) = &prev {
            for i in 0..hs.len() {
                if in_frame(&pose, centroids[i]) {
                    worst = worst.max((hs[i] - p[i]).abs());
                }
            }
        }
        prev = Some(hs);
    }
    (worst, lowered as f64 / 1800.0)
}

#[test]
#[ignore = "needs the Midtown fixture"]
fn gate10_untouched() {
    for (name, file) in [
        ("city", root().join("tests/city-flight.toml")),
        ("orbit", root().join("tests/see-through-flight.toml")),
    ] {
        let (worst, mean) = untouched(&file);
        println!("gate10 {name}: largest in-frame step {worst:.2} m; lowered a frame {mean:.1}");
        assert!(
            worst <= 10.0,
            "{name}: a building in frame changed by {worst} m"
        );
    }
}

// ── Gate 11: the credit, either cache ────────────────────────────────────────

const CREDIT: &str = "© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1 · Microsoft ML Buildings (ODbL) · USGS Lidar";

fn flight_options() -> RenderOptions {
    RenderOptions {
        project: midtown(),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: Some(300.0),
        to: Some(360.0),
        speedup: Some(1.0),
        fps: 30,
        width: 1920,
        height: 1080,
    }
}

#[test]
#[ignore = "needs the Midtown fixture and the GPU"]
fn gate11_credit() {
    for name in ["buildings-parts.geojson", "buildings.geojson"] {
        let job = Job::prepare_with(&flight_options(), None, Some(&midtown().join(name))).unwrap();
        println!("gate11 {name}: {:?}", job.credit());
        assert_eq!(job.credit(), Some(CREDIT), "{name}");
    }
}

// ── Gate 12: Midtown's traffic with parts ────────────────────────────────────

/// Box pixels in the centre, the middle and the whole frame, over every 15th frame
/// (Phase 3 gate 8's count), with the parts cache.
fn box_pixels(flight: &Path, on: bool) -> [u64; 3] {
    let cache = midtown().join("buildings-parts.geojson");
    let mut job =
        Job::prepare_without_credit(&flight_options(), Some(flight), Some(&cache)).unwrap();
    job.set_see_through(on);
    let mut tot = [0u64; 3];
    for n in (0..1800u64).step_by(15) {
        let a = job.render_frame(n).unwrap();
        let e = job.render_empty().unwrap();
        for (i, (x, y)) in a.chunks(4).zip(e.chunks(4)).enumerate() {
            if x != y {
                let (px, py) = ((i % 1920) as f64 + 0.5, (i / 1920) as f64 + 0.5);
                let mid = (px - 960.0).abs() <= 480.0 && (py - 540.0).abs() <= 270.0;
                let cen = (px - 960.0).abs() <= 240.0 && (py - 540.0).abs() <= 135.0;
                tot[0] += cen as u64;
                tot[1] += mid as u64;
                tot[2] += 1;
            }
        }
    }
    tot
}

fn within_1pct(name: &str, got: [u64; 3], want: [u64; 3]) -> bool {
    let mut ok = true;
    for (k, region) in ["centre", "middle", "frame"].iter().enumerate() {
        let dev = (got[k] as f64 - want[k] as f64) / want[k] as f64;
        println!(
            "gate12 {name} {region}: {} (probe {}, {:+.3} %)",
            got[k],
            want[k],
            100.0 * dev
        );
        ok &= dev.abs() <= 0.01;
    }
    ok
}

#[test]
#[ignore = "needs the Midtown fixture and the GPU"]
fn gate12_traffic() {
    let orbit = root().join("tests/see-through-flight.toml");
    let city_flight = root().join("tests/city-flight.toml");
    let mut ok = within_1pct(
        "orbit off",
        box_pixels(&orbit, false),
        [62_082, 92_016, 147_200],
    );
    ok &= within_1pct(
        "orbit on",
        box_pixels(&orbit, true),
        [159_502, 259_247, 387_679],
    );
    ok &= within_1pct(
        "city off",
        box_pixels(&city_flight, false),
        [4_033, 55_729, 111_115],
    );
    ok &= within_1pct(
        "city on",
        box_pixels(&city_flight, true),
        [4_886, 57_610, 123_130],
    );
    assert!(ok, "a count is more than 1 % from the probe's");
}
