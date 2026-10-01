//! vis-002 Phase 1 exit gates 6–12 and 14 (`specs/city_spec.md`): the projection, reading,
//! heights, the mesh, alignment, top-down coverage, the shapes in perspective and `B`.
//! The headless ones run in plain `cargo test`. Those that need the Midtown fixture of
//! `scripts/fixture.sh midtown` or the GPU are ignored; run everything with
//! `cargo test --release --test buildings -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 3–5, 7's CLI cases and 13 are `scripts/gates-city.sh`.

use assimilator_video::buildings::{lnglat_to_xy, xy_to_lnglat};

/// Midtown's `metadata.map_origin`, `[lng, lat]`.
const MIDTOWN_ORIGIN: [f64; 2] = [-73.9775152177763, 40.76472024499192];

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
