//! Headless Bevy (vis-001 §2.3): no window, a camera rendering to an offscreen image, and
//! a lossless readback of every frame. The camera is Phase 1's top-down orthographic one,
//! or, for a keyframed render, a perspective one set from a pose each frame (§2.11.2).
//! With buildings (vis-002), the scene adds their mesh and the sun.
//!
//! The update loop is pumped by hand. Each frame sets the vehicle boxes, schedules a
//! screenshot of the target image, and updates until that screenshot has been read back,
//! so a readback always belongs to the frame that asked for it.

use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};
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
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;

use crate::buildings::Buildings;
use crate::camera::Pose;
use crate::draw;
use crate::motion::TrackPos;
use crate::place::Placed;
use crate::scene::{self, Camera as SceneCamera, Strip};

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

pub struct Renderer {
    apps: SubApps,
    target: Handle<Image>,
    camera: SceneCamera,
    pool: Vec<Entity>,
    materials: Vec<Handle<StandardMaterial>>,
    slot: Arc<Mutex<Option<Vec<u8>>>>,
    /// The perspective camera's entity; `None` on the orthographic path.
    perspective: Option<Entity>,
    /// Metres of lift per rank.
    lift: f64,
}

impl Renderer {
    /// Build the scene: road strips in link order, and `pool` hidden boxes that frames
    /// fill in `vehicle_id` order. With `buildings`, their mesh and the sun too, and the
    /// orthographic eye rises above the tallest roof (`draw::ortho_eye`).
    pub fn new(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        buildings: Option<&Buildings>,
    ) -> Result<Self> {
        Self::build(strips, camera, pool, None, buildings)
    }

    /// The same scene through a perspective camera at `pose` (§2.11.2): shaded boxes and
    /// `draw::RANK_LIFT_3D`. `camera` is the fit, the scene's bake origin and image size.
    pub fn new_perspective(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        pose: &Pose,
        buildings: Option<&Buildings>,
    ) -> Result<Self> {
        Self::build(strips, camera, pool, Some(pose), buildings)
    }

    fn build(
        strips: &[Strip],
        camera: SceneCamera,
        pool: usize,
        pose: Option<&Pose>,
        buildings: Option<&Buildings>,
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
                match pose {
                    None => meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                    Some(_) => meshes.add(draw::shaded_box_mesh()),
                },
            )
        };

        let perspective = match pose {
            None => {
                let (eye, far) = draw::ortho_eye(buildings.map_or(0.0, |b| b.tallest));
                world.spawn((
                    draw::camera_fixed(),
                    RenderTarget::Image(target.clone().into()),
                    draw::projection(
                        (camera.width as f64 * camera.k) as f32,
                        (camera.height as f64 * camera.k) as f32,
                        far as f32,
                    ),
                    draw::look_down(0.0, 0.0, eye as f32),
                ));
                None
            }
            Some(pose) => {
                let aspect = camera.width as f64 / camera.height as f64;
                let (transform, projection) =
                    draw::perspective(pose, camera.cx, camera.cy, aspect);
                Some(
                    world
                        .spawn((
                            draw::camera_fixed(),
                            RenderTarget::Image(target.clone().into()),
                            projection,
                            transform,
                        ))
                        .id(),
                )
            }
        };
        world.spawn((
            Mesh3d(road_mesh),
            MeshMaterial3d(road_mat),
            Transform::IDENTITY,
        ));
        let pool = draw::spawn_pool(world, &box_mesh, &speed_mats[0], pool);
        // Nothing is spawned without buildings, so such a render is unchanged.
        if let Some(b) = buildings {
            draw::spawn_buildings(world, b, (camera.cx, camera.cy));
        }

        let mut r = Renderer {
            apps,
            target,
            camera,
            pool,
            materials: speed_mats,
            slot: Arc::new(Mutex::new(None)),
            perspective,
            lift: match pose {
                None => scene::RANK_LIFT,
                Some(_) => draw::RANK_LIFT_3D,
            },
        };
        // Let assets and pipelines settle before the first frame that counts.
        for _ in 0..3 {
            r.render(&[])?;
        }
        Ok(r)
    }

    pub fn camera(&self) -> &SceneCamera {
        &self.camera
    }

    /// Set the perspective camera to `pose` for the next frames; nothing on the
    /// orthographic path.
    pub fn set_pose(&mut self, pose: &Pose) {
        let Some(e) = self.perspective else { return };
        let cam = self.camera;
        let aspect = cam.width as f64 / cam.height as f64;
        let (transform, projection) = draw::perspective(pose, cam.cx, cam.cy, aspect);
        let mut ent = self.apps.main.world_mut().entity_mut(e);
        *ent.get_mut::<Transform>().unwrap() = transform;
        *ent.get_mut::<Projection>().unwrap() = projection;
    }

    /// Render one frame with exactly these boxes (in draw order) and return its RGBA8
    /// (sRGB) pixels, rows top to bottom, `width · height · 4` bytes.
    pub fn render(&mut self, boxes: &[VehicleBox]) -> Result<Vec<u8>> {
        if boxes.len() > self.pool.len() {
            bail!(
                "internal: {} vehicles in a frame, pool of {}",
                boxes.len(),
                self.pool.len()
            );
        }
        let cam = self.camera;
        let world = self.apps.main.world_mut();
        draw::fill_pool(
            world,
            &self.pool,
            &self.materials,
            boxes,
            (cam.cx, cam.cy),
            self.lift,
        );
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
