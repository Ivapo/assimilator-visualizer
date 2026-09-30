//! Headless Bevy (vis-001 §2.3): no window, a top-down orthographic camera rendering to
//! an offscreen image, and a lossless readback of every frame.
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
}

impl Renderer {
    /// Build the scene: road strips in link order, and `pool` hidden boxes that frames
    /// fill in `vehicle_id` order.
    pub fn new(strips: &[Strip], camera: SceneCamera, pool: usize) -> Result<Self> {
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
                meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            )
        };

        world.spawn((
            draw::camera_fixed(),
            RenderTarget::Image(target.clone().into()),
            draw::projection(
                (camera.width as f64 * camera.k) as f32,
                (camera.height as f64 * camera.k) as f32,
            ),
            draw::look_down(0.0, 0.0),
        ));
        world.spawn((
            Mesh3d(road_mesh),
            MeshMaterial3d(road_mat),
            Transform::IDENTITY,
        ));
        let pool = draw::spawn_pool(world, &box_mesh, &speed_mats[0], pool);

        let mut r = Renderer {
            apps,
            target,
            camera,
            pool,
            materials: speed_mats,
            slot: Arc::new(Mutex::new(None)),
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
            scene::RANK_LIFT,
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
