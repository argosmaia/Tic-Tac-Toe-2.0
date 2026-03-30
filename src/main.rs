//! Ponto de entrada do Velha 2.0.
//!
//! Inicializa o runtime tokio (necessário para iroh) e lança o app eframe.

mod ai;
mod app;
mod game;
mod network;
mod storage;
mod ui;

use app::AppState;
use eframe::NativeOptions;
use egui::ViewportBuilder;

fn main() -> eframe::Result<()> {
    // Configura janela nativa
    let opções = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("Velha 2.0 — Ultimate Tic-Tac-Toe")
            .with_inner_size([800.0, 640.0])
            .with_min_inner_size([720.0, 540.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Velha 2.0",
        opções,
        Box::new(|cc| Box::new(AppState::new(cc))),
    )
}
