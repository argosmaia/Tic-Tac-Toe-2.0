//! Tipos de domínio do jogo Ultimate Tic-Tac-Toe.
//!
//! Este módulo define apenas tipos primitivos usados no domínio do jogo.
//! Não tem dependência de UI, banco de dados ou rede.
//! Qualquer alteração aqui implica em alterações em toda a lógica do jogo.

use serde::{Deserialize, Serialize};

/// Representa um dos dois jogadores: X ou O.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
}

impl Player {
    /// Retorna o oponente deste jogador.
    pub fn opponent(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }

    /// Símbolo textual do jogador para exibição.
    pub fn symbol(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }
}

/// Estado de uma célula individual dentro de um mini-tabuleiro.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cell {
    /// Célula vazia — disponível para jogada.
    #[default]
    Empty,
    /// Célula ocupada pelo jogador especificado.
    Taken(Player),
}

/// Estado de um quadrante macro após ser disputado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadState {
    /// Quadrante ainda em disputa.
    Open,
    /// Quadrante vencido pelo jogador especificado.
    Won(Player),
    /// Quadrante empatado — ninguém pode mais jogar aqui.
    Draw,
}

/// Resultado final de uma partida completa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// Partida vencida pelo jogador especificado.
    Winner(Player),
    /// Partida encerrada em empate.
    Draw,
}

/// Modo de jogo selecionado pelo usuário.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameMode {
    /// Dois jogadores na mesma máquina.
    Local,
    /// Um jogador contra a CPU.
    VsCpu,
    /// Partida P2P via iroh.
    P2P,
}

impl GameMode {
    pub fn label(self) -> &'static str {
        match self {
            GameMode::Local => "Local",
            GameMode::VsCpu => "vs CPU",
            GameMode::P2P => "P2P",
        }
    }
}
