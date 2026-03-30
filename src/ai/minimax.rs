//! Algoritmo Minimax com poda Alpha-Beta para o motor de IA.
//!
//! Responsabilidade: dada uma posição do tabuleiro, encontrar a melhor jogada
//! dentro de uma profundidade máxima configurável.
//! Depende apenas de `game/` e `ai/heuristic.rs`.

use crate::game::{rules, Board, GameResult, Player};

use super::heuristic;

/// Score de vitória imediata — alto o suficiente para nunca ser superado pela heurística.
const SCORE_VITORIA: i32 = 100_000;

/// Executa o algoritmo Minimax com poda Alpha-Beta.
///
/// # Parâmetros
/// - `board`: estado atual do tabuleiro (clonado para exploração)
/// - `depth`: profundidade máxima restante de exploração
/// - `alpha`: melhor score garantido para o maximizador (X)
/// - `beta`: melhor score garantido para o minimizador (O)
/// - `maximizing`: `true` quando é a vez de X (maximizador)
///
/// # Retorna
/// Score numérico do estado. Positivo = vantagem de X. Negativo = vantagem de O.
pub fn minimax(
    board: &Board,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
) -> i32 {
    // Caso base: jogo encerrado
    if let Some(resultado) = &board.result {
        return match resultado {
            GameResult::Winner(Player::X) => SCORE_VITORIA + depth as i32, // vitória mais rápida = melhor
            GameResult::Winner(Player::O) => -SCORE_VITORIA - depth as i32,
            GameResult::Draw => 0,
        };
    }

    // Profundidade esgotada: avaliação heurística
    if depth == 0 {
        return heuristic::evaluate(board);
    }

    let jogadas = rules::valid_moves(board);

    if jogadas.is_empty() {
        // Sem jogadas válidas = empate por posição
        return 0;
    }

    if maximizing {
        let mut melhor = i32::MIN;

        for (quad, cell) in jogadas {
            let mut novo_tabuleiro = board.clone();
            novo_tabuleiro.make_move(quad, cell);

            let score = minimax(&novo_tabuleiro, depth - 1, alpha, beta, false);
            melhor = melhor.max(score);
            alpha = alpha.max(score);

            // Poda Beta: o minimizador não vai permitir este ramo
            if beta <= alpha {
                break;
            }
        }

        melhor
    } else {
        let mut melhor = i32::MAX;

        for (quad, cell) in jogadas {
            let mut novo_tabuleiro = board.clone();
            novo_tabuleiro.make_move(quad, cell);

            let score = minimax(&novo_tabuleiro, depth - 1, alpha, beta, true);
            melhor = melhor.min(score);
            beta = beta.min(score);

            // Poda Alpha: o maximizador não vai permitir este ramo
            if beta <= alpha {
                break;
            }
        }

        melhor
    }
}

/// Encontra a melhor jogada para o jogador atual dado uma profundidade máxima.
///
/// # Retorna
/// O par `(quadrante, célula)` da melhor jogada encontrada.
/// Retorna `None` se não houver jogadas disponíveis (não deveria ocorrer em partida ativa).
pub fn best_move_at_depth(board: &Board, depth: u8) -> Option<(usize, usize)> {
    let jogadas = rules::valid_moves(board);

    if jogadas.is_empty() {
        return None;
    }

    let maximizing = board.current_player == Player::X;
    let mut melhor_jogada = jogadas[0];
    let mut melhor_score = if maximizing { i32::MIN } else { i32::MAX };

    for (quad, cell) in jogadas {
        let mut novo_tabuleiro = board.clone();
        novo_tabuleiro.make_move(quad, cell);

        let score = minimax(&novo_tabuleiro, depth, i32::MIN, i32::MAX, !maximizing);

        let é_melhor = if maximizing {
            score > melhor_score
        } else {
            score < melhor_score
        };

        if é_melhor {
            melhor_score = score;
            melhor_jogada = (quad, cell);
        }
    }

    Some(melhor_jogada)
}
