//! `assimilator-video view` (vis-001 §2.9): a window over a finished run. Each frame turns
//! Bevy's input into a [`ViewInput`], applies it to the [`ViewState`], prints a returned
//! keyframe line on stdout, and draws the state: the camera, the boxes at `t` and the
//! readout.

pub mod state;

use std::io::Write;

use anyhow::{Result, bail};
use bevy::input::keyboard::Key;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};

use crate::draw;
use crate::run::{self, LoadOptions, Run};
pub use state::{Fit, ViewInput, ViewState};

/// What `view` needs, after CLI parsing.
#[derive(Debug, Clone)]
pub struct ViewOptions {
    pub load: LoadOptions,
    /// The window, logical pixels.
    pub width: u32,
    pub height: u32,
    /// The hidden `--bench <s>` (vis-001 Phase 3): record `s` seconds of frame times.
    pub bench: Option<f64>,
}

/// `--bench` starts here, playing at this speed index (2×).
const BENCH_T: f64 = 140.0;
const BENCH_SPEED: usize = 4;
const BENCH_WARMUP: f64 = 3.0;

#[derive(Resource)]
struct Viewer {
    run: Run,
    state: ViewState,
    pool: Vec<Entity>,
    materials: Vec<Handle<StandardMaterial>>,
    camera: Entity,
    readout: Entity,
    bench: Option<Bench>,
}

struct Bench {
    secs: f64,
    elapsed: f64,
    frame_ms: Vec<f64>,
}

/// Every input check runs here, before any `App` is built; then the window opens and
/// runs on this (the main) thread until it is closed.
pub fn run(o: &ViewOptions) -> Result<()> {
    if o.width == 0 || o.height == 0 {
        bail!(
            "--width and --height must be positive (got {}x{})",
            o.width,
            o.height
        );
    }
    if let Some(s) = o.bench
        && !(s.is_finite() && s > 0.0)
    {
        bail!("--bench must be positive (got {s})");
    }
    let run = run::load(&o.load)?;
    let fit = Fit::new(&run.strips, o.width, o.height);
    let snaps = run.fcd.snapshots.iter().map(|s| s.time).collect();
    let mut state = ViewState::new(run.from, run.to, snaps, fit);
    if o.bench.is_some() {
        state.t = BENCH_T.clamp(state.from, state.to);
        state.playing = true;
        state.speed = BENCH_SPEED;
    }

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "assimilator-video view".into(),
            resolution: WindowResolution::new(o.width, o.height),
            ..default()
        }),
        ..default()
    }));
    let world = app.world_mut();
    let (road_mat, speed_mats) =
        draw::materials(&mut world.resource_mut::<Assets<StandardMaterial>>());
    let (road_mesh, box_mesh) = {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        (
            meshes.add(draw::road_mesh(&run.strips, fit.cx, fit.cy)),
            meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        )
    };
    let camera = world
        .spawn((
            draw::camera_fixed(),
            draw::projection((fit.size.0 * fit.k) as f32, (fit.size.1 * fit.k) as f32),
            draw::look_down(0.0, 0.0),
        ))
        .id();
    world.spawn((
        Mesh3d(road_mesh),
        MeshMaterial3d(road_mat),
        Transform::IDENTITY,
    ));
    let pool = draw::spawn_pool(world, &box_mesh, &speed_mats[0], run.motion.max_drawn());
    let readout = world
        .spawn((
            Text::new(state.readout()),
            TextFont::from_font_size(FontSize::Px(14.0)),
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(6.0),
                left: Val::Px(8.0),
                ..default()
            },
        ))
        .id();
    world.insert_resource(Viewer {
        run,
        state,
        pool,
        materials: speed_mats,
        camera,
        readout,
        bench: o.bench.map(|secs| Bench {
            secs,
            elapsed: 0.0,
            frame_ms: Vec::new(),
        }),
    });
    app.add_systems(Update, frame);
    app.run();
    Ok(())
}

/// Bevy's input for this frame, as a [`ViewInput`].
fn input(world: &mut World) -> ViewInput {
    let mut q = world.query_filtered::<&Window, With<PrimaryWindow>>();
    let (size, cursor) = match q.iter(world).next() {
        Some(w) => (
            (w.width() as f64, w.height() as f64),
            w.cursor_position().map(|c| (c.x as f64, c.y as f64)),
        ),
        None => ((1.0, 1.0), None),
    };
    let keys = world.resource::<ButtonInput<KeyCode>>();
    // Speed keys by their logical character, so `+` works on any layout; the keypad by
    // position.
    let chars = world.resource::<ButtonInput<Key>>();
    let char_pressed = |f: fn(&str) -> bool| {
        chars
            .get_just_pressed()
            .any(|k| matches!(k, Key::Character(c) if f(c)))
    };
    let mouse = world.resource::<ButtonInput<MouseButton>>();
    let scroll = world.resource::<AccumulatedMouseScroll>();
    let (scroll_lines, scroll_pixels) = match scroll.unit {
        MouseScrollUnit::Line => (scroll.delta.y as f64, 0.0),
        MouseScrollUnit::Pixel => (0.0, scroll.delta.y as f64),
    };
    ViewInput {
        pressed: state::Pressed {
            space: keys.just_pressed(KeyCode::Space),
            plus: char_pressed(state::is_speed_up) || keys.just_pressed(KeyCode::NumpadAdd),
            minus: char_pressed(state::is_speed_down) || keys.just_pressed(KeyCode::NumpadSubtract),
            left: keys.just_pressed(KeyCode::ArrowLeft),
            right: keys.just_pressed(KeyCode::ArrowRight),
            esc: keys.just_pressed(KeyCode::Escape),
            k: keys.just_pressed(KeyCode::KeyK),
        },
        held: state::Held {
            shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            w: keys.pressed(KeyCode::KeyW),
            a: keys.pressed(KeyCode::KeyA),
            s: keys.pressed(KeyCode::KeyS),
            d: keys.pressed(KeyCode::KeyD),
        },
        cursor,
        press: mouse.just_pressed(MouseButton::Left),
        release: mouse.just_released(MouseButton::Left),
        scroll_lines,
        scroll_pixels,
        size,
        dt: world.resource::<Time<Real>>().delta_secs_f64(),
    }
}

fn frame(world: &mut World) {
    let input = input(world);
    world.resource_scope(|world, mut v: Mut<Viewer>| {
        let v = &mut *v;
        let run = &v.run;
        if let Some(line) = v.state.frame(&input, |t| run.boxes_at(t)) {
            let mut out = std::io::stdout().lock();
            let _ = writeln!(out, "{line}");
            let _ = out.flush();
        }

        let s = &v.state;
        let (fx, fy) = (s.fit.cx, s.fit.cy);
        let mut cam = world.entity_mut(v.camera);
        *cam.get_mut::<Transform>().unwrap() =
            draw::look_down((s.cx - fx) as f32, -(s.cy - fy) as f32);
        *cam.get_mut::<Projection>().unwrap() =
            draw::projection((s.size.0 * s.k) as f32, (s.size.1 * s.k) as f32);
        let boxes = run.boxes_at(s.t);
        draw::fill_pool(world, &v.pool, &v.materials, &boxes, fx, fy);
        let text = s.readout();
        let mut ent = world.entity_mut(v.readout);
        let mut t = ent.get_mut::<Text>().unwrap();
        if t.0 != text {
            t.0 = text;
        }

        if let Some(b) = v.bench.as_mut() {
            b.elapsed += input.dt;
            if b.elapsed > BENCH_WARMUP {
                b.frame_ms.push(input.dt * 1000.0);
            }
            if b.frame_ms.iter().sum::<f64>() >= b.secs * 1000.0 {
                eprintln!("{}", bench_json(&b.frame_ms));
                v.bench = None;
                world.write_message(AppExit::Success);
            }
        }
    });
}

fn bench_json(ms: &[f64]) -> String {
    let mut sorted = ms.to_vec();
    sorted.sort_by(f64::total_cmp);
    let n = sorted.len();
    let q = |p: f64| sorted[((p * (n - 1) as f64).round() as usize).min(n - 1)];
    let total: f64 = ms.iter().sum();
    format!(
        "{{\"frames\": {n}, \"mean_fps\": {:.2}, \"median_ms\": {:.2}, \"p99_ms\": {:.2}, \"worst_ms\": {:.2}}}",
        n as f64 / (total / 1000.0),
        q(0.5),
        q(0.99),
        sorted[n - 1]
    )
}
