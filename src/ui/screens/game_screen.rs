//! Tela de jogo: tabuleiro, placar e indicadores de turno.

use egui::Ui;

use crate::game::{Board, GameResult, Player};
use crate::ui::components::board_widget::render_board;
use crate::ui::components::player_card::render_player_card;
use crate::ui::theme::{cor_jogador, cores, espacamentos, tipografia};

/// Ação solicitada na tela de jogo.
#[derive(Debug, Clone)]
pub enum GameScreenAction {
    /// Jogador humano fez uma jogada.
    JogadaRealizada { quad: usize, cell: usize },
    /// Voltar ao menu (desistência).
    Desistir,
    /// Nova partida com o mesmo modo.
    NovaPartida,
    /// Nenhuma ação — aguardando input ou CPU jogando.
    Nenhuma,
}

/// Estado de placar durante uma sessão de jogo.
#[derive(Default)]
pub struct Placar {
    pub pontos_x: u32,
    pub pontos_o: u32,
}

/// Renderiza a tela de jogo completa.
///
/// # Parâmetros
/// - `board`: estado atual do tabuleiro
/// - `nome_x`, `nome_o`: nomes dos jogadores
/// - `placar`: pontuação acumulada da sessão
/// - `interativo`: false quando a CPU está "pensando"
pub fn render_game_screen(
    ui: &mut Ui,
    board: &Board,
    nome_x: &str,
    nome_o: &str,
    placar: &Placar,
    interativo: bool,
) -> GameScreenAction {
    let mut ação = GameScreenAction::Nenhuma;

    // Layout: cards de jogador laterais + tabuleiro central
    ui.vertical(|ui| {
        ui.add_space(16.0);

        // Header com título
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("← Desistir")
                            .size(tipografia::PEQUENO)
                            .color(cores::TEXTO_MUDO),
                    )
                    .fill(egui::Color32::TRANSPARENT),
                )
                .clicked()
            {
                ação = GameScreenAction::Desistir;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let turno_label = if board.is_over() {
                    "Partida encerrada".to_string()
                } else {
                    format!(
                        "Vez de {}",
                        if board.current_player == Player::X {
                            nome_x
                        } else {
                            nome_o
                        }
                    )
                };
                ui.label(
                    egui::RichText::new(turno_label)
                        .size(tipografia::CORPO)
                        .color(cor_jogador(&board.current_player)),
                );
            });
        });

        ui.add_space(12.0);

        // Cards dos jogadores + tabuleiro
        ui.horizontal(|ui| {
            // Card X (esquerda)
            ui.vertical(|ui| {
                ui.set_width(130.0);
                render_player_card(
                    ui,
                    Player::X,
                    nome_x,
                    placar.pontos_x,
                    board.current_player == Player::X && !board.is_over(),
                );
            });

            ui.add_space(espacamentos::PADDING_PAINEL);

            // Tabuleiro central
            ui.vertical(|ui| {
                let mut jogada_pendente: Option<(usize, usize)> = None;

                render_board(
                    ui,
                    board,
                    &mut |quad, cell| {
                        jogada_pendente = Some((quad, cell));
                    },
                    interativo,
                );

                if let Some((quad, cell)) = jogada_pendente {
                    ação = GameScreenAction::JogadaRealizada { quad, cell };
                }
            });

            ui.add_space(espacamentos::PADDING_PAINEL);

            // Card O (direita)
            ui.vertical(|ui| {
                ui.set_width(130.0);
                render_player_card(
                    ui,
                    Player::O,
                    nome_o,
                    placar.pontos_o,
                    board.current_player == Player::O && !board.is_over(),
                );
            });
        });

        ui.add_space(16.0);

        // Tela de resultado se jogo encerrado
        if let Some(resultado) = &board.result {
            render_resultado(ui, resultado, nome_x, nome_o, &mut ação);
        }
    });

    ação
}

/// Renderiza o painel de resultado ao fim da partida.
fn render_resultado(
    ui: &mut Ui,
    resultado: &GameResult,
    nome_x: &str,
    nome_o: &str,
    ação: &mut GameScreenAction,
) {
    ui.separator();
    ui.add_space(8.0);

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        let (mensagem, cor) = match resultado {
            GameResult::Winner(Player::X) => (
                format!("🏆  {} venceu!", nome_x),
                cores::JOGADOR_X,
            ),
            GameResult::Winner(Player::O) => (
                format!("🏆  {} venceu!", nome_o),
                cores::JOGADOR_O,
            ),
            GameResult::Draw => ("🤝  Empate!".to_string(), cores::TEXTO_SECUNDARIO),
        };

        ui.label(
            egui::RichText::new(&mensagem)
                .size(tipografia::SUBTITULO)
                .color(cor)
                .strong(),
        );

        ui.add_space(12.0);

        if ui
            .add_sized(
                [180.0, 40.0],
                egui::Button::new(
                    egui::RichText::new("🔄  Nova Partida")
                        .size(tipografia::CORPO)
                        .color(cores::BOTAO_TEXTO),
                )
                .fill(cores::BOTAO_PRIMARIO)
                .rounding(espacamentos::RAIO_BORDA),
            )
            .clicked()
        {
            *ação = GameScreenAction::NovaPartida;
        }
    });
}
