//! Níveis de dificuldade da IA e dispatcher de jogadas.
//!
//! Responsabilidade: expor `best_move(board, level)` que despacha para a estratégia
//! apropriada conforme o nível. Isola o resto do sistema dos detalhes de cada nível.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::game::{rules, Board, Player};

use super::minimax::best_move_at_depth;

/// Nível de dificuldade da IA.
///
/// - `Noob`: aleatoriedade pura com 20% de chance de jogar a melhor jogada "por acidente"
/// - `Jogadora`: heurística simples sem lookahead (ganhar se puder, bloquear se necessário)
/// - `Master`: Minimax com Alpha-Beta, profundidade máxima 4, heurística local
/// - `Killer`: Minimax com Alpha-Beta, profundidade máxima 6, heurística macro+micro combinada
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiLevel {
    Noob,
    Jogadora,
    Master,
    Killer,
}

impl AiLevel {
    pub fn label(self) -> &'static str {
        match self {
            AiLevel::Noob => "Noob",
            AiLevel::Jogadora => "Jogadora",
            AiLevel::Master => "Master",
            AiLevel::Killer => "Killer 💀",
        }
    }
}

/// Calcula a melhor jogada para o jogador atual conforme o nível de dificuldade.
///
/// # Retorna
/// Par `(quadrante, célula)` da jogada escolhida.
/// Retorna `None` se não houver jogadas disponíveis (jogo já encerrado).
pub fn best_move(board: &Board, level: AiLevel) -> Option<(usize, usize)> {
    let jogadas = rules::valid_moves(board);

    if jogadas.is_empty() {
        return None;
    }

    match level {
        AiLevel::Noob => jogar_noob(board, &jogadas),
        AiLevel::Jogadora => jogar_jogadora(board, &jogadas),
        AiLevel::Master => best_move_at_depth(board, 4),
        AiLevel::Killer => best_move_at_depth(board, 6),
    }
}

/// Nível Noob: 80% aleatório, 20% melhor jogada por "sorte".
fn jogar_noob(board: &Board, jogadas: &[(usize, usize)]) -> Option<(usize, usize)> {
    let mut rng = rand::thread_rng();

    // 20% de chance de jogar a melhor jogada acidentalmente
    if rng.gen_bool(0.20) {
        return best_move_at_depth(board, 1);
    }

    // Jogada completamente aleatória
    let idx = rng.gen_range(0..jogadas.len());
    Some(jogadas[idx])
}

/// Nível Jogadora: ganhar se puder, bloquear se necessário, senão jogar melhor posição.
fn jogar_jogadora(board: &Board, jogadas: &[(usize, usize)]) -> Option<(usize, usize)> {
    let jogador_atual = board.current_player;
    let oponente = jogador_atual.opponent();

    // Prioridade 1: pode ganhar agora?
    for &(quad, cell) in jogadas {
        let mut teste = board.clone();
        teste.make_move(quad, cell);
        if let Some(resultado) = &teste.result {
            if *resultado == crate::game::GameResult::Winner(jogador_atual) {
                return Some((quad, cell));
            }
        }
    }

    // Prioridade 2: precisa bloquear o oponente?
    for &(quad, cell) in jogadas {
        let mut teste = board.clone();
        // Simula como se fosse o oponente jogando aqui
        teste.current_player = oponente;
        teste.cells[quad][cell] = crate::game::Cell::Taken(oponente);
        use crate::game::rules::evaluate_quad;
        teste.quad_states[quad] = evaluate_quad(&teste.cells[quad]);

        if let Some(res) = crate::game::rules::check_game_result(&teste) {
            if res == crate::game::GameResult::Winner(oponente) {
                return Some((quad, cell));
            }
        }
    }

    // Prioridade 3: minimax com profundidade 1 (pega o melhor imediato)
    best_move_at_depth(board, 1)
}


