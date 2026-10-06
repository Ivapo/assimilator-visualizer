//! Headless Bevy (vis-001 §2.3): no window, a camera rendering to an offscreen image, and
//! a lossless readback of every frame. The camera is Phase 1's top-down orthographic one,
//! or, for a keyframed render, a perspective one set from a pose each frame (§2.11.2).
//! With buildings (vis-002), the scene adds their mesh and the sun. With a credit line
//! (vis-002 §2.14), a UI tree draws it in the bottom-right corner of the image, fitted
//! to the frame's width after the settle frames. With see-through on (vis-002 §2.15), a
//! perspective renderer redraws the buildings at each pose's heights. With streets
//! (vis-002 §2.17), two more meshes: the junction surfaces and median fills, and the
//! markings, whose colours a perspective renderer fades for each pose.
//!
//! The update loop is pumped by hand. Each frame sets the vehicle boxes, schedules a
//! screenshot of the target image, and updates until that screenshot has been read back,
//! so a readback always belongs to the frame that asked for it.

use std::sync::{Arc, Mutex};

use anyhow::{Result, anyhow, bail};
use bevy::app::{SubApps, TerminalCtrlCHandlerPlugin};
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_resource::{
    Extent3d, PollType, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::ui::{ComputedNode, UiGlobalTransform};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;

use crate::buildings::Buildings;
use crate::camera::Pose;
use crate::credit;
use crate::draw::{self, BuildingsCut, CreditNodes, StreetsDrawn};
use crate::motion::TrackPos;
use crate::place::Placed;
use crate::scene::{self, Camera as SceneCamera, Strip};
use crate::see_through;
use crate::streets::{self, Streets};

/// One box to draw.
#[derive(Debug, Clone, Copy)]
pub struct VehicleBox {
    pub vehicle_id: u64,
    pub at: Placed,
    pub length: f64,
    /// FCD speed, linear between rows; sets the colour.
    pub speed: f64,
    /// Where the vehicle is on its track (vis-001 §2.8).
    pub track: TrackPos,
}

/// How many updates a readback may take before it is an error.
const MAX_UPDATES_PER_FRAME: usize = 200;

/// The renders that let assets, pipelines and the UI layout settle.
const SETTLE_RENDERS: usize = 3;

/// The credit line as drawn: its nodes, its font size after the fit, and its outline
/// offset (the first size's, §2.14.5).
struct Credit {
    nodes: CreditNodes,
    size: u32,
    offset: u32,
}

/// How the boxes are drawn: a pool of entities on the orthographic path, one mesh on the
/// perspective path (vis-001 §2.12.2).
enum Boxes {
    Pool {
        pool: Vec<Entity>,
        materials: Vec<Handle<StandardMaterial>>,
    },
    Mesh(draw::BoxesEntity),
}

/// See-through as kept by a perspective renderer with buildings (vis-002 §2.15.5).
struct Cut {
    buildings: Buildings,
    mesh: BuildingsCut,
    /// The heights the buildings' mesh is drawn at now, in file order.
    drawn: Vec<f64>,
    /// Off after on: the next `set_pose` restores every height, then drops the cut.
    on: bool,
}

pub struct Renderer {
    apps: SubApps,
    target: Handle<Image>,
    camera: SceneCamera,
    /// The most boxes a frame may hold.
    pool: usize,
    boxes: Boxes,
    slot: Arc<Mutex<Option<Vec<u8>>>>,
    /// The perspective camera's entity; `None` on the orthographic path.
    perspective: Option<Entity>,
    /// Metres of lift per rank.
    lift: f64,
    /// The credit line; `None` without one.
    credit: Option<Credit>,
    /// The buildings' mesh entity; `None` without buildings.
    buildings_mesh: Option<Entity>,
    /// See-through; `None` while it is off.
    cut: Option<Cut>,
    /// The road's material, which the junction surfaces share (vis-002 §2.17.9).
    road_mat: Handle<StandardMaterial>,
    /// The perspective camera's pose now; `None` on the orthographic path.
    pose: Option<Pose>,
    /// The streets; `None` while they are off.
    streets: Option<StreetsDrawn>,
}

impl Renderer {
    /// Build the scene: road strips in link order, and `pool` hidden boxes that frames
    /// fill in `vehicle_id` order. With `buildings`, their mesh and the sun too, and the
    /// orthographic eye rises above the tallest roof (`draw::ortho_eye`). With `credit`,
    /// the line too, fitted to the frame (vis-002 §2.14.5).
    pub fn new(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        buildings: Option<&Buildings>,
        credit: Option<&str>,
    ) -> Result<Self> {
        Self::build(strips, camera, pool, None, buildings, credit)
    }

    /// The same scene through a perspective camera at `pose` (§2.11.2): shaded boxes and
    /// `draw::RANK_LIFT_3D`, drawn as one mesh in `vehicle_id` order (§2.12.2). `camera`
    /// is the fit, the scene's bake origin and image size.
    pub fn new_perspective(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        pose: &Pose,
        buildings: Option<&Buildings>,
        credit: Option<&str>,
    ) -> Result<Self> {
        Self::build(strips, camera, pool, Some(pose), buildings, credit)
    }

    fn build(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        pose: Option<&Pose>,
        buildings: Option<&Buildings>,
        credit: Option<&str>,
    ) -> Result<Self> {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(RenderPlugin {
                    synchronous_pipeline_compilation: true,
                    ..default()
                })
                // No window and no OS event loop (vis-001 §2.9.6): `WinitPlugin::build`
                // creates the event loop, which on macOS panics off the main thread, and
                // with no primary window the default exit condition would write
                // `AppExit` on every update.
                .set(WindowPlugin {
                    primary_window: None,
                    exit_condition: ExitCondition::DontExit,
                    close_when_requested: false,
                    ..default()
                })
                .disable::<WinitPlugin>()
                .disable::<PipelinedRenderingPlugin>()
                // Its handler only asks the app to exit, and the loop here is pumped by
                // hand: keep the default SIGINT behaviour instead.
                .disable::<TerminalCtrlCHandlerPlugin>(),
        );
        app.finish();
        app.cleanup();
        let mut apps = std::mem::take(app.sub_apps_mut());
        let world = apps.main.world_mut();

        let mut image = Image::new_uninit(
            Extent3d {
                width: camera.width,
                height: camera.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        );
        image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
        let target = world.resource_mut::<Assets<Image>>().add(image);

        let (road_mat, speed_mats) =
            draw::materials(&mut world.resource_mut::<Assets<StandardMaterial>>());
        let (road_mesh, box_mesh) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(draw::road_mesh(strips, camera.cx, camera.cy)),
                pose.is_none()
                    .then(|| meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            )
        };

        let (image_camera, perspective) = match pose {
            None => {
                let (eye, far) = draw::ortho_eye(buildings.map_or(0.0, |b| b.tallest));
                let e = world
                    .spawn((
                        draw::camera_fixed(),
                        RenderTarget::Image(target.clone().into()),
                        draw::projection(
                            (camera.width as f64 * camera.k) as f32,
                            (camera.height as f64 * camera.k) as f32,
                            far as f32,
                        ),
                        draw::look_down(0.0, 0.0, eye as f32),
                    ))
                    .id();
                (e, None)
            }
            Some(pose) => {
                let aspect = camera.width as f64 / camera.height as f64;
                let (transform, projection) = draw::perspective(pose, camera.cx, camera.cy, aspect);
                let e = world
                    .spawn((
                        draw::camera_fixed(),
                        RenderTarget::Image(target.clone().into()),
                        projection,
                        transform,
                    ))
                    .id();
                (e, Some(e))
            }
        };
        world.spawn((
            Mesh3d(road_mesh),
            MeshMaterial3d(road_mat.clone()),
            Transform::IDENTITY,
        ));
        let boxes = match box_mesh {
            Some(box_mesh) => Boxes::Pool {
                pool: draw::spawn_pool(world, &box_mesh, &speed_mats[0], pool),
                materials: speed_mats,
            },
            None => Boxes::Mesh(draw::spawn_boxes(world)),
        };
        // Nothing is spawned without buildings, so such a render is unchanged.
        let buildings_mesh =
            buildings.map(|b| draw::spawn_buildings(world, b, (camera.cx, camera.cy)).mesh);
        // Nor without a credit line: a synthetic network's render is unchanged.
        let credit = credit.map(|line| {
            let size = credit::font_size(camera.width, camera.height);
            let offset = credit::outline_offset(size);
            let font = world
                .resource_mut::<Assets<Font>>()
                .add(Font::from_bytes(draw::CREDIT_FONT.to_vec()));
            let nodes = draw::spawn_credit(
                world,
                line,
                image_camera,
                font,
                size,
                credit::margin(size),
                offset,
            );
            Credit {
                nodes,
                size,
                offset,
            }
        });

        let mut r = Renderer {
            apps,
            target,
            camera,
            pool,
            boxes,
            slot: Arc::new(Mutex::new(None)),
            perspective,
            lift: match pose {
                None => scene::RANK_LIFT,
                Some(_) => draw::RANK_LIFT_3D,
            },
            credit,
            buildings_mesh,
            cut: None,
            road_mat,
            pose: pose.copied(),
            streets: None,
        };
        // Let assets and pipelines settle before the first frame that counts.
        r.settle()?;
        r.fit_credit()?;
        Ok(r)
    }

    fn settle(&mut self) -> Result<()> {
        for _ in 0..SETTLE_RENDERS {
            self.render(&[])?;
        }
        Ok(())
    }

    /// The fit (vis-002 §2.14.5): while the fill is wider than `W − 2·margin(S)` for the
    /// first size `S`, shrink all nine texts and settle again. A final size under
    /// `credit::MIN_SIZE` is an error (§2.14.7).
    fn fit_credit(&mut self) -> Result<()> {
        let Some(first) = self.credit.as_ref().map(|c| c.size) else {
            return Ok(());
        };
        let (w, h) = (self.camera.width, self.camera.height);
        let avail = w as f64 - 2.0 * credit::margin(first) as f64;
        loop {
            let c = self.credit.as_ref().unwrap();
            let (size, nodes) = (c.size, c.nodes);
            let width = self.fill_rect(nodes.fill)?.size().x as f64;
            let fit = credit::fit_size(size, width, avail).map_err(|e| {
                anyhow!(
                    "the credit line would be {} px in a {w}×{h} frame, under {} px; render a larger frame",
                    e.0,
                    credit::MIN_SIZE
                )
            })?;
            if fit == size {
                return Ok(());
            }
            draw::set_credit_size(self.apps.main.world_mut(), &nodes, fit);
            self.credit.as_mut().unwrap().size = fit;
            self.settle()?;
        }
    }

    /// The fill text's computed rectangle, in pixels.
    fn fill_rect(&self, fill: Entity) -> Result<Rect> {
        let world = self.apps.main.world();
        let (Some(node), Some(at)) = (
            world.get::<ComputedNode>(fill),
            world.get::<UiGlobalTransform>(fill),
        ) else {
            bail!("internal: the credit line has no layout");
        };
        Ok(Rect::from_center_size(at.translation, node.size))
    }

    /// The credit line's font size after the fit; `None` without a line.
    pub fn credit_size(&self) -> Option<u32> {
        self.credit.as_ref().map(|c| c.size)
    }

    /// The credit box (vis-002 §2.14.5): the fill text's computed rectangle as laid out
    /// now, grown by the outline offset on every side and rounded outward to whole pixels,
    /// as `[x0, y0, x1, y1]` with `x1` and `y1` exclusive. `None` without a line.
    pub fn credit_box(&self) -> Option<[u32; 4]> {
        let c = self.credit.as_ref()?;
        let r = self.fill_rect(c.nodes.fill).ok()?;
        let o = c.offset as f32;
        let (w, h) = (self.camera.width as f32, self.camera.height as f32);
        Some([
            (r.min.x - o).floor().clamp(0.0, w) as u32,
            (r.min.y - o).floor().clamp(0.0, h) as u32,
            (r.max.x + o).ceil().clamp(0.0, w) as u32,
            (r.max.y + o).ceil().clamp(0.0, h) as u32,
        ])
    }

    pub fn camera(&self) -> &SceneCamera {
        &self.camera
    }

    /// See-through (vis-002 §2.15.6): `Some(b)`, with `b` the buildings this renderer was
    /// built with, turns it on, keeping a copy of them and the mesh's `f64` positions;
    /// `None` turns it off. It takes effect at the next [`Renderer::set_pose`]. It has no
    /// effect on an orthographic renderer or one without buildings.
    pub fn set_see_through(&mut self, cut: Option<&Buildings>) {
        if self.perspective.is_none() || self.buildings_mesh.is_none() {
            return;
        }
        match (cut, self.cut.as_mut()) {
            (Some(_), Some(c)) => c.on = true,
            (Some(b), None) => {
                let cam = self.camera;
                self.cut = Some(Cut {
                    buildings: b.clone(),
                    mesh: BuildingsCut::new(b, (cam.cx, cam.cy)),
                    drawn: b.buildings.iter().map(|x| x.top()).collect(),
                    on: true,
                });
            }
            (None, Some(c)) => c.on = false,
            (None, None) => {}
        }
    }

    /// Streets (vis-002 §2.17.12): `Some` spawns their two meshes, the markings faded for
    /// the orthographic `k` or the current pose, then settles again; `None` despawns them.
    /// Off from the start, nothing is spawned.
    pub fn set_streets(&mut self, streets: Option<&Streets>) -> Result<()> {
        let world = self.apps.main.world_mut();
        if let Some(old) = self.streets.take() {
            draw::despawn_streets(world, old);
        }
        let Some(st) = streets else {
            return Ok(());
        };
        let cam = self.camera;
        let at = (cam.cx, cam.cy);
        self.streets = Some(match &self.pose {
            None => draw::spawn_streets(world, st, &self.road_mat, at, |_| cam.k),
            Some(pose) => draw::spawn_streets(
                world,
                st,
                &self.road_mat,
                at,
                streets::mpp_at(pose, cam.height as f64),
            ),
        });
        self.settle()
    }

    /// Set the perspective camera to `pose` for the next frames; nothing on the
    /// orthographic path. With see-through on, the buildings are redrawn at `pose`'s
    /// heights when any differs from the last drawn; with streets, the markings are
    /// recoloured for `pose` when any colour differs.
    pub fn set_pose(&mut self, pose: &Pose) {
        let Some(e) = self.perspective else { return };
        self.pose = Some(*pose);
        let cam = self.camera;
        let aspect = cam.width as f64 / cam.height as f64;
        let (transform, projection) = draw::perspective(pose, cam.cx, cam.cy, aspect);
        let world = self.apps.main.world_mut();
        let mut ent = world.entity_mut(e);
        *ent.get_mut::<Transform>().unwrap() = transform;
        *ent.get_mut::<Projection>().unwrap() = projection;
        if let Some(d) = self.streets.as_mut() {
            draw::set_street_colours(world, d, streets::mpp_at(pose, cam.height as f64));
        }
        let (Some(mesh), Some(c)) = (self.buildings_mesh, self.cut.as_mut()) else {
            return;
        };
        let want = if c.on {
            see_through::heights(&c.buildings, pose)
        } else {
            c.buildings.buildings.iter().map(|x| x.top()).collect()
        };
        if want != c.drawn {
            draw::set_building_heights(world, mesh, &c.mesh, &want);
            c.drawn = want;
        }
        if !c.on {
            self.cut = None;
        }
    }

    /// Render one frame with exactly these boxes (in draw order) and return its RGBA8
    /// (sRGB) pixels, rows top to bottom, `width · height · 4` bytes.
    pub fn render(&mut self, boxes: &[VehicleBox]) -> Result<Vec<u8>> {
        if boxes.len() > self.pool {
            bail!(
                "internal: {} vehicles in a frame, pool of {}",
                boxes.len(),
                self.pool
            );
        }
        let cam = self.camera;
        let world = self.apps.main.world_mut();
        match &self.boxes {
            Boxes::Pool { pool, materials } => {
                draw::fill_pool(world, pool, materials, boxes, (cam.cx, cam.cy), self.lift)
            }
            Boxes::Mesh(entity) => {
                draw::set_boxes(world, entity, boxes, (cam.cx, cam.cy), self.lift)
            }
        }
        *self.slot.lock().unwrap() = None;
        let slot = self.slot.clone();
        world.spawn(Screenshot::image(self.target.clone())).observe(
            move |captured: On<ScreenshotCaptured>| {
                *slot.lock().unwrap() = Some(captured.image.data.clone().unwrap_or_default());
            },
        );
        for _ in 0..MAX_UPDATES_PER_FRAME {
            self.update()?;
            if let Some(data) = self.slot.lock().unwrap().take() {
                let want = cam.width as usize * cam.height as usize * 4;
                if data.len() != want {
                    bail!(
                        "internal: readback of {} bytes, expected {want}",
                        data.len()
                    );
                }
                return Ok(data);
            }
        }
        bail!("render: no readback after {MAX_UPDATES_PER_FRAME} updates")
    }

    fn update(&mut self) -> Result<()> {
        self.apps.update();
        self.apps
            .main
            .world()
            .resource::<RenderDevice>()
            .wgpu_device()
            .poll(PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .map_err(|e| anyhow::anyhow!("render: GPU poll failed: {e}"))?;
        Ok(())
    }
}
