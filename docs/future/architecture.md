# Arquitetura Eficiente — Jogo da Velha 2.0

> Análise do consumo de recursos atual e proposta de mudanças arquiteturais
> para reduzir uso de memória, CPU e disco sem sacrificar funcionalidade.

---

## Diagnóstico do Estado Atual

### Pontos de Pressão Identificados

| Problema | Localização | Impacto |
|---|---|---|
| Repaint contínuo a cada 100ms | `app.rs:679` | CPU ociosa em lógica de rede |
| `Board` clonado a cada nó do minimax | `ai/minimax.rs:57` | Alocações excessivas no heap |
| Histórico carregado todo na memória | `app.rs:120-123` | Cresce indefinidamente |
| `Vec<(String, usize, usize)>` por jogada | `app.rs:55` | String alocada por turno |
| `Connection` SQLite sem pool | `storage/db.rs:84` | Locks sequenciais desnecessários |
| Font Garet embutida em memória estática | `app.rs:32` | ~120KB sempre residente |
| `tokio::runtime::Builder::new_multi_thread` | `main.rs:20` | 4-8 threads para 1 conexão P2P |
| `serde_json` para mensagens P2P | `network/manager.rs:355` | Overhead de serialização textual |

---

## Proposta 1 — Render On-Demand (Remover Repaint Forçado)

### Situação Atual

```rust
// app.rs:678-680
if precisa_repaint {
    ctx.request_repaint_after(std::time::Duration::from_millis(100));
}
```

O app pede repaint a cada 100ms enquanto a CPU ou rede está ativa. Isso equivale a 10 frames/segundo de CPU mesmo sem nada acontecer na tela.

### Solução

Usar `ctx.request_repaint()` **apenas quando um evento real ocorrer**, não periodicamente.

```rust
// ai/levels.rs — ao spawnar a task da CPU
tokio::spawn(async move {
    let jogada = best_move(&board, nivel);
    let _ = tx.send(jogada).await;
    ctx_clone.request_repaint(); // <-- já existe! Manter apenas isso.
});
```

Para a rede, o `NetworkEvent` já dispara via canal — ao drenar eventos em `processar_eventos_rede`, chamar `ctx.request_repaint()` apenas se houver evento real. Remover o loop de 100ms.

**Ganho estimado:** 80-95% de redução de uso de CPU em estados de espera.

---

## Proposta 2 — Board como Valor Compacto (Eliminar Clones no Minimax)

### Situação Atual

`Board` contém dois arrays heap-allocated indiretamente e é clonado em cada nó da árvore minimax:

```rust
// minimax.rs:57
let mut novo_tabuleiro = board.clone(); // alocação por nó
```

Com profundidade 9 e fator de ramificação ~30, isso gera ~30^9 / (poda) ~ milhares de clones intermediários.

### Solução — Board Compacto com Copy

Substituir `Board` por uma struct que caiba em 2 registradores de 64 bits:

```rust
/// Representação compacta do tabuleiro usando bitmaps.
/// Cada jogador tem 81 bits de estado (9 quads × 9 cells).
/// Cabe em 2 × u128 = 256 bits total.
#[derive(Clone, Copy)]  // <-- Copy elimina toda alocação de clone
pub struct BoardCompact {
    bits_x: u128,       // bit i = célula i pertence a X
    bits_o: u128,       // bit i = célula i pertence a O
    active_quad: u8,    // 0-8 ou 0xFF = livre
    current_player: u8, // 0=X, 1=O
    result: u8,         // 0=ongoing, 1=X wins, 2=O wins, 3=draw
}
```

**Como acessar:**
- Célula `(quad, cell)` pertence a X: `(bits_x >> (quad * 9 + cell)) & 1 == 1`
- Definir célula: `bits_x |= 1u128 << (quad * 9 + cell)`

**Ganho estimado:** Eliminação de 100% das alocações de heap no minimax. Melhora de cache (board cabe em 1-2 cache lines de 64 bytes).

**Migração:** `Board` atual permanece para a UI (legibilidade). `BoardCompact` é usado apenas dentro de `ai/minimax.rs` e `ai/heuristic.rs`, com conversão no ponto de entrada do minimax.

---

## Proposta 3 — Histórico com Paginação Preguiçosa

### Situação Atual

```rust
// app.rs:120-123
let historico_cache = db
    .as_ref()
    .and_then(|d| d.list_matches(50).ok())
    .unwrap_or_default();
```

50 registros são sempre carregados na inicialização e mantidos em `Vec<MatchRecord>` no `AppState`. Com o tempo, serão centenas de partidas por sessão.

### Solução — Cache Paginado

```rust
// storage/history.rs — nova assinatura
pub fn list_matches_page(&self, limit: u32, offset: u32) -> SqlResult<Vec<MatchRecord>>;
pub fn count_matches(&self) -> SqlResult<u32>;
```

```rust
// AppState
historico_cache: Vec<MatchRecord>,   // apenas a página atual (20 registros)
historico_pagina: u32,               // página atual
historico_total: u32,                // total para calcular número de páginas
```

A tela de histórico carrega apenas a página visível. Ao navegar, busca a próxima. O cache é invalidado apenas ao salvar nova partida.

**Ganho estimado:** O `historico_cache` que hoje retém ~50 `MatchRecord` passa a reter ~20. Cada `MatchRecord` tem ~5 `String` de ~10-20 bytes = ~100 bytes. Economiza ~3KB constantes, crescentes com o uso.

---

## Proposta 4 — Jogadas Buffer com Tipo Compacto

### Situação Atual

```rust
// app.rs:55
jogadas_buffer: Vec<(String, usize, usize)>,
```

Cada jogada aloca uma `String` com o nome do jogador. Uma partida tem ~50-80 turnos, o que significa 50-80 alocações desnecessárias durante o jogo.

### Solução — Referência ao Config

```rust
// Substituir Vec<(String, usize, usize)> por:
jogadas_buffer: Vec<(Player, usize, usize)>,
```

`Player` é um enum de 1 byte (`Copy`). O nome do jogador está em `config.nome_x` / `config.nome_o` e é resolvido apenas na hora de persistir (fim da partida), não a cada jogada.

**Ganho:** Elimina 50-80 alocações de heap por partida. Menor uso de memória durante o jogo.

---

## Proposta 5 — Runtime Tokio Enxuto

### Situação Atual

```rust
// main.rs:20-23
let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("Falha ao criar runtime tokio");
```

`new_multi_thread()` cria o número de threads igual aos núcleos da CPU (4-16 threads). Para o jogo, são usadas no máximo:
1. Uma task de rede P2P
2. Uma task de cálculo de IA

### Solução — Runtime de Thread Única com Thread Pool Mínima

```rust
// main.rs
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(2)           // IA + rede, nunca precisamos de mais
    .enable_time()               // apenas timers (para keepalive)
    .enable_io()                 // apenas I/O de rede
    .build()
    .expect("Falha ao criar runtime tokio");
```

**Ganho estimado:** Reduz de N threads para 2 threads fixas. Em uma máquina de 8 núcleos, economiza 6 threads (cada uma reserva ~8MB de stack por padrão no Linux = ~48MB de memória virtual).

---

## Proposta 6 — Serialização Binária para P2P

### Situação Atual

```rust
// network/manager.rs:355
let dados = serde_json::to_vec(msg).context("Falha ao serializar mensagem")?;
```

Mensagem de jogada em JSON:
```json
{"tipo":"jogada","quad":4,"cell":4}
```
= ~30 bytes por jogada

### Solução — Encoding Binário Compacto

Uma jogada é apenas `(quad: 4 bits, cell: 4 bits)` = **1 byte**. Com tipo de mensagem, cabe em **2 bytes**:

```rust
// network/protocol.rs
impl GameMessage {
    pub fn to_bytes(&self) -> [u8; 2] {
        match self {
            GameMessage::Jogada { quad, cell } => [0x01, (*quad as u8) << 4 | (*cell as u8)],
            GameMessage::Handshake { .. }      => [0x02, 0x00], // nome via stream separado
            GameMessage::Desistir              => [0x03, 0x00],
            GameMessage::Heartbeat             => [0x04, 0x00],
        }
    }
}
```

**Ganho:** Reduz overhead de rede de ~30 bytes para 2 bytes por jogada (93% menor). Menos pressão no buffer do QUIC. Também remove a dependência de `serde_json` no caminho crítico de rede (mantendo apenas para storage).

---

## Proposta 7 — Persistência Assíncrona Parcial (apenas resultados e jogadas)

### Situação Atual

A conexão SQLite é bloqueante e executada na thread principal do egui durante `registrar_resultado`. Embora rápida no geral, pode causar jank ao persistir dezenas de `match_moves` ao fim de uma partida.

### Escopo Crítico — O que NÃO pode ser assíncrono

> **Atenção:** `record_player_move` e `get_move_heatmap` (tabela `move_stats`) **devem permanecer síncronos**.
>
> O fluxo do nível "The Experience" é:
> 1. Humano joga → `record_player_move` escreve em `move_stats` **imediatamente**
> 2. CPU calcula → `get_move_heatmap` lê `move_stats` para montar o heatmap
>
> Se a escrita for diferida via canal assíncrono, a leitura do heatmap na mesma partida
> veria dados desatualizados — a IA perderia o padrão de jogada acabado de registrar.

### Solução — Persistência Assíncrona apenas para resultados e replay

Separar os comandos de banco em duas categorias:

```rust
enum DbCommand {
    // ASSÍNCRONO — seguro: só é lido na tela de histórico, não durante o jogo
    SalvarPartida { player_x, player_o, mode, result, duration_s, abandoned_by },
    SalvarJogadaReplay { match_id, turn, player, quad, cell },

    // SÍNCRONO — nunca assincronizar: lido pela IA na mesma partida
    // record_player_move  -> permanece chamada direta em processar_jogada
    // get_move_heatmap    -> permanece chamada direta em tick_cpu
}
```

Apenas `SalvarPartida` e `SalvarJogadaReplay` vão para o canal tokio. As escritas em
`move_stats` continuam síncronas na thread da UI — são rápidas (uma linha por turno)
e exigem consistência imediata para o "The Experience".

**Ganho:** Elimina o jank ao persistir as dezenas de `match_moves` no encerramento da partida, sem comprometer a integridade do heatmap do "The Experience".

---

## Proposta 8 — Limpeza Automática de Dados Antigos

### Situação Atual

`match_moves` acumula uma linha por jogada de cada partida. Com 50 turnos/partida e 100 partidas, são 5.000 linhas. Com 1.000 partidas, são 50.000 linhas que nunca são limpas.

### Solução — Retenção Configurável

```rust
// storage/history.rs
pub fn limpar_partidas_antigas(&self, manter_ultimas: u32) -> SqlResult<()> {
    self.conn.execute(
        "DELETE FROM match_moves WHERE match_id NOT IN (
            SELECT id FROM matches ORDER BY played_at DESC LIMIT ?1
         )",
        rusqlite::params![manter_ultimas],
    )?;
    self.conn.execute(
        "DELETE FROM matches WHERE id NOT IN (
            SELECT id FROM matches ORDER BY played_at DESC LIMIT ?1
         )",
        rusqlite::params![manter_ultimas],
    )?;
    self.conn.execute_batch("VACUUM;")?;
    Ok(())
}
```

Chamado ao iniciar a aplicação, mantendo apenas as últimas 200 partidas com suas jogadas.

**Ganho:** Banco nunca cresce acima de ~2MB (200 partidas × ~80 jogadas × ~50 bytes/linha).

---

## Proposta 9 — Heurística com SIMD (Longo Prazo)

Para `heuristic.rs`, as verificações de linhas vencedoras são 8 verificações de 3 posições sobre arrays de 9 elementos. Com `std::simd` (stabilizado no Rust nightly), é possível vetorizar estas verificações processando todos os 8 padrões simultaneamente.

```rust
// Atualmente: loop sequencial sobre 8 padrões
for linha in &LINHAS_VENCEDORAS { ... }

// Futuro: bitwise AND em u16 com máscara de padrão
const MASKS: [u16; 8] = [0b111, 0b111000, ...]; // cada linha como bitmask
let ocupacao_x: u16 = /* bits das células de X */;
let venceu = MASKS.iter().any(|&m| (ocupacao_x & m) == m);
```

Esta transformação se alinha com a Proposta 2 (BoardCompact com bitmaps).

**Ganho:** Verificação de vitória em O(1) bitwise em vez de O(8×3) iterativo.

---

## Resumo das Mudanças por Arquivo

| Arquivo | Mudança | Ganho Principal |
|---|---|---|
| `main.rs` | `worker_threads(2)` no runtime | -48MB virtual (8-core) |
| `app.rs` | Remover repaint de 100ms | -80% CPU em espera |
| `app.rs` | `jogadas_buffer: Vec<(Player, usize, usize)>` | -80 alocações/partida |
| `app.rs` | Paginação de histórico | Memória constante |
| `ai/minimax.rs` | `BoardCompact` com `Copy` | Sem alocações no minimax |
| `storage/history.rs` | Paginação + limpeza automática | Banco < 2MB sempre |
| `network/manager.rs` | Serialização binária 2 bytes | -93% bytes por jogada |
| `network/manager.rs` | Task tokio de DB | UI sem jank de disco |

---

## Ordem de Implementação Recomendada

1. **Remover repaint de 100ms** — Uma linha de código, ganho imediato e mensurável.
2. **`worker_threads(2)`** — Uma linha de código, ganho em memória virtual.
3. **`jogadas_buffer` compacto** — Mudança localizada, sem impacto em testes externos.
4. **Paginação do histórico** — Médio esforço, necessário conforme base de partidas cresce.
5. **`BoardCompact`** — Alto esforço, mas necessário para desbloquear profundidades maiores de IA.
6. **Serialização binária P2P** — Médio esforço, remove `serde_json` do caminho crítico.
7. **Limpeza automática do banco** — Baixo esforço, importante para uso de longo prazo.
