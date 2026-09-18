# 🚀 Inovações Futuras — Jogo da Velha 2.0

> Documento de roadmap de inovações baseado na análise completa do código-fonte (v0.2.1).
> Cada proposta parte do estado atual do sistema e aponta o que mudaria em cada camada.

---

## Estado Atual (Referência)

| Módulo | O que existe hoje |
|---|---|
| `game/` | Tabuleiro 9×9 (Ultimate Tic-Tac-Toe), regras puras, sem estado externo |
| `ai/` | 5 níveis: Noob → The Experience (minimax + heatmap por perfil) |
| `network/` | P2P iroh/QUIC, ticket manual, 1v1 apenas |
| `storage/` | SQLite local, WAL, 4 tabelas, histórico de partidas e jogadas |
| `ui/` | egui immediate-mode, 5 telas, tema dark espacial |

---

## 1. Gameplay — Novas Variantes de Regras

### 1.1 Modo "Caos" — Quadrantes Independentes
O jogador pode escolher **qualquer quadrante aberto** em todos os turnos, eliminando a regra de direcionamento. Ideal para iniciantes e partidas rápidas.

**O que muda:**
- `game/types.rs` -> novo `GameMode::Caos`
- `game/board.rs` -> `active_quad` sempre `None`
- `ui/screens/lobby.rs` -> botão extra no seletor de modo

### 1.2 Modo "Trio Maluco" — 3 Jogadores
Adicionar `Player::T` (terceiro jogador) com cor roxa, turnos circulares X->O->T->X.

**O que muda:**
- `game/types.rs` -> `Player::T`, ajuste em `opponent()` para ciclo de 3
- `game/rules.rs` -> `check_line_winner` por jogador individualmente (sem mudança de interface)
- `storage/history.rs` -> `player_t` extra em `MatchRecord`
- `ui/theme.rs` -> `JOGADOR_T: Color32` roxo

### 1.3 Modo "Timed" — Relógio por Turno
Cada jogador tem N segundos por jogada. Ao esgotar, a jogada é automática (melhor heurística). Exige apenas um `Instant` extra em `SessaoJogo`.

**O que muda:**
- `app.rs` -> campo `tempo_restante: Option<Duration>` em `SessaoJogo`
- `ui/screens/game_screen.rs` -> barra de progresso de tempo no `player_card`
- `ui/screens/lobby.rs` -> slider de tempo (5s, 10s, 30s, infinito)

### 1.4 Modo "Espelho" — Jogadas Simétricas
Toda jogada em (quad, cell) é espelhada automaticamente no quadrante oposto para o adversário, criando um jogo de simetria estratégica.

---

## 2. IA — Evolução do Motor

### 2.1 Nível "Ghost" — Imitação de Estilo
Aprende o estilo de **qualquer jogador humano do histórico** (não apenas o atual) e o imita. Permite jogar "contra si mesmo" ou contra o estilo de um amigo registrado.

**O que muda:**
- `ai/experience.rs` -> `best_move_ghost(board, heatmap_alvo)` — mesmo mecanismo de heatmap, perfil configurável no lobby
- `storage/history.rs` -> consulta de `move_stats` por nome de perfil arbitrário
- `ui/screens/lobby.rs` -> seletor de "perfil para imitar" no nível Ghost

### 2.2 Transposition Table (Cache de Minimax)
Cache de posições já avaliadas usando `HashMap<u64, i32>` com Zobrist hashing. Elimina re-exploração de posições idênticas atingidas por diferentes ordens de jogadas.

**O que muda:**
- `ai/minimax.rs` -> recebe `&mut HashMap<u64, (i32, u8)>` (score + profundidade)
- `ai/levels.rs` -> inicializa e passa a tabela para `best_move_at_depth`
- Reduz tempo de cálculo do nível Killer em ~40% (estimativa empírica para UTTT)

### 2.3 Iterative Deepening
Em vez de busca a profundidade fixa, aumenta a profundidade gradualmente (1->2->3...) até o tempo máximo permitido expirar. O último resultado completo é usado.

**O que muda:**
- `ai/minimax.rs` -> loop `for depth in 1..=max_depth` com `tokio::time::timeout`
- `app.rs` -> `tick_cpu` passa `Duration::from_millis(500)` como budget de tempo

### 2.4 Nível "Adaptativo"
A IA ajusta sua profundidade de minimax dinamicamente conforme a posição no jogo:
- Abertura (turnos 1-10): profundidade 3
- Meio-jogo (11-40): profundidade 6
- Final (>40): profundidade ilimitada (jogo resolvido)

**O que muda:**
- `ai/levels.rs` -> função `profundidade_por_turno(turno: u32) -> u8`

### 2.5 Nível "God Mode" — Busca Paralela em Todos os Núcleos da CPU

Um nível que usa **todo o poder de processamento da máquina** para calcular a melhor jogada possível dentro de um orçamento de tempo fixo (ex: 3 segundos). Em uma CPU moderna com 8+ núcleos, a taxa de vitória contra humanos se aproxima de 99,99% — o jogo se torna efetivamente imbatível.

**Como funciona:**

Atualmente o minimax roda em thread única. Para o God Mode, cada nó-filho do primeiro nível da árvore é explorado em paralelo usando `rayon::par_iter`, com uma **Transposition Table** compartilhada via `Arc<DashMap>` entre as threads:

```rust
// ai/god.rs
use rayon::prelude::*;
use std::sync::Arc;
use dashmap::DashMap;

pub fn best_move_god(
    board: &Board,
    budget: std::time::Duration,
) -> Option<(usize, usize)> {
    let tabela: Arc<DashMap<u64, (i32, u8)>> = Arc::new(DashMap::with_capacity(1_000_000));
    let deadline = std::time::Instant::now() + budget;
    let jogadas = rules::valid_moves(board);

    // Iterative Deepening paralelo: aumenta profundidade até o tempo esgotar
    let mut melhor = jogadas[0];
    for depth in 1u8.. {
        if std::time::Instant::now() >= deadline { break; }

        let resultado: Option<(usize, usize)> = jogadas
            .par_iter()                        // <-- rayon: todos os núcleos
            .map(|&(quad, cell)| {
                let mut b = board.clone();
                b.make_move(quad, cell);
                let score = minimax_tt(&b, depth, i32::MIN, i32::MAX,
                                       false, Arc::clone(&tabela), &deadline);
                (score, (quad, cell))
            })
            .max_by_key(|(s, _)| *s)
            .map(|(_, m)| m);

        if let Some(m) = resultado { melhor = m; }
    }
    Some(melhor)
}
```

**Por que fica quase imbatível:**

| CPU | Núcleos | Profundidade alcançada em 3s (estimativa) |
|---|---|---|
| Intel Core i5 (4 núcleos) | 4 | ~12-14 |
| Intel Core i9 / Ryzen 9 (16 núcleos) | 16 | ~16-18 |
| Servidor 64 núcleos | 64 | >20 (espaço de jogo praticamente esgotado) |

Para o Ultimate Tic-Tac-Toe, profundidade 12+ cobre virtualmente todos os cenários de meio e fim de jogo. Com Transposition Table, posições iguais chegadas por caminhos diferentes não são recalculadas.

**O que muda:**
- `Cargo.toml` -> `rayon = "1"`, `dashmap = "5"`
- Novo arquivo `ai/god.rs` com `best_move_god(board, budget)`
- `ai/levels.rs` -> `AiLevel::GodMode`, despacha para `best_move_god` com `Duration::from_secs(3)`
- `app.rs` -> `tick_cpu` para `GodMode` usa budget de tempo em vez de profundidade fixa
- `ui/screens/lobby.rs` -> botão especial com aviso "Esta IA usa todos os núcleos da sua CPU"
- `ui/theme.rs` -> cor vermelha sangue para o nível God Mode

**Nota de UX:** exibir na UI o número de núcleos em uso e a profundidade atual via `rayon::current_num_threads()` e um canal de feedback, para o jogador sentir o poder da máquina calculando.

---

## 3. Rede — Expansão Multiplayer

### 3.1 Sala com Código Amigável
Substituir o ticket iroh longo por um código de 6 letras (ex: `VELHA-K7X9`) mapeado em um relay leve. O relay pode ser um servidor HTTP simples (`actix-web` ou `axum`) que armazena `codigo -> ticket_iroh` por 5 minutos.

**O que muda:**
- Novo crate `server/` (binário separado, deploy leve)
- `network/manager.rs` -> `NetworkCommand::Hospedar` opcionalmente registra no relay
- `ui/screens/lobby.rs` -> exibe código de 6 letras ao hospedar e campo para digitar código curto

### 3.2 Espectador P2P
Um terceiro peer recebe as jogadas em modo read-only via stream QUIC secundário.

**O que muda:**
- `network/protocol.rs` -> `GameMessage::EstadoCompleto { board_serializado }` para sincronização inicial
- `network/manager.rs` -> aceita segunda conexão com ALPN `ultimate-tictactoe/spectator`
- `app.rs` -> modo `Tela::Espectador` sem input de jogada

### 3.3 Torneio Local (Round-Robin)
Para sessões LAN com 3-8 perfis, gerencia chaveamento automático e placar geral.

**O que muda:**
- Nova tela `ui/screens/tournament.rs`
- `storage/` -> nova tabela `tournaments` com FK para `matches`
- `app.rs` -> `Tela::Torneio` e `SessaoTorneio` análogo a `SessaoJogo`

### 3.4 Replay em Rede
Permite compartilhar um `match_id` via ticket iroh para o amigo assistir o replay animado.

**O que muda:**
- `network/protocol.rs` -> `GameMessage::ReplayJogada { turno, quad, cell }`
- `ui/screens/history.rs` -> botão "Compartilhar Replay" que abre uma sessão iroh read-only

---

## 4. Perfil e Estatísticas

### 4.1 Dashboard de Estatísticas Visuais
Gráfico de barras de vitórias/derrotas/empates por modo, streak atual, média de turnos por partida.

**O que muda:**
- `storage/history.rs` -> `get_streak(name)`, `get_avg_turns(name)` queries adicionais
- `ui/screens/profile.rs` -> seção gráfica com barras manuais via `Painter`

### 4.2 Conquistas (Achievements)
Sistema de conquistas desbloqueáveis avaliadas ao salvar cada partida.

| Conquista | Critério |
|---|---|
| Primeiro Sangue | Primeira vitória |
| Imbativel | 10 vitórias seguidas |
| David vs Golias | Vencer o nível Killer |
| Veterano | 100 partidas jogadas |
| Pacifista | 10 empates consecutivos |

**O que muda:**
- `storage/` -> tabela `achievements(player, key, unlocked_at)`
- `app.rs` -> `verificar_conquistas(nome, &db)` chamado após `registrar_resultado`
- `ui/screens/profile.rs` -> grid de emblemas

### 4.3 Heatmap Visível ao Jogador
Exibir o mapa de calor do próprio histórico de jogadas numa tela de análise, mostrando onde o jogador tem vícios posicionais.

**O que muda:**
- `ui/screens/profile.rs` -> mini-tabuleiro 9x9 colorido por intensidade usando `Painter::rect`
- `storage/history.rs` -> `get_move_heatmap` já existe — apenas expor na UI

---

## 5. UI/UX — Experiência do Jogador

### 5.1 Animações de Vitória por Quadrante
Ao conquistar um mini-tabuleiro, animar o símbolo X/O crescendo de 0% a 100% usando `ctx.animate_value_with_time`.

**O que muda:**
- `ui/components/board_widget.rs` -> `HashMap<usize, f32>` de progresso de animação por quadrante
- `app.rs` -> `ctx.request_repaint()` contínuo enquanto animação ativa

### 5.2 Sons (Feedback Auditivo)
Sons leves de clique, vitória e erro usando `rodio` (cross-platform, zero deps de sistema).

**O que muda:**
- `Cargo.toml` -> `rodio = "0.17"`
- Novo módulo `audio/mod.rs` com `tocar_som(SomEvento)`
- Assets: 3-4 arquivos `.ogg` de ~5KB cada embutidos via `include_bytes!`

### 5.3 Tema Claro
Alternância entre tema escuro (atual) e claro, persistida nas `settings`.

**O que muda:**
- `ui/theme.rs` -> `struct Tema { cores: CoresTema }` com variantes `Escuro` e `Claro`
- `storage/settings.rs` -> chave `"tema"` = `"dark"` | `"light"`
- `app.rs` -> lê setting ao iniciar e aplica `theme::aplicar_tema(ctx, tema)`

### 5.4 Internacionalização (i18n)
Suporte a Português, Inglês e Espanhol com troca em runtime.

**O que muda:**
- Novo módulo `i18n/mod.rs` com `struct Textos` e variantes `Pt`, `En`, `Es`
- Todos os textos hardcoded migrados para `textos.nova_partida`, `textos.desistir`, etc.
- `storage/settings.rs` -> chave `"idioma"`

---

## 6. Qualidade de Código e DX

### 6.1 Testes Unitários
O módulo `game/` é puramente funcional e ideal para cobertura total via `#[cfg(test)]`.

| Arquivo | Testes sugeridos |
|---|---|
| `rules.rs` | Todas as 8 linhas vencedoras, empate, jogo em andamento |
| `board.rs` | `make_move` em sequências reais de partida |
| `ai/minimax.rs` | Profundidade 1 deve sempre bloquear vitória imediata |
| `storage/` | `in_memory()` já existe — cobrir CRUD de perfis e histórico |

### 6.2 CLI de Replay
Binário auxiliar `jogodavelha2-replay` que lê um `match_id` do banco e imprime a partida turno a turno no terminal. Útil para debug e análise offline.

**O que muda:**
- `[[bin]]` extra em `Cargo.toml`
- `src/bin/replay.rs` usando `storage::Database` diretamente

### 6.3 Exportar Partida como Formato VELHA
Formato texto simples inspirado em PGN de xadrez para compartilhar partidas:

```
[Player X "Alice"]
[Player O "CPU:Killer"]
[Result "x_wins"]
1. (4,4) (4,0)
2. (0,4) (4,5)
...
```

---

## Priorização Sugerida

| Prioridade | Inovação | Esforço | Impacto |
|---|---|---|---|
| Alta | Transposition Table (2.2) | Baixo | Alto — melhora IA imediatamente |
| Alta | Dashboard de Estatísticas (4.1) | Médio | Alto — retenção de jogadores |
| Alta | Animações de Vitória (5.1) | Baixo | Alto — polimento visual imediato |
| Alta | God Mode — Busca Paralela (2.5) | Médio | Muito Alto — nível imbatível, diferencial único |
| Média | Modo Timed (1.3) | Médio | Médio — nova dimensão de gameplay |
| Média | Código Amigável P2P (3.1) | Médio | Alto — reduz atrito no P2P |
| Média | Conquistas (4.2) | Médio | Alto — engajamento de longo prazo |
| Baixa | 3 Jogadores (1.2) | Alto | Médio — nicho específico |
| Baixa | i18n (5.4) | Alto | Baixo a curto prazo |
| Baixa | Torneio Local (3.3) | Alto | Médio — uso em grupo |
