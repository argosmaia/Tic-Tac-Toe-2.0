//! Regras do Ultimate Tic-Tac-Toe.
//!
//! Responsabilidade única: verificação de condições de vitória, empate,
//! geração de jogadas válidas e avanço de turno.
//! Não tem conhecimento de UI, IA, rede ou banco de dados.

use super::{
    board::Board,
    types::{Cell, GameResult, Player, QuadState},
};

/// Padrões vencedores em um tabuleiro 3×3 (índices de 0 a 8).
const WINNING_LINES: [[usize; 3]; 8] = [
    // Linhas
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    // Colunas
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    // Diagonais
    [0, 4, 8],
    [2, 4, 6],
];

/// Verifica se um jogador venceu em um array de 9 células (mini ou macro-tabuleiro).
///
/// Retorna `Some(Player)` se houver vencedor, `None` caso contrário.
pub fn check_line_winner(cells: &[bool; 9]) -> bool {
    WINNING_LINES
        .iter()
        .any(|line| line.iter().all(|&idx| cells[idx]))
}

/// Verifica o estado de um quadrante específico dado seu array de células.
///
/// # Retorna
/// - `QuadState::Won(player)` se o jogador venceu o quadrante
/// - `QuadState::Draw` se não há mais células disponíveis e ninguém venceu
/// - `QuadState::Open` se o quadrante ainda está em disputa
pub fn evaluate_quad(cells: &[Cell; 9]) -> QuadState {
    for player in [Player::X, Player::O] {
        let mask: [bool; 9] = cells
            .iter()
            .map(|&c| c == Cell::Taken(player))
            .collect::<Vec<_>>()
            .try_into()
            .expect("sempre 9 células");

        if check_line_winner(&mask) {
            return QuadState::Won(player);
        }
    }

    // Sem vencedor — verifica empate (todas preenchidas)
    if cells.iter().all(|c| *c != Cell::Empty) {
        QuadState::Draw
    } else {
        QuadState::Open
    }
}

/// Verifica o resultado do macro-tabuleiro com base nos estados dos quadrantes.
///
/// # Retorna
/// - `Some(GameResult::Winner(player))` se o jogador venceu o macro
/// - `Some(GameResult::Draw)` se todos os quadrantes estão resolvidos sem vencedor
/// - `None` se o jogo ainda está em andamento
pub fn check_game_result(board: &Board) -> Option<GameResult> {
    for player in [Player::X, Player::O] {
        let mask: [bool; 9] = board
            .quad_states
            .iter()
            .map(|&qs| qs == QuadState::Won(player))
            .collect::<Vec<_>>()
            .try_into()
            .expect("sempre 9 quadrantes");

        if check_line_winner(&mask) {
            return Some(GameResult::Winner(player));
        }
    }

    // Verifica empate: todos os quadrantes resolvidos (Won ou Draw)
    let todos_resolvidos = board
        .quad_states
        .iter()
        .all(|qs| !matches!(qs, QuadState::Open));

    if todos_resolvidos {
        Some(GameResult::Draw)
    } else {
        None
    }
}

/// Retorna a lista de jogadas válidas no estado atual do tabuleiro.
///
/// Cada jogada é um par `(quadrante, célula)` onde ambos são índices de 0 a 8.
/// Respeita a regra do Ultimate Tic-Tac-Toe: o próximo quadrante é determinado
/// pela célula jogada anteriormente.
pub fn valid_moves(board: &Board) -> Vec<(usize, usize)> {
    let mut moves = Vec::new();

    let quads_disponíveis: Vec<usize> = match board.active_quad {
        Some(q) if board.quad_states[q] == QuadState::Open => vec![q],
        // Quadrante indicado já foi resolvido: jogador pode escolher qualquer aberto
        _ => (0..9)
            .filter(|&q| board.quad_states[q] == QuadState::Open)
            .collect(),
    };

    for quad in quads_disponíveis {
        for cell in 0..9 {
            if board.cells[quad][cell] == Cell::Empty {
                moves.push((quad, cell));
            }
        }
    }

    moves
}
