//! Camada de networking P2P via iroh.
//!
//! Responsabilidade: sessões P2P, protocolo de mensagens e estado de peers.
//! Depende apenas de `game/` para serialização de jogadas.

pub mod peer;
pub mod protocol;
pub mod session;

pub use peer::PeerStatus;
pub use protocol::GameMessage;
pub use session::{GameSession, SessionId};
