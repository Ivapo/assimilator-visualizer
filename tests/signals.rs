//! vis-003 Phase 1 exit gates 5, 6, 7, 8 and 15's `L` (`specs/data_spec.md`): the
//! signals replayed with the engine's own code on a synthetic crossing and on both
//! fixtures, checked against a second `SignalState` and against the run's FCD, their dots
//! placed and meshed, and `L`. The headless ones without a fixture run in plain
//! `cargo test`. Those that need the fixtures of `scripts/fixture.sh` and
//! `scripts/fixture.sh midtown` are ignored; run everything with
//! `cargo test --release --test signals -- --include-ignored --test-threads=1 --nocapture`.
//! Gates 2, 10 and 15's CLI cases are `scripts/gates-signals.sh`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::time::Instant;

use assimilator_config::network::{ControlType, NetworkConfig};
use assimilator_config::types::{LaneIdx, LinkId};
use assimilator_core::systems::signal::SignalState;
use assimilator_video::camera::{self, Pose};
use assimilator_video::inputs;
use assimilator_video::place::Placement;
use assimilator_video::render::VehicleBox;
use assimilator_video::run::{self, LoadOptions, Run};
use assimilator_video::signals::{AMBER, Colour, Eye, GREEN, OUTLINE, RED, Signals, dots, scale};
use assimilator_video::streets::{Kind, Streets};
use assimilator_video::view::state::Follow;
use assimilator_video::view::{Fit, ViewInput, ViewState};

use Colour::{Amber as A, Green as G, Red as R};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// vis-002 gate 5's crossing (`tests/look.rs`), with a plan on `C` (vis-003 gate 5): its
/// `cycle_time` is not the phases' 45 s, and its `offset` is not 0; the engine reads
/// neither.
const SYNTHETIC: &str = r#"
schema_version: 1
metadata: {name: signals synthetic, coordinate_system: metric, z_enabled: false}
nodes:
  - {id: C, point: [0, 0], type: junction}
  - {id: W, point: [-60, 0], type: endpoint}
  - {id: E, point: [60, 0], type: endpoint}
  - {id: N, point: [0, 60], type: endpoint}
  - {id: S, point: [0, -60], type: endpoint}
links:
  - {id: L_WC, from_node: W, to_node: C, geometry: [[-60, 0], [0, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}, {id: 2, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CW, from_node: C, to_node: W, geometry: [[0, 0], [-60, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_EC, from_node: E, to_node: C, geometry: [[60, 0], [0, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}, {id: 2, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CE, from_node: C, to_node: E, geometry: [[0, 0], [60, 0]], median_gap: 0.5, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_NC, from_node: N, to_node: C, geometry: [[0, 60], [0, 0]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CN, from_node: C, to_node: N, geometry: [[0, 0], [0, 60]], median_gap: 0, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
  - {id: L_SC, from_node: S, to_node: C, geometry: [[0, -60], [0, 0]], median_gap: 4, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}]}
  - {id: L_CS, from_node: C, to_node: S, geometry: [[0, 0], [0, -60]], median_gap: 4, lanes: [{id: 0, width: 3.5, speed_limit: 13.9}, {id: 1, width: 3.5, speed_limit: 13.9}]}
junctions:
  - node_id: C
    control: signal
    geometry: {setback: 3}
    stop_line_offsets: {L_EC: {'0': 5}}
    movements:
      - {id: W_U, from_link: L_WC, to_link: L_CW, from_lanes: [0], to_lanes: [0], type: u-turn}
      - {id: W_L, from_link: L_WC, to_link: L_CN, from_lanes: [1], to_lanes: [0], type: left}
      - {id: W_T, from_link: L_WC, to_link: L_CE, from_lanes: [1, 2], to_lanes: [0, 1], type: through}
      - {id: W_R, from_link: L_WC, to_link: L_CS, from_lanes: [2], to_lanes: [1], type: right}
      - {id: E_L, from_link: L_EC, to_link: L_CS, from_lanes: [0], to_lanes: [0], type: left}
      - {id: E_U, from_link: L_EC, to_link: L_CE, from_lanes: [0], to_lanes: [0], type: u-turn}
      - {id: E_T, from_link: L_EC, to_link: L_CW, from_lanes: [1], to_lanes: [1], type: through}
      - {id: E_R, from_link: L_EC, to_link: L_CN, from_lanes: [2], to_lanes: [1], type: right}
      - {id: N_L, from_link: L_NC, to_link: L_CE, from_lanes: [0], to_lanes: [0], type: left}
      - {id: N_T, from_link: L_NC, to_link: L_CS, from_lanes: [0], to_lanes: [0], type: through}
      - {id: N_R, from_link: L_NC, to_link: L_CW, from_lanes: [0], to_lanes: [1], type: right}
      - {id: S_L, from_link: L_SC, to_link: L_CW, from_lanes: [0], to_lanes: [0], type: left}
      - {id: S_R, from_link: L_SC, to_link: L_CE, from_lanes: [0], to_lanes: [1], type: right}
    signal_plan:
      cycle_time: 60
      offset: 7
      phases:
        - {id: P1, duration: 20, green_movements: [W_T, W_R, E_T, E_R], permitted_movements: [W_L, E_L], amber_time: 3, all_red_time: 2}
        - {id: P2, duration: 15, green_movements: [N_T, N_R, S_R], permitted_movements: [N_L, S_L], amber_time: 3, all_red_time: 2}
"#;

fn synthetic() -> Placement {
    let cfg: NetworkConfig = serde_yaml::from_str(SYNTHETIC).expect("the synthetic network");
    Placement::new(cfg)
}

fn links(sig: &Signals) -> Vec<&str> {
    sig.approaches.iter().map(|a| a.link.as_str()).collect()
}

// ── Gate 5: the synthetic crossing with a plan ───────────────────────────────

#[test]
fn gate5_synthetic_replay() {
    let pl = synthetic();
    let sig = Signals::build(&pl, 0.1, 100.0);
    assert_eq!(links(&sig), ["L_EC", "L_NC", "L_SC", "L_WC"]);
    assert!(sig.approaches.iter().all(|a| a.node == "C"));
    let want = [(10.0, 2.0), (-1.75, 13.75), (3.75, -13.5), (-12.0, -1.75)];
    for (a, (x, y)) in sig.approaches.iter().zip(want) {
        println!("gate 5 {} dot ({:.9}, {:.9})", a.link, a.at[0], a.at[1]);
        assert!(
            (a.at[0] - x).abs() <= 1e-9 && (a.at[1] - y).abs() <= 1e-9,
            "{}",
            a.link
        );
    }
    // L_EC's dot is at its link's end, 5 m ahead of its lane 0's own line.
    let ec = LinkId("L_EC".into());
    let (len, base) = (pl.data.link_length(&ec), pl.data.stop_line_offset(&ec));
    let lane0 = pl.data.lane_stop_line_offset(&ec, LaneIdx(0));
    println!("gate 5 L_EC: length {len}, stop_line_offset {base}, lane 0's {lane0}");
    assert_eq!(base, 0.0, "L_EC's dot at its link's end");
    assert!((lane0 - base - 5.0).abs() <= 1e-9, "L_EC's lane 0 delta");

    // At 0.1 s: a step after the plan's seconds, where the float sum falls short.
    let ew = vec![
        (0, G),
        (200, A),
        (230, R),
        (451, G),
        (650, A),
        (680, R),
        (901, G),
    ];
    let ns = vec![
        (0, R),
        (250, G),
        (401, A),
        (431, R),
        (700, G),
        (851, A),
        (881, R),
    ];
    assert_eq!(sig.t.len(), 1001, "1000 steps");
    for (a, want) in sig.approaches.iter().zip([&ew, &ns, &ns, &ew]) {
        println!("gate 5 dt 0.1 {}: {:?}", a.link, a.changes);
        assert_eq!(&a.changes, want, "{} at dt 0.1", a.link);
    }
    // At 0.25 s, exact in binary: on the plan's seconds.
    let sig25 = Signals::build(&pl, 0.25, 100.0);
    assert_eq!(sig25.t.len(), 401, "400 steps");
    assert_eq!(sig25.t[400], 100.0);
    let ew = vec![
        (0, G),
        (80, A),
        (92, R),
        (180, G),
        (260, A),
        (272, R),
        (360, G),
    ];
    let ns = vec![
        (0, R),
        (100, G),
        (160, A),
        (172, R),
        (280, G),
        (340, A),
        (352, R),
    ];
    for (a, want) in sig25.approaches.iter().zip([&ew, &ns, &ns, &ew]) {
        println!("gate 5 dt 0.25 {}: {:?}", a.link, a.changes);
        assert_eq!(&a.changes, want, "{} at dt 0.25", a.link);
    }

    let table: [(f64, [Colour; 4]); 16] = [
        (0.0, [G, R, R, G]),
        (10.0, [G, R, R, G]),
        (19.95, [G, R, R, G]),
        (20.0, [A, R, R, A]),
        (22.99, [A, R, R, A]),
        (23.0, [R, R, R, R]),
        (24.0, [R, R, R, R]),
        (25.0, [R, G, G, R]),
        (39.99, [R, G, G, R]),
        (40.0, [R, G, G, R]),
        (43.0, [R, A, A, R]),
        (44.99, [R, R, R, R]),
        (45.0, [R, R, R, R]),
        (46.0, [G, R, R, G]),
        (60.0, [G, R, R, G]),
        (61.0, [G, R, R, G]),
    ];
    for (t, want) in table {
        let got = sig.colours_at(t);
        println!("gate 5 colours_at {t}: {got:?}");
        assert_eq!(got, want, "colours_at {t}");
    }
    println!("gate 5: 4 approaches, their dots, the changes at dt 0.1 and 0.25, colours_at: PASS");
}

#[test]
fn gate5_synthetic_mesh() {
    let pl = synthetic();
    let sig = Signals::build(&pl, 0.1, 100.0);
    let k = 0.05;
    let cols = sig.colours_at(10.0);
    let d = dots(&sig, &cols, Eye::Ortho(k), 1920.0, 1080.0);
    println!(
        "gate 5 mesh at k {k}, 1920x1080: {} vertices, {} triangles",
        d.positions.len(),
        d.indices.len() / 3
    );
    assert_eq!(d.positions.len(), 388);
    assert_eq!(d.indices.len() / 3, 384);
    assert_eq!(d.colours.len(), 388);
    let mut worst: f64 = 0.0;
    for (i, (a, c)) in sig.approaches.iter().zip(&cols).enumerate() {
        let v = &d.positions[i * 97..(i + 1) * 97];
        let vc = &d.colours[i * 97..(i + 1) * 97];
        for (j, p) in v.iter().enumerate() {
            let r = ((p[0] - a.at[0]).powi(2) + (p[1] - a.at[1]).powi(2)).sqrt();
            let want = match j {
                0..32 => 4.5 * k,
                32..64 => 5.5 * k,
                64 => 0.0,
                _ => 4.5 * k,
            };
            worst = worst.max((r - want).abs());
            assert!((r - want).abs() <= 1e-9, "{} vertex {j}: r {r}", a.link);
            assert!(
                (p[2] - 0.03).abs() <= 1e-9,
                "{} vertex {j}: z {}",
                a.link,
                p[2]
            );
            let colour = if j < 64 { OUTLINE } else { c.srgb() };
            assert_eq!(vc[j], colour, "{} vertex {j}'s colour", a.link);
        }
    }
    println!(
        "gate 5 mesh: every vertex within {worst:.1e} m of its circle or centre, at 0.03 m: PASS"
    );
}

#[test]
fn gate5_perspective_dot() {
    let pl = synthetic();
    let sig = Signals::build(&pl, 0.1, 100.0);
    let pose = Pose {
        cx: 0.0,
        cy: 0.0,
        height_m: 60.0,
        yaw_deg: 30.0,
        pitch_deg: 30.0,
    };
    let (w, h) = (1920.0, 1080.0);
    let cols = sig.colours_at(10.0);
    let d = dots(&sig, &cols, Eye::Pose(&pose), w, h);
    assert_eq!(d.positions.len(), 388, "no dot left out");
    let (mut err, mut low): (f64, f64) = (0.0, 0.0);
    for (i, a) in sig.approaches.iter().enumerate() {
        let v = &d.positions[i * 97..(i + 1) * 97];
        let ctr = camera::project(&pose, w, h, v[64]);
        for (j, p) in v.iter().enumerate() {
            if j == 64 {
                continue;
            }
            let want = if (32..64).contains(&j) { 5.5 } else { 4.5 };
            let q = camera::project(&pose, w, h, *p);
            let r = ((q.0 - ctr.0).powi(2) + (q.1 - ctr.1).powi(2)).sqrt();
            err = err.max((r - want).abs());
            assert!((r - want).abs() <= 1e-6, "{} vertex {j}: {r} px", a.link);
        }
        let zmin = v.iter().map(|p| p[2]).fold(f64::INFINITY, f64::min);
        low = low.max((zmin - 0.03).abs());
        println!(
            "gate 5 perspective {}: lowest vertex at {zmin:.12} m",
            a.link
        );
        assert!((zmin - 0.03).abs() <= 1e-9, "{}'s lowest vertex", a.link);
    }
    println!(
        "gate 5 perspective: every vertex within {err:.1e} px of its circle, lowest within {low:.1e} m of 0.03: PASS"
    );
}

/// A temporary `results.db` with one completed `runs` row.
fn results_db(tag: &str, mode: Option<&str>, duration: f64, config: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("vis003-gate5-{}-{tag}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute_batch(
        "CREATE TABLE runs (scenario TEXT NOT NULL, seed INTEGER NOT NULL, mode TEXT,
         status TEXT NOT NULL, duration REAL, resolved_config TEXT,
         PRIMARY KEY (scenario, seed));",
    )
    .unwrap();
    conn.execute(
        "INSERT INTO runs VALUES ('baseline', 42, ?1, 'completed', ?2, ?3)",
        rusqlite::params![mode, duration, config],
    )
    .unwrap();
    path
}

#[test]
fn gate5_run_clock() {
    let micro = r#"{"scenario":"baseline","sim_params":{"duration":100.0,"random_seed":42,"timestep":0.1,"warm_up":10.0}}"#;
    let no_dt = r#"{"scenario":"baseline","sim_params":{"duration":100.0,"random_seed":42,"warm_up":10.0}}"#;
    let zero_dt = r#"{"scenario":"baseline","sim_params":{"duration":100.0,"random_seed":42,"timestep":0.0,"warm_up":10.0}}"#;
    let meso = r#"{"scenario":"baseline","mode":"meso","sim_params":{"duration":100.0,"warm_up":10.0,"random_seed":42},"meso":{}}"#;
    let clock = |p: &Path| inputs::run_clock(p, "baseline", 42);
    let timestep_err = |p: &Path, e: &str| {
        let head = format!(
            "{}: the run's timestep is not readable (runs.resolved_config): ",
            p.display()
        );
        assert!(e.starts_with(&head), "{e}");
        assert!(e.ends_with("; --no-signals renders without signals"), "{e}");
    };

    let p = results_db("micro", Some("micro"), 100.0, micro);
    let got = clock(&p).unwrap();
    println!("gate 5 run_clock, a micro row: {got:?}");
    assert_eq!(got, (0.1, 100.0));

    for (tag, config) in [("no-timestep", no_dt), ("zero-timestep", zero_dt)] {
        let p = results_db(tag, Some("micro"), 100.0, config);
        let e = format!("{:#}", clock(&p).unwrap_err());
        println!("gate 5 run_clock, {tag}: {e}");
        timestep_err(&p, &e);
        let _ = std::fs::remove_file(&p);
    }

    let p = results_db("meso", Some("meso"), 100.0, meso);
    let e = format!("{:#}", clock(&p).unwrap_err());
    println!("gate 5 run_clock, the meso runner's row: {e}");
    assert_eq!(
        e,
        format!(
            "{}: the run is meso; signals are replayed only for micro runs (vis-003 OQ-3); --no-signals renders without signals",
            p.display()
        )
    );
    let _ = std::fs::remove_file(&p);

    let p = results_db("no-mode", None, 100.0, micro);
    let e = format!("{:#}", clock(&p).unwrap_err());
    println!("gate 5 run_clock, a NULL mode: {e}");
    assert!(e.contains(": the run is not recorded; "), "{e}");
    let _ = std::fs::remove_file(&p);
    println!(
        "gate 5 run_clock: the micro row's clock, the timestep message, the mode message first: PASS"
    );
}

// ── Gate 8: the colours, the scale ───────────────────────────────────────────

fn hex(s: &str) -> [u8; 3] {
    let v = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).unwrap();
    [v(1), v(3), v(5)]
}

#[test]
fn gate8_colours_and_scale() {
    // colors.ts at 90b39292, l. 84, 85, 86 and 121.
    assert_eq!(RED, hex("#ef4444"));
    assert_eq!(AMBER, hex("#f59e0b"));
    assert_eq!(GREEN, hex("#22c55e"));
    assert_eq!(OUTLINE, hex("#000000"));
    assert_eq!(Colour::of("green"), G);
    assert_eq!(Colour::of("amber"), A);
    assert_eq!(Colour::of("red"), R);
    for other in ["", "off", "flashing", "Green"] {
        assert_eq!(Colour::of(other), R, "{other:?}");
    }
    assert_eq!((G.srgb(), A.srgb(), R.srgb()), (GREEN, AMBER, RED));
    for (w, h, want) in [
        (1280.0, 720.0, 2.0 / 3.0),
        (1920.0, 1080.0, 1.0),
        (3840.0, 2160.0, 2.0),
    ] {
        println!("gate 8 scale {w}x{h}: {}", scale(w, h));
        assert_eq!(scale(w, h), want);
    }
    println!("gate 8: the copied colours, Colour's mapping, scale: PASS");
}

// ── Gates 6, 7 and 8: the fixtures ───────────────────────────────────────────

fn load(dir: &str) -> Run {
    run::load(&LoadOptions {
        project: root().join("scratch").join(dir),
        scenario: "baseline".into(),
        seed: 42,
        results: None,
        fcd: None,
        from: None,
        to: None,
    })
    .unwrap()
}

/// The fixture's run, its clock from `results.db`, and its timeline.
fn replay(dir: &str) -> (Run, f64, f64, Signals) {
    let run = load(dir);
    let results = root().join("scratch").join(dir).join("results.db");
    let (dt, duration) = inputs::run_clock(&results, "baseline", 42).unwrap();
    let t0 = Instant::now();
    let sig = Signals::build(&run.placement, dt, duration);
    println!(
        "{dir}: timeline built in {:.1} ms (recorded, not asserted)",
        t0.elapsed().as_secs_f64() * 1e3
    );
    (run, dt, duration, sig)
}

struct Expect6 {
    clock: (f64, f64),
    junctions: usize,
    phases: &'static [(usize, usize)],
    approaches: usize,
    steps: usize,
    changes: usize,
    /// Run lengths in steps: green, amber, red after an amber.
    green: &'static [usize],
    amber: &'static [usize],
    red: &'static [usize],
    /// The red an approach starts the run with: steps, and how many.
    first_red: (usize, usize),
    fcd_times: usize,
    compared: usize,
    at_line: usize,
    /// The one dot off its lane 0's line, m ahead; `None` for none.
    off_line: Option<f64>,
    pairs: usize,
    partial: usize,
}

fn gate6(dir: &str, e: Expect6) {
    let (run, dt, duration, sig) = replay(dir);
    let pl = &run.placement;
    let (net, nd) = (&pl.network, &pl.data);
    println!("gate 6 {dir}: run_clock ({dt}, {duration})");
    assert_eq!((dt, duration), e.clock);

    let mut junctions = 0;
    let mut phases: BTreeMap<usize, usize> = BTreeMap::new();
    for j in &net.junctions {
        if j.control == ControlType::Signal
            && let Some(sp) = &j.signal_plan
        {
            junctions += 1;
            *phases.entry(sp.phases.len()).or_default() += 1;
        }
    }
    let phases: Vec<(usize, usize)> = phases.into_iter().collect();
    println!("gate 6 {dir}: {junctions} signalised junctions; phases per plan {phases:?}");
    assert_eq!(junctions, e.junctions);
    assert_eq!(phases, e.phases);

    let all = SignalState::new(nd).signalized_approach_links().len();
    let changes: usize = sig.approaches.iter().map(|a| a.changes.len()).sum();
    println!(
        "gate 6 {dir}: {} approaches = dots, {} left out; {} steps; {changes} changes",
        sig.approaches.len(),
        all - sig.approaches.len(),
        sig.t.len() - 1
    );
    assert_eq!((sig.approaches.len(), all), (e.approaches, e.approaches));
    assert_eq!(sig.t.len() - 1, e.steps);
    assert_eq!(changes, e.changes);

    // Run lengths: from one entry of `changes` to the next; the last is cut off.
    let (mut green, mut amber, mut red) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    let mut first_red: BTreeMap<usize, usize> = BTreeMap::new();
    for a in &sig.approaches {
        for (j, w) in a.changes.windows(2).enumerate() {
            let n = (w[1].0 - w[0].0) as usize;
            match (w[0].1, j) {
                (G, _) => green.insert(n),
                (A, _) => amber.insert(n),
                (R, 0) => {
                    *first_red.entry(n).or_default() += 1;
                    true
                }
                (R, _) => red.insert(n),
            };
        }
    }
    let first_red: Vec<(usize, usize)> = first_red.into_iter().collect();
    println!(
        "gate 6 {dir}: run lengths in steps: green {green:?}; amber {amber:?}; red after an amber {red:?}; the red from step 0 {first_red:?}"
    );
    assert_eq!(green.into_iter().collect::<Vec<_>>(), e.green);
    assert_eq!(amber.into_iter().collect::<Vec<_>>(), e.amber);
    assert_eq!(red.into_iter().collect::<Vec<_>>(), e.red);
    assert_eq!(first_red, [e.first_red]);

    // Every FCD time is a T_k, bit for bit.
    let mut times: Vec<f64> = run
        .fcd
        .vehicles
        .iter()
        .flat_map(|v| v.rows.iter().map(|r| r.time))
        .collect();
    times.sort_by(f64::total_cmp);
    times.dedup();
    let exact = times
        .iter()
        .filter(|&&t| sig.t[sig.step_at(t)] == t)
        .count();
    println!(
        "gate 6 {dir}: FCD times bit-equal to a T_k: {exact} of {}",
        times.len()
    );
    assert_eq!((exact, times.len()), (e.fcd_times, e.fcd_times));

    // colours_at against a second SignalState stepped alongside.
    let mut probes: Vec<f64> = times.clone();
    for &t in &times {
        probes.push(t + 0.05);
        probes.push(t + 0.5);
    }
    probes.sort_by(f64::total_cmp);
    let ids: Vec<LinkId> = sig
        .approaches
        .iter()
        .map(|a| LinkId(a.link.clone()))
        .collect();
    let mut s = SignalState::new(nd);
    let (mut k, mut eq, mut ne) = (0, 0, 0);
    for &t in &probes {
        let want = sig.step_at(t);
        while k < want {
            s.advance(dt);
            k += 1;
        }
        for (c, id) in sig.colours_at(t).iter().zip(&ids) {
            if Colour::of(s.link_signal_color(nd, id)) == *c {
                eq += 1;
            } else {
                ne += 1;
            }
        }
    }
    println!("gate 6 {dir}: colours_at against a second SignalState: {eq} equal, {ne} differ");
    assert_eq!((eq, ne), (e.compared, 0));

    // Each dot at its lane-0 stop line's junction-facing midpoint: the `Kind::Stop`
    // quad's centroid, 0.2 m along the heading at the link's end.
    let st = Streets::build(pl);
    let (mut at_line, mut off) = (0, Vec::new());
    for a in &sig.approaches {
        let li = net.links.iter().position(|l| l.id.0 == a.link).unwrap();
        let q: Vec<_> = st
            .markings
            .iter()
            .filter(|m| m.kind == Kind::Stop && m.link == Some(li) && m.lane == Some(0))
            .collect();
        assert_eq!(q.len(), 1, "{}: one lane-0 stop line", a.link);
        let poly = &q[0].polygon;
        let n = poly.len() as f64;
        let c = poly
            .iter()
            .fold([0.0, 0.0], |s, p| [s[0] + p[0] / n, s[1] + p[1] / n]);
        let len = nd.link_length(&LinkId(a.link.clone()));
        let hd = pl.place(&a.link, len, 0).unwrap().heading.to_radians();
        let m = [c[0] + 0.2 * hd.sin(), c[1] + 0.2 * hd.cos()];
        let d = ((m[0] - a.at[0]).powi(2) + (m[1] - a.at[1]).powi(2)).sqrt();
        if d <= 0.001 {
            at_line += 1;
        } else {
            off.push(d);
        }
    }
    println!(
        "gate 6 {dir}: dots at their lane-0 stop line's front midpoint within 1 mm: {at_line} of {}; others {off:?} m",
        sig.approaches.len()
    );
    assert_eq!(at_line, e.at_line);
    match e.off_line {
        None => assert!(off.is_empty()),
        Some(d) => {
            assert_eq!(off.len(), 1);
            assert!((off[0] - d).abs() < 5e-5, "{} m", off[0]);
        }
    }

    // (approach, phase) pairs: green by permitted movements alone; green while a
    // movement from the link is in neither list.
    let (mut pairs, mut permitted_only, mut partial) = (0, 0, 0);
    for a in &sig.approaches {
        let j = net
            .junctions
            .iter()
            .find(|j| j.node_id.0 == a.node)
            .unwrap();
        let sp = j.signal_plan.as_ref().unwrap();
        let mv: Vec<_> = j
            .movements
            .iter()
            .filter(|m| m.from_link.0 == a.link)
            .collect();
        for p in &sp.phases {
            pairs += 1;
            let g: Vec<bool> = mv
                .iter()
                .map(|m| p.green_movements.contains(&m.id))
                .collect();
            let pe: Vec<bool> = mv
                .iter()
                .map(|m| p.permitted_movements.contains(&m.id))
                .collect();
            let (any_g, any_p) = (g.contains(&true), pe.contains(&true));
            if any_g || any_p {
                if !any_g {
                    permitted_only += 1;
                }
                if g.iter().zip(&pe).any(|(a, b)| !a && !b) {
                    partial += 1;
                }
            }
        }
    }
    println!(
        "gate 6 {dir}: (approach, phase) pairs {pairs}: green by permitted alone {permitted_only}; green with a movement in no list {partial}"
    );
    assert_eq!((pairs, permitted_only, partial), (e.pairs, 0, e.partial));
    println!("gate 6 {dir}: PASS");
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate6_midtown_against_engine() {
    gate6(
        "midtown",
        Expect6 {
            clock: (0.1, 1200.0),
            junctions: 83,
            phases: &[(1, 2), (2, 81)],
            approaches: 188,
            steps: 12_000,
            changes: 11_708,
            green: &[250],
            amber: &[30],
            red: &[20, 320],
            first_red: (300, 91),
            fcd_times: 1199,
            compared: 676_236,
            at_line: 188,
            off_line: None,
            pairs: 372,
            partial: 2,
        },
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate6_urban_grid_against_engine() {
    gate6(
        "urban_grid",
        Expect6 {
            clock: (0.1, 300.0),
            junctions: 9,
            phases: &[(2, 9)],
            approaches: 36,
            steps: 3000,
            changes: 576,
            green: &[250],
            amber: &[30],
            red: &[320],
            first_red: (300, 18),
            fcd_times: 291,
            compared: 31_428,
            at_line: 35,
            off_line: Some(5.0),
            pairs: 72,
            partial: 0,
        },
    );
}

#[derive(Debug, PartialEq)]
struct Check {
    /// Junction entries: green, amber, amber then red, red throughout.
    entries: [usize; 4],
    episodes: usize,
    rows: usize,
    /// The colour at an episode's first row: red, amber, green.
    first: [usize; 3],
    /// The colour at the vehicle's next row after it: green, amber, red, no next row.
    next: [usize; 4],
    /// The most common delay from green to that next row, and its count.
    delay: (String, usize),
}

/// §2.7's check on the whole run, with `S_{k + shift}` read where it reads `S_k`, and with
/// each junction's offset applied when `offsets` (`S_{k − round(offset/dt)}`), clamped to
/// the run's steps.
fn fcd_check(run: &Run, sig: &Signals, dt: f64, shift: i64, offsets: bool) -> Check {
    let pl = &run.placement;
    let (net, nd) = (&pl.network, &pl.data);
    let steps = (sig.t.len() - 1) as i64;
    let idx: HashMap<&str, usize> = sig
        .approaches
        .iter()
        .enumerate()
        .map(|(i, a)| (a.link.as_str(), i))
        .collect();
    let off: Vec<i64> = sig
        .approaches
        .iter()
        .map(|a| {
            if !offsets {
                return 0;
            }
            let j = net
                .junctions
                .iter()
                .find(|j| j.node_id.0 == a.node)
                .unwrap();
            -((j.signal_plan.as_ref().unwrap().offset / dt).round() as i64)
        })
        .collect();
    let colour = |i: usize, k: usize| -> Colour {
        let kk = (k as i64 + shift + off[i]).clamp(0, steps) as usize;
        let ch = &sig.approaches[i].changes;
        ch[ch.partition_point(|&(s, _)| s as usize <= kk) - 1].1
    };

    // Junction entries: each visit to an approach followed by a row on another link. The
    // rows stay on the approach with `position` frozen through the transit, so the entry
    // is in (the row before the first moving row of the trailing frozen run, that row].
    let mut entries = [0; 4];
    for v in &run.fcd.vehicles {
        let rows = &v.rows;
        let mut i = 0;
        while i < rows.len() {
            let mut j = i;
            while j + 1 < rows.len() && rows[j + 1].link == rows[i].link {
                j += 1;
            }
            let link = run.fcd.links[rows[i].link as usize].as_str();
            if let (Some(&ai), true) = (idx.get(link), j + 1 < rows.len()) {
                let mut f0 = j;
                while f0 > i && rows[f0 - 1].position == rows[j].position {
                    f0 -= 1;
                }
                let a = (f0..=j).find(|&q| rows[q].speed >= 0.1).unwrap_or(j);
                let t_prev = if a > 0 {
                    rows[a - 1].time
                } else {
                    rows[a].time - 1.0
                };
                let (k0, k1) = (sig.step_at(t_prev) + 1, sig.step_at(rows[a].time));
                let seen: Vec<Colour> = (k0..=k1).map(|k| colour(ai, k)).collect();
                let class = if seen.contains(&G) {
                    0
                } else if seen.contains(&A) && seen.contains(&R) {
                    2
                } else if seen.contains(&A) {
                    1
                } else {
                    3
                };
                entries[class] += 1;
            }
            i = j + 1;
        }
    }

    // The first stopped boxes (vis-002 Phase 6 gate 7's rule), at signals only.
    let mut back: HashMap<(&str, u32), f64> = HashMap::new();
    for a in &sig.approaches {
        let l = LinkId(a.link.clone());
        let lanes = net.links.iter().find(|x| x.id == l).unwrap().lanes.len() as u32;
        for lane in 0..lanes {
            let s1 = nd.link_length(&l) - nd.lane_stop_line_offset(&l, LaneIdx(lane));
            back.insert((a.link.as_str(), lane), s1 - 0.4);
        }
    }
    // (vehicle, link, first time, last time)
    let mut eps: Vec<(u64, &str, f64, f64)> = Vec::new();
    let mut last: HashMap<u64, usize> = HashMap::new();
    let mut rows_n = 0;
    for snap in &run.fcd.snapshots {
        let rs = &run.fcd.rows[snap.start..snap.end];
        let mut lead: HashMap<(u32, u32), f64> = HashMap::new();
        for r in rs {
            let e = lead.entry((r.link, r.lane)).or_insert(f64::NEG_INFINITY);
            *e = e.max(r.position);
        }
        for r in rs {
            if r.speed >= 0.1 {
                continue;
            }
            let link = run.fcd.links[r.link as usize].as_str();
            let Some(b) = back.get(&(link, r.lane)) else {
                continue;
            };
            if lead[&(r.link, r.lane)] > r.position {
                continue;
            }
            if (b - (r.position + r.length / 2.0)).abs() > 5.0 {
                continue;
            }
            rows_n += 1;
            match last.get(&r.vehicle_id) {
                Some(&e) if snap.time - eps[e].3 <= 1.5 => eps[e].3 = snap.time,
                _ => {
                    eps.push((r.vehicle_id, link, snap.time, snap.time));
                    last.insert(r.vehicle_id, eps.len() - 1);
                }
            }
        }
    }
    let vehicle: HashMap<u64, usize> = run
        .fcd
        .vehicles
        .iter()
        .enumerate()
        .map(|(i, v)| (v.vehicle_id, i))
        .collect();
    let (mut first, mut next) = ([0; 3], [0; 4]);
    let mut delays: BTreeMap<String, usize> = BTreeMap::new();
    for &(vid, link, t_first, t_last) in &eps {
        let ai = idx[link];
        first[match colour(ai, sig.step_at(t_first)) {
            R => 0,
            A => 1,
            G => 2,
        }] += 1;
        let v = &run.fcd.vehicles[vehicle[&vid]];
        let Some(nr) = v.rows.iter().find(|r| r.time > t_last + 1e-6) else {
            next[3] += 1;
            continue;
        };
        let k_end = sig.step_at(nr.time);
        let c = colour(ai, k_end);
        next[match c {
            G => 0,
            A => 1,
            R => 2,
        }] += 1;
        if c == G {
            let mut kk = k_end;
            while kk > 0 && colour(ai, kk - 1) == G && k_end - kk < 2000 {
                kk -= 1;
            }
            *delays
                .entry(format!("{:.2}", nr.time - sig.t[kk]))
                .or_default() += 1;
        }
    }
    // The most common delay; a tie goes to the shorter (BTreeMap order).
    let delay = delays.iter().fold((String::new(), 0), |best, (d, &n)| {
        if n > best.1 { (d.clone(), n) } else { best }
    });
    Check {
        entries,
        episodes: eps.len(),
        rows: rows_n,
        first,
        next,
        delay,
    }
}

fn gate7(dir: &str, want: Check, control: &[(i64, bool, usize)]) {
    let (run, dt, _, sig) = replay(dir);
    let got = fcd_check(&run, &sig, dt, 0, false);
    println!(
        "gate 7 {dir}: junction entries {} (green, amber, amber then red, red throughout {:?}); first stopped boxes {} episodes ({} rows), first row red/amber/green {:?}, next row green/amber/red/none {:?}; most common delay from green {} s ({})",
        got.entries.iter().sum::<usize>(),
        got.entries,
        got.episodes,
        got.rows,
        got.first,
        got.next,
        got.delay.0,
        got.delay.1
    );
    assert_eq!(got, want, "{dir}");
    for &(shift, offsets, red) in control {
        let c = fcd_check(&run, &sig, dt, shift, offsets);
        println!(
            "gate 7 {dir} control (shift {shift} steps, offsets {offsets}): entries on red throughout {}",
            c.entries[3]
        );
        assert_eq!(
            c.entries[3], red,
            "{dir} control shift {shift} offsets {offsets}"
        );
    }
    println!("gate 7 {dir}: PASS");
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate7_midtown_against_fcd() {
    gate7(
        "midtown",
        Check {
            entries: [6896, 570, 70, 0],
            episodes: 2490,
            rows: 48_896,
            first: [2301, 27, 162],
            next: [2406, 10, 7, 67],
            delay: ("1.10".into(), 2194),
        },
        // The dots 1 s early: 70 on red throughout; 1 s late, the wrong sign: 0.
        &[(10, false, 70), (-10, false, 0)],
    );
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate7_urban_grid_against_fcd() {
    gate7(
        "urban_grid",
        Check {
            entries: [373, 49, 0, 0],
            episodes: 129,
            rows: 2555,
            first: [118, 0, 11],
            next: [107, 0, 0, 22],
            delay: ("0.10".into(), 96),
        },
        // The plans' offsets applied: 90 on red throughout.
        &[(0, true, 90)],
    );
}

fn gate8_place(dir: &str) {
    let run = load(dir);
    let pl = &run.placement;
    let sig = Signals::build(pl, 0.1, 1.0);
    for a in &sig.approaches {
        let l = LinkId(a.link.clone());
        let s = (pl.data.link_length(&l) - pl.data.stop_line_offset(&l)).max(0.0);
        let p = pl.place(&a.link, s, 0).unwrap();
        assert_eq!(a.at, [p.x, p.y], "{}", a.link);
    }
    println!(
        "gate 8 {dir}: {} dots each at Placement::place(link, max(L − stop_line_offset, 0), 0), bit for bit: PASS",
        sig.approaches.len()
    );
}

#[test]
#[ignore = "needs scratch/midtown (scripts/fixture.sh midtown)"]
fn gate8_midtown_place() {
    gate8_place("midtown");
}

#[test]
#[ignore = "needs scratch/urban_grid (scripts/fixture.sh)"]
fn gate8_urban_grid_place() {
    gate8_place("urban_grid");
}

// ── Gate 15: `L`, headless ───────────────────────────────────────────────────

const W: f64 = 1280.0;
const H: f64 = 720.0;
const DT: f64 = 1.0 / 60.0;

/// vis-001 Phase 3's `plain_state`: `[0, 300]`, centre (0, 0), `k` = 1, 1280×720.
fn plain_state() -> ViewState {
    let fit = Fit {
        cx: 0.0,
        cy: 0.0,
        k: 1.0,
        rect: (-1e4, -1e4, 1e4, 1e4),
        size: (W, H),
    };
    ViewState::new(0.0, 300.0, vec![], fit)
}

fn at(c: (f64, f64)) -> ViewInput {
    ViewInput {
        cursor: Some(c),
        pointer: Some(c),
        size: (W, H),
        dt: DT,
        ..Default::default()
    }
}

fn no_boxes(_: f64) -> Vec<VehicleBox> {
    vec![]
}

/// `input` with `l` pressed flips the flag and does exactly what `input` alone does.
fn step_l(s: &mut ViewState, input: ViewInput) {
    let mut without = s.clone();
    let line = without.frame(&input, no_boxes);
    let before = s.signals_shown;
    let mut with_l = input;
    with_l.pressed.l = true;
    assert_eq!(s.frame(&with_l, no_boxes), line, "the keyframe line");
    assert_eq!(s.signals_shown, !before, "l flips the flag");
    let mut flipped_back = s.clone();
    flipped_back.signals_shown = before;
    assert_eq!(flipped_back, without, "l changes nothing else");
}

fn step(s: &mut ViewState, input: ViewInput) {
    let before = s.signals_shown;
    s.frame(&input, no_boxes);
    assert_eq!(s.signals_shown, before, "only l flips the flag");
}

#[test]
fn gate15_l() {
    let mut s = plain_state();
    assert!(!s.signals_shown, "off at new");
    let (buildings, see_through, streets) = (s.buildings_shown, s.see_through, s.streets_shown);
    step_l(&mut s, at((640.0, 360.0)));
    assert!(s.signals_shown);
    step_l(&mut s, at((640.0, 360.0)));
    assert!(!s.signals_shown);

    // Playing, following, with K pressed in the same frame.
    s.playing = true;
    s.follow = Some(Follow {
        vehicle_id: 7,
        drawn: false,
    });
    let mut k = at((640.0, 360.0));
    k.pressed.k = true;
    step_l(&mut s, k);
    step_l(&mut s, at((640.0, 360.0)));
    s.playing = false;
    s.follow = None;

    // A right-button orbit.
    step(
        &mut s,
        ViewInput {
            right_press: true,
            ..at((640.0, 360.0))
        },
    );
    assert!(s.orbit.is_some());
    step_l(&mut s, at((700.0, 330.0)));
    assert!(s.orbit.is_some(), "the orbit goes on");
    step(
        &mut s,
        ViewInput {
            right_release: true,
            ..at((700.0, 330.0))
        },
    );
    assert!(s.orbit.is_none());

    // A left drag.
    step(
        &mut s,
        ViewInput {
            press: true,
            ..at((400.0, 300.0))
        },
    );
    step_l(&mut s, at((460.0, 340.0)));
    assert!(s.press.is_some(), "the drag goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((460.0, 340.0))
        },
    );

    // A scrub.
    step(
        &mut s,
        ViewInput {
            press: true,
            ..at((640.0, 706.0))
        },
    );
    assert!(s.scrub.is_some());
    step_l(&mut s, at((300.0, 706.0)));
    assert!(s.scrub.is_some(), "the scrub goes on");
    step(
        &mut s,
        ViewInput {
            release: true,
            ..at((300.0, 706.0))
        },
    );
    assert!(s.scrub.is_none());
    assert_eq!(
        (s.buildings_shown, s.see_through, s.streets_shown),
        (buildings, see_through, streets),
        "l never touches buildings_shown, see_through or streets_shown"
    );
    println!(
        "gate 15: l flips signals_shown and nothing else, through an orbit, a drag and a scrub: PASS"
    );
}
