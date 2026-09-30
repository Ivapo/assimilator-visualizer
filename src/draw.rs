//! What `render` and `view` draw the same way (vis-001 §2.9.6): the road mesh, the box
//! pool's materials and transforms, and the camera's fixed parts. Moved from
//! `src/render.rs`, so `render`'s frames do not change.

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::view::Msaa;

use crate::render::VehicleBox;
use crate::scene::{self, Strip};

pub fn srgb(c: [u8; 3]) -> Color {
    Color::srgb_u8(c[0], c[1], c[2])
}

fn unlit(materials: &mut Assets<StandardMaterial>, c: [u8; 3]) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: srgb(c),
        unlit: true,
        ..default()
    })
}

/// The road material, and one material per speed bin (`scene::SPEED_BINS`).
pub fn materials(
    mats: &mut Assets<StandardMaterial>,
) -> (Handle<StandardMaterial>, Vec<Handle<StandardMaterial>>) {
    // No culling: a strip's winding depends on its direction of travel.
    let road = mats.add(StandardMaterial {
        base_color: srgb(scene::ROAD),
        unlit: true,
        cull_mode: None,
        ..default()
    });
    let speed: Vec<_> = scene::SPEED_BINS
        .iter()
        .map(|(_, c)| unlit(mats, *c))
        .collect();
    (road, speed)
}

/// The camera's fixed parts: straight down, north (−Z) at the top, MSAA ×4, no
/// tonemapping or dither, the background as clear colour. The caller adds the target and
/// the projection.
pub fn camera_fixed() -> impl Bundle {
    (
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(srgb(scene::BACKGROUND)),
            ..default()
        },
        Tonemapping::None,
        DebandDither::Disabled,
        Msaa::Sample4,
    )
}

/// Straight down from `(x, 500, z)`, north (−Z) at the top of the image.
pub fn look_down(x: f32, z: f32) -> Transform {
    Transform::from_xyz(x, 500.0, z).looking_at(Vec3::new(x, 0.0, z), Vec3::NEG_Z)
}

/// The ortho projection showing `width × height` metres.
pub fn projection(width: f32, height: f32) -> Projection {
    Projection::from(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed { width, height },
        near: 0.0,
        far: 1000.0,
        ..OrthographicProjection::default_3d()
    })
}

/// Box `b` at rank `i` in `vehicle_id` order, with the scene baked relative to
/// `(cx, cy)`.
pub fn box_transform(b: &VehicleBox, i: usize, cx: f64, cy: f64) -> Transform {
    let x = (b.at.x - cx) as f32;
    let z = -(b.at.y - cy) as f32;
    // Local +X along the heading: rotate by 90° − heading about +Y.
    let yaw = (90.0 - b.at.heading).to_radians() as f32;
    Transform {
        // Rank `i` in vehicle_id order sets the depth order (scene::RANK_LIFT).
        translation: Vec3::new(
            x,
            (scene::BOX_HEIGHT / 2.0 + 0.05 + i as f64 * scene::RANK_LIFT) as f32,
            z,
        ),
        rotation: Quat::from_rotation_y(yaw),
        scale: Vec3::new(
            b.length as f32,
            scene::BOX_HEIGHT as f32,
            scene::BOX_WIDTH as f32,
        ),
    }
}

/// Fill the pool with `boxes` in draw order and hide the rest.
pub fn fill_pool(
    world: &mut World,
    pool: &[Entity],
    materials: &[Handle<StandardMaterial>],
    boxes: &[VehicleBox],
    cx: f64,
    cy: f64,
) {
    for (i, &e) in pool.iter().enumerate() {
        let mut ent = world.entity_mut(e);
        match boxes.get(i) {
            Some(b) => {
                *ent.get_mut::<Transform>().unwrap() = box_transform(b, i, cx, cy);
                ent.get_mut::<MeshMaterial3d<StandardMaterial>>().unwrap().0 =
                    materials[scene::speed_bin(b.speed)].clone();
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
}

/// Spawn `n` hidden unit cuboids for [`fill_pool`].
pub fn spawn_pool(
    world: &mut World,
    box_mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    n: usize,
) -> Vec<Entity> {
    (0..n)
        .map(|_| {
            world
                .spawn((
                    Mesh3d(box_mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    Visibility::Hidden,
                ))
                .id()
        })
        .collect()
}

/// All strips as one flat mesh at height 0, in link order, relative to `(cx, cy)`.
pub fn road_mesh(strips: &[Strip], cx: f64, cy: f64) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();
    let to = |p: [f64; 2]| [(p[0] - cx) as f32, 0.0, -(p[1] - cy) as f32];
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
