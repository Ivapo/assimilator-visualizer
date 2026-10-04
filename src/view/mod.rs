//! `assimilator-video view` (vis-001 §2.9): a window over a finished run. Each frame turns
//! Bevy's input into a [`ViewInput`], applies it to the [`ViewState`], prints a returned
//! keyframe line on stdout, and draws the state: the perspective camera at the state's
//! pose (§2.11), the shaded boxes at `t`, the readout and the time slider (§2.10). With
//! `--buildings` (vis-002 §2.8), the buildings and the sun, shown or hidden by `B`, with
//! the buildings in the way cut to stubs while see-through is on (§2.15.6), flipped by `X`.

pub mod slider;
pub mod state;

use std::io::Write;

use anyhow::{Result, anyhow, bail};
use bevy::input::keyboard::Key;
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::window::{CursorMoved, PrimaryWindow, WindowResolution};

use crate::buildings::Buildings;
use crate::draw::{self, BuildingsCut};
use crate::run::{self, LoadOptions, Run};
use crate::see_through;
pub use state::{Fit, ViewInput, ViewState};

/// What `view` needs, after CLI parsing.
#[derive(Debug, Clone)]
pub struct ViewOptions {
    pub load: LoadOptions,
    /// The `--buildings` file.
    pub buildings: Option<std::path::PathBuf>,
    /// See-through at launch (vis-002 §2.15.6): true unless `--no-see-through`.
    pub see_through: bool,
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
    boxes: draw::BoxesEntity,
    camera: Entity,
    /// The buildings, their mesh and sun; `None` without `--buildings`.
    buildings: Option<ViewBuildings>,
    readout: Entity,
    bar: BarNodes,
    bench: Option<Bench>,
}

/// The buildings as drawn, for `B` and `X`.
struct ViewBuildings {
    entities: draw::BuildingEntities,
    buildings: Buildings,
    /// The heights the mesh is drawn at now, in file order.
    drawn: Vec<f64>,
    /// Built the first time see-through is on.
    cut: Option<BuildingsCut>,
}

/// The time slider's `bevy_ui` nodes, placed each frame from [`ViewState::bar`].
struct BarNodes {
    track: Entity,
    handle: Entity,
    /// A pool, grown when a frame needs more ticks; unused ones are hidden.
    ticks: Vec<Entity>,
}

/// The primary window's last `CursorMoved` position this frame, logical and unbounded
/// ([`ViewInput::pointer`]).
#[derive(Resource, Default)]
struct Pointer(Option<(f64, f64)>);

const TRACK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.35);
const TICK_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.7);
const HANDLE_COLOR: Color = Color::WHITE;

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
    let buildings = match &o.buildings {
        None => None,
        Some(file) => Some(
            crate::buildings::read(file, &run.placement.network)
                .map_err(|e| anyhow!("--buildings {}: {e}", file.display()))?,
        ),
    };
    let fit = Fit::new(&run.strips, o.width, o.height);
    let snaps = run.fcd.snapshots.iter().map(|s| s.time).collect();
    let mut state = ViewState::new(run.from, run.to, snaps, fit);
    state.see_through = o.see_through;
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
    let (road_mat, _) = draw::materials(&mut world.resource_mut::<Assets<StandardMaterial>>());
    let road_mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(draw::road_mesh(&run.strips, fit.cx, fit.cy));
    let (transform, projection) = draw::perspective(
        &state.pose(),
        fit.cx,
        fit.cy,
        fit.size.0 / fit.size.1,
    );
    let camera = world
        .spawn((draw::camera_fixed(), projection, transform))
        .id();
    world.spawn((
        Mesh3d(road_mesh),
        MeshMaterial3d(road_mat),
        Transform::IDENTITY,
    ));
    let boxes = draw::spawn_boxes(world);
    let buildings = buildings.map(|b| ViewBuildings {
        entities: draw::spawn_buildings(world, &b, (fit.cx, fit.cy)),
        drawn: b.buildings.iter().map(|x| x.top()).collect(),
        buildings: b,
        cut: None,
    });
    let readout = world
        .spawn((
            Text::new(state.readout()),
            TextFont::from_font_size(FontSize::Px(14.0)),
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(slider::READOUT_BOTTOM as f32),
                left: Val::Px(slider::INSET as f32),
                ..default()
            },
        ))
        .id();
    // Ticks are drawn over the track and under the handle.
    let bar = BarNodes {
        track: spawn_rect(world, TRACK_COLOR, 0),
        handle: spawn_rect(world, HANDLE_COLOR, 2),
        ticks: Vec::new(),
    };
    world.init_resource::<Pointer>();
    world.insert_resource(Viewer {
        run,
        state,
        boxes,
        camera,
        buildings,
        readout,
        bar,
        bench: o.bench.map(|secs| Bench {
            secs,
            elapsed: 0.0,
            frame_ms: Vec::new(),
        }),
    });
    app.add_systems(Update, (track_pointer, frame).chain());
    app.run();
    Ok(())
}

/// Keep the primary window's last `CursorMoved` position of this frame. Unlike
/// `Window::cursor_position`, it is not bounded to the window: while a button is held,
/// winit on macOS keeps sending it outside (vis-001 §2.10.3).
fn track_pointer(
    mut moved: MessageReader<CursorMoved>,
    primary: Query<Entity, With<PrimaryWindow>>,
    mut pointer: ResMut<Pointer>,
) {
    let window = primary.single().ok();
    pointer.0 = moved
        .read()
        .filter(|m| Some(m.window) == window)
        .last()
        .map(|m| (m.position.x as f64, m.position.y as f64));
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
            q: keys.just_pressed(KeyCode::KeyQ),
            e: keys.just_pressed(KeyCode::KeyE),
            r: keys.just_pressed(KeyCode::KeyR),
            f: keys.just_pressed(KeyCode::KeyF),
            b: keys.just_pressed(KeyCode::KeyB),
            x: keys.just_pressed(KeyCode::KeyX),
        },
        held: state::Held {
            shift: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            w: keys.pressed(KeyCode::KeyW),
            a: keys.pressed(KeyCode::KeyA),
            s: keys.pressed(KeyCode::KeyS),
            d: keys.pressed(KeyCode::KeyD),
            ctrl: keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        },
        cursor,
        pointer: world.resource::<Pointer>().0,
        press: mouse.just_pressed(MouseButton::Left),
        release: mouse.just_released(MouseButton::Left),
        right_press: mouse.just_pressed(MouseButton::Right),
        right_release: mouse.just_released(MouseButton::Right),
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
        let (transform, projection) = draw::perspective(&s.pose(), fx, fy, s.size.0 / s.size.1);
        let mut cam = world.entity_mut(v.camera);
        *cam.get_mut::<Transform>().unwrap() = transform;
        *cam.get_mut::<Projection>().unwrap() = projection;
        let boxes = run.boxes_at(s.t);
        draw::set_boxes(world, &v.boxes, &boxes, (fx, fy), draw::RANK_LIFT_3D);
        if let Some(b) = v.buildings.as_mut() {
            let want = if s.buildings_shown {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            for e in [b.entities.mesh, b.entities.sun] {
                let mut ent = world.entity_mut(e);
                let mut vis = ent.get_mut::<Visibility>().unwrap();
                if *vis != want {
                    *vis = want;
                }
            }
            // See-through (vis-002 §2.15.6): the mesh is replaced only when a height
            // changes, and on every frame of `--bench` while it is on.
            if s.see_through && b.cut.is_none() {
                b.cut = Some(BuildingsCut::new(&b.buildings, (fx, fy)));
            }
            let heights = if s.see_through && s.buildings_shown {
                see_through::heights(&b.buildings, &s.pose())
            } else {
                b.buildings.buildings.iter().map(|x| x.top()).collect()
            };
            if let Some(cut) = &b.cut
                && (heights != b.drawn || (v.bench.is_some() && s.see_through))
            {
                draw::set_building_heights(world, b.entities.mesh, cut, &heights);
                b.drawn = heights;
            }
        }
        let text = s.readout();
        let mut ent = world.entity_mut(v.readout);
        let mut t = ent.get_mut::<Text>().unwrap();
        if t.0 != text {
            t.0 = text;
        }
        draw_bar(world, &mut v.bar, s);

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

/// An absolutely positioned rectangle, hidden until placed.
fn spawn_rect(world: &mut World, color: Color, z: i32) -> Entity {
    world
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            BackgroundColor(color),
            ZIndex(z),
        ))
        .id()
}

/// Show `e` at `(left, top, width, height)` logical pixels, or hide it. The node is
/// written only when it changes, so a still frame triggers no layout.
fn place(world: &mut World, e: Entity, r: Option<(f64, f64, f64, f64)>) {
    let mut ent = world.entity_mut(e);
    let mut node = ent.get_mut::<Node>().unwrap();
    let Some((l, t, w, h)) = r else {
        if node.display != Display::None {
            node.display = Display::None;
        }
        return;
    };
    let px = |v: f64| Val::Px(v as f32);
    let want = (Display::Flex, px(l), px(t), px(w), px(h));
    if (node.display, node.left, node.top, node.width, node.height) != want {
        (node.display, node.left, node.top, node.width, node.height) = want;
    }
}

/// Place the track, the handle at `x(t)` and the ticks from the state's geometry
/// (§2.10.1, §2.10.4); hide them all when there is no bar.
fn draw_bar(world: &mut World, nodes: &mut BarNodes, s: &ViewState) {
    let Some(b) = s.bar() else {
        place(world, nodes.track, None);
        place(world, nodes.handle, None);
        for &e in &nodes.ticks {
            place(world, e, None);
        }
        return;
    };
    let th = slider::TRACK_H;
    place(
        world,
        nodes.track,
        Some((b.x0, b.y_bar - th / 2.0, b.len(), th)),
    );
    let hs = slider::HANDLE;
    let hx = b.x(s.t, s.from, s.to);
    place(
        world,
        nodes.handle,
        Some((hx - hs / 2.0, b.y_bar - hs / 2.0, hs, hs)),
    );
    let ticks = b.ticks(s.from, s.to);
    while nodes.ticks.len() < ticks.len() {
        nodes.ticks.push(spawn_rect(world, TICK_COLOR, 1));
    }
    let (tw, tk) = (slider::TICK_W, slider::TICK_H);
    for (i, &e) in nodes.ticks.iter().enumerate() {
        let r = ticks
            .get(i)
            .map(|&t| (b.x(t, s.from, s.to) - tw / 2.0, b.y_bar - tk / 2.0, tw, tk));
        place(world, e, r);
    }
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
