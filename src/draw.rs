//! What `render` and `view` draw the same way (vis-001 §2.9.6): the road mesh, the box
//! pool's materials and transforms, and the camera's fixed parts. Moved from
//! `src/render.rs`, so `render`'s frames do not change. The perspective path (§2.11.2)
//! adds a camera built from a pose, a shaded box mesh and a smaller rank lift, and since
//! Phase 6 (§2.12) draws every box of a frame as one mesh in `vehicle_id` order; the pool
//! is the orthographic path's. vis-002 adds the buildings: one lit mesh, a sun and the
//! orthographic eye's rule, and, for `render` only, the credit line's UI tree (§2.14.5).

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
use crate::render::VehicleBox;
use crate::scene::{self, Strip};

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
/// the speed colour (linear) × the face's shade. One draw keeps its triangles' order, so
/// a depth tie goes to the later box, the higher id. A box whose placed point, heading and
/// length equal a later box's bit for bit is left out; ranks are counted first, so no
/// other box moves. `None` when no box is drawn.
pub fn boxes_mesh(boxes: &[VehicleBox], (cx, cy): (f64, f64), lift: f64) -> Option<Mesh> {
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
        let c = srgb(scene::SPEED_BINS[scene::speed_bin(b.speed)].1).to_linear();
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
    let mesh = boxes_mesh(&[placeholder], (0.0, 0.0), 0.0).expect("one box is drawn");
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

/// Draw `boxes` as [`boxes_mesh`]'s one mesh, `lift` metres per rank, in place of the last
/// frame's; hide the entity when no box is drawn.
pub fn set_boxes(
    world: &mut World,
    entity: &BoxesEntity,
    boxes: &[VehicleBox],
    (cx, cy): (f64, f64),
    lift: f64,
) {
    let want = match boxes_mesh(boxes, (cx, cy), lift) {
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
