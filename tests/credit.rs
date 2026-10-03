//! vis-002 Phase 2 exit gates 3 (the `Buildings` equality), 6–8, 10 and 11
//! (`specs/city_spec.md`): the rule on made-up inputs, Midtown's lines, the numbers, the
//! line confined to its box, and legible. Gates 6 and 8 are headless and run in plain
//! `cargo test`. Those that need the Midtown fixture of `scripts/fixture.sh midtown` (its
//! cache fetched again in Phase 2's form) or the GPU are ignored; run everything with
//! `cargo test --release --test credit -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 4, 5, 9 and 14 are `scripts/gates-credit.sh`.

use std::path::{Path, PathBuf};

use assimilator_config::network::NetworkConfig;
use assimilator_video::credit::{self, FILL, OUTLINE, TooSmall};
use assimilator_video::{Job, RenderOptions, buildings, inputs};
use serde_json::{Value, json};

const MIDTOWN_ROADS: &str =
    "© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1";
const MIDTOWN_CITY: &str = "© OpenStreetMap contributors (ODbL) · Overture Maps Foundation, release 2026-09-23.1 · Microsoft ML Buildings (ODbL) · USGS Lidar";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn midtown() -> PathBuf {
    root().join("scratch/midtown")
}

fn cache() -> PathBuf {
    midtown().join("buildings.geojson")
}

/// A network with no links, with or without Midtown's `map_origin`.
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

/// A fresh case directory under `scratch/out/credit-tests/`.
fn case_dir(name: &str) -> PathBuf {
    let dir = root().join("scratch/out/credit-tests").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// `tests/shapes.geojson` with `description` and, per shape in order, `sources` set
/// from `sources` (`None` leaves the member out), written to `path`. Not map data.
fn shapes_with(path: &Path, description: Option<&str>, sources: &[Option<Value>]) {
    let mut doc: Value = serde_json::from_str(
        &std::fs::read_to_string(root().join("tests/shapes.geojson")).unwrap(),
    )
    .unwrap();
    if let Some(d) = description {
        doc["description"] = json!(d);
    }
    for (f, s) in doc["features"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .zip(sources)
    {
        if let Some(s) = s {
            f["properties"]["sources"] = s.clone();
        }
    }
    std::fs::write(path, serde_json::to_string(&doc).unwrap()).unwrap();
}

// ── Gate 6: the rule ─────────────────────────────────────────────────────────

enum Report<'a> {
    Absent,
    Text(&'a str),
}

/// One case: a project directory with the report, and a buildings file if any.
fn run_case(
    name: &str,
    map_origin: bool,
    report: Report,
    buildings: Option<(Option<&str>, Vec<Option<Value>>)>,
) -> (PathBuf, Option<PathBuf>, anyhow::Result<Option<String>>) {
    let dir = case_dir(name);
    if let Report::Text(t) = report {
        std::fs::write(dir.join("import_report.json"), t).unwrap();
    }
    let file = buildings.map(|(desc, sources)| {
        let p = dir.join("buildings.geojson");
        shapes_with(&p, desc, &sources);
        p
    });
    let got = credit::line(&test_network(map_origin), &dir, file.as_deref());
    (dir, file, got)
}

const OVERTURE_R1: &str =
    r#"{"source": {"type": "Overture", "bbox": [0, 0, 1, 1], "release": "R1"}}"#;

fn all(v: Value) -> Vec<Option<Value>> {
    vec![Some(v); 6]
}

#[test]
fn gate6_rule() {
    let osm = "© OpenStreetMap contributors (ODbL)";
    let r1 = format!("{osm} · Overture Maps Foundation, release R1");
    let r2 = format!("{osm} · Overture Maps Foundation, release R2");
    let desc = Some("Overture Maps buildings, release R2");
    let mut n = 0;
    let mut ok = |name: &str, map_origin, report, b, want: Option<&str>| {
        let (_, _, got) = run_case(name, map_origin, report, b);
        let got = got.unwrap_or_else(|e| panic!("{name}: {e}"));
        println!("gate6 {name}: {got:?}");
        assert_eq!(got.as_deref(), want, "{name}");
        n += 1;
    };
    ok(
        "no_map_origin",
        false,
        Report::Text("{ not json"),
        None,
        None,
    );
    ok("absent", true, Report::Absent, None, Some(osm));
    ok(
        "osm",
        true,
        Report::Text(r#"{"source": {"type": "Osm", "bbox": [0, 0, 1, 1]}}"#),
        None,
        Some(osm),
    );
    ok(
        "null",
        true,
        Report::Text(r#"{"source": null, "node_count": 3}"#),
        None,
        Some(osm),
    );
    ok("overture", true, Report::Text(OVERTURE_R1), None, Some(&r1));
    ok(
        "buildings_osm",
        true,
        Report::Text(OVERTURE_R1),
        Some((desc, all(json!(["OpenStreetMap"])))),
        Some(&r2),
    );
    ok(
        "buildings_none",
        true,
        Report::Absent,
        Some((
            desc,
            vec![
                Some(json!(null)),
                Some(json!([])),
                Some(json!(null)),
                Some(json!([])),
                Some(json!(null)),
                Some(json!([])),
            ],
        )),
        Some(&r2),
    );
    let mut one = all(json!(["OpenStreetMap"]));
    one[3] = Some(json!(["Esri Community Maps", "OpenStreetMap"]));
    ok(
        "buildings_esri",
        true,
        Report::Text(OVERTURE_R1),
        Some((desc, one)),
        Some(&format!(
            "{r2} · Esri Community Maps contributors (CC BY 4.0)"
        )),
    );
    let six = [
        "Zeta Lab",
        "USGS Lidar",
        "Microsoft ML Buildings",
        "Alpha Survey",
        "Google Open Buildings",
        "Esri Community Maps",
    ]
    .map(|d| Some(json!([d])))
    .to_vec();
    ok(
        "buildings_six",
        true,
        Report::Text(OVERTURE_R1),
        Some((desc, six)),
        Some(&format!(
            "{r2} · Esri Community Maps contributors (CC BY 4.0) · Google Open Buildings (CC BY 4.0) · Microsoft ML Buildings (ODbL) · USGS Lidar · Alpha Survey · Zeta Lab"
        )),
    );

    // The report's errors: each names the file and its cause.
    for (name, text, cause) in [
        ("report_not_json", "{ not json", "not JSON"),
        (
            "report_source_string",
            r#"{"source": "Overture"}"#,
            "source is neither null nor an object with a string type",
        ),
        (
            "report_no_release",
            r#"{"source": {"type": "Overture", "bbox": [0, 0, 1, 1]}}"#,
            "source type \"Overture\" has no string release",
        ),
        (
            "report_here",
            r#"{"source": {"type": "Here", "path": "x"}}"#,
            "source type \"Here\" has no credit (vis-002 OQ-11)",
        ),
        (
            "report_foo",
            r#"{"source": {"type": "Foo"}}"#,
            "source type \"Foo\" has no credit (vis-002 OQ-11)",
        ),
    ] {
        let (dir, _, got) = run_case(name, true, Report::Text(text), None);
        let e = got.expect_err(name).to_string();
        println!("gate6 {name}: {e}");
        let file = dir.join("import_report.json");
        assert!(
            e.starts_with(&format!("{}: ", file.display())),
            "{name}: {e}"
        );
        assert!(e.contains(cause), "{name}: {e}");
        n += 1;
    }

    // The buildings file's errors: each names the flag and file; the last two the feature.
    let mut bad_str = all(json!(["OpenStreetMap"]));
    bad_str[2] = Some(json!("OpenStreetMap"));
    let mut bad_num = all(json!(["OpenStreetMap"]));
    bad_num[4] = Some(json!([1]));
    for (name, d, sources, cause) in [
        (
            "buildings_no_description",
            None,
            all(json!(["OpenStreetMap"])),
            "no release: fetched before vis-002 Phase 2; fetch it again with scripts/fetch-buildings.sh",
        ),
        (
            "buildings_description",
            Some("buildings"),
            all(json!(["OpenStreetMap"])),
            "no release: fetched before vis-002 Phase 2",
        ),
        (
            "buildings_sources_string",
            desc,
            bad_str,
            "feature shape-l: sources \"OpenStreetMap\" is not an array of strings",
        ),
        (
            "buildings_sources_number",
            desc,
            bad_num,
            "feature shape-both: sources [1] is not an array of strings",
        ),
    ] {
        let (_, file, got) = run_case(name, true, Report::Text(OVERTURE_R1), Some((d, sources)));
        let e = got.expect_err(name).to_string();
        println!("gate6 {name}: {e}");
        assert!(
            e.starts_with(&format!("--buildings {}: ", file.unwrap().display())),
            "{name}: {e}"
        );
        assert!(e.contains(cause), "{name}: {e}");
        n += 1;
    }
    println!("gate6: {n} cases");
    assert_eq!(n, 18);
}

// ── Gate 8: the numbers ──────────────────────────────────────────────────────

/// WCAG 2 relative luminance of an sRGB colour.
fn luminance(c: [u8; 3]) -> f64 {
    let lin = |v: u8| {
        let v = v as f64 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

#[test]
fn gate8_numbers() {
    for (w, h, s) in [
        (1280, 720, 13),
        (1920, 1080, 20),
        (3840, 2160, 40),
        (1080, 1920, 11),
    ] {
        println!("gate8 font_size({w}, {h}) = {}", credit::font_size(w, h));
        assert_eq!(credit::font_size(w, h), s, "{w}x{h}");
    }
    for (s, o) in [(13, 1), (20, 2), (40, 3)] {
        assert_eq!(credit::outline_offset(s), o, "size {s}");
        assert_eq!(credit::margin(s), s, "size {s}");
    }
    let ratio = (luminance(FILL) + 0.05) / (luminance(OUTLINE) + 0.05);
    println!("gate8 contrast {ratio:.2}:1");
    assert!((ratio - 18.4).abs() < 0.05 && ratio >= 7.0, "{ratio}");
    for (args, want) in [
        ((13, 785.0, 1254.0), Ok(13)),
        ((20, 1961.0, 1880.0), Ok(19)),
        ((20, 4826.0, 1880.0), Err(TooSmall(7))),
        ((40, 9651.0, 3760.0), Ok(15)),
        ((7, 300.0, 626.0), Err(TooSmall(7))),
    ] {
        let got = credit::fit_size(args.0, args.1, args.2);
        println!("gate8 fit_size{args:?} = {got:?}");
        assert_eq!(got, want, "{args:?}");
    }
}

// ── Gates 3 and 7: the Midtown cache and lines ───────────────────────────────

fn midtown_network() -> NetworkConfig {
    inputs::load_network(&midtown(), "baseline").unwrap()
}

#[test]
#[ignore = "needs the Midtown fixture, its cache fetched again"]
fn gate3_buildings_equal() {
    let net = midtown_network();
    let old = buildings::read(&root().join("scratch/buildings-vis002p1.geojson"), &net).unwrap();
    let new = buildings::read(&cache(), &net).unwrap();
    println!(
        "gate3 Phase 1's cache: {} buildings {:?}; Phase 2's: {} {:?}",
        old.buildings.len(),
        old.counts,
        new.buildings.len(),
        new.counts
    );
    assert_eq!(old, new);
}

#[test]
#[ignore = "needs the Midtown fixture, its cache fetched again"]
fn gate7_midtown_lines() {
    let net = midtown_network();
    let roads = credit::line(&net, &midtown(), None).unwrap();
    let city = credit::line(&net, &midtown(), Some(&cache())).unwrap();
    println!("gate7 without buildings: {roads:?}");
    println!("gate7 with buildings: {city:?}");
    assert_eq!(roads.as_deref(), Some(MIDTOWN_ROADS));
    assert_eq!(city.as_deref(), Some(MIDTOWN_CITY));
}

// ── Gates 10 and 11: through the GPU ─────────────────────────────────────────

fn options(width: u32, height: u32) -> RenderOptions {
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
        width,
        height,
    }
}

/// The pixels that differ between `a` and `b`, as `(x, y)`.
fn differing(a: &[u8], b: &[u8], width: u32) -> Vec<(u32, u32)> {
    assert_eq!(a.len(), b.len());
    (0..a.len() / 4)
        .filter(|&k| a[4 * k..4 * k + 4] != b[4 * k..4 * k + 4])
        .map(|k| ((k as u32) % width, (k as u32) / width))
        .collect()
}

fn inside(b: [u32; 4], (x, y): (u32, u32)) -> bool {
    b[0] <= x && x < b[2] && b[1] <= y && y < b[3]
}

/// The differing pixels outside and inside `bx`.
fn split(a: &[u8], b: &[u8], width: u32, bx: [u32; 4]) -> (usize, usize) {
    let d = differing(a, b, width);
    let n_in = d.iter().filter(|&&p| inside(bx, p)).count();
    (d.len() - n_in, n_in)
}

#[test]
#[ignore = "needs the Midtown fixture and the GPU"]
fn gate10_confined() {
    let (w, h) = (1920u32, 1080u32);
    let cache = cache();
    for (label, b, want) in [
        ("roads", None, MIDTOWN_ROADS),
        ("city", Some(cache.as_path()), MIDTOWN_CITY),
    ] {
        let o = options(w, h);
        let mut with = Job::prepare_with(&o, None, b).unwrap();
        let mut without = Job::prepare_without_credit(&o, None, b).unwrap();
        assert_eq!(with.credit(), Some(want), "{label}");
        assert_eq!(without.credit(), None);
        assert_eq!(without.credit_box(), None);
        let bx = with.credit_box().unwrap();
        let s = with.credit_size().unwrap();
        println!("gate10 {label}: size {s}, box {bx:?}");
        assert_eq!(s, 20);
        assert_eq!((bx[2], bx[3]), (1902, 1062), "{label}: right and bottom");
        assert!(bx[1].abs_diff(1034) <= 1, "{label}: top {}", bx[1]);
        let n = with.clock.frames;
        let times = [0, n / 2, n - 1].map(|k| with.clock.time_of(k));
        let pairs = std::iter::once((
            "empty".to_string(),
            with.render_empty().unwrap(),
            without.render_empty().unwrap(),
        ))
        .chain(times.iter().map(|&t| {
            (
                format!("t={t:.3}"),
                with.render_at(t).unwrap(),
                without.render_at(t).unwrap(),
            )
        }))
        .collect::<Vec<_>>();
        for (at, a, c) in &pairs {
            let (out, inn) = split(a, c, w, bx);
            println!("gate10 {label} {at}: {out} pixels differ outside the box, {inn} inside");
            assert_eq!(out, 0, "{label} {at}");
            assert!(inn > 0, "{label} {at}");
            assert_eq!(with.credit_box(), Some(bx), "{label} {at}: the box moved");
        }
    }

    // Once through a one-keyframe camera, with buildings, at render_empty only (OQ-8).
    let dir = case_dir("gate10-camera");
    let cam = dir.join("camera.toml");
    std::fs::write(
        &cam,
        "keyframes = [\n  { t = 300.000, x = 0.00, y = 0.00, height_m = 900.00, yaw_deg = 0.00, pitch_deg = 60.00 },\n]\n",
    )
    .unwrap();
    let o = options(w, h);
    let mut with = Job::prepare_with(&o, Some(&cam), Some(&cache)).unwrap();
    let mut without = Job::prepare_without_credit(&o, Some(&cam), Some(&cache)).unwrap();
    let bx = with.credit_box().unwrap();
    let (out, inn) = split(
        &with.render_empty().unwrap(),
        &without.render_empty().unwrap(),
        w,
        bx,
    );
    println!("gate10 camera: box {bx:?}, {out} pixels differ outside the box, {inn} inside");
    assert_eq!(out, 0);
    assert!(inn > 0);
}

#[test]
#[ignore = "needs the Midtown fixture and the GPU"]
fn gate11_legible() {
    let cache = cache();
    for (w, h, size, span) in [(1280u32, 720u32, 13u32, 16u32), (3840, 2160, 40, 49)] {
        let o = options(w, h);
        let mut with = Job::prepare_with(&o, None, Some(&cache)).unwrap();
        let mut without = Job::prepare_without_credit(&o, None, Some(&cache)).unwrap();
        let s = with.credit_size().unwrap();
        let bx = with.credit_box().unwrap();
        let a = with.render_empty().unwrap();
        let c = without.render_empty().unwrap();
        let d = differing(&a, &c, w);
        let (out, inn) = split(&a, &c, w, bx);
        let rows: Vec<u32> = d.iter().filter(|&&p| inside(bx, p)).map(|p| p.1).collect();
        let ink = rows.iter().max().unwrap() - rows.iter().min().unwrap() + 1;
        let off = credit::outline_offset(size);
        let fill_w = (bx[2] - bx[0] - 2 * off) as f64;
        let em = fill_w / s as f64;
        let px = |x: u32, y: u32| {
            let k = 4 * (y * w + x) as usize;
            [a[k], a[k + 1], a[k + 2]]
        };
        let pixels: Vec<[u8; 3]> = (bx[1]..bx[3])
            .flat_map(|y| (bx[0]..bx[2]).map(move |x| (x, y)))
            .map(|(x, y)| px(x, y))
            .collect();
        let lightest = pixels
            .iter()
            .map(|p| *p.iter().min().unwrap())
            .max()
            .unwrap();
        let darkest = pixels
            .iter()
            .map(|p| *p.iter().max().unwrap())
            .min()
            .unwrap();
        println!(
            "gate11 {w}x{h}: size {s}, box {bx:?}, ink rows {ink}, fill {fill_w} px = {em:.2} em, {out} outside / {inn} inside, lightest min-channel {lightest}, darkest max-channel {darkest}"
        );
        assert_eq!(s, size);
        assert!(ink.abs_diff(span) <= 2, "ink rows {ink}");
        assert!((em / 60.37 - 1.0).abs() <= 0.03, "{em} em");
        assert!(fill_w <= (w - 2 * credit::margin(size)) as f64);
        assert!(lightest >= 220 && darkest <= 20, "{lightest} {darkest}");
        assert_eq!(out, 0);
    }

    // The shrink: the made-up *six sources* file at 1920×1080.
    let dir = case_dir("gate11-six");
    let six = dir.join("six.geojson");
    let sources = [
        "Esri Community Maps",
        "Google Open Buildings",
        "Microsoft ML Buildings",
        "USGS Lidar",
        "Alpha Survey",
        "Zeta Lab",
    ]
    .map(|d| Some(json!([d])))
    .to_vec();
    shapes_with(
        &six,
        Some("Overture Maps buildings, release test"),
        &sources,
    );
    let (w, h) = (1920u32, 1080u32);
    let o = options(w, h);
    let mut with = Job::prepare_with(&o, None, Some(&six)).unwrap();
    let mut without = Job::prepare_without_credit(&o, None, Some(&six)).unwrap();
    let s = with.credit_size().unwrap();
    let bx = with.credit_box().unwrap();
    let off = credit::outline_offset(credit::font_size(w, h));
    let fill_w = bx[2] - bx[0] - 2 * off;
    let (out, inn) = split(
        &with.render_empty().unwrap(),
        &without.render_empty().unwrap(),
        w,
        bx,
    );
    println!(
        "gate11 six sources: {:?}\ngate11 six sources: size {s}, box {bx:?}, fill {fill_w} px, {out} outside / {inn} inside",
        with.credit()
    );
    assert!(s == 17 || s == 18, "size {s}");
    assert!(fill_w <= w - 2 * credit::margin(20));
    assert_eq!(out, 0);
}
