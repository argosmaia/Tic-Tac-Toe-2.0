//! Card de jogador: exibe nome, símbolo e placar.

use egui::Ui;

use crate::game::Player;
use crate::ui::theme::{cor_jogador, cores, espacamentos, tipografia};

/// Renderiza o card de um jogador com nome, símbolo e placar.
///
/// Destaca visualmente o card quando é o turno deste jogador.
pub fn render_player_card(
    ui: &mut Ui,
    player: Player,
    nome: &str,
    pontos: u32,
    turno_ativo: bool,
) {
    let cor = cor_jogador(&player);
    let borda = if turno_ativo { cor } else { cores::BORDA };
    let fundo = if turno_ativo {
        cores::SUPERFICIE_ELEVADA
    } else {
        cores::SUPERFICIE
    };

    egui::Frame::none()
        .fill(fundo)
        .stroke(egui::Stroke::new(if turno_ativo { 2.0 } else { 1.0 }, borda))
        .rounding(espacamentos::RAIO_BORDA)
        .inner_margin(egui::Margin::same(espacamentos::PADDING_PAINEL))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Símbolo do jogador
                ui.label(
                    egui::RichText::new(player.symbol())
                        .size(tipografia::SUBTITULO)
                        .color(cor)
                        .strong(),
                );

                ui.vertical(|ui| {
                    // Nome do jogador
                    ui.label(
                        egui::RichText::new(nome)
                            .size(tipografia::CORPO)
                            .color(if turno_ativo {
                                cores::TEXTO_PRIMARIO
                            } else {
                                cores::TEXTO_SECUNDARIO
                            }),
                    );

                    // Placar
                    ui.label(
                        egui::RichText::new(format!("{} pts", pontos))
                            .size(tipografia::PEQUENO)
                            .color(cor),
                    );
                });

                // Indicador de turno ativo
                if turno_ativo {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new("▶")
                                    .size(tipografia::PEQUENO)
                                    .color(cor),
                            );
                        },
                    );
                }
            });
        });
}
