//! Camada de persistência local via SQLite.
//!
//! Responsabilidade: acesso a dados de perfis, partidas e configurações.
//! Independente de UI, jogo e rede.

pub mod db;
pub mod history;
pub mod profile;

pub use db::Database;
pub use history::{MatchRecord, ProfileStats};
pub use profile::Profile;
