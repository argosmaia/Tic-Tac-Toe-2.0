//! Estado de conexão e ciclo de vida do peer P2P.
//!
//! Responsabilidade: rastrear o estado atual da conexão com o peer remoto.

/// Estado atual da conexão com o peer remoto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerStatus {
    /// Sem conexão ativa.
    Disconnected,
    /// Tentando conectar ao peer.
    Connecting,
    /// Conexão estabelecida e pronta para jogo.
    Connected { peer_name: String },
    /// Conexão perdida inesperadamente.
    Lost,
}

impl PeerStatus {
    /// Retorna `true` se a conexão está estabelecida.
    pub fn is_connected(&self) -> bool {
        matches!(self, PeerStatus::Connected { .. })
    }
}

impl Default for PeerStatus {
    fn default() -> Self {
        PeerStatus::Disconnected
    }
}
