//! Persistência de configurações da aplicação via SQLite.
//!
//! Responsabilidade: ler e gravar pares chave/valor na tabela `settings`.
//! Nenhuma lógica de negócio — apenas infraestrutura de preferências.

use rusqlite::Result as SqlResult;

use super::db::Database;

impl Database {
    /// Lê o valor de uma configuração pelo nome da chave.
    ///
    /// Retorna `None` se a chave não existir no banco.
    pub fn get_setting(&self, chave: &str) -> SqlResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM settings WHERE key = ?1")?;

        let mut linhas = stmt.query_map(rusqlite::params![chave], |row| {
            row.get::<_, String>(0)
        })?;

        Ok(linhas.next().transpose()?)
    }

    /// Grava (ou sobrescreve) o valor de uma configuração.
    ///
    /// Usa `INSERT OR REPLACE` — idempotente e seguro para chamadas repetidas.
    pub fn set_setting(&self, chave: &str, valor: &str) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![chave, valor],
        )?;
        Ok(())
    }
}
