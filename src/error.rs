//! Gestão centralizada de erros da aplicação.
//!
//! Responsabilidade: encapsular falhas de banco, rede e lógica interna
//! em um único enum tipado, permitindo que a UI renderize alertas padronizados.

use std::fmt;

/// Erros que podem ocorrer em qualquer camada da aplicação.
#[derive(Debug)]
pub enum AppError {
    /// Falha de persistência SQLite.
    Banco(rusqlite::Error),
    /// Falha de rede ou protocolo P2P.
    Rede(anyhow::Error),
    /// Inconsistência interna — estado inesperado da lógica de domínio.
    Interno(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Banco(e) => write!(f, "Erro de banco: {e}"),
            AppError::Rede(e) => write!(f, "Erro de rede: {e}"),
            AppError::Interno(msg) => write!(f, "Erro interno: {msg}"),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Banco(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Rede(e)
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Interno(msg)
    }
}
