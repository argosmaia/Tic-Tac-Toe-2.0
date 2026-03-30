//! Registro de partidas e estatísticas.
//!
//! Responsabilidade: salvar resultados de partidas e calcular estatísticas por perfil.
//! Não tem lógica de jogo — apenas registros de histórico.

use rusqlite::Result as SqlResult;
use std::time::{SystemTime, UNIX_EPOCH};

use super::db::Database;

/// Registro completo de uma partida disputada.
#[derive(Debug, Clone)]
pub struct MatchRecord {
    pub id: i64,
    pub player_x: String,    // nome do perfil ou "CPU:Killer"
    pub player_o: String,
    pub mode: String,         // "local" | "p2p" | "cpu"
    pub result: String,       // "x_wins" | "o_wins" | "draw"
    pub duration_s: Option<i64>,
    pub played_at: i64,       // Unix timestamp
}

/// Estatísticas agregadas de um perfil.
#[derive(Debug, Default)]
pub struct ProfileStats {
    pub total: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

impl Database {
    /// Registra o resultado de uma partida no histórico.
    pub fn save_match(
        &self,
        player_x: &str,
        player_o: &str,
        mode: &str,
        result: &str,
        duration_s: Option<i64>,
    ) -> SqlResult<i64> {
        let now = unix_now();

        self.conn.execute(
            "INSERT INTO matches (player_x, player_o, mode, result, duration_s, played_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![player_x, player_o, mode, result, duration_s, now],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Lista as últimas N partidas em ordem cronológica decrescente.
    pub fn list_matches(&self, limit: u32) -> SqlResult<Vec<MatchRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, player_x, player_o, mode, result, duration_s, played_at
             FROM matches
             ORDER BY played_at DESC
             LIMIT ?1",
        )?;

        let registros = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok(MatchRecord {
                    id: row.get(0)?,
                    player_x: row.get(1)?,
                    player_o: row.get(2)?,
                    mode: row.get(3)?,
                    result: row.get(4)?,
                    duration_s: row.get(5)?,
                    played_at: row.get(6)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(registros)
    }

    /// Calcula estatísticas de vitória/derrota/empate para um perfil.
    ///
    /// Considera partidas onde o perfil jogou como X ou como O.
    pub fn get_stats_for_profile(&self, name: &str) -> SqlResult<ProfileStats> {
        let mut stmt = self.conn.prepare(
            "SELECT player_x, player_o, result FROM matches
             WHERE player_x = ?1 OR player_o = ?1",
        )?;

        let mut stats = ProfileStats::default();

        let rows = stmt.query_map(rusqlite::params![name], |row| {
            Ok((
                row.get::<_, String>(0)?, // player_x
                row.get::<_, String>(1)?, // player_o
                row.get::<_, String>(2)?, // result
            ))
        })?;

        for row in rows {
            let (px, po, result) = row?;
            stats.total += 1;

            let jogando_como_x = px == name;

            match result.as_str() {
                "x_wins" => {
                    if jogando_como_x {
                        stats.wins += 1;
                    } else {
                        stats.losses += 1;
                    }
                }
                "o_wins" => {
                    if !jogando_como_x {
                        stats.wins += 1;
                    } else {
                        stats.losses += 1;
                    }
                }
                "draw" => stats.draws += 1,
                _ => {} // resultado desconhecido — ignora
            }

            // Suprime warning de variável não usada
            let _ = po;
        }

        Ok(stats)
    }
}

/// Retorna o timestamp Unix atual em segundos.
fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
