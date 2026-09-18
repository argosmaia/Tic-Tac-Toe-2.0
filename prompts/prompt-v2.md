# Análise do Codebase e Roadmap: Velha 2.0 (Ultimate Tic-Tac-Toe)

Este documento apresenta uma análise profunda do codebase atual do projeto **Velha 2.0**, identificando o que está correto (funcionando como esperado), o que está incorreto (bugs lógicos, de rede e performance), o que falta implementar (funcionalidades pendentes) e as mudanças arquiteturais necessárias.

---

## 1. O Que Está Correto (Funcionando Adequadamente)
O projeto possui uma base sólida e modular em Rust, com separação clara de responsabilidades:
- **Lógica de Domínio (`src/game/`)**: As regras do jogo, verificação de vitória de quadrantes e global, e geração de jogadas válidas estão corretas e testadas contra panics. O tabuleiro macro 9x9 (`Board`) gerencia corretamente o turno e o quadrante ativo.
- **Banco de Dados Local (`src/storage/`)**: A infraestrutura do SQLite (`Database::open` com modo WAL e migração V1 inicial) está configurada corretamente. A gravação e leitura cronológica do histórico de partidas funcionam bem.
- **Minimax e Heurísticas Básicas (`src/ai/`)**: O Minimax com Poda Alpha-Beta básico está estruturado. A função de avaliação heurística (`heuristic::evaluate`) está bem desenhada para o Ultimate Tic-Tac-Toe, pesando quadrantes macros ganhos, ameaças de vitória em linha (macro e micro) e valor posicional.
- **Interface e Apresentação (`src/ui/`)**: O design system (`theme.rs`) está bem implementado com cores espaciais escuras e neon. Os widgets como `board_widget.rs` e `player_card.rs` são stateless e funcionam de forma fluida. O carregamento de fonte Garet personalizado está ativo.
- **Estrutura Assíncrona de Rede (`src/network/manager.rs`)**: A arquitetura de rodar o gerenciador de rede em uma thread/task tokio separada, comunicando-se com a UI via canais (`NetworkCommand` e `NetworkEvent`), é excelente e segue boas práticas do egui.

---

## 2. O Que Está Errado (Bugs e Correções Críticas)

### A. Bug Crítico de Rede: Inicialização Simétrica de Streams (Hanging)
* **Arquivo**: [manager.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/network/manager.rs) (Linha 251 em `run_game_loop`)
* **Problema**: Tanto o Host quanto o Guest chamam `conn.open_bi().await` ao mesmo tempo no início de `run_game_loop`. No protocolo QUIC do `iroh`, isso faz com que ambas as pontas iniciem streams bidirecionais independentes. Como nenhuma das pontas chama `conn.accept_bi().await` para aceitar a conexão do outro, os dados enviados por um jogador nunca chegam ao outro e a partida trava (hang) indefinidamente após o handshake.
* **Correção**: A inicialização deve ser assimétrica. O Host deve abrir a stream (`open_bi`) e o Guest deve aceitá-la (`accept_bi`).
  ```rust
  // Adicionar parâmetro `is_host: bool` ao `run_game_loop`
  let (mut send_jog, mut recv_jog) = if is_host {
      conn.open_bi().await.context("Falha ao abrir stream de jogo (Host)")?
  } else {
      conn.accept_bi().await.context("Falha ao aceitar stream de jogo (Guest)")?
  };
  ```

### B. Bug de Rede/UI: Inversão de Nomes no Guest
* **Arquivo**: [app.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/app.rs) (Linha 168 em `processar_eventos_rede`)
* **Problema**: O Host joga como `Player::X` (inicia a partida) e o Guest joga como `Player::O`. No entanto, na tela do Guest, o nome local do jogador é mapeado incorretamente para `nome_x`, e o nome do Host é atribuído a `nome_o`.
  Isso causa anomalias visuais e lógicas:
  - O Guest vê o card de `Player::X` com seu próprio nome e destacado como se fosse sua vez, mas não consegue clicar pois `aguardando_peer` é `true`.
  - Quando o Host (Player X real) joga, a peça vermelha de X é exibida como se o Guest a tivesse jogado.
* **Correção**: Ajustar a atribuição de nomes no Guest ao conectar:
  ```rust
  let is_host = self.ticket_p2p.is_some();
  let config = self.lobby_state.config.clone();
  let config_com_peer = if is_host {
      LobbyConfig { nome_o: nome_peer, ..config }
  } else {
      LobbyConfig {
          nome_x: nome_peer, // Host é X
          nome_o: config.nome_x.clone(), // Guest é O
          ..config
      }
  };
  ```

### C. Ineficiência de Performance: Falta de Poda Alpha-Beta na Raiz do Minimax
* **Arquivo**: [minimax.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/ai/minimax.rs) (Linha 97 em `best_move_at_depth`)
* **Problema**: No nó raiz (`best_move_at_depth`), cada chamada para `minimax` passa os valores padrão `alpha = i32::MIN` e `beta = i32::MAX` sempre. Isso significa que os limites descobertos em um ramo filho não são propagados para os ramos seguintes na raiz. A poda Alpha-Beta é desabilitada no nível mais alto, gerando uma grande lentidão ao explorar estados complexos. Além disso, a profundidade máxima avaliada é deslocada por +1 (se depth=6, o algoritmo avalia profundidade 7 de fato).
* **Correção**: Propagar `alpha` e `beta` no loop raiz e ajustar a profundidade inicial:
  ```rust
  let mut alpha = i32::MIN;
  let mut beta = i32::MAX;
  for (quad, cell) in jogadas {
      let mut novo_tabuleiro = board.clone();
      novo_tabuleiro.make_move(quad, cell);
      let score = minimax(&novo_tabuleiro, depth - 1, alpha, beta, !maximizing);
      if maximizing {
          if score > melhor_score {
              melhor_score = score;
              melhor_jogada = (quad, cell);
          }
          alpha = alpha.max(melhor_score);
      } else {
          if score < melhor_score {
              melhor_score = score;
              melhor_jogada = (quad, cell);
          }
          beta = beta.min(melhor_score);
      }
  }
  ```

### D. Protocolo de Desistência Incompleto (Dead Code)
* **Arquivo**: [manager.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/network/manager.rs) (Linha 270) / [app.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/app.rs) (Linha 397)
* **Problema**: O enum `GameMessage::Desistir` está definido em `protocol.rs` e mapeado no `manager.rs` para disparar `NetworkEvent::PeerDesconectado`. No entanto, quando um jogador clica em "Desistir" no jogo P2P, o app apenas envia `NetworkCommand::Desconectar`, fechando a conexão QUIC diretamente. O outro peer detecta a queda como um erro genérico e não como uma desistência limpa.
* **Correção**: Modificar o comando de desistência para enviar explicitamente o pacote `GameMessage::Desistir` antes de fechar a conexão, garantindo um encerramento gracioso.

---

## 3. O Que Falta Implementar (Funcionalidades Pendentes)

### A. Tela de Gerenciamento de Perfil (👤 Perfil)
* **Arquivo**: [app.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/app.rs) (Linha 310 - `MenuAction::IrParaPerfil`)
* **Problema**: A tela de perfil está marcada como `TODO` e não existe na UI. O banco de dados já possui a tabela `profiles` e as funções de CRUD (`create_profile`, `list_profiles`, `delete_profile`), mas elas nunca são chamadas.
* **Requisito**: Implementar `src/ui/screens/profile.rs` para permitir criar, deletar e visualizar estatísticas aggregadas de vitórias/derrotas de perfis.

### B. Integração de Perfis no Lobby
* **Arquivo**: [lobby.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/ui/screens/lobby.rs)
* **Problema**: Atualmente, a seleção de nomes no lobby é feita digitando livremente em campos de texto.
* **Requisito**: Substituir (ou complementar) os inputs de texto por um seletor (combobox) dos perfis salvos no banco de dados, vinculando as partidas do histórico aos perfis reais.

### C. Persistência de Configurações (Settings)
* **Arquivo**: [db.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/storage/db.rs)
* **Problema**: A tabela `settings` é criada no SQLite (`MIGRATION_V1`), mas o módulo `storage` não expõe métodos para ler/escrever configurações, e o app não persiste nenhuma opção.
* **Requisito**: Adicionar métodos em `Database` para salvar chaves/valores de configurações (ex: volume, tema, último perfil ativo).

### D. Sistema de Save-State & Crash Recovery (Conforme Roadmap)
* **Problema**: Fechamentos abruptos do app fazem com que partidas locais (Vs CPU e Local) em andamento sejam perdidas.
* **Requisito**:
  - Salvar o estado da partida (`Board` e `LobbyConfig`) no SQLite após cada jogada de forma assíncrona/rápida.
  - Ao iniciar o app, verificar se há alguma partida inacabada no banco.
  - Se houver, exibir um botão "Retomar Partida" no menu principal.
  - **Atenção**: Partidas P2P *nunca* devem ser restauradas (devem ser expurgadas se interrompidas).

### E. Keepalive via Heartbeat P2P
* **Arquivo**: [manager.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/network/manager.rs)
* **Problema**: O enum `GameMessage::Heartbeat` está definido no protocolo, mas nunca é enviado nem tratado de forma ativa. Conexões ociosas em redes complexas podem sofrer timeout e cair.
* **Requisito**: Implementar um timer periódico no loop de rede para enviar pings (`Heartbeat`) quando não houver atividade de jogadas.

---

## 4. Mudanças Arquiteturais Recomendadas

### A. Desacoplamento da IA da Thread de UI (Async CPU Thinking)
* **Arquivo**: [app.rs](file:///home/dgti/Projetos/tic-tac-toe-2.0/src/app.rs) (Linha 233 - `tick_cpu` e `update`)
* **Problema**: A IA (Minimax de profundidade 6) roda sincronamente no loop de renderização do egui na thread principal. Em posições complexas da IA `Killer`, o cálculo do minimax pode travar a tela (bloqueio do render) por mais de um segundo, causando travamento perceptível e quebrando o 60 FPS da UI.
* **Refatoração**: A IA deve ser disparada em background.
  - Quando for o turno da CPU, disparar a computação via `tokio::spawn(async move { best_move(...) })`.
  - Enviar o resultado de volta para o estado central através de um canal de mpsc (similar à rede) ou de forma similar para ser consumido na tela de jogo sem travar a interface gráfica.
  - Exibir um indicador visual de "CPU pensando..." enquanto a task de background calcula a jogada.

### B. Gestão Centralizada de Erros
* **Problema**: Atualmente, erros de rede e banco de dados utilizam strings genéricas ou panic bounds.
* **Refatoração**: Criar um enum centralizado `AppError` em `src/error.rs` para encapsular falhas de banco (`rusqlite::Error`), rede (`anyhow::Error`), e lógica interna, permitindo que a UI renderize de forma padronizada alertas de erros para o usuário.
