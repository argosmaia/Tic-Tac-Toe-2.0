//! Protocolo de mensagens entre peers P2P.
//!
//! Todas as mensagens são serializadas com serde_json antes do envio.
//! O receptor valida a mensagem contra o estado atual do jogo antes de aplicá-la.

use serde::{Deserialize, Serialize};

use crate::game::Player;

/// Protocolo de mensagens entre peers na sessão P2P.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMessage {
    /// Jogada realizada: quadrante e célula dentro do tabuleiro.
    Move {
        quad: usize,
        cell: usize,
        player: Player,
    },
    /// Desistência — jogador encerra a partida antes do fim.
    Resign,
    /// Ping para manutenção de conexão ativa.
    Heartbeat,
    /// Informações de sessão enviadas ao peer que entrou.
    SessionInfo {
        session_id: String,
        host_name: String,
    },
}

impl GameMessage {
    /// Serializa a mensagem para JSON em bytes.
    ///
    /// # Erros
    /// Retorna erro apenas se tipos internos forem não-serializáveis (não deve ocorrer).
    pub fn to_bytes(&self) -> serde_json::Result<Vec<u8>> {
        serde_json::to_vec(self)
    }

    /// Desserializa uma mensagem de bytes JSON.
    pub fn from_bytes(bytes: &[u8]) -> serde_json::Result<Self> {
        serde_json::from_slice(bytes)
    }
}
