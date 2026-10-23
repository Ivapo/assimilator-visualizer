//! What `render` and `view` draw the same way (vis-001 §2.9.6): the road mesh, the box
//! pool's materials and transforms, and the camera's fixed parts. Moved from
//! `src/render.rs`, so `render`'s frames do not change. The perspective path (§2.11.2)
//! adds a camera built from a pose, a shaded box mesh and a smaller rank lift, and since
//! Phase 6 (§2.12) draws every box of a frame as one mesh in `vehicle_id` order; the pool
//! is the orthographic path's. vis-002 adds the buildings: one lit mesh, a sun and the
//! orthographic eye's rule, and, for `render` only, the credit line's UI tree (§2.14.5).
//! See-through (vis-002 §2.15.5) rebuilds the buildings' mesh with lowered heights.
//! Streets (vis-002 §2.17.11) add two meshes: the junction surfaces and median noses,
//! and the markings, recoloured as the fade follows the camera. Signals (vis-003 §2.12)
//! add one more: every dot, rebuilt each frame. The HUD (vis-003 §2.20.9) is a UI tree
//! drawn as the credit line's, in the top-left corner, in `render` and `view`. Link
//! colours (vis-003 §2.21) add the tint, one mesh over the road strips replaced when a
//! colour changes, a legend under the HUD drawn as its line, and the boxes in one colour.

use std::collections::HashSet;

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::view::Msaa;

use crate::buildings::{Buildings, MeshData};
use crate::camera::{self, FAR_PER_D, FOV_DEG, Pose};
use crate::credit;
use crate::hud;
use crate::links::{self, Legend, TintData};
use crate::render::VehicleBox;
use crate::scene::{self, Strip};
use crate::signals::DotsData;
use crate::streets::{self, MarkingsData, Streets};

/// The rank lift of the perspective path, metres (§2.11.2): 0.01 m per rank would float
/// a 110th box 1.09 m over the road at a tilt. The orthographic path keeps
/// `scene::RANK_LIFT`.
pub const RANK_LIFT_3D: f64 = 0.001;

/// The shaded box's faces, as fractions of the speed colour (linear, multiplied in by the
/// vertex colour): the top keeps it; the long sides, the ends and the bottom are darker.
pub const SHADE_TOP: f32 = 1.0;
pub const SHADE_SIDE: f32 = 0.55;
pub const SHADE_END: f32 = 0.4;
pub const SHADE_BOTTOM: f32 = 0.25;

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

/// A box's colour by its speed: its bin's (`scene::SPEED_BINS`).
pub fn speed_colour(speed: f64) -> [u8; 3] {
    scene::SPEED_BINS[scene::speed_bin(speed)].1
}

/// A box's colour while link colours are on, whatever its speed (vis-003 OQ-8).
pub fn cars_colour(_speed: f64) -> [u8; 3] {
    links::CARS
}

/// The one material of every pooled box while link colours are on.
pub fn cars_material(mats: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    unlit(mats, links::CARS)
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

/// Straight down from `(x, eye, z)`, north (−Z) at the top of the image; `eye` is
/// [`ortho_eye`]'s.
pub fn look_down(x: f32, z: f32, eye: f32) -> Transform {
    Transform::from_xyz(x, eye, z).looking_at(Vec3::new(x, 0.0, z), Vec3::NEG_Z)
}

/// The ortho projection showing `width × height` metres, to `far` ([`ortho_eye`]'s).
pub fn projection(width: f32, height: f32, far: f32) -> Projection {
    Projection::from(OrthographicProjection {
        scaling_mode: bevy::camera::ScalingMode::Fixed { width, height },
        near: 0.0,
        far,
        ..OrthographicProjection::default_3d()
    })
}

/// The perspective camera at `pose`, in the scene baked relative to `(fx, fy)`: world
/// `(x, y, z)` is Bevy `(x − fx, z, −(y − fy))`. A `Transform` at the eye looking at the
/// look-at point with the image's up, and a projection of `φ` with `far = 20·d`.
pub fn perspective(pose: &Pose, fx: f64, fy: f64, aspect: f64) -> (Transform, Projection) {
    let to = |p: [f64; 3]| Vec3::new((p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32);
    let (_, u, _) = pose.axes();
    let eye = to(pose.eye());
    let target = to([pose.cx, pose.cy, 0.0]);
    let up = Vec3::new(u[0] as f32, u[2] as f32, -u[1] as f32);
    let d = camera::distance(pose.height_m);
    (
        Transform::from_translation(eye).looking_at(target, up),
        Projection::from(PerspectiveProjection {
            fov: FOV_DEG.to_radians() as f32,
            aspect_ratio: aspect as f32,
            far: (FAR_PER_D * d) as f32,
            ..default()
        }),
    )
}

/// A unit cuboid whose faces are shaded by vertex colour (§2.11.2), so a tilted box reads
/// as a solid under the unlit speed materials.
pub fn shaded_box_mesh() -> Mesh {
    let mut mesh = Mesh::from(Cuboid::new(1.0, 1.0, 1.0));
    let normals: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(bevy::mesh::VertexAttributeValues::Float32x3(n)) => n.clone(),
        _ => unreachable!("a Cuboid mesh has normals"),
    };
    // Local +X is the length, +Y up, +Z the width (`box_transform`).
    let colours: Vec<[f32; 4]> = normals
        .iter()
        .map(|n| {
            let f = if n[1] > 0.5 {
                SHADE_TOP
            } else if n[1] < -0.5 {
                SHADE_BOTTOM
            } else if n[2].abs() > 0.5 {
                SHADE_SIDE
            } else {
                SHADE_END
            };
            [f, f, f, 1.0]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    mesh
}

/// Box `b` at rank `i` in `vehicle_id` order, lifted `lift` metres per rank, with the
/// scene baked relative to `(cx, cy)`.
pub fn box_transform(b: &VehicleBox, i: usize, cx: f64, cy: f64, lift: f64) -> Transform {
    let x = (b.at.x - cx) as f32;
    let z = -(b.at.y - cy) as f32;
    // Local +X along the heading: rotate by 90° − heading about +Y.
    let yaw = (90.0 - b.at.heading).to_radians() as f32;
    Transform {
        // Rank `i` in vehicle_id order sets the depth order (scene::RANK_LIFT).
        translation: Vec3::new(
            x,
            (scene::BOX_HEIGHT / 2.0 + 0.05 + i as f64 * lift) as f32,
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

/// Fill the pool with `boxes` in draw order, `lift` metres per rank, and hide the rest.
pub fn fill_pool(
    world: &mut World,
    pool: &[Entity],
    materials: &[Handle<StandardMaterial>],
    boxes: &[VehicleBox],
    (cx, cy): (f64, f64),
    lift: f64,
) {
    for (i, &e) in pool.iter().enumerate() {
        let mut ent = world.entity_mut(e);
        match boxes.get(i) {
            Some(b) => {
                *ent.get_mut::<Transform>().unwrap() = box_transform(b, i, cx, cy, lift);
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

/// Every box of `boxes` as one mesh, in slice (`vehicle_id`) order (vis-001 §2.12.2): the
/// shaded cuboid's vertices through [`box_transform`] at rank = slice index, coloured by
/// `colour` of its speed ([`speed_colour`], or [`cars_colour`]) (linear) × the face's
/// shade. One draw keeps its triangles' order, so
/// a depth tie goes to the later box, the higher id. A box whose placed point, heading and
/// length equal a later box's bit for bit is left out; ranks are counted first, so no
/// other box moves. `None` when no box is drawn.
pub fn boxes_mesh(
    boxes: &[VehicleBox],
    (cx, cy): (f64, f64),
    lift: f64,
    colour: fn(f64) -> [u8; 3],
) -> Option<Mesh> {
    // The last box is always drawn, so only an empty slice draws nothing.
    if boxes.is_empty() {
        return None;
    }
    let mut seen = HashSet::new();
    let mut drawn = vec![false; boxes.len()];
    for (i, b) in boxes.iter().enumerate().rev() {
        let key = (
            b.at.x.to_bits(),
            b.at.y.to_bits(),
            b.at.heading.to_bits(),
            b.length.to_bits(),
        );
        drawn[i] = seen.insert(key);
    }
    let unit = shaded_box_mesh();
    let vec3s = |attr| match unit.attribute(attr) {
        Some(bevy::mesh::VertexAttributeValues::Float32x3(v)) => v.clone(),
        _ => unreachable!("the shaded box has positions and normals"),
    };
    let (unit_pos, unit_normal) = (
        vec3s(Mesh::ATTRIBUTE_POSITION),
        vec3s(Mesh::ATTRIBUTE_NORMAL),
    );
    let shades: Vec<f32> = match unit.attribute(Mesh::ATTRIBUTE_COLOR) {
        Some(bevy::mesh::VertexAttributeValues::Float32x4(v)) => v.iter().map(|c| c[0]).collect(),
        _ => unreachable!("the shaded box has vertex colours"),
    };
    let unit_idx: Vec<u32> = match unit.indices() {
        Some(Indices::U32(v)) => v.clone(),
        Some(Indices::U16(v)) => v.iter().map(|&i| i as u32).collect(),
        None => unreachable!("a Cuboid mesh is indexed"),
    };
    let n = drawn.iter().filter(|&&d| d).count();
    let mut pos = Vec::with_capacity(n * unit_pos.len());
    let mut normals = Vec::with_capacity(n * unit_pos.len());
    let mut colours = Vec::with_capacity(n * unit_pos.len());
    let mut idx = Vec::with_capacity(n * unit_idx.len());
    for (i, b) in boxes.iter().enumerate().filter(|&(i, _)| drawn[i]) {
        let t = box_transform(b, i, cx, cy, lift);
        let c = srgb(colour(b.speed)).to_linear();
        let base = pos.len() as u32;
        for ((p, nrm), s) in unit_pos.iter().zip(&unit_normal).zip(&shades) {
            pos.push(t.transform_point(Vec3::from(*p)).to_array());
            normals.push((t.rotation * Vec3::from(*nrm)).to_array());
            colours.push([c.red * s, c.green * s, c.blue * s, 1.0]);
        }
        idx.extend(unit_idx.iter().map(|&k| base + k));
    }
    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_indices(Indices::U32(idx)),
    )
}

/// The perspective path's boxes: one entity, and the handle of the mesh [`set_boxes`]
/// replaces each frame.
#[derive(Debug, Clone)]
pub struct BoxesEntity {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
}

/// Spawn the boxes' entity, hidden, never culled, with one white unlit material: the
/// vertex colours carry the speed colours. Its first mesh is a placeholder, never shown.
pub fn spawn_boxes(world: &mut World) -> BoxesEntity {
    let placeholder = VehicleBox {
        vehicle_id: 0,
        at: crate::place::Placed {
            x: 0.0,
            y: 0.0,
            heading: 0.0,
        },
        length: 1.0,
        speed: 0.0,
        track: crate::motion::TrackPos {
            piece: crate::motion::Piece::Link(0),
            along: 0.0,
            lateral: 0.0,
            odo: 0.0,
        },
    };
    let mesh = boxes_mesh(&[placeholder], (0.0, 0.0), 0.0, speed_colour).expect("one box is drawn");
    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        });
    let entity = world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            Visibility::Hidden,
            NoFrustumCulling,
        ))
        .id();
    BoxesEntity { entity, mesh }
}

/// Draw `boxes` as [`boxes_mesh`]'s one mesh, `lift` metres per rank, coloured by
/// `colour`, in place of the last frame's; hide the entity when no box is drawn.
pub fn set_boxes(
    world: &mut World,
    entity: &BoxesEntity,
    boxes: &[VehicleBox],
    (cx, cy): (f64, f64),
    lift: f64,
    colour: fn(f64) -> [u8; 3],
) {
    let want = match boxes_mesh(boxes, (cx, cy), lift, colour) {
        Some(mesh) => {
            world
                .resource_mut::<Assets<Mesh>>()
                .insert(entity.mesh.id(), mesh)
                .expect("the boxes' mesh handle is held");
            Visibility::Visible
        }
        None => Visibility::Hidden,
    };
    let mut v = world.get_mut::<Visibility>(entity.entity).unwrap();
    if *v != want {
        *v = want;
    }
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

/// The orthographic eye's height without buildings, metres; its depth range is the eye's
/// height + [`ORTHO_DEPTH_BELOW_M`].
pub const ORTHO_EYE_M: f64 = 500.0;
pub const ORTHO_DEPTH_BELOW_M: f64 = 500.0;

/// The orthographic eye's height and far plane for a scene whose tallest roof is
/// `tallest_m` (vis-002 §2.8): the eye rises only for a roof that would reach it, to
/// `max(500, tallest + 10)` m, and far is the eye's height + 500 m. A job without
/// buildings passes 0 and gets `(500, 1000)`.
pub fn ortho_eye(tallest_m: f64) -> (f64, f64) {
    let eye = ORTHO_EYE_M.max(tallest_m + 10.0);
    (eye, eye + ORTHO_DEPTH_BELOW_M)
}

/// The sun (vis-002 §2.7): where the light comes from, clockwise from north, and its
/// elevation, degrees. Above 45°, every roof is lighter than every wall.
pub const SUN_AZIMUTH_DEG: f64 = 210.0;
pub const SUN_ELEVATION_DEG: f64 = 60.0;
/// The sun's illuminance, lux, and the ambient light's brightness: low enough that a
/// roof stays below full white (the camera has no tonemapping), the ambient weaker than
/// the sun.
pub const SUN_LUX: f32 = 4000.0;
pub const AMBIENT_BRIGHTNESS: f32 = 250.0;
/// The buildings' one material: neutral grey (sRGB), so a building pixel is never a road
/// or background pixel.
pub const BUILDING_GREY: [u8; 3] = [188, 188, 188];

/// The direction the sun's light travels, in Bevy coordinates: world
/// `(−cos e·sin a, −cos e·cos a, −sin e)` is Bevy `(−cos e·sin a, −sin e, cos e·cos a)`.
pub fn sun_direction(azimuth_deg: f64, elevation_deg: f64) -> Vec3 {
    let (a, e) = (azimuth_deg.to_radians(), elevation_deg.to_radians());
    Vec3::new(
        (-e.cos() * a.sin()) as f32,
        (-e.sin()) as f32,
        (e.cos() * a.cos()) as f32,
    )
}

/// The buildings' mesh data as one Bevy mesh, baked relative to `(fx, fy)` like the
/// roads: world `(x, y, z)` is Bevy `(x − fx, z, −(y − fy))`, converted to `f32` here.
pub fn buildings_mesh(data: &MeshData, fx: f64, fy: f64) -> Mesh {
    let pos: Vec<[f32; 3]> = data
        .positions
        .iter()
        .map(|p| [(p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32])
        .collect();
    let normals: Vec<[f32; 3]> = data
        .normals
        .iter()
        .map(|n| [n[0] as f32, n[2] as f32, -n[1] as f32])
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(data.indices.clone()))
}

/// What [`spawn_buildings`] spawned, for `view`'s `B`.
#[derive(Debug, Clone, Copy)]
pub struct BuildingEntities {
    pub mesh: Entity,
    pub sun: Entity,
}

/// Spawn the buildings as one lit, never-culled mesh, the sun and the ambient light
/// (vis-002 §2.7). Called only with buildings: without them nothing is spawned.
pub fn spawn_buildings(
    world: &mut World,
    buildings: &Buildings,
    (fx, fy): (f64, f64),
) -> BuildingEntities {
    let data = crate::buildings::mesh_data(buildings);
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(buildings_mesh(&data, fx, fy));
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: srgb(BUILDING_GREY),
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            metallic: 0.0,
            ..default()
        });
    let mesh = world
        .spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            // One mesh, always submitted: no tower drops out of a low, zoomed shot.
            NoFrustumCulling,
        ))
        .id();
    let sun = world
        .spawn((
            DirectionalLight {
                illuminance: SUN_LUX,
                shadow_maps_enabled: false,
                contact_shadows_enabled: false,
                ..default()
            },
            Transform::IDENTITY
                .looking_to(sun_direction(SUN_AZIMUTH_DEG, SUN_ELEVATION_DEG), Vec3::Y),
        ))
        .id();
    world.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });
    BuildingEntities { mesh, sun }
}

/// The buildings' mesh kept for see-through (vis-002 §2.15.5): [`crate::buildings::mesh_data`]'s
/// positions in `f64` and each vertex's building, so the mesh can be rebuilt with any
/// building's height lowered. Normals and indices never change.
pub struct BuildingsCut {
    positions: Vec<[f64; 3]>,
    owner: Vec<u32>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    fx: f64,
    fy: f64,
}

impl BuildingsCut {
    /// Each building's `building_mesh` in file order, appended as `mesh_data` appends
    /// them, baked relative to `(fx, fy)` as [`buildings_mesh`] bakes them.
    pub fn new(buildings: &Buildings, (fx, fy): (f64, f64)) -> BuildingsCut {
        let mut cut = BuildingsCut {
            positions: Vec::new(),
            owner: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            fx,
            fy,
        };
        for (i, b) in buildings.buildings.iter().enumerate() {
            let m = crate::buildings::building_mesh(b);
            let base = cut.positions.len() as u32;
            cut.owner
                .extend(std::iter::repeat_n(i as u32, m.positions.len()));
            cut.positions.extend(m.positions);
            cut.normals.extend(
                m.normals
                    .iter()
                    .map(|n| [n[0] as f32, n[2] as f32, -n[1] as f32]),
            );
            cut.indices.extend(m.indices.iter().map(|k| base + k));
        }
        cut
    }

    /// The mesh with every vertex of building `b` at `z′ = min(z, heights[b])`. With
    /// every building's own height, it equals [`buildings_mesh`] of the same data.
    pub fn mesh(&self, heights: &[f64]) -> Mesh {
        let pos: Vec<[f32; 3]> = self
            .positions
            .iter()
            .zip(&self.owner)
            .map(|(p, &b)| {
                [
                    (p[0] - self.fx) as f32,
                    p[2].min(heights[b as usize]) as f32,
                    -(p[1] - self.fy) as f32,
                ]
            })
            .collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()))
    }
}

/// Draw the buildings at `heights` (see-through, vis-002 §2.15.5): [`BuildingsCut::mesh`]
/// in place of the asset behind `mesh`'s `Mesh3d`, the entity [`spawn_buildings`] spawned.
pub fn set_building_heights(world: &mut World, mesh: Entity, cut: &BuildingsCut, heights: &[f64]) {
    let handle = world
        .get::<Mesh3d>(mesh)
        .expect("the buildings' entity has a mesh")
        .0
        .id();
    world
        .resource_mut::<Assets<Mesh>>()
        .insert(handle, cut.mesh(heights))
        .expect("the buildings' mesh handle is held");
}

/// The credit line's font (vis-002 §2.14.6): Fira Sans Medium 4.203, under the SIL Open
/// Font License 1.1 (`assets/fonts/OFL.txt`). Embedded, so `render` reads no font file.
pub const CREDIT_FONT: &[u8] = include_bytes!("../assets/fonts/FiraSans-Medium.ttf");

/// What [`spawn_credit`] spawned: the root, the fill text and its eight outline copies.
#[derive(Debug, Clone, Copy)]
pub struct CreditNodes {
    pub root: Entity,
    pub fill: Entity,
    pub outline: [Entity; 8],
}

/// One copy of the line: never wrapped, so the fill's width is the whole line's.
fn credit_text(line: &str, font: &Handle<Font>, size: u32, color: [u8; 3]) -> impl Bundle {
    (
        Text::new(line),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size as f32),
            ..default()
        },
        TextColor(srgb(color)),
        TextLayout::no_wrap(),
    )
}

/// Spawn the credit line's UI tree (vis-002 §2.14.5), drawn by `camera` into its image:
/// a root `margin` px from the right and bottom edges, the fill text in its flow, and
/// eight copies in the outline colour beside it, each `offset` px away in one of eight
/// directions. The copies are the fill's siblings at `ZIndex(-1)`, so they draw under it
/// (`ZIndex` orders siblings only). Called only with a line: without one nothing is
/// spawned, so such a render is unchanged.
pub fn spawn_credit(
    world: &mut World,
    line: &str,
    camera: Entity,
    font: Handle<Font>,
    size: u32,
    margin: u32,
    offset: u32,
) -> CreditNodes {
    let root = world
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(margin as f32),
                bottom: Val::Px(margin as f32),
                ..default()
            },
            UiTargetCamera(camera),
        ))
        .id();
    let fill = world
        .spawn((credit_text(line, &font, size, credit::FILL), ChildOf(root)))
        .id();
    let o = offset as f32;
    let mut outline = [Entity::PLACEHOLDER; 8];
    let offsets = [-o, 0.0, o]
        .into_iter()
        .flat_map(|dy| [-o, 0.0, o].into_iter().map(move |dx| (dx, dy)))
        .filter(|&d| d != (0.0, 0.0));
    for (slot, (dx, dy)) in outline.iter_mut().zip(offsets) {
        *slot = world
            .spawn((
                credit_text(line, &font, size, credit::OUTLINE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(dx),
                    top: Val::Px(dy),
                    ..default()
                },
                ZIndex(-1),
                ChildOf(root),
            ))
            .id();
    }
    CreditNodes {
        root,
        fill,
        outline,
    }
}

/// Set the font size of all nine of the line's texts, for the fit (§2.14.5).
pub fn set_credit_size(world: &mut World, nodes: &CreditNodes, size: u32) {
    for e in std::iter::once(nodes.fill).chain(nodes.outline) {
        world.get_mut::<TextFont>(e).unwrap().font_size = FontSize::Px(size as f32);
    }
}

/// The streets as drawn (vis-002 §2.17.11): the surface's and the markings' entities,
/// each spawned only if its mesh has a triangle, and what recolouring the markings needs.
pub struct StreetsDrawn {
    pub surface: Option<Entity>,
    pub markings: Option<Entity>,
    /// The markings' mesh, replaced by [`set_street_colours`].
    mesh: Option<Handle<Mesh>>,
    /// The markings' data in `f64` world metres, and the fit's centre it is baked about.
    data: MarkingsData,
    at: (f64, f64),
    /// The colours the markings' mesh is drawn with now.
    drawn: Vec<[f32; 4]>,
}

impl StreetsDrawn {
    /// Forget the colours drawn, so the next [`set_street_colours`] replaces the mesh:
    /// `view --bench` does it every frame.
    pub fn force(&mut self) {
        self.drawn.clear();
    }

    /// Every entity spawned.
    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.surface.into_iter().chain(self.markings)
    }
}

fn markings_mesh(d: &MarkingsData, colours: Vec<[f32; 4]>, (fx, fy): (f64, f64)) -> Mesh {
    let pos: Vec<[f32; 3]> = d
        .positions
        .iter()
        .map(|p| [(p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32])
        .collect();
    let n = pos.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(d.indices.clone()))
}

/// Spawn the streets (vis-002 §2.17.11): the junction surfaces and median noses with the
/// road's material, and the markings with per-vertex colours faded at `mpp` (metres per
/// pixel at a world point) on one unlit white material that does not cull, never
/// frustum-culled. A mesh with no triangle is not spawned (§2.17.12).
pub fn spawn_streets(
    world: &mut World,
    streets: &Streets,
    road: &Handle<StandardMaterial>,
    (fx, fy): (f64, f64),
    mpp: impl Fn([f64; 3]) -> f64,
) -> StreetsDrawn {
    let surface = streets.surface();
    let surface = (!surface.indices.is_empty()).then(|| {
        let pos: Vec<[f32; 3]> = surface
            .positions
            .iter()
            .map(|p| [(p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32])
            .collect();
        let n = pos.len();
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
        .with_inserted_indices(Indices::U32(surface.indices));
        let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);
        world
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(road.clone()),
                Transform::IDENTITY,
            ))
            .id()
    });
    let data = streets.markings();
    let mut drawn = StreetsDrawn {
        surface,
        markings: None,
        mesh: None,
        drawn: Vec::new(),
        data,
        at: (fx, fy),
    };
    if !drawn.data.indices.is_empty() {
        let colours = streets::vertex_colours(
            &drawn.data.positions,
            &drawn.data.colours,
            &drawn.data.widths,
            mpp,
        );
        let mesh = world.resource_mut::<Assets<Mesh>>().add(markings_mesh(
            &drawn.data,
            colours.clone(),
            drawn.at,
        ));
        let material = world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                cull_mode: None,
                ..default()
            });
        drawn.markings = Some(
            world
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::IDENTITY,
                    NoFrustumCulling,
                ))
                .id(),
        );
        drawn.mesh = Some(mesh);
        drawn.drawn = colours;
    }
    drawn
}

/// Recolour the markings for `mpp` (vis-002 §2.17.10): the mesh is replaced, with the
/// same positions and indices, only when a colour differs from the last drawn.
pub fn set_street_colours(world: &mut World, d: &mut StreetsDrawn, mpp: impl Fn([f64; 3]) -> f64) {
    let Some(handle) = d.mesh.as_ref() else {
        return;
    };
    let colours = streets::vertex_colours(&d.data.positions, &d.data.colours, &d.data.widths, mpp);
    if colours == d.drawn {
        return;
    }
    let mesh = markings_mesh(&d.data, colours.clone(), d.at);
    world
        .resource_mut::<Assets<Mesh>>()
        .insert(handle.id(), mesh)
        .expect("the markings' mesh handle is held");
    d.drawn = colours;
}

/// Despawn the streets.
pub fn despawn_streets(world: &mut World, d: StreetsDrawn) {
    for e in d.entities() {
        world.despawn(e);
    }
}

/// The signals' dots: one entity, and the handle of the mesh [`set_dots`] replaces each
/// frame.
#[derive(Debug, Clone)]
pub struct DotsEntity {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
}

fn dots_mesh(d: &DotsData, (fx, fy): (f64, f64)) -> Mesh {
    let pos: Vec<[f32; 3]> = d
        .positions
        .iter()
        .map(|p| [(p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32])
        .collect();
    let n = pos.len();
    let colours: Vec<[f32; 4]> = d.colours.iter().map(|c| streets::colour(*c, 1.0)).collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(d.indices.clone()))
}

/// Spawn the dots' entity (vis-003 §2.12), hidden, never culled, with one white unlit
/// material that does not cull: the vertex colours carry the colours, linear from sRGB as
/// the markings'. Its first mesh is a placeholder, never shown.
pub fn spawn_dots(world: &mut World) -> DotsEntity {
    let placeholder = DotsData {
        positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        colours: vec![[0, 0, 0]; 3],
        indices: vec![0, 1, 2],
    };
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(dots_mesh(&placeholder, (0.0, 0.0)));
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            cull_mode: None,
            ..default()
        });
    let entity = world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            Visibility::Hidden,
            NoFrustumCulling,
        ))
        .id();
    DotsEntity { entity, mesh }
}

/// Draw `dots` about the bake origin `(fx, fy)` in place of the last frame's; hide the
/// entity when there is no triangle.
pub fn set_dots(world: &mut World, entity: &DotsEntity, dots: &DotsData, at: (f64, f64)) {
    let want = if dots.indices.is_empty() {
        Visibility::Hidden
    } else {
        world
            .resource_mut::<Assets<Mesh>>()
            .insert(entity.mesh.id(), dots_mesh(dots, at))
            .expect("the dots' mesh handle is held");
        Visibility::Visible
    };
    let mut v = world.get_mut::<Visibility>(entity.entity).unwrap();
    if *v != want {
        *v = want;
    }
}

/// Despawn the dots.
pub fn despawn_dots(world: &mut World, d: DotsEntity) {
    world.despawn(d.entity);
}

/// What [`spawn_hud`] spawned: the root, the fill text and its eight outline copies.
#[derive(Debug, Clone, Copy)]
pub struct HudNodes {
    pub root: Entity,
    pub fill: Entity,
    pub outline: [Entity; 8],
}

/// One copy of the HUD's line: never wrapped, with tabular figures, so a digit that
/// changes moves nothing after it (vis-003 §2.20.8).
fn hud_text(font: &Handle<Font>, size: u32, color: [u8; 3]) -> impl Bundle {
    (
        Text::new(hud::GLYPHS),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size as f32),
            font_features: [bevy::text::FontFeatureTag::TABULAR_FIGURES].into(),
            ..default()
        },
        TextColor(srgb(color)),
        TextLayout::no_wrap(),
    )
}

/// The eight outline copies' offsets from the fill, `o` px, in spawn order.
fn hud_offsets(size: u32) -> impl Iterator<Item = (f32, f32)> {
    let o = hud::outline_offset(size) as f32;
    [-o, 0.0, o]
        .into_iter()
        .flat_map(move |dy| [-o, 0.0, o].into_iter().map(move |dx| (dx, dy)))
        .filter(|&d| d != (0.0, 0.0))
}

/// Spawn the HUD's UI tree (vis-003 §2.20.9), drawn as the credit line is: a root
/// `hud::margin(size)` px from the top and left edges, the fill text in its flow, and
/// eight copies in the outline colour at `ZIndex(-1)`, each `hud::outline_offset(size)`
/// px away in one of eight directions. With `camera`, drawn by it into its image
/// (`render`); without, into the window (`view`). Every text holds `hud::GLYPHS` until
/// [`set_hud_text`], so the first layout meets every glyph the HUD can draw.
pub fn spawn_hud(
    world: &mut World,
    camera: Option<Entity>,
    font: Handle<Font>,
    size: u32,
) -> HudNodes {
    let m = hud::margin(size) as f32;
    let mut root = world.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(m),
        top: Val::Px(m),
        ..default()
    });
    if let Some(c) = camera {
        root.insert(UiTargetCamera(c));
    }
    let root = root.id();
    let fill = world
        .spawn((hud_text(&font, size, hud::FILL), ChildOf(root)))
        .id();
    let mut outline = [Entity::PLACEHOLDER; 8];
    for (slot, (dx, dy)) in outline.iter_mut().zip(hud_offsets(size)) {
        *slot = world
            .spawn((
                hud_text(&font, size, hud::OUTLINE),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(dx),
                    top: Val::Px(dy),
                    ..default()
                },
                ZIndex(-1),
                ChildOf(root),
            ))
            .id();
    }
    HudNodes {
        root,
        fill,
        outline,
    }
}

/// Set the HUD's line on all nine texts, each only where it differs, so an unchanged
/// line is not laid out again.
pub fn set_hud_text(world: &mut World, nodes: &HudNodes, line: &str) {
    for e in std::iter::once(nodes.fill).chain(nodes.outline) {
        let mut t = world.get_mut::<Text>(e).unwrap();
        if t.0 != line {
            t.0 = line.to_string();
        }
    }
}

/// Set the HUD's size: all nine texts' font size, the root's margin and the copies'
/// offsets, for `view`'s window when it is resized (§2.20.10).
pub fn set_hud_size(world: &mut World, nodes: &HudNodes, size: u32) {
    let m = hud::margin(size) as f32;
    let mut root = world.get_mut::<Node>(nodes.root).unwrap();
    root.left = Val::Px(m);
    root.top = Val::Px(m);
    for e in std::iter::once(nodes.fill).chain(nodes.outline) {
        world.get_mut::<TextFont>(e).unwrap().font_size = FontSize::Px(size as f32);
    }
    for (e, (dx, dy)) in nodes.outline.into_iter().zip(hud_offsets(size)) {
        let mut n = world.get_mut::<Node>(e).unwrap();
        n.left = Val::Px(dx);
        n.top = Val::Px(dy);
    }
}

/// The link colours' tint (vis-003 §2.21.10): one entity, the handle of the mesh
/// [`set_tint`] replaces, and the colours it was last drawn with.
#[derive(Debug, Clone)]
pub struct TintEntity {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    pub drawn: Vec<Option<[u8; 3]>>,
}

/// The triangles of every unit with a colour, in unit order, each vertex its unit's
/// colour, linear from sRGB as the markings'; `None` when no unit has one.
fn tint_mesh(d: &TintData, colours: &[Option<[u8; 3]>], (fx, fy): (f64, f64)) -> Option<Mesh> {
    let mut idx = Vec::new();
    for (t, c) in d.tris.iter().zip(colours) {
        if c.is_some() {
            idx.extend_from_slice(t);
        }
    }
    if idx.is_empty() {
        return None;
    }
    let pos: Vec<[f32; 3]> = d
        .positions
        .iter()
        .map(|p| [(p[0] - fx) as f32, p[2] as f32, -(p[1] - fy) as f32])
        .collect();
    let n = pos.len();
    let cols: Vec<[f32; 4]> = d
        .unit
        .iter()
        .map(|&u| streets::colour(colours[u as usize].unwrap_or([0, 0, 0]), 1.0))
        .collect();
    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, cols)
        .with_inserted_indices(Indices::U32(idx)),
    )
}

/// Spawn the tint's entity, hidden, never culled, with one white unlit material that does
/// not cull, as the dots'. Its first mesh is a placeholder, never shown.
pub fn spawn_tint(world: &mut World) -> TintEntity {
    let placeholder = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32, 0.0, 0.0]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32, 0.0, 0.0, 1.0]; 3])
    .with_inserted_indices(Indices::U32(vec![0, 1, 2]));
    let mesh = world.resource_mut::<Assets<Mesh>>().add(placeholder);
    let material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            cull_mode: None,
            ..default()
        });
    let entity = world
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::IDENTITY,
            Visibility::Hidden,
            NoFrustumCulling,
        ))
        .id();
    TintEntity {
        entity,
        mesh,
        drawn: Vec::new(),
    }
}

/// Draw the units with these colours about the bake origin `at`, replacing the mesh only
/// when a colour differs from the last drawn, or always when `force` (`view --bench`);
/// hide the entity when no unit has a colour.
pub fn set_tint(
    world: &mut World,
    e: &mut TintEntity,
    d: &TintData,
    colours: &[Option<[u8; 3]>],
    at: (f64, f64),
    force: bool,
) {
    if !force && e.drawn == colours {
        return;
    }
    let want = match tint_mesh(d, colours, at) {
        Some(mesh) => {
            world
                .resource_mut::<Assets<Mesh>>()
                .insert(e.mesh.id(), mesh)
                .expect("the tint's mesh handle is held");
            Visibility::Visible
        }
        None => Visibility::Hidden,
    };
    let mut v = world.get_mut::<Visibility>(e.entity).unwrap();
    if *v != want {
        *v = want;
    }
    e.drawn = colours.to_vec();
}

/// Despawn the tint.
pub fn despawn_tint(world: &mut World, e: TintEntity) {
    world.despawn(e.entity);
}

/// The legend's top, px (§2.21.9): under the HUD's line box and `0.4·S`, or the HUD's
/// own top when the HUD is off.
pub fn legend_top(size: u32, hud_on: bool) -> f32 {
    let m = hud::margin(size) as f32;
    if hud_on {
        m + 1.2 * size as f32 + 0.4 * size as f32
    } else {
        m
    }
}

/// One copy of a legend text: never wrapped, proportional figures.
fn legend_text(s: &str, font: &Handle<Font>, size: u32, color: [u8; 3]) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(size as f32),
            ..default()
        },
        TextColor(srgb(color)),
        TextLayout::no_wrap(),
    )
}

/// An outlined text item in `parent`'s row, with a right margin of `0.4·S`: the fill in
/// its flow and the eight black copies `o` px away at `ZIndex(-1)`, as the HUD's.
fn legend_item(world: &mut World, parent: Entity, s: &str, font: &Handle<Font>, size: u32) {
    let item = world
        .spawn((
            Node {
                margin: UiRect::right(Val::Px(size as f32 * 0.4)),
                ..default()
            },
            ChildOf(parent),
        ))
        .id();
    world.spawn((legend_text(s, font, size, hud::FILL), ChildOf(item)));
    for (dx, dy) in hud_offsets(size) {
        world.spawn((
            legend_text(s, font, size, hud::OUTLINE),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(dx),
                top: Val::Px(dy),
                ..default()
            },
            ZIndex(-1),
            ChildOf(item),
        ));
    }
}

/// Spawn the legend's UI tree (vis-003 §2.21.9) at (`hud::margin(size)`, `top`): a row
/// with its items centred, the title, then each step's swatch (`round(0.75·S)` px square,
/// an `o`-px black border, a right margin of `0.25·S`) and its range. With `camera`,
/// drawn by it into its image (`render`); without, into the window (`view`). Returns the
/// root.
pub fn spawn_legend(
    world: &mut World,
    camera: Option<Entity>,
    font: Handle<Font>,
    size: u32,
    top: f32,
    legend: &Legend,
) -> Entity {
    let (title, items) = legend;
    let m = hud::margin(size) as f32;
    let mut root = world.spawn(Node {
        position_type: PositionType::Absolute,
        left: Val::Px(m),
        top: Val::Px(top),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..default()
    });
    if let Some(c) = camera {
        root.insert(UiTargetCamera(c));
    }
    let root = root.id();
    legend_item(world, root, title, &font, size);
    let o = hud::outline_offset(size) as f32;
    let side = (size as f32 * 0.75).round();
    for (label, colour) in items {
        world.spawn((
            Node {
                width: Val::Px(side),
                height: Val::Px(side),
                border: UiRect::all(Val::Px(o)),
                margin: UiRect::right(Val::Px(size as f32 * 0.25)),
                ..default()
            },
            BackgroundColor(srgb(*colour)),
            BorderColor::all(srgb(hud::OUTLINE)),
            ChildOf(root),
        ));
        legend_item(world, root, label, &font, size);
    }
    root
}

/// Despawn the legend's tree.
pub fn despawn_legend(world: &mut World, root: Entity) {
    world.entity_mut(root).despawn();
}
