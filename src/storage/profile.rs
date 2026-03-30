//! CRUD de perfis de jogador.
//!
//! Responsabilidade: criar, listar e buscar perfis no banco local.
//! Um perfil é simplesmente um nome de jogador com timestamp de criação.

use rusqlite::Result as SqlResult;
use std::time::{SystemTime, UNIX_EPOCH};

use super::db::Database;

/// Representa um perfil de jogador armazenado no banco.
#[derive(Debug, Clone)]
pub struct Profile {
    pub id: i64,
    pub name: String,
    pub created_at: i64, // Unix timestamp
}

impl Database {
    /// Cria um novo perfil com o nome fornecido.
    ///
    /// # Erros
    /// Retorna erro se o nome já existir (UNIQUE constraint) ou em caso de falha no banco.
    pub fn create_profile(&self, name: &str) -> SqlResult<Profile> {
        let now = unix_now();

        self.conn.execute(
            "INSERT INTO profiles (name, created_at) VALUES (?1, ?2)",
            rusqlite::params![name, now],
        )?;

        let id = self.conn.last_insert_rowid();

        Ok(Profile {
            id,
            name: name.to_owned(),
            created_at: now,
        })
    }

    /// Lista todos os perfis em ordem de criação.
    pub fn list_profiles(&self) -> SqlResult<Vec<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM profiles ORDER BY created_at ASC",
        )?;

        let perfis = stmt
            .query_map([], |row| {
                Ok(Profile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    created_at: row.get(2)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(perfis)
    }

    /// Busca um perfil pelo nome exato.
    ///
    /// Retorna `None` se o perfil não existir.
    pub fn get_profile_by_name(&self, name: &str) -> SqlResult<Option<Profile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM profiles WHERE name = ?1",
        )?;

        let mut rows = stmt.query_map(rusqlite::params![name], |row| {
            Ok(Profile {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;

        // Retorna o primeiro resultado (nome é unique)
        Ok(rows.next().transpose()?)
    }

    /// Remove um perfil pelo ID.
    ///
    /// # Erros
    /// Retorna erro se o ID não existir ou em caso de falha no banco.
    pub fn delete_profile(&self, id: i64) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}

/// Retorna o timestamp Unix atual em segundos.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
