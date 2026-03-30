//! Motor de Inteligência Artificial para o Ultimate Tic-Tac-Toe.
//!
//! Depende apenas de `game/`. Não tem conhecimento de UI, rede ou banco de dados.
//! Ponto de entrada público: `levels::best_move(board, level)`.

pub mod heuristic;
pub mod levels;
pub mod minimax;

// Re-export principal
pub use levels::{best_move, AiLevel};
