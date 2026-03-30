//! Tabuleiro do Ultimate Tic-Tac-Toe.
//!
//! Responsabilidade: manter o estado completo de uma partida e aplicar jogadas válidas.
//! Não valida regras de negócio — use `rules::valid_moves` antes de chamar `make_move`.
//! Não tem conhecimento de UI, IA, rede ou banco de dados.

use super::{
    rules::{check_game_result, evaluate_quad},
    types::{Cell, GameResult, Player, QuadState},
};

/// O tabuleiro completo de Ultimate Tic-Tac-Toe.
///
/// Composto por 9 mini-tabuleiros (quadrantes) organizados em uma grade 3×3.
/// Cada mini-tabuleiro tem 9 células. A indexação segue a convenção:
/// índice de quadrante [0..9] mapeado como grade 3×3 (0=top-left, 8=bottom-right).
/// Dentro de cada quadrante, células seguem a mesma convenção.
///
/// # Invariantes
/// - `active_quad` é `None` quando qualquer quadrante aberto pode ser jogado
/// - `active_quad` nunca aponta para um quadrante já resolvido (`QuadState != Open`)
#[derive(Debug, Clone)]
pub struct Board {
    /// Estado das 81 células (9 quadrantes × 9 células cada).
    pub cells: [[Cell; 9]; 9],
    /// Estado resolvido de cada quadrante macro.
    pub quad_states: [QuadState; 9],
    /// Quadrante onde o próximo jogador deve jogar, ou None para livre escolha.
    pub active_quad: Option<usize>,
    /// Jogador cujo turno é o atual.
    pub current_player: Player,
    /// Resultado da partida, se já encerrada.
    pub result: Option<GameResult>,
}

impl Board {
    /// Cria um novo tabuleiro zerado, com X para começar.
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; 9]; 9],
            quad_states: [QuadState::Open; 9],
            active_quad: None, // primeiro turno: livre escolha
            current_player: Player::X,
            result: None,
        }
    }

    /// Retorna `true` se a partida já terminou.
    pub fn is_over(&self) -> bool {
        self.result.is_some()
    }

    /// Aplica uma jogada no tabuleiro.
    ///
    /// # Pré-condição
    /// A jogada deve ser válida (verificada via `rules::valid_moves`).
    /// Chamar com jogada inválida causa panic em debug e comportamento indefinido em release.
    ///
    /// # Retorna
    /// O resultado da partida se ela terminou após esta jogada, ou `None` se continua.
    pub fn make_move(&mut self, quad: usize, cell: usize) -> Option<GameResult> {
        debug_assert!(quad < 9, "índice de quadrante fora do range");
        debug_assert!(cell < 9, "índice de célula fora do range");
        debug_assert!(
            self.cells[quad][cell] == Cell::Empty,
            "célula já ocupada"
        );
        debug_assert!(
            self.quad_states[quad] == QuadState::Open,
            "quadrante já resolvido"
        );

        // Registra a jogada
        self.cells[quad][cell] = Cell::Taken(self.current_player);

        // Reavalia o estado do quadrante após a jogada
        self.quad_states[quad] = evaluate_quad(&self.cells[quad]);

        // Verifica resultado global
        if let Some(resultado) = check_game_result(self) {
            self.result = Some(resultado);
            return Some(resultado);
        }

        // Determina o próximo quadrante ativo
        // A regra: o quadrante ativo é o mesmo índice da célula jogada
        self.active_quad = if self.quad_states[cell] == QuadState::Open {
            Some(cell)
        } else {
            // Quadrante-alvo já resolvido: jogador livre
            None
        };

        // Passa o turno
        self.current_player = self.current_player.opponent();

        None
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
