//! Отладочная egui-панель: FPS и время кадра.

// Системные параметры Bevy (`Res`, `Time`, …) обязаны передаваться по значению;
// clippy-линт здесь — ложное срабатывание.
#![allow(clippy::needless_pass_by_value)]

use bevy::diagnostic::{Diagnostic, DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

/// Плагин отладочной панели: egui-окно с FPS и временем кадра, toggle по F3.
pub struct DebugPanelPlugin;

impl Plugin for DebugPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EguiPlugin::default(), FrameTimeDiagnosticsPlugin::default()))
            .init_resource::<DebugPanel>()
            .add_observer(customize_egui_context)
            .add_systems(Update, toggle_panel)
            .add_systems(EguiPrimaryContextPass, draw_panel);
    }
}

/// Один раз при создании egui-контекста отключает тень у окон.
fn customize_egui_context(trigger: On<Add, EguiContext>, mut contexts: Query<&mut EguiContext>) {
    let mut context = contexts
        .get_mut(trigger.entity)
        .expect("just-added egui context");
    let ctx = context.get_mut();
    for theme in [egui::Theme::Dark, egui::Theme::Light] {
        ctx.style_mut_of(theme, |style| {
            style.visuals.window_shadow = egui::Shadow::NONE;
        });
    }
}

/// Состояние панели: видимость (переключается по F3).
///
/// По умолчанию включена в debug-сборке и выключена в release.
#[derive(Resource)]
struct DebugPanel {
    visible: bool,
}

impl Default for DebugPanel {
    fn default() -> Self {
        Self {
            visible: cfg!(debug_assertions),
        }
    }
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
