//! Registro de partidas, jogadas individuais e estatísticas.
//!
//! Responsabilidade: salvar resultados de partidas, cada jogada realizada e calcular
//! estatísticas por perfil. Não tem lógica de jogo — apenas registros de histórico.

use rusqlite::Result as SqlResult;
use std::time::{SystemTime, UNIX_EPOCH};

use super::db::Database;

/// Registro completo de uma partida disputada.
#[derive(Debug, Clone)]
pub struct MatchRecord {
    pub id: i64,
    pub player_x: String,         // nome do perfil ou "CPU:Killer"
    pub player_o: String,
    pub mode: String,             // "local" | "p2p" | "cpu"
    pub result: String,           // "x_wins" | "o_wins" | "draw" | "abandoned"
    pub abandoned_by: Option<String>, // "x" | "o" | None
    pub duration_s: Option<i64>,
    pub played_at: i64,           // Unix timestamp
}

/// Registro de uma jogada individual dentro de uma partida.
#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub id: i64,
    pub match_id: i64,
    pub turn: u32,
    pub player: String, // "x" | "o"
    pub quad: usize,
    pub cell: usize,
}

impl MoveRecord {
    /// Retorna uma descrição curta para a tela de histórico.
    pub fn resumo(&self) -> String {
        format!(
            "movimento #{} da partida #{} · turno {} · {} · quadrante {}, célula {}",
            self.id, self.match_id, self.turn, self.player, self.quad + 1, self.cell + 1
        )
    }
}

/// Estatísticas agregadas de um perfil.
#[derive(Debug, Default)]
pub struct ProfileStats {
    pub total: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub abandoned: u32, // partidas em que o perfil desistiu
}

impl Database {
    /// Registra o resultado de uma partida no histórico.
    ///
    /// # Parâmetros
    /// - `abandoned_by`: `Some("x")` ou `Some("o")` quando houve desistência; `None` caso contrário.
    ///
    /// # Retorna
    /// O ID da partida criada, usado para vincular jogadas via `save_match_move`.
    pub fn save_match(
        &self,
        player_x: &str,
        player_o: &str,
        mode: &str,
        result: &str,
        duration_s: Option<i64>,
        abandoned_by: Option<&str>,
    ) -> SqlResult<i64> {
        let now = unix_now();

        self.conn.execute(
            "INSERT INTO matches (player_x, player_o, mode, result, duration_s, played_at, abandoned_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![player_x, player_o, mode, result, duration_s, now, abandoned_by],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Registra uma jogada individual associada a uma partida.
    ///
    /// Deve ser chamada para cada jogada (humana e CPU) após `save_match` retornar o `match_id`.
    pub fn save_match_move(
        &self,
        match_id: i64,
        turn: u32,
        player: &str,
        quad: usize,
        cell: usize,
    ) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO match_moves (match_id, turn, player, quad, cell)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![match_id, turn, player, quad as i64, cell as i64],
        )?;
        Ok(())
    }

    /// Retorna todas as jogadas de uma partida em ordem de turno.
    pub fn get_match_moves(&self, match_id: i64) -> SqlResult<Vec<MoveRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, match_id, turn, player, quad, cell
             FROM match_moves
             WHERE match_id = ?1
             ORDER BY turn ASC",
        )?;

        let moves = stmt
            .query_map(rusqlite::params![match_id], |row| {
                Ok(MoveRecord {
                    id: row.get(0)?,
                    match_id: row.get(1)?,
                    turn: row.get::<_, u32>(2)?,
                    player: row.get(3)?,
                    quad: row.get::<_, i64>(4)? as usize,
                    cell: row.get::<_, i64>(5)? as usize,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(moves)
    }

    /// Lista as últimas N partidas em ordem cronológica decrescente.
    pub fn list_matches(&self, limit: u32) -> SqlResult<Vec<MatchRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, player_x, player_o, mode, result, abandoned_by, duration_s, played_at
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
                    abandoned_by: row.get(5)?,
                    duration_s: row.get(6)?,
                    played_at: row.get(7)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;

        Ok(registros)
    }

    /// Calcula estatísticas de vitória/derrota/empate/desistência para um perfil.
    ///
    /// Considera partidas onde o perfil jogou como X ou como O.
    /// Desistências são contadas separadamente; para estatísticas de W/L,
    /// abandonar conta como derrota para quem desistiu.
    pub fn get_stats_for_profile(&self, name: &str) -> SqlResult<ProfileStats> {
        let mut stmt = self.conn.prepare(
            "SELECT player_x, player_o, result, abandoned_by FROM matches
             WHERE player_x = ?1 OR player_o = ?1",
        )?;

        let mut stats = ProfileStats::default();

        let rows = stmt.query_map(rusqlite::params![name], |row| {
            Ok((
                row.get::<_, String>(0)?, // player_x
                row.get::<_, String>(1)?, // player_o
                row.get::<_, String>(2)?, // result
                row.get::<_, Option<String>>(3)?, // abandoned_by
            ))
        })?;

        for row in rows {
            let (px, po, result, abandoned_by) = row?;
            stats.total += 1;

            let jogando_como_x = px == name;
            // Suprime warning de variável não usada
            let _ = po;

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
                "abandoned" => {
                    stats.abandoned += 1;
                    // Quem desistiu leva derrota; quem ficou leva vitória
                    match abandoned_by.as_deref() {
                        Some("x") => {
                            if jogando_como_x {
                                stats.losses += 1;
                            } else {
                                stats.wins += 1;
                            }
                        }
                        Some("o") => {
                            if !jogando_como_x {
                                stats.losses += 1;
                            } else {
                                stats.wins += 1;
                            }
                        }
                        _ => {} // abandoned sem distinção — ignora W/L
                    }
                }
                _ => {} // resultado desconhecido — ignora
            }
        }

        Ok(stats)
    }

    /// Incrementa o contador de jogadas do jogador na posição (quad, cell).
    ///
    /// Usa INSERT OR REPLACE para criar ou atualizar o registro atomicamente.
    /// Chamado a cada jogada humana na sessão vs CPU "The Experience".
    pub fn record_player_move(&self, player: &str, quad: usize, cell: usize) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO move_stats (player, quad, cell, count)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(player, quad, cell) DO UPDATE SET count = count + 1",
            rusqlite::params![player, quad as i64, cell as i64],
        )?;
        Ok(())
    }

    /// Retorna um mapa de calor 9×9 (quadrante × célula) com frequência relativa (0.0–1.0)
    /// de cada posição para o jogador informado.
    ///
    /// O valor de pico é 1.0. Se o jogador não tem histórico, retorna tudo 0.0.
    /// Usado internamente pelo nível "The Experience" para ordenar jogadas e
    /// priorizar bloqueios — o heatmap é conhecimento da IA, não exposto ao jogador.
    pub fn get_move_heatmap(&self, player: &str) -> SqlResult<[[f32; 9]; 9]> {
        let mut heatmap = [[0.0f32; 9]; 9];

        let mut stmt = self.conn.prepare(
            "SELECT quad, cell, count FROM move_stats WHERE player = ?1",
        )?;

        let rows = stmt.query_map(rusqlite::params![player], |row| {
            Ok((
                row.get::<_, i64>(0)? as usize,
                row.get::<_, i64>(1)? as usize,
                row.get::<_, i64>(2)? as f32,
            ))
        })?;

        let mut max_count = 1.0f32; // evita divisão por zero

        for row in rows {
            let (q, c, count) = row?;
            if q < 9 && c < 9 {
                heatmap[q][c] = count;
                if count > max_count {
                    max_count = count;
                }
            }
        }

        // Normaliza para 0.0–1.0
        for q in 0..9 {
            for c in 0..9 {
                heatmap[q][c] /= max_count;
            }
        }

        Ok(heatmap)
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
