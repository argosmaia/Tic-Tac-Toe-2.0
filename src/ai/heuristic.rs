//! Heurística de avaliação de tabuleiro para os níveis Master e Killer da IA.
//!
//! Responsabilidade: calcular um score numérico do tabuleiro do ponto de vista de um jogador.
//! Score positivo = vantagem para X. Score negativo = vantagem para O.
//! Não toma decisões de jogada — apenas avalia o estado atual.

use crate::game::{Board, Cell, Player, QuadState};

/// Valor de cada posição em um tabuleiro 3×3 (centro > cantos > arestas).
/// Reflete a importância estratégica posicional.
const VALOR_POSICIONAL: [i32; 9] = [
    3, 1, 3, // top-left, top-center, top-right
    1, 5, 1, // mid-left,    center,  mid-right
    3, 1, 3, // bot-left, bot-center, bot-right
];

/// Padrões vencedores — os 8 alinhamentos possíveis em um tabuleiro 3×3.
const LINHAS_VENCEDORAS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// Avalia o tabuleiro completo e retorna um score numérico.
///
/// Score positivo favorece X. Score negativo favorece O.
/// Chamado pelo minimax quando a profundidade limite é atingida.
///
/// # Critérios de avaliação
/// - Quadrantes ganhos no macro-tabuleiro (peso alto)
/// - Ameaças de vitória em dois níveis (macro e micro)
/// - Valor posicional de centro e cantos (macro e micro)
pub fn evaluate(board: &Board) -> i32 {
    let mut score = 0i32;

    // 1. Avaliação do macro-tabuleiro (quadrantes resolvidos)
    for (idx, &estado) in board.quad_states.iter().enumerate() {
        match estado {
            QuadState::Won(Player::X) => {
                // Quadrante ganho vale posição + bônus fixo
                score += 100 + VALOR_POSICIONAL[idx] * 10;
            }
            QuadState::Won(Player::O) => {
                score -= 100 + VALOR_POSICIONAL[idx] * 10;
            }
            _ => {}
        }
    }

    // 2. Ameaças no macro-tabuleiro (dois quadrantes ganhos numa linha)
    score += avaliar_ameaças_macro(board);

    // 3. Avaliação de cada mini-tabuleiro ainda aberto
    for (quad_idx, &estado_quad) in board.quad_states.iter().enumerate() {
        if estado_quad != QuadState::Open {
            continue; // Quadrante já resolvido — sem pontos extras por células
        }
        score += avaliar_mini_tabuleiro(board, quad_idx);
    }

    score
}

/// Avalia ameaças de vitória no macro-tabuleiro (dois quadrantes ganhos por linha).
fn avaliar_ameaças_macro(board: &Board) -> i32 {
    let mut score = 0i32;

    for linha in &LINHAS_VENCEDORAS {
        let (x_count, o_count, open_count) = contar_linha_quad(board, linha);

        // Dois quadrantes ganhos numa linha = ameaça forte
        if x_count == 2 && open_count == 1 {
            score += 50;
        }
        if o_count == 2 && open_count == 1 {
            score -= 50;
        }
        // Um quadrante ganho numa linha = ameaça moderada
        if x_count == 1 && open_count == 2 {
            score += 10;
        }
        if o_count == 1 && open_count == 2 {
            score -= 10;
        }
    }

    score
}

/// Conta quantos quadrantes numa linha pertencem a X, O e estão abertos.
fn contar_linha_quad(board: &Board, linha: &[usize; 3]) -> (i32, i32, i32) {
    let mut x = 0i32;
    let mut o = 0i32;
    let mut open = 0i32;

    for &idx in linha {
        match board.quad_states[idx] {
            QuadState::Won(Player::X) => x += 1,
            QuadState::Won(Player::O) => o += 1,
            QuadState::Open => open += 1,
            QuadState::Draw => {} // neutro
        }
    }

    (x, o, open)
}

/// Avalia um mini-tabuleiro individual pelo valor posicional das células.
fn avaliar_mini_tabuleiro(board: &Board, quad_idx: usize) -> i32 {
    let mut score = 0i32;
    let células = &board.cells[quad_idx];

    for (idx, &célula) in células.iter().enumerate() {
        match célula {
            Cell::Taken(Player::X) => score += VALOR_POSICIONAL[idx],
            Cell::Taken(Player::O) => score -= VALOR_POSICIONAL[idx],
            Cell::Empty => {}
        }
    }

    // Ameaças dentro do mini-tabuleiro
    for linha in &LINHAS_VENCEDORAS {
        let (x_cnt, o_cnt, open_cnt) = contar_linha_células(células, linha);
        if x_cnt == 2 && open_cnt == 1 {
            score += 8;
        }
        if o_cnt == 2 && open_cnt == 1 {
            score -= 8;
        }
    }

    score
}

/// Conta células de X, O e vazias numa linha específica.
fn contar_linha_células(células: &[Cell; 9], linha: &[usize; 3]) -> (i32, i32, i32) {
    let mut x = 0i32;
    let mut o = 0i32;
    let mut open = 0i32;

    for &idx in linha {
        match células[idx] {
            Cell::Taken(Player::X) => x += 1,
            Cell::Taken(Player::O) => o += 1,
            Cell::Empty => open += 1,
        }
    }

    (x, o, open)
}
