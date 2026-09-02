//! Estado central da aplicação e orquestração de telas.
//!
//! Responsabilidade: gerenciar navegação entre telas, injetar dependências,
//! processar eventos de jogo (jogadas humanas e da CPU) e persistir resultados.
//! É a camada de cola entre UI e domínio — não contém lógica de negócio.

use std::path::PathBuf;
use std::time::Instant;

use directories::ProjectDirs;
use eframe::CreationContext;
use egui::Context;

use crate::ai::{best_move, best_move_with_heatmap, AiLevel};
use crate::game::{rules, Board, GameMode, GameResult, Player};
use crate::network::{
    iniciar_network_manager, NetworkCommand, NetworkEvent, NetworkHandle,
};
use crate::network::peer::PeerStatus;
use crate::network::session::GameSession;
use crate::storage::Database;
use crate::ui::screens::{
    game_screen::{GameScreenAction, Placar},
    history::{HistoricoAction, HistoricoState},
    lobby::{LobbyAction, LobbyConfig, LobbyState},
    main_menu::MenuAction,
    profile::{PerfilAction, PerfilState},
};
use crate::ui::theme;

// Fonte Garet embutida no binário
const FONTE_GARET: &[u8] = include_bytes!("../assets/fonts/Garet-Book.ttf");

/// Telas disponíveis na aplicação.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Tela {
    MenuPrincipal,
    Lobby,
    Jogo,
    Historico,
    Perfil,
}

/// Sessão de jogo ativa.
struct SessaoJogo {
    board: Board,
    config: LobbyConfig,
    placar: Placar,
    inicio: Instant,
    cpu_turno: bool,       // true quando a CPU deve jogar neste frame
    aguardando_cpu: bool,  // true quando a CPU está calculando a jogada em background
    aguardando_peer: bool, // true quando é turno do peer P2P (bloqueia input local)
    /// Buffer de jogadas da partida atual (jogador, quad, cell).
    /// Drenado e persistido em `match_moves` ao fim da partida.
    jogadas_buffer: Vec<(String, usize, usize)>,
    /// Contador de turnos (incrementado a cada jogada válida).
    turno_contador: u32,
}

/// Estado global da aplicação.
pub struct AppState {
    tela_atual: Tela,
    lobby_state: LobbyState,
    sessao: Option<SessaoJogo>,
    db: Option<Database>,
    historico_cache: Vec<crate::storage::MatchRecord>,
    /// Estado da tela de histórico (filtro ativo).
    historico_state: HistoricoState,
    /// Estado da tela de perfis.
    perfil_state: PerfilState,
    /// Handle de rede P2P, presente apenas durante uma sessão P2P.
    network: Option<NetworkHandle>,
    /// Estado da sessão P2P atual, quando houver uma conexão em andamento.
    sessao_p2p: Option<GameSession>,
    /// Ticket P2P gerado pelo host, exibido no lobby para compartilhamento.
    pub ticket_p2p: Option<String>,
    /// Mensagem de status de rede ("Conectando...", "Erro: ...", etc.).
    pub status_rede: Option<String>,
    /// Transmissor de jogadas da CPU calculadas em background.
    tx_cpu_move: tokio::sync::mpsc::Sender<Option<(usize, usize)>>,
    /// Receptor de jogadas da CPU calculadas em background.
    rx_cpu_move: tokio::sync::mpsc::Receiver<Option<(usize, usize)>>,
}

impl AppState {
    /// Inicializa o estado da aplicação, abre o banco e carrega a fonte.
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Carrega a fonte Garet no contexto egui
        let mut fontes = egui::FontDefinitions::default();
        fontes.font_data.insert(
            "Garet".to_owned(),
            egui::FontData::from_static(FONTE_GARET),
        );
        // Garet como fonte primária proporcional
        fontes
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Garet".to_owned());

        cc.egui_ctx.set_fonts(fontes);

        // Aplica o tema visual do design system
        theme::aplicar_tema(&cc.egui_ctx);

        // Tenta abrir o banco de dados local
        let db = match Self::abrir_banco() {
            Ok(db) => Some(db),
            Err(error) => {
                eprintln!("Falha ao abrir banco local: {error}");
                None
            }
        };
        if let Some(database) = &db {
            let versao = database.get_setting("app_version").ok().flatten();
            if versao.as_deref() != Some(env!("CARGO_PKG_VERSION")) {
                let _ = database.set_setting("app_version", env!("CARGO_PKG_VERSION"));
            }
        }
        let historico_cache = db
            .as_ref()
            .and_then(|d| d.list_matches(50).ok())
            .unwrap_or_default();

        let (tx_cpu_move, rx_cpu_move) = tokio::sync::mpsc::channel(16);

        // Carrega perfis para o lobby e estado de perfis
        let perfis = db
            .as_ref()
            .and_then(|d| d.list_profiles().ok())
            .unwrap_or_default();

        let mut lobby_state = LobbyState::default();
        lobby_state.perfis_disponiveis = perfis.clone();

        let mut perfil_state = PerfilState::default();
        if let Some(banco) = &db {
            perfil_state.recarregar(banco);
        }

        Self {
            tela_atual: Tela::MenuPrincipal,
            lobby_state,
            sessao: None,
            db,
            historico_cache,
            historico_state: HistoricoState::default(),
            perfil_state,
            network: None,
            sessao_p2p: None,
            ticket_p2p: None,
            status_rede: None,
            tx_cpu_move,
            rx_cpu_move,
        }
    }

    /// Tenta abrir (ou criar) o banco local em `~/.velha2/data.db`.
    fn abrir_banco() -> Result<Database, crate::error::AppError> {
        let mut caminho = dirs_next_or_home();
        caminho.push("data.db");

        // Garante que o diretório pai existe
        if let Some(pai) = caminho.parent() {
            let _ = std::fs::create_dir_all(pai);
        }

        Ok(Database::open(&caminho)?)
    }

    /// Inicia uma nova sessão de jogo com a configuração do lobby.
    fn iniciar_jogo(&mut self, config: LobbyConfig) {
        self.sessao = Some(SessaoJogo {
            board: Board::new(),
            config,
            placar: Placar::default(),
            inicio: Instant::now(),
            cpu_turno: false,
            aguardando_cpu: false,
            aguardando_peer: false,
            jogadas_buffer: Vec::new(),
            turno_contador: 0,
        });
        self.tela_atual = Tela::Jogo;
    }

    /// Inicia uma sessão P2P como host.
    fn hospedar_p2p(&mut self, nosso_nome: String) {
        self.sessao_p2p = Some(GameSession::new_as_host(nosso_nome.clone()));
        let handle = iniciar_network_manager();
        let _ = handle
            .tx_cmd
            .try_send(NetworkCommand::Hospedar { nosso_nome });
        self.network = Some(handle);
        self.ticket_p2p = None;
        self.status_rede = Some("Aguardando ticket iroh...".to_owned());
    }

    /// Conecta a uma sessão P2P existente usando o ticket do host.
    fn conectar_p2p(&mut self, ticket: String, nosso_nome: String) {
        self.sessao_p2p = Some(GameSession::new_as_guest(ticket.clone(), nosso_nome.clone()));
        let handle = iniciar_network_manager();
        let _ = handle.tx_cmd.try_send(NetworkCommand::Conectar {
            ticket_str: ticket,
            nosso_nome,
        });
        self.network = Some(handle);
        self.status_rede = Some("Conectando ao host...".to_owned());
    }

    /// Drena os eventos de rede chegando do manager e atualiza o estado da UI.
    fn processar_eventos_rede(&mut self) {
        // Precisamos de mut borrow separado para o network e o resto do estado.
        let eventos: Vec<NetworkEvent> = self
            .network
            .as_mut()
            .map(|h| std::iter::from_fn(|| h.rx_evt.try_recv().ok()).collect())
            .unwrap_or_default();

        for evento in eventos {
            match evento {
                NetworkEvent::HostPronto { ticket } => {
                    if let Some(sessao) = &mut self.sessao_p2p {
                        sessao.session_id = ticket.clone();
                        let identificador = sessao.display_id().chars().take(12).collect::<String>();
                        self.status_rede = Some(format!(
                            "Ticket pronto! Sessão {}...",
                            identificador
                        ));
                    } else {
                        self.status_rede = Some("Ticket pronto! Compartilhe com seu amigo.".to_owned());
                    }
                    self.ticket_p2p = Some(ticket);
                }
                NetworkEvent::PeerConectado { nome_peer } => {
                    if let Some(sessao) = &mut self.sessao_p2p {
                        sessao.host_name = nome_peer.clone();
                        sessao.peer_status = PeerStatus::Connected {
                            peer_name: nome_peer.clone(),
                        };
                        let papel = if sessao.is_host { "host" } else { "convidado" };
                        if sessao.peer_status.is_connected() {
                            self.status_rede = Some(format!(
                                "Conectado como {} com {}",
                                papel,
                                sessao.host_name
                            ));
                        }
                    }
                    let config = self.lobby_state.config.clone();
                    let is_host = self.ticket_p2p.is_some();
                    let config_com_peer = if is_host {
                        LobbyConfig {
                            nome_o: nome_peer,
                            ..config
                        }
                    } else {
                        LobbyConfig {
                            nome_x: nome_peer, // Host é X
                            nome_o: config.nome_x.clone(), // Guest é O
                            ..config
                        }
                    };
                    self.iniciar_jogo(config_com_peer);
                    // O host joga com X (primeiro turno), guest aguarda
                    if let Some(sessao) = &mut self.sessao {
                        // guest: is_host = false → aguarda o host jogar
                        sessao.aguardando_peer = !is_host;
                    }
                }
                NetworkEvent::JogadaRecebida { quad, cell } => {
                    self.processar_jogada(quad, cell);
                    if let Some(sessao) = &mut self.sessao {
                        sessao.aguardando_peer = false;
                    }
                }
                NetworkEvent::PeerDesconectado => {
                    if let Some(sessao) = &mut self.sessao_p2p {
                        sessao.peer_status = PeerStatus::Lost;
                    }
                    self.status_rede = Some("⚠️ Amigo desconectou.".to_owned());
                    if let Some(sessao) = &mut self.sessao {
                        sessao.aguardando_peer = false;
                    }
                }
                NetworkEvent::Erro { mensagem } => {
                    if let Some(sessao) = &mut self.sessao_p2p {
                        sessao.peer_status = PeerStatus::Lost;
                    }
                    self.status_rede = Some(format!("❌ {}", mensagem));
                }
            }
        }
    }

    /// Processa uma jogada humana ou da CPU sobre a sessão ativa.
    fn processar_jogada(&mut self, quad: usize, cell: usize) {
        let Some(sessao) = &mut self.sessao else {
            return;
        };

        // Valida a jogada antes de aplicar
        let válida = rules::valid_moves(&sessao.board).contains(&(quad, cell));
        if !válida {
            return; // Jogada inválida — ignora silenciosamente
        }

        // Identifica o jogador atual antes de fazer a jogada
        let jogador_atual = sessao.board.current_player;
        let player_str = if jogador_atual == Player::X { "x" } else { "o" };

        // Atualiza o mapa de calor do humano quando vs TheExperience
        // (apenas jogadas humanas, i.e., turno de X em modo VsCpu)
        if sessao.config.modo == crate::game::GameMode::VsCpu
            && sessao.config.nivel_cpu == AiLevel::TheExperience
            && jogador_atual == Player::X
        {
            let nome_x = sessao.config.nome_x.clone();
            if let Some(db) = &self.db {
                let _ = db.record_player_move(&nome_x, quad, cell);
            }
        }

        // Incrementa turno e registra no buffer (humano e CPU)
        sessao.turno_contador += 1;
        let turno = sessao.turno_contador;
        let nome_jogador = if jogador_atual == Player::X {
            sessao.config.nome_x.clone()
        } else {
            match sessao.config.modo {
                GameMode::VsCpu => format!("CPU:{}", sessao.config.nivel_cpu),
                _ => sessao.config.nome_o.clone(),
            }
        };
        sessao.jogadas_buffer.push((nome_jogador, quad, cell));
        let _ = (player_str, turno); // usados abaixo via buffer

        let resultado = sessao.board.make_move(quad, cell);

        // Após a jogada, verifica se é turno da CPU
        if sessao.config.modo == GameMode::VsCpu
            && sessao.board.current_player == Player::O
            && resultado.is_none()
        {
            sessao.cpu_turno = true;
        }

        // Persiste o resultado se a partida terminou
        if let Some(resultado) = resultado {
            self.registrar_resultado(resultado, None);
        }
    }

    /// Drena as jogadas da CPU calculadas em background.
    fn processar_jogadas_cpu(&mut self) {
        while let Ok(jogada) = self.rx_cpu_move.try_recv() {
            if let Some(sessao) = &mut self.sessao {
                sessao.aguardando_cpu = false;
            }
            if let Some((quad, cell)) = jogada {
                // Registra a jogada da CPU pelo processador padrão
                // (inclui buffer + heatmap + resultado)
                self.processar_jogada(quad, cell);
            }
        }
    }

    /// Executa a jogada da CPU se for seu turno, disparando a busca em background.
    fn tick_cpu(&mut self, ctx: &Context) {
        let Some(sessao) = &mut self.sessao else {
            return;
        };

        if !sessao.cpu_turno || sessao.board.is_over() || sessao.aguardando_cpu {
            return;
        }

        sessao.cpu_turno = false;
        sessao.aguardando_cpu = true;

        let board = sessao.board.clone();
        let nivel = sessao.config.nivel_cpu;
        let nome_x = sessao.config.nome_x.clone();
        let tx = self.tx_cpu_move.clone();
        let ctx_clone = ctx.clone();

        // Para o nível "The Experience", carrega o heatmap do banco e usa a função específica.
        if nivel == AiLevel::TheExperience {
            let heatmap = self
                .db
                .as_ref()
                .and_then(|db| db.get_move_heatmap(&nome_x).ok())
                .unwrap_or([[0.0f32; 9]; 9]);

            tokio::spawn(async move {
                let jogada = best_move_with_heatmap(&board, &heatmap);
                let _ = tx.send(jogada).await;
                ctx_clone.request_repaint();
            });
        } else {
            tokio::spawn(async move {
                let jogada = best_move(&board, nivel);
                let _ = tx.send(jogada).await;
                ctx_clone.request_repaint();
            });
        }
    }

    /// Registra o resultado de uma partida no banco, persiste as jogadas do buffer e atualiza o placar.
    ///
    /// # Parâmetros
    /// - `abandoned_by`: `Some("x")` ou `Some("o")` em caso de desistência; `None` para fim normal.
    fn registrar_resultado(&mut self, resultado: GameResult, abandoned_by: Option<&str>) {
        let Some(sessao) = &mut self.sessao else {
            return;
        };

        // Atualiza placar da sessão (exceto desistências — placar não muda)
        if abandoned_by.is_none() {
            match resultado {
                GameResult::Winner(Player::X) => sessao.placar.pontos_x += 1,
                GameResult::Winner(Player::O) => sessao.placar.pontos_o += 1,
                GameResult::Draw => {}
            }
        }

        // Monta strings de resultado e modo
        let duração = sessao.inicio.elapsed().as_secs() as i64;
        let result_str = if abandoned_by.is_some() {
            "abandoned"
        } else {
            match resultado {
                GameResult::Winner(Player::X) => "x_wins",
                GameResult::Winner(Player::O) => "o_wins",
                GameResult::Draw => "draw",
            }
        };
        let modo_str = sessao.config.modo.label().to_lowercase();
        let nome_x = sessao.config.nome_x.clone();
        let nome_o = match sessao.config.modo {
            GameMode::VsCpu => format!("CPU:{}", sessao.config.nivel_cpu),
            _ => sessao.config.nome_o.clone(),
        };

        // Drena o buffer de jogadas para persistência
        let buffer_snapshot = std::mem::take(&mut sessao.jogadas_buffer);

        if let Some(db) = &self.db {
            // Salva a partida e obtém o ID para vincular as jogadas
            if let Ok(match_id) = db.save_match(
                &nome_x,
                &nome_o,
                &modo_str,
                result_str,
                Some(duração),
                abandoned_by,
            ) {
                // Persiste cada jogada do buffer
                for (turno, (_nome, quad, cell)) in buffer_snapshot.iter().enumerate() {
                    let player_str = if turno % 2 == 0 { "x" } else { "o" };
                    let _ = db.save_match_move(match_id, (turno + 1) as u32, player_str, *quad, *cell);
                }
            }
            // Recarrega o cache do histórico
            self.historico_cache = db.list_matches(50).unwrap_or_default();
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Drena eventos de rede (non-blocking)
        self.processar_eventos_rede();
        // Drena jogadas da CPU (non-blocking)
        self.processar_jogadas_cpu();

        // Executa turno da CPU se necessário
        self.tick_cpu(ctx);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(crate::ui::theme::cores::FUNDO_ESCURO)
                    .inner_margin(egui::Margin::same(0.0)),
            )
            .show(ctx, |ui| {
                match self.tela_atual.clone() {
                    Tela::MenuPrincipal => {
                        let ação = crate::ui::screens::main_menu::render_main_menu(ui);
                        match ação {
                            MenuAction::IrParaLobby => {
                                // Atualiza perfis disponíveis ao entrar no lobby
                                if let Some(banco) = &self.db {
                                    self.lobby_state.perfis_disponiveis =
                                        banco.list_profiles().unwrap_or_default();
                                }
                                self.tela_atual = Tela::Lobby;
                            }
                            MenuAction::IrParaHistorico => self.tela_atual = Tela::Historico,
                            MenuAction::IrParaPerfil => {
                                // Recarrega perfis ao entrar na tela
                                if let Some(banco) = &self.db {
                                    self.perfil_state.recarregar(banco);
                                }
                                self.perfil_state.mensagem_feedback = None;
                                self.tela_atual = Tela::Perfil;
                            }
                            MenuAction::Sair => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                            MenuAction::Nenhuma => {}
                        }
                    }

                    Tela::Lobby => {
                        let ação =
                            crate::ui::screens::lobby::render_lobby(
                                ui,
                                &mut self.lobby_state,
                                self.ticket_p2p.as_deref(),
                                self.status_rede.as_deref(),
                            );
                        match ação {
                            LobbyAction::IniciarPartida(config) => match config.modo {
                                GameMode::P2P => {
                                    if self.lobby_state.config.session_id_entrada.trim().is_empty() {
                                        // Hospedar
                                        self.hospedar_p2p(config.nome_x.clone());
                                    } else {
                                        // Conectar como guest
                                        let ticket = config.session_id_entrada.trim().to_owned();
                                        self.conectar_p2p(ticket, config.nome_x.clone());
                                    }
                                }
                                _ => self.iniciar_jogo(config),
                            },
                            LobbyAction::Voltar => {
                                // Cancela qualquer processo de rede em andamento
                                if let Some(h) = &self.network {
                                    let _ = h.tx_cmd.try_send(NetworkCommand::Desconectar);
                                }
                                self.network = None;
                                self.ticket_p2p = None;
                                self.status_rede = None;
                                self.tela_atual = Tela::MenuPrincipal;
                            }
                            LobbyAction::Nenhuma => {}
                        }
                    }

                    Tela::Jogo => {
                        if let Some(sessao) = &self.sessao {
                            let interativo = !sessao.cpu_turno
                                && !sessao.aguardando_cpu
                                && !sessao.aguardando_peer
                                && !sessao.board.is_over();
                            let nome_x = sessao.config.nome_x.clone();
                            let nome_o = match sessao.config.modo {
                                GameMode::VsCpu => {
                                    format!("CPU ({})", sessao.config.nivel_cpu)
                                }
                                _ => sessao.config.nome_o.clone(),
                            };
                            let board = sessao.board.clone();
                            let pontos_x = sessao.placar.pontos_x;
                            let pontos_o = sessao.placar.pontos_o;

                            let placar_render = Placar {
                                pontos_x,
                                pontos_o,
                            };

                            let ação = crate::ui::screens::game_screen::render_game_screen(
                                ui,
                                &board,
                                &nome_x,
                                &nome_o,
                                &placar_render,
                                interativo,
                            );

                            match ação {
                                GameScreenAction::JogadaRealizada { quad, cell } => {
                                    self.processar_jogada(quad, cell);
                                    // Em modo P2P, envia a jogada para o peer
                                    if let Some(h) = &self.network {
                                        let _ = h.tx_cmd.try_send(NetworkCommand::EnviarJogada { quad, cell });
                                    }
                                    // Marca que agora aguardamos o peer responder
                                    if let Some(sessao) = &mut self.sessao {
                                        if sessao.config.modo == GameMode::P2P {
                                            sessao.aguardando_peer = true;
                                        }
                                    }
                                    ctx.request_repaint();
                                }
                                GameScreenAction::Desistir => {
                                    // Registra desistência: quem clicou em desistir é sempre X (humano)
                                    // em modo VsCpu. Em modo local, quem desistiu é o jogador atual.
                                    if let Some(sess) = &self.sessao {
                                        if !sess.board.is_over() {
                                            let abandoned_by = if sess.config.modo == GameMode::VsCpu {
                                                Some("x") // humano sempre joga como X
                                            } else {
                                                // Em local/P2P, quem desistiu é o jogador atual
                                                if sess.board.current_player == Player::X {
                                                    Some("x")
                                                } else {
                                                    Some("o")
                                                }
                                            };
                                            // Usa resultado fictício para desistência
                                            let resultado_ficticio = GameResult::Winner(
                                                if abandoned_by == Some("x") { Player::O } else { Player::X },
                                            );
                                            self.registrar_resultado(resultado_ficticio, abandoned_by);
                                        }
                                    }
                                    // Notifica o peer que desistimos
                                    if let Some(h) = &self.network {
                                        let _ = h.tx_cmd.try_send(NetworkCommand::Desconectar);
                                    }
                                    self.network = None;
                                    self.ticket_p2p = None;
                                    self.status_rede = None;
                                    self.sessao = None;
                                    self.tela_atual = Tela::MenuPrincipal;
                                }
                                GameScreenAction::NovaPartida => {
                                    if let Some(sess) = &self.sessao {
                                        let config = sess.config.clone();
                                        // P2P não suporta "nova partida" direto — volta ao lobby
                                        if config.modo == GameMode::P2P {
                                            self.tela_atual = Tela::Lobby;
                                        } else {
                                            self.iniciar_jogo(config);
                                        }
                                    }
                                }
                                GameScreenAction::Nenhuma => {}
                            }
                        }
                    }

                    Tela::Historico => {
                        let ação = crate::ui::screens::history::render_historico(
                            ui,
                            &self.historico_cache,
                            &mut self.historico_state,
                            self.db.as_ref(),
                        );
                        match ação {
                            HistoricoAction::Voltar => self.tela_atual = Tela::MenuPrincipal,
                            HistoricoAction::Nenhuma => {}
                        }
                    }

                    Tela::Perfil => {
                        let ação = crate::ui::screens::profile::render_perfil(
                            ui,
                            &mut self.perfil_state,
                            self.db.as_ref(),
                        );
                        match ação {
                            PerfilAction::Voltar => self.tela_atual = Tela::MenuPrincipal,
                            PerfilAction::Nenhuma => {}
                        }
                    }
                }
            });

        // Repaint contínuo quando CPU ou rede estão ativos
        let precisa_repaint = self
            .sessao
            .as_ref()
            .map(|s| s.cpu_turno || s.aguardando_cpu || s.aguardando_peer)
            .unwrap_or(false)
            || self.network.is_some();

        if precisa_repaint {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}

/// Retorna o diretório de dados da aplicação, multiplataforma:
/// - Linux:   ~/.local/share/velha2/
/// - macOS:   ~/Library/Application Support/velha2/
/// - Windows: C:\Users\<user>\AppData\Roaming\velha2\
fn dirs_next_or_home() -> PathBuf {
    ProjectDirs::from("br", "HappyCode", "velha2")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

impl std::fmt::Display for AiLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}
