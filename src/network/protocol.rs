//! Protocolo de mensagens da rede P2P Velha 2.0.
//!
//! Serializado como JSON e transportado sobre streams QUIC do iroh.
//! Cada mensagem é precedida por 4 bytes (big-endian) com o tamanho do payload.

use serde::{Deserialize, Serialize};

/// Mensagens trocadas entre os dois peers durante uma sessão P2P.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum GameMessage {
    /// Troca de nomes ao estabelecer a conexão.
    Handshake { nome: String },

    /// Jogada realizada por um peer.
    Jogada { quad: usize, cell: usize },

    /// Peer está desistindo da partida.
    Desistir,

    /// Mensagem keepalive (previne timeouts em conexões ociosas).
    Heartbeat,
}
