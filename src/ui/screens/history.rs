//! Tela de histórico de partidas.

use egui::Ui;

use crate::storage::MatchRecord;
use crate::ui::theme::{cores, espacamentos, tipografia};

/// Ação solicitada na tela de histórico.
pub enum HistoricoAction {
    Voltar,
    Nenhuma,
}

/// Renderiza a tela de histórico de partidas.
pub fn render_historico(ui: &mut Ui, partidas: &[MatchRecord]) -> HistoricoAction {
    let mut ação = HistoricoAction::Nenhuma;

    ui.vertical(|ui| {
        ui.add_space(24.0);

        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("← Voltar")
                            .size(tipografia::CORPO)
                            .color(cores::TEXTO_MUDO),
                    )
                    .fill(egui::Color32::TRANSPARENT),
                )
                .clicked()
            {
                ação = HistoricoAction::Voltar;
            }

            ui.label(
                egui::RichText::new("Histórico de Partidas")
                    .size(tipografia::TITULO)
                    .color(cores::TEXTO_PRIMARIO)
                    .strong(),
            );
        });

        ui.add_space(16.0);

        if partidas.is_empty() {
            ui.label(
                egui::RichText::new("Nenhuma partida registrada ainda.")
                    .size(tipografia::CORPO)
                    .color(cores::TEXTO_MUDO),
            );
            return;
        }

        // Cabeçalho da tabela
        ui.horizontal(|ui| {
            ui.set_min_height(28.0);
            ui.label(egui::RichText::new("X").size(tipografia::CORPO).color(cores::JOGADOR_X).strong());
            ui.add_space(60.0);
            ui.label(egui::RichText::new("vs").size(tipografia::PEQUENO).color(cores::TEXTO_MUDO));
            ui.add_space(8.0);
            ui.label(egui::RichText::new("O").size(tipografia::CORPO).color(cores::JOGADOR_O).strong());
            ui.add_space(40.0);
            ui.label(egui::RichText::new("Resultado").size(tipografia::CORPO).color(cores::TEXTO_SECUNDARIO));
            ui.add_space(20.0);
            ui.label(egui::RichText::new("Modo").size(tipografia::CORPO).color(cores::TEXTO_SECUNDARIO));
        });

        ui.separator();

        // Linhas da tabela
        egui::ScrollArea::vertical().show(ui, |ui| {
            for partida in partidas {
                egui::Frame::none()
                    .fill(cores::SUPERFICIE)
                    .stroke(egui::Stroke::new(1.0, cores::BORDA))
                    .rounding(espacamentos::RAIO_BORDA)
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&partida.player_x)
                                    .size(tipografia::CORPO)
                                    .color(cores::JOGADOR_X),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("vs")
                                    .size(tipografia::PEQUENO)
                                    .color(cores::TEXTO_MUDO),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(&partida.player_o)
                                    .size(tipografia::CORPO)
                                    .color(cores::JOGADOR_O),
                            );
                            ui.add_space(16.0);

                            let (resultado_texto, cor_resultado) = match partida.result.as_str() {
                                "x_wins" => ("X venceu", cores::JOGADOR_X),
                                "o_wins" => ("O venceu", cores::JOGADOR_O),
                                _ => ("Empate", cores::TEXTO_MUDO),
                            };
                            ui.label(
                                egui::RichText::new(resultado_texto)
                                    .size(tipografia::CORPO)
                                    .color(cor_resultado),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                egui::RichText::new(&partida.mode)
                                    .size(tipografia::PEQUENO)
                                    .color(cores::TEXTO_MUDO),
                            );
                        });
                    });

                ui.add_space(4.0);
            }
        });
    });

    ação
}
