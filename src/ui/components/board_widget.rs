//! Widget do tabuleiro Ultimate Tic-Tac-Toe.
//!
//! Responsabilidade: renderizar o tabuleiro 9x9 de forma completamente stateless.
//! Não muta estado — recebe &Board e um callback on_move.
//! Toda lógica de estado fica em AppState.

use egui::{Color32, Pos2, Rect, Sense, Ui, Vec2};

use crate::game::{Board, Cell, Player, QuadState};
use crate::ui::theme::{cores, espacamentos, tipografia};

/// Renderiza o tabuleiro completo do Ultimate Tic-Tac-Toe.
///
/// # Parâmetros
/// - `ui`: contexto de renderização do egui
/// - `board`: estado atual do tabuleiro (somente leitura)
/// - `on_move`: callback chamado com (quadrante, célula) quando o jogador clica
/// - `interativo`: se false, desabilita todo input (modo spectator/CPU jogando)
pub fn render_board(
    ui: &mut Ui,
    board: &Board,
    on_move: &mut impl FnMut(usize, usize),
    interativo: bool,
) {
    let tamanho_celula = espacamentos::TAMANHO_CELULA;
    let gap_celula = espacamentos::GAP_CELULA;
    let gap_quad = espacamentos::GAP_QUADRANTE;

    // Tamanho de um quadrante: 3 células + 2 gaps internos
    let tamanho_quad = tamanho_celula * 3.0 + gap_celula * 2.0;
    // Tamanho total do tabuleiro: 3 quadrantes + 2 gaps externos
    let tamanho_total = tamanho_quad * 3.0 + gap_quad * 2.0;

    // Aloca espaço fixo para o tabuleiro
    let (resposta_area, painter) = ui.allocate_painter(
        Vec2::splat(tamanho_total),
        Sense::hover(),
    );

    let origem = resposta_area.rect.min;

    // Renderiza cada um dos 9 quadrantes
    for quad_idx in 0..9 {
        let quad_col = (quad_idx % 3) as f32;
        let quad_row = (quad_idx / 3) as f32;

        let quad_origem = Pos2 {
            x: origem.x + quad_col * (tamanho_quad + gap_quad),
            y: origem.y + quad_row * (tamanho_quad + gap_quad),
        };

        let quad_rect = Rect::from_min_size(quad_origem, Vec2::splat(tamanho_quad));

        // Determina se este quadrante está ativo
        let quad_ativo = match board.active_quad {
            Some(q) => q == quad_idx,
            None => board.quad_states[quad_idx] == QuadState::Open,
        };

        // Renderiza fundo do quadrante
        render_fundo_quadrante(&painter, quad_rect, quad_ativo, board.quad_states[quad_idx]);

        // Overlay de vitória ou empate sobre quadrantes resolvidos
        match board.quad_states[quad_idx] {
            QuadState::Won(jogador) => {
                render_overlay_vitoria(&painter, quad_rect, jogador);
            }
            QuadState::Draw => {
                painter.rect_filled(quad_rect, 4.0, cores::OVERLAY_EMPATE);
            }
            QuadState::Open => {
                // Renderiza células individuais
                render_celulas(
                    ui,
                    &painter,
                    board,
                    quad_idx,
                    quad_origem,
                    tamanho_celula,
                    gap_celula,
                    quad_ativo && interativo && !board.is_over(),
                    on_move,
                );
            }
        }
    }
}

/// Renderiza o fundo de um quadrante com borda de destaque quando ativo.
fn render_fundo_quadrante(
    painter: &egui::Painter,
    rect: Rect,
    ativo: bool,
    estado: QuadState,
) {
    let cor_fundo = if ativo {
        cores::SUPERFICIE_ELEVADA
    } else {
        cores::SUPERFICIE
    };

    painter.rect_filled(rect, 6.0, cor_fundo);

    if ativo && estado == QuadState::Open {
        // Borda colorida + glow para quadrante ativo
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(espacamentos::ESPESSURA_BORDA_ATIVA, cores::BORDA_ATIVA),
        );
        // Glow: borda mais larga e transparente
        painter.rect_stroke(
            rect.expand(3.0),
            8.0,
            egui::Stroke::new(6.0, cores::GLOW_ATIVO),
        );
    } else {
        painter.rect_stroke(rect, 6.0, egui::Stroke::new(1.0, cores::BORDA));
    }
}

/// Renderiza o overlay de vitória sobre um quadrante resolvido.
fn render_overlay_vitoria(painter: &egui::Painter, rect: Rect, jogador: Player) {
    let (cor_overlay, cor_texto) = match jogador {
        Player::X => (cores::OVERLAY_VITORIA_X, cores::JOGADOR_X),
        Player::O => (cores::OVERLAY_VITORIA_O, cores::JOGADOR_O),
    };

    painter.rect_filled(rect, 6.0, cor_overlay);

    // Símbolo grande centralizado no quadrante
    let centro = rect.center();
    let simbolo = jogador.symbol();

    painter.text(
        centro,
        egui::Align2::CENTER_CENTER,
        simbolo,
        egui::FontId::proportional(tipografia::SIMBOLO_OVERLAY),
        cor_texto,
    );
}

/// Renderiza as 9 células de um quadrante e processa cliques.
fn render_celulas(
    ui: &mut Ui,
    painter: &egui::Painter,
    board: &Board,
    quad_idx: usize,
    quad_origem: Pos2,
    tamanho_celula: f32,
    gap_celula: f32,
    clicavel: bool,
    on_move: &mut impl FnMut(usize, usize),
) {
    for cell_idx in 0..9 {
        let cell_col = (cell_idx % 3) as f32;
        let cell_row = (cell_idx / 3) as f32;

        let cell_origem = Pos2 {
            x: quad_origem.x + cell_col * (tamanho_celula + gap_celula),
            y: quad_origem.y + cell_row * (tamanho_celula + gap_celula),
        };

        let cell_rect = Rect::from_min_size(cell_origem, Vec2::splat(tamanho_celula));
        let célula = board.cells[quad_idx][cell_idx];

        // Fundo da célula
        let cor_fundo = match célula {
            Cell::Empty => cores::FUNDO_ESCURO,
            Cell::Taken(_) => cores::SUPERFICIE,
        };
        painter.rect_filled(cell_rect, 4.0, cor_fundo);
        painter.rect_stroke(cell_rect, 4.0, egui::Stroke::new(1.0, cores::BORDA));

        match célula {
            Cell::Taken(jogador) => {
                // Célula ocupada — renderiza o símbolo
                let cor = match jogador {
                    Player::X => cores::JOGADOR_X,
                    Player::O => cores::JOGADOR_O,
                };
                painter.text(
                    cell_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    jogador.symbol(),
                    egui::FontId::proportional(tipografia::SIMBOLO_TABULEIRO),
                    cor,
                );
            }
            Cell::Empty if clicavel => {
                // Célula vazia e jogável — registra área clicável
                let resposta = ui.allocate_rect(cell_rect, Sense::click());

                if resposta.hovered() {
                    // Efeito hover: preenchimento suave
                    painter.rect_filled(cell_rect, 4.0, cores::GLOW_ATIVO);
                }

                if resposta.clicked() {
                    on_move(quad_idx, cell_idx);
                }
            }
            _ => {} // Célula vazia mas não clicável — sem ação
        }
    }
}
