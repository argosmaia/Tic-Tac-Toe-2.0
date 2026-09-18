//! Tela de histórico de partidas.

use egui::Ui;

use crate::storage::{Database, MatchRecord};
use crate::ui::theme::{cores, espacamentos, tipografia};

/// Filtro de exibição do histórico.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum FiltroHistorico {
    #[default]
    Todos,
    Vitorias,
    Derrotas,
    Empates,
    Desistencias,
}

impl FiltroHistorico {
    fn label(self) -> &'static str {
        match self {
            FiltroHistorico::Todos => "Todos",
            FiltroHistorico::Vitorias => "✅ Vitórias",
            FiltroHistorico::Derrotas => "❌ Derrotas",
            FiltroHistorico::Empates => "🤝 Empates",
            FiltroHistorico::Desistencias => "⚡ Desistências",
        }
    }
}

/// Ação solicitada na tela de histórico.
pub enum HistoricoAction {
    Voltar,
    Nenhuma,
}

/// Estado persistente da tela de histórico (filtro selecionado).
#[derive(Default)]
pub struct HistoricoState {
    pub filtro: FiltroHistorico,
}

/// Renderiza a tela de histórico de partidas.
pub fn render_historico(
    ui: &mut Ui,
    partidas: &[MatchRecord],
    state: &mut HistoricoState,
    db: Option<&Database>,
) -> HistoricoAction {
    let mut ação = HistoricoAction::Nenhuma;

    ui.vertical(|ui| {
        ui.add_space(24.0);

        // Header
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

        ui.add_space(12.0);

        // Barra de filtros
        ui.horizontal(|ui| {
            for filtro in [
                FiltroHistorico::Todos,
                FiltroHistorico::Vitorias,
                FiltroHistorico::Derrotas,
                FiltroHistorico::Empates,
                FiltroHistorico::Desistencias,
            ] {
                let selecionado = state.filtro == filtro;
                let cor_fundo = if selecionado {
                    cores::BOTAO_PRIMARIO
                } else {
                    cores::SUPERFICIE
                };
                let cor_texto = if selecionado {
                    cores::BOTAO_TEXTO
                } else {
                    cores::TEXTO_SECUNDARIO
                };

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(filtro.label())
                                .size(tipografia::PEQUENO)
                                .color(cor_texto),
                        )
                        .fill(cor_fundo)
                        .rounding(espacamentos::RAIO_BORDA),
                    )
                    .clicked()
                {
                    state.filtro = filtro;
                }

                ui.add_space(4.0);
            }
        });

        ui.add_space(12.0);

        if partidas.is_empty() {
            ui.label(
                egui::RichText::new("Nenhuma partida registrada ainda.")
                    .size(tipografia::CORPO)
                    .color(cores::TEXTO_MUDO),
            );
            return;
        }

        // Filtra partidas conforme seleção
        let partidas_filtradas: Vec<&MatchRecord> = partidas
            .iter()
            .filter(|p| match state.filtro {
                FiltroHistorico::Todos => true,
                FiltroHistorico::Vitorias => p.result == "x_wins" || p.result == "o_wins",
                FiltroHistorico::Derrotas => p.result == "o_wins",
                FiltroHistorico::Empates => p.result == "draw",
                FiltroHistorico::Desistencias => p.result == "abandoned",
            })
            .collect();

        // Contagem
        ui.label(
            egui::RichText::new(format!("{} partida(s)", partidas_filtradas.len()))
                .size(tipografia::PEQUENO)
                .color(cores::TEXTO_MUDO),
        );
        ui.add_space(6.0);

        // Linhas da tabela
        egui::ScrollArea::vertical().show(ui, |ui| {
            for partida in &partidas_filtradas {
                let (emoji, result_texto, cor_resultado) =
                    resultado_display(partida);

                egui::Frame::none()
                    .fill(cores::SUPERFICIE)
                    .stroke(egui::Stroke::new(1.0, cores::BORDA))
                    .rounding(espacamentos::RAIO_BORDA)
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Emoji de resultado
                            ui.label(
                                egui::RichText::new(emoji)
                                    .size(tipografia::SUBTITULO),
                            );
                            ui.add_space(8.0);

                            // Jogadores
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(&partida.player_x)
                                            .size(tipografia::CORPO)
                                            .color(cores::JOGADOR_X)
                                            .strong(),
                                    );
                                    ui.label(
                                        egui::RichText::new(" vs ")
                                            .size(tipografia::PEQUENO)
                                            .color(cores::TEXTO_MUDO),
                                    );
                                    ui.label(
                                        egui::RichText::new(&partida.player_o)
                                            .size(tipografia::CORPO)
                                            .color(cores::JOGADOR_O)
                                            .strong(),
                                    );
                                });

                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new(result_texto)
                                            .size(tipografia::PEQUENO)
                                            .color(cor_resultado),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(&partida.mode)
                                            .size(tipografia::PEQUENO)
                                            .color(cores::TEXTO_MUDO),
                                    );
                                    ui.add_space(12.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "#{} · {}",
                                            partida.id,
                                            partida.played_at
                                        ))
                                        .size(tipografia::PEQUENO)
                                        .color(cores::TEXTO_MUDO),
                                    );
                                    if let Some(banco) = db {
                                        if let Ok(jogadas) = banco.get_match_moves(partida.id) {
                                            ui.add_space(12.0);
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "{} jogadas",
                                                    jogadas.len()
                                                ))
                                                .size(tipografia::PEQUENO)
                                                .color(cores::TEXTO_MUDO),
                                            );
                                            if let Some(ultima_jogada) = jogadas.last() {
                                                ui.add_space(12.0);
                                                ui.label(
                                                    egui::RichText::new(ultima_jogada.resumo())
                                                        .size(tipografia::PEQUENO)
                                                        .color(cores::TEXTO_MUDO),
                                                );
                                            }
                                        }
                                    }
                                    // Duração formatada
                                    if let Some(dur) = partida.duration_s {
                                        ui.add_space(12.0);
                                        ui.label(
                                            egui::RichText::new(format_duracao(dur))
                                                .size(tipografia::PEQUENO)
                                                .color(cores::TEXTO_MUDO),
                                        );
                                    }
                                });
                            });
                        });
                    });

                ui.add_space(4.0);
            }
        });
    });

    ação
}

/// Retorna emoji, texto e cor para um resultado de partida.
fn resultado_display(p: &MatchRecord) -> (&'static str, String, egui::Color32) {
    match p.result.as_str() {
        "x_wins" => ("🏆", format!("{} venceu", p.player_x), cores::JOGADOR_X),
        "o_wins" => ("🏆", format!("{} venceu", p.player_o), cores::JOGADOR_O),
        "draw" => ("🤝", "Empate".to_owned(), cores::TEXTO_MUDO),
        "abandoned" => {
            let quem = match p.abandoned_by.as_deref() {
                Some("x") => format!("{} desistiu", p.player_x),
                Some("o") => format!("{} desistiu", p.player_o),
                _ => "Desistência".to_owned(),
            };
            // Laranja — cor de alerta/warning
            ("⚡", quem, egui::Color32::from_rgb(255, 160, 50))
        }
        _ => ("❓", "Desconhecido".to_owned(), cores::TEXTO_MUDO),
    }
}

/// Formata duração em segundos como mm:ss.
fn format_duracao(segundos: i64) -> String {
    let m = segundos / 60;
    let s = segundos % 60;
    format!("⏱ {:02}:{:02}", m, s)
}
