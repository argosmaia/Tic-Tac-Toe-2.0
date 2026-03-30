//! Inicialização do banco SQLite e execução de migrations.
//!
//! Responsabilidade: abrir (ou criar) o banco de dados local e garantir que o schema
//! está atualizado. Não contém lógica de negócio — apenas infraestrutura de persistência.

use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

/// Wrapper sobre a conexão SQLite com migrations automáticas na inicialização.
pub struct Database {
    pub(crate) conn: Connection,
}

/// SQL de criação do schema completo da aplicação.
///
/// As tabelas usam IF NOT EXISTS — seguro executar múltiplas vezes.
const MIGRATION_V1: &str = "
    CREATE TABLE IF NOT EXISTS profiles (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        name        TEXT NOT NULL UNIQUE,
        created_at  INTEGER NOT NULL   -- Unix timestamp
    );

    CREATE TABLE IF NOT EXISTS matches (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        player_x    TEXT NOT NULL,     -- nome do perfil ou 'CPU:Killer'
        player_o    TEXT NOT NULL,
        mode        TEXT NOT NULL,     -- 'local' | 'p2p' | 'cpu'
        result      TEXT NOT NULL,     -- 'x_wins' | 'o_wins' | 'draw'
        duration_s  INTEGER,           -- duração da partida em segundos
        played_at   INTEGER NOT NULL   -- Unix timestamp
    );

    CREATE TABLE IF NOT EXISTS settings (
        key         TEXT PRIMARY KEY,
        value       TEXT NOT NULL
    );
";

impl Database {
    /// Abre (ou cria) o banco de dados no caminho especificado e aplica migrations.
    ///
    /// # Erros
    /// Retorna erro se o arquivo não puder ser criado ou se a migration falhar.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;

        // Performance: WAL mode para escrita concorrente sem locks longos
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Executa migration inicial
        conn.execute_batch(MIGRATION_V1)?;

        Ok(Self { conn })
    }

    /// Abre banco de dados em memória (para testes).
    #[cfg(test)]
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(MIGRATION_V1)?;
        Ok(Self { conn })
    }
}
