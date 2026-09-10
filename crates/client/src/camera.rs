//! Контроллер first-person камеры: WASD + мышь, без коллизий.
//!
//! Сейчас — только полёт для проверки рендера. В фазе E добавятся коллизии,
//! гравитация и режим полёт/ходьба.

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

/// Плагин контроллера камеры: спавн, обзор, полёт, захват курсора.
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (look, fly).chain())
            .add_systems(Update, toggle_cursor);
    }
}

/// Состояние контроллера камеры: углы обзора.
///
/// Позже расширится скоростью/режимом полёта при добавлении физики.
#[derive(Component)]
pub struct CameraController {
    yaw: f32,
    pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: START_PITCH,
        }
    }
}

impl CameraController {
    /// Ориентация камеры по текущим углам (порядок YXZ: yaw, затем pitch).
    fn rotation(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0)
    }
}

/// Спавнит камеру и захватывает курсор.
fn setup(mut commands: Commands, mut cursor: Query<&mut CursorOptions>) {
    let controller = CameraController::default();
    let rotation = controller.rotation();
    commands.spawn((
        Camera3d::default(),
        controller,
        Transform::from_xyz(0.0, 8.0, 16.0).with_rotation(rotation),
    ));

    for mut options in &mut cursor {
        options.grab_mode = CursorGrabMode::Locked;
        options.visible = false;
    }
}

/// Обзор мышью: движение мыши меняет yaw/pitch (только при захваченном курсоре).
fn look(
    cursor: Query<&CursorOptions>,
    mut motion: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
) {
    let mut delta = Vec2::ZERO;
    for event in motion.read() {
        delta += event.delta;
    }

    let locked = cursor.iter().any(|c| c.grab_mode == CursorGrabMode::Locked);
    if !locked || delta == Vec2::ZERO {
        return;
    }

    for (mut transform, mut controller) in &mut query {
        controller.yaw -= delta.x * LOOK_SENSITIVITY;
        controller.pitch =
            (controller.pitch - delta.y * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = controller.rotation();
    }
}

/// Полёт: WASD — параллельно земле, Space/Shift — вверх/вниз.
fn fly(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<CameraController>>,
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
fn toggle_cursor(keys: Res<ButtonInput<KeyCode>>, mut cursor: Query<&mut CursorOptions>) {
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
