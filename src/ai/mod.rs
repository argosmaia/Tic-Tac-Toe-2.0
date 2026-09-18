//! Motor de Inteligência Artificial para o Ultimate Tic-Tac-Toe.
//!
//! Depende apenas de `game/`. Não tem conhecimento de UI, rede ou banco de dados.
//! Ponto de entrada público: `levels::best_move(board, level)`
//! Para o nível "The Experience": `levels::best_move_with_heatmap(board, heatmap)`

pub mod experience;
pub mod heuristic;
pub mod levels;
pub mod minimax;

// Re-exports principais
pub use levels::{best_move, best_move_with_heatmap, AiLevel};
