//! Bevy-клиент (presentation): рендер, ввод, отладка.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use simulation::block::{BlockRegistry, BuiltinBlocks};
use simulation::chunk::Chunk;
use simulation::coord::{LocalPos, CHUNK_SIZE_U8};

pub mod camera;

/// Цвет фона (неба), очищающий экран каждый кадр.
const CLEAR_COLOR: Color = Color::srgb_u8(135, 206, 250);

/// Запускает игровой клиент.
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(CLEAR_COLOR))
        .add_systems(Startup, (camera::setup, setup_light, spawn_hardcoded_chunk))
        .add_systems(Update, (camera::look, camera::fly).chain())
        .add_systems(Update, camera::toggle_cursor)
        .run();
}

/// Спавнит временный направленный свет (до полноценного освещения в харденинге).
fn setup_light(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            (-45f32).to_radians(),
            45f32.to_radians(),
            0.0,
        )),
    ));
}

/// Спавнит хардкод-чанк: платформа + колонна. Временный спайк до настоящего
/// мешера (шаг 15) и генерации мира (шаг 16).
fn spawn_hardcoded_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let blocks = BlockRegistry::new().builtin_blocks();
    let chunk = hardcoded_chunk(blocks);
    let mesh = chunk_to_mesh(&chunk);
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.6, 0.4),
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(material),
        Transform::default(),
    ));
}

/// Собирает хардкод-чанк: платформа 8×1×8 из камня и колонна в центре.
fn hardcoded_chunk(blocks: BuiltinBlocks) -> Chunk {
    let mut chunk = Chunk::empty();
    for x in 0..8 {
        for z in 0..8 {
            chunk.set(LocalPos::new(x, 0, z), blocks.stone);
        }
    }
    for y in 1..4 {
        chunk.set(LocalPos::new(0, y, 0), blocks.dirt);
    }
    chunk.set(LocalPos::new(0, 4, 0), blocks.grass);
    chunk
}

/// Наивный мешер: 6 граней на каждый непустой блок, без face-culling.
///
/// Временный — заменяется чистым мешером с culling в шаге 15.
fn chunk_to_mesh(chunk: &Chunk) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..CHUNK_SIZE_U8 {
        for z in 0..CHUNK_SIZE_U8 {
            for x in 0..CHUNK_SIZE_U8 {
                let pos = LocalPos::new(x, y, z);
                if chunk.get(pos) == BlockRegistry::AIR {
                    continue;
                }
                emit_cube(
                    Vec3::new(f32::from(x), f32::from(y), f32::from(z)),
                    &mut positions,
                    &mut normals,
                    &mut indices,
                );
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Грани куба: (нормаль, 4 угла против часовой стрелки снаружи).
const FACES: [(Vec3, [Vec3; 4]); 6] = [
    // +X
    (
        Vec3::X,
        [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
        ],
    ),
    // -X
    (
        Vec3::NEG_X,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 0.0),
        ],
    ),
    // +Y
    (
        Vec3::Y,
        [
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
        ],
    ),
    // -Y
    (
        Vec3::NEG_Y,
        [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
        ],
    ),
    // +Z
    (
        Vec3::Z,
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 1.0, 1.0),
        ],
    ),
    // -Z
    (
        Vec3::NEG_Z,
        [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        ],
    ),
];

/// Добавляет 6 граней куба с нижним углом в `origin`.
fn emit_cube(
    origin: Vec3,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
) {
    for (normal, corners) in FACES {
        let base = u32::try_from(positions.len()).expect("mesh vertex count fits in u32");
        for corner in corners {
            let v = origin + corner;
            positions.push(v.to_array());
            normals.push(normal.to_array());
        }
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}
