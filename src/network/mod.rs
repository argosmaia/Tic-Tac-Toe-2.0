//! Camada de networking P2P via iroh 0.29.
//!
//! Responsabilidade: initiate/accept conexões P2P com hole punching via DERP servers da n0.
//! Toda a I/O de rede acontece em tasks tokio separadas — nunca bloqueia a UI.

pub mod manager;
pub mod peer;
pub mod protocol;
pub mod session;

pub use manager::{iniciar_network_manager, NetworkCommand, NetworkEvent, NetworkHandle};
pub use protocol::GameMessage;
