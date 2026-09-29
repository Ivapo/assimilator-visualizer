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
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::render_resource::{
    Extent3d, PollType, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::view::Msaa;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};

use crate::place::Placed;
use crate::scene::{self, Camera as SceneCamera, Strip};

/// One box to draw.
#[derive(Debug, Clone, Copy)]
pub struct VehicleBox {
    pub at: Placed,
    pub length: f64,
    pub speed: f64,
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

fn srgb(c: [u8; 3]) -> Color {
    Color::srgb_u8(c[0], c[1], c[2])
}

fn unlit(materials: &mut Assets<StandardMaterial>, c: [u8; 3]) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: srgb(c),
        unlit: true,
        ..default()
    })
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

        let (road_mat, speed_mats) = {
            let mut mats = world.resource_mut::<Assets<StandardMaterial>>();
            // No culling: a strip's winding depends on its direction of travel.
            let road = mats.add(StandardMaterial {
                base_color: srgb(scene::ROAD),
                unlit: true,
                cull_mode: None,
                ..default()
            });
            let speed: Vec<_> = scene::SPEED_BINS
                .iter()
                .map(|(_, c)| unlit(&mut mats, *c))
                .collect();
            (road, speed)
        };
        let (road_mesh, box_mesh) = {
            let mut meshes = world.resource_mut::<Assets<Mesh>>();
            (
                meshes.add(road_mesh(strips, &camera)),
                meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            )
        };

        world.spawn((
            Camera3d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(srgb(scene::BACKGROUND)),
                ..default()
            },
            RenderTarget::Image(target.clone().into()),
            Projection::from(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: (camera.width as f64 * camera.k) as f32,
                    height: (camera.height as f64 * camera.k) as f32,
                },
                near: 0.0,
                far: 1000.0,
                ..OrthographicProjection::default_3d()
            }),
            Tonemapping::None,
            DebandDither::Disabled,
            Msaa::Sample4,
            // Straight down, north (−Z) at the top of the image.
            Transform::from_xyz(0.0, 500.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z),
        ));
        world.spawn((
            Mesh3d(road_mesh),
            MeshMaterial3d(road_mat),
            Transform::IDENTITY,
        ));
        let pool: Vec<Entity> = (0..pool)
            .map(|_| {
                world
                    .spawn((
                        Mesh3d(box_mesh.clone()),
                        MeshMaterial3d(speed_mats[0].clone()),
                        Transform::IDENTITY,
                        Visibility::Hidden,
                    ))
                    .id()
            })
            .collect();

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
        for (i, &e) in self.pool.iter().enumerate() {
            let mut ent = world.entity_mut(e);
            match boxes.get(i) {
                Some(b) => {
                    let x = (b.at.x - cam.cx) as f32;
                    let z = -(b.at.y - cam.cy) as f32;
                    // Local +X along the heading: rotate by 90° − heading about +Y.
                    let yaw = (90.0 - b.at.heading).to_radians() as f32;
                    *ent.get_mut::<Transform>().unwrap() = Transform {
                        translation: Vec3::new(x, (scene::BOX_HEIGHT / 2.0) as f32 + 0.05, z),
                        rotation: Quat::from_rotation_y(yaw),
                        scale: Vec3::new(
                            b.length as f32,
                            scene::BOX_HEIGHT as f32,
                            scene::BOX_WIDTH as f32,
                        ),
                    };
                    ent.get_mut::<MeshMaterial3d<StandardMaterial>>().unwrap().0 =
                        self.materials[scene::speed_bin(b.speed)].clone();
                    *ent.get_mut::<Visibility>().unwrap() = Visibility::Visible;
                }
                None => {
                    let mut v = ent.get_mut::<Visibility>().unwrap();
                    if *v != Visibility::Hidden {
                        *v = Visibility::Hidden;
                    }
                }
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

/// All strips as one flat mesh at height 0, in link order.
fn road_mesh(strips: &[Strip], cam: &SceneCamera) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    let to = |p: [f64; 2]| [(p[0] - cam.cx) as f32, 0.0, -(p[1] - cam.cy) as f32];
    for s in strips {
        let base = pos.len() as u32;
        for (l, r) in s.left.iter().zip(&s.right) {
            pos.push(to(*l));
            pos.push(to(*r));
        }
        for i in 0..(s.left.len() as u32 - 1) {
            let (l0, r0, l1, r1) = (
                base + 2 * i,
                base + 2 * i + 1,
                base + 2 * i + 2,
                base + 2 * i + 3,
            );
            idx.extend_from_slice(&[l0, r0, l1, r0, r1, l1]);
        }
    }
    let n = pos.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
    .with_inserted_indices(Indices::U32(idx))
}
