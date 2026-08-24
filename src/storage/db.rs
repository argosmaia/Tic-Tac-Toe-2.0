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
        result      TEXT NOT NULL,     -- 'x_wins' | 'o_wins' | 'draw' | 'abandoned'
        duration_s  INTEGER,           -- duração da partida em segundos
        played_at   INTEGER NOT NULL   -- Unix timestamp
    );

    CREATE TABLE IF NOT EXISTS settings (
        key         TEXT PRIMARY KEY,
        value       TEXT NOT NULL
    );
";

/// Migration V2 — tabela de estatísticas de jogadas por posição.
///
/// Usada pelo nível "The Experience" para construir o mapa de calor do jogador.
const MIGRATION_V2: &str = "
    CREATE TABLE IF NOT EXISTS move_stats (
        id      INTEGER PRIMARY KEY AUTOINCREMENT,
        player  TEXT    NOT NULL,       -- nome do perfil humano
        quad    INTEGER NOT NULL,       -- 0-8 (quadrante macro)
        cell    INTEGER NOT NULL,       -- 0-8 (célula no mini-tabuleiro)
        count   INTEGER NOT NULL DEFAULT 0,
        UNIQUE(player, quad, cell)
    );
";

/// Migration V3 — tabela de jogadas individuais por partida + coluna de desistência.
///
/// `match_moves`: cada linha é uma jogada (turno, jogador, posição) ligada a uma partida.
/// `abandoned_by`: identifica quem desistiu ('x' | 'o' | NULL), complementando o campo `result`.
///
/// As colunas novas usam ALTER TABLE com IF NOT EXISTS via PRAGMA para compatibilidade SQLite.
const MIGRATION_V3: &str = "
    CREATE TABLE IF NOT EXISTS match_moves (
        id        INTEGER PRIMARY KEY AUTOINCREMENT,
        match_id  INTEGER NOT NULL,     -- FK → matches.id
        turn      INTEGER NOT NULL,     -- número do turno (1, 2, 3…)
        player    TEXT    NOT NULL,     -- 'x' | 'o'
        quad      INTEGER NOT NULL,     -- 0-8
        cell      INTEGER NOT NULL      -- 0-8
    );
";

/// Adiciona a coluna `abandoned_by` à tabela `matches` se ainda não existir.
///
/// SQLite não suporta `ADD COLUMN IF NOT EXISTS` diretamente;
/// a abordagem segura é ignorar o erro caso a coluna já exista.
const MIGRATION_V3_ALTER: &str =
    "ALTER TABLE matches ADD COLUMN abandoned_by TEXT;";

impl Database {
    /// Abre (ou cria) o banco de dados no caminho especificado e aplica migrations.
    ///
    /// # Erros
    /// Retorna erro se o arquivo não puder ser criado ou se a migration falhar.
    pub fn open(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;

        // Performance: WAL mode para escrita concorrente sem locks longos
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Executa migrations em ordem
        conn.execute_batch(MIGRATION_V1)?;
        conn.execute_batch(MIGRATION_V2)?;
        conn.execute_batch(MIGRATION_V3)?;
        // ALTER TABLE ignora erro se coluna já existe (idempotente)
        let _ = conn.execute_batch(MIGRATION_V3_ALTER);

        Ok(Self { conn })
    }

    /// Abre banco de dados em memória (para testes).
    #[cfg(test)]
    pub fn in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(MIGRATION_V1)?;
        conn.execute_batch(MIGRATION_V2)?;
        conn.execute_batch(MIGRATION_V3)?;
        let _ = conn.execute_batch(MIGRATION_V3_ALTER);
        Ok(Self { conn })
    }
}
