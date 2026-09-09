//! Bevy-клиент (presentation): рендер, ввод, отладка.

use bevy::prelude::*;

/// Запускает игровой клиент.
pub fn run() {
    App::new().add_plugins(DefaultPlugins).run();
}
