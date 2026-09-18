//! Nível "The Experience" — Minimax com viés estatístico do jogador humano.
//!
//! Responsabilidade: combinar a força do Minimax com Alpha-Beta (profundidade 9)
//! com um mapa de calor das posições favoritas do humano para:
//!   1. Ordenar jogadas de forma que o Alpha-Beta explore primeiro as posições
//!      mais usadas pelo humano (melhor poda E maior pressão de bloqueio).
//!   2. Desempatar posições de score igual preferindo posições de alto calor
//!      (bloquear onde o humano gosta de jogar).

use crate::game::{rules, Board, Player};

use super::minimax::{best_move_at_depth, minimax};

/// Profundidade máxima de busca para o nível "The Experience".
///
/// 9 é o limite prático para Ultimate Tic-Tac-Toe com Alpha-Beta:
/// cobre todo o espaço de jogadas nas fases iniciais e finais.
const EXPERIENCIA_DEPTH: u8 = 9;

/// Calcula a melhor jogada para o nível "The Experience".
///
/// # Parâmetros
/// - `board`: estado atual do tabuleiro
/// - `heatmap`: frequência relativa (0.0–1.0) das posições do jogador humano
///              indexada por [quadrante][célula]
///
/// # Estratégia
/// 1. Obtém jogadas válidas e as ordena por calor decrescente (posições favoritas
///    do humano primeiro — Alpha-Beta pode eliminá-las mais rápido ao ver que a
///    IA quer bloquear ali).
/// 2. Executa minimax com poda Alpha-Beta em profundidade máxima.
/// 3. Em caso de empate de score, prefere a posição de maior calor (bloqueio
///    preventivo nos padrões históricos do humano).
pub fn best_move_experience(board: &Board, heatmap: &[[f32; 9]; 9]) -> Option<(usize, usize)> {
    let mut jogadas = rules::valid_moves(board);

    if jogadas.is_empty() {
        return None;
    }

    // Ordena jogadas por calor decrescente para melhor poda Alpha-Beta.
    // Posições de alto calor = onde o humano joga mais = prioritárias para bloqueio.
    jogadas.sort_by(|&(qa, ca), &(qb, cb)| {
        heatmap[qb][cb]
            .partial_cmp(&heatmap[qa][ca])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let maximizing = board.current_player == Player::X;
    let mut melhor_jogada = jogadas[0];
    let mut melhor_score = if maximizing { i32::MIN } else { i32::MAX };
    let mut melhor_calor = -1.0f32; // calor da jogada escolhida (desempate)

    let mut alpha = i32::MIN;
    let mut beta = i32::MAX;
    let proxima_profundidade = EXPERIENCIA_DEPTH.saturating_sub(1);

    for &(quad, cell) in &jogadas {
        let mut novo_tabuleiro = board.clone();
        novo_tabuleiro.make_move(quad, cell);

        let score = minimax(
            &novo_tabuleiro,
            proxima_profundidade,
            alpha,
            beta,
            !maximizing,
        );

        let calor = heatmap[quad][cell];

        if maximizing {
            // Maximizador (CPU como X): escolhe maior score; desempata por maior calor
            if score > melhor_score || (score == melhor_score && calor > melhor_calor) {
                melhor_score = score;
                melhor_jogada = (quad, cell);
                melhor_calor = calor;
            }
            alpha = alpha.max(melhor_score);
        } else {
            // Minimizador (CPU como O): escolhe menor score; desempata por maior calor
            if score < melhor_score || (score == melhor_score && calor > melhor_calor) {
                melhor_score = score;
                melhor_jogada = (quad, cell);
                melhor_calor = calor;
            }
            beta = beta.min(melhor_score);
        }
    }

    Some(melhor_jogada)
}

/// Versão de fallback sem heatmap — usa minimax puro em profundidade máxima.
///
/// Usada quando o banco de dados não está disponível ou o jogador não tem histórico.
pub fn best_move_experience_fallback(board: &Board) -> Option<(usize, usize)> {
    best_move_at_depth(board, EXPERIENCIA_DEPTH)
}
