//! Полётная first-person камера: WASD + мышь, без коллизий.
//!
//! Временное управление для проверки рендера: полёт в 6DoF и обзор мышью.
//! Коллизии появятся в фазе E.

// Системные параметры Bevy (`Res`, `Time`, …) обязаны передаваться по значению;
// clippy-линт здесь — ложное срабатывание.
#![allow(clippy::needless_pass_by_value)]

use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

/// Скорость полёта в блоках в секунду.
const MOVE_SPEED: f32 = 12.0;
/// Чувствительность мыши в радианах на пиксель.
const LOOK_SENSITIVITY: f32 = 0.002;
/// Предел наклона обзора (чуть меньше 90°, чтобы не было флипа).
const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
/// Начальный наклон вниз, чтобы видеть платформу.
const START_PITCH: f32 = -0.4;

/// Углы обзора полётной камеры.
#[derive(Component)]
pub struct FlyingCamera {
    yaw: f32,
    pitch: f32,
}

impl Default for FlyingCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: START_PITCH,
        }
    }
}

impl FlyingCamera {
    /// Ориентация камеры по текущим углам (порядок YXZ: yaw, затем pitch).
    fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }
}

/// Спавнит полётную камеру и захватывает курсор.
pub fn setup(mut commands: Commands, mut cursor: Query<&mut CursorOptions>) {
    let camera = FlyingCamera::default();
    let rotation = camera.rotation();
    commands.spawn((
        Camera3d::default(),
        camera,
        Transform::from_xyz(0.0, 8.0, 16.0).with_rotation(rotation),
    ));

    for mut options in &mut cursor {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}

/// Обзор мышью: движение мыши меняет yaw/pitch (только при захваченном курсоре).
pub fn look(
    cursor: Query<&CursorOptions>,
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyingCamera), With<Camera3d>>,
) {
    let mut delta = Vec2::ZERO;
    for event in motion.read() {
        delta += event.delta;
    }

    let locked = cursor.iter().any(|c| c.grab_mode == CursorGrabMode::Locked);
    if !locked || delta == Vec2::ZERO {
        return;
    }

    for (mut transform, mut camera) in &mut query {
        camera.yaw -= delta.x * LOOK_SENSITIVITY;
        camera.pitch = (camera.pitch - delta.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = camera.rotation();
    }
}

/// Полёт: WASD — параллельно земле, Space/Shift — вверх/вниз.
pub fn fly(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<FlyingCamera>>,
) {
    for mut transform in &mut query {
        let mut forward = *transform.forward();
        forward.y = 0.0;
        let forward = forward.normalize_or_zero();
        let right = *transform.right();
        let mut direction = Vec3::ZERO;
        if keys.pressed(KeyCode::KeyW) {
            direction += forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            direction -= forward;
        }
        if keys.pressed(KeyCode::KeyD) {
            direction += right;
        }
        if keys.pressed(KeyCode::KeyA) {
            direction -= right;
        }
        if keys.pressed(KeyCode::Space) {
            direction += Vec3::Y;
        }
        if keys.pressed(KeyCode::ShiftLeft) {
            direction -= Vec3::Y;
        }
        transform.translation += direction.normalize_or_zero() * MOVE_SPEED * time.delta_secs();
    }
}

/// Переключает захват курсора по `Escape` (чтобы можно было закрыть окно).
pub fn toggle_cursor(keys: Res<ButtonInput<KeyCode>>, mut cursor: Query<&mut CursorOptions>) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    for mut options in &mut cursor {
        let new_mode = if options.grab_mode == CursorGrabMode::Locked {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
        options.grab_mode = new_mode;
        options.visible = new_mode == CursorGrabMode::None;
    }
}
