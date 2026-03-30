//! Domínio puro do jogo Ultimate Tic-Tac-Toe.
//!
//! Este módulo não tem dependência de UI, banco de dados ou rede.
//! Expõe apenas tipos e lógica de domínio para uso pelos módulos superiores.

pub mod board;
pub mod rules;
pub mod types;

// Re-exports convenientes
pub use board::Board;
pub use types::{Cell, GameMode, GameResult, Player, QuadState};
