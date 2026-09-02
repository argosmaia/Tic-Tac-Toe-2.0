//! Níveis de dificuldade da IA e dispatcher de jogadas.
//!
//! Responsabilidade: expor `best_move(board, level)` que despacha para a estratégia
//! apropriada conforme o nível. Isola o resto do sistema dos detalhes de cada nível.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::game::{rules, Board};

use super::experience::{best_move_experience, best_move_experience_fallback};
use super::minimax::best_move_at_depth;

/// Nível de dificuldade da IA.
///
/// - `Noob`: aleatoriedade pura com 20% de chance de jogar a melhor jogada "por acidente"
/// - `Player`: heurística simples sem lookahead (ganhar se puder, bloquear se necessário)
/// - `Master`: Minimax com Alpha-Beta, profundidade máxima 4, heurística local
/// - `Killer`: Minimax com Alpha-Beta, profundidade máxima 6, heurística macro+micro combinada
/// - `TheExperience`: Minimax profundidade 9 + viés estatístico do histórico do jogador
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiLevel {
    Noob,
    Player,
    Master,
    Killer,
    TheExperience,
}

impl AiLevel {
    pub fn label(self) -> &'static str {
        match self {
            AiLevel::Noob => "Noob",
            AiLevel::Player => "Player",
            AiLevel::Master => "Master",
            AiLevel::Killer => "Killer 💀",
            AiLevel::TheExperience => "The Experience 🧠",
        }
    }
}

/// Calcula a melhor jogada para o jogador atual conforme o nível de dificuldade.
///
/// Para o nível `TheExperience`, use `best_move_with_heatmap` para fornecer o mapa
/// de calor do jogador. Esta função usa o fallback (minimax puro) nesse caso.
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
        AiLevel::Player => jogar_player(board, &jogadas),
        AiLevel::Master => best_move_at_depth(board, 4),
        AiLevel::Killer => best_move_at_depth(board, 6),
        AiLevel::TheExperience => best_move_experience_fallback(board),
    }
}

/// Calcula a melhor jogada para o nível "The Experience" usando o mapa de calor do jogador.
///
/// Deve ser chamada em vez de `best_move` quando o nível é `TheExperience`
/// e há histórico de jogadas do humano disponível.
///
/// # Retorna
/// Par `(quadrante, célula)` da jogada escolhida com viés estatístico.
pub fn best_move_with_heatmap(board: &Board, heatmap: &[[f32; 9]; 9]) -> Option<(usize, usize)> {
    let jogadas = rules::valid_moves(board);
    if jogadas.is_empty() {
        return None;
    }
    best_move_experience(board, heatmap)
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

/// Nível Player: ganhar se puder, bloquear se necessário, senão jogar melhor posição.
fn jogar_player(board: &Board, jogadas: &[(usize, usize)]) -> Option<(usize, usize)> {
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
