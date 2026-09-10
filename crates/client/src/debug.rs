//! Отладочная egui-панель: FPS и время кадра.

// Системные параметры Bevy (`Res`, `Time`, …) обязаны передаваться по значению;
// clippy-линт здесь — ложное срабатывание.
#![allow(clippy::needless_pass_by_value)]

use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

/// Плагин отладочной панели: egui-окно с FPS и временем кадра, toggle по F3.
pub struct DebugPanelPlugin;

impl Plugin for DebugPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
            .init_resource::<DebugPanel>()
            .add_systems(Update, toggle_panel)
            .add_systems(EguiPrimaryContextPass, draw_panel);
    }
}

/// Состояние панели: видимость (переключается по F3).
#[derive(Resource, Default)]
struct DebugPanel {
    visible: bool,
}

/// Переключает видимость панели по F3.
fn toggle_panel(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<DebugPanel>) {
    if keys.just_pressed(KeyCode::F3) {
        panel.visible = !panel.visible;
    }
}

/// Рисует панель с FPS и временем кадра.
fn draw_panel(
    panel: Res<DebugPanel>,
    diagnostics: Res<DiagnosticsStore>,
    mut contexts: EguiContexts,
) {
    if !panel.visible {
        return;
    }
    let ctx = contexts.ctx_mut().expect("egui context");
    egui::Window::new("Debug").show(ctx, |ui| {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(Diagnostic::smoothed)
        {
            ui.label(format!("FPS: {fps:.1}"));
        }
        if let Some(frame_time) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .and_then(Diagnostic::smoothed)
        {
            ui.label(format!("Frame time: {frame_time:.2} ms"));
        }
    });
}
