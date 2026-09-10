//! Bevy-клиент (presentation): рендер, ввод, отладка.

use bevy::prelude::*;

/// Цвет фона (неба), очищающий экран каждый кадр.
const CLEAR_COLOR: Color = Color::srgb_u8(135, 206, 250);

/// Запускает игровой клиент.
pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(CLEAR_COLOR))
        .add_systems(Startup, setup_camera)
        .run();
}

/// Спавнит 3D-камеру, смотрящую на начало мира.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 8.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
