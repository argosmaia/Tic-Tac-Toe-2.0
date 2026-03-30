//! Gerenciamento de sessão P2P via iroh.
//!
//! Responsabilidade: criar e entrar em sessões P2P.
//! A sessão usa iroh para hole punching via DERP, após o qual o tráfego é direto.

use crate::network::peer::PeerStatus;

/// Identificador único de uma sessão P2P.
///
/// É o NodeAddr do host serializado em base32 — pode ser copiado e colado pelo jogador.
pub type SessionId = String;

/// Estado de uma sessão P2P ativa.
pub struct GameSession {
    pub session_id: SessionId,
    pub host_name: String,
    pub peer_status: PeerStatus,
    pub is_host: bool,
}

impl GameSession {
    /// Cria uma nova sessão como host (placeholder — implementação iroh completa futura).
    ///
    /// Em produção: inicializa um iroh Endpoint, obtém o NodeAddr e o serializa como session_id.
    pub fn new_as_host(host_name: String) -> Self {
        // Gera um session_id temporário baseado em timestamp
        let session_id = format!(
            "velha2-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        Self {
            session_id,
            host_name,
            peer_status: PeerStatus::Disconnected,
            is_host: true,
        }
    }

    /// Retorna o session ID para exibição/compartilhamento.
    pub fn display_id(&self) -> &str {
        &self.session_id
    }
}
