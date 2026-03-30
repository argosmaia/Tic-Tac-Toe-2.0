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

use crate::ai::{best_move, AiLevel};
use crate::game::{rules, Board, GameMode, GameResult, Player};
use crate::network::{
    iniciar_network_manager, NetworkCommand, NetworkEvent, NetworkHandle,
};
use crate::storage::Database;
use crate::ui::screens::{
    game_screen::{GameScreenAction, Placar},
    history::HistoricoAction,
    lobby::{LobbyAction, LobbyConfig, LobbyState},
    main_menu::MenuAction,
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
}

/// Sessão de jogo ativa.
struct SessaoJogo {
    board: Board,
    config: LobbyConfig,
    placar: Placar,
    inicio: Instant,
    cpu_turno: bool,      // true quando a CPU deve jogar neste frame
    aguardando_peer: bool, // true quando é turno do peer P2P (bloqueia input local)
}

/// Estado global da aplicação.
pub struct AppState {
    tela_atual: Tela,
    lobby_state: LobbyState,
    sessao: Option<SessaoJogo>,
    db: Option<Database>,
    historico_cache: Vec<crate::storage::MatchRecord>,
    /// Handle de rede P2P, presente apenas durante uma sessão P2P.
    network: Option<NetworkHandle>,
    /// Ticket P2P gerado pelo host, exibido no lobby para compartilhamento.
    pub ticket_p2p: Option<String>,
    /// Mensagem de status de rede ("Conectando...", "Erro: ...", etc.).
    pub status_rede: Option<String>,
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
        let db = Self::abrir_banco();
        let historico_cache = db
            .as_ref()
            .and_then(|d| d.list_matches(50).ok())
            .unwrap_or_default();

        Self {
            tela_atual: Tela::MenuPrincipal,
            lobby_state: LobbyState::default(),
            sessao: None,
            db,
            historico_cache,
            network: None,
            ticket_p2p: None,
            status_rede: None,
        }
    }

    /// Tenta abrir (ou criar) o banco local em `~/.velha2/data.db`.
    fn abrir_banco() -> Option<Database> {
        let mut caminho = dirs_next_or_home();
        caminho.push("data.db");

        // Garante que o diretório pai existe
        if let Some(pai) = caminho.parent() {
            let _ = std::fs::create_dir_all(pai);
        }

        Database::open(&caminho).ok()
    }

    /// Inicia uma nova sessão de jogo com a configuração do lobby.
    fn iniciar_jogo(&mut self, config: LobbyConfig) {
        self.sessao = Some(SessaoJogo {
            board: Board::new(),
            config,
            placar: Placar::default(),
            inicio: Instant::now(),
            cpu_turno: false,
            aguardando_peer: false,
        });
        self.tela_atual = Tela::Jogo;
    }

    /// Inicia uma sessão P2P como host.
    fn hospedar_p2p(&mut self, nosso_nome: String) {
        let mut handle = iniciar_network_manager();
        let _ = handle
            .tx_cmd
            .try_send(NetworkCommand::Hospedar { nosso_nome });
        self.network = Some(handle);
        self.ticket_p2p = None;
        self.status_rede = Some("Aguardando ticket iroh...".to_owned());
    }

    /// Conecta a uma sessão P2P existente usando o ticket do host.
    fn conectar_p2p(&mut self, ticket: String, nosso_nome: String) {
        let mut handle = iniciar_network_manager();
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
                    self.ticket_p2p = Some(ticket);
                    self.status_rede = Some("Ticket pronto! Compartilhe com seu amigo.".to_owned());
                }
                NetworkEvent::PeerConectado { nome_peer } => {
                    self.status_rede = None;
                    // Inicia a sessao com config P2P real
                    let config = self.lobby_state.config.clone();
                    let config_com_peer = LobbyConfig {
                        nome_o: nome_peer,
                        ..config
                    };
                    self.iniciar_jogo(config_com_peer);
                    // O host joga com X (primeiro turno), guest aguarda
                    if let Some(sessao) = &mut self.sessao {
                        // guest: is_host = false → aguarda o host jogar
                        sessao.aguardando_peer = self.ticket_p2p.is_none();
                    }
                }
                NetworkEvent::JogadaRecebida { quad, cell } => {
                    self.processar_jogada(quad, cell);
                    if let Some(sessao) = &mut self.sessao {
                        sessao.aguardando_peer = false;
                    }
                }
                NetworkEvent::PeerDesconectado => {
                    self.status_rede = Some("⚠️ Amigo desconectou.".to_owned());
                    if let Some(sessao) = &mut self.sessao {
                        sessao.aguardando_peer = false;
                    }
                }
                NetworkEvent::Erro { mensagem } => {
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
        let válida = rules::valid_moves(&sessao.board)
            .contains(&(quad, cell));

        if !válida {
            return; // Jogada inválida — ignora silenciosamente
        }

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
            self.registrar_resultado(resultado);
        }
    }

    /// Executa a jogada da CPU se for seu turno.
    fn tick_cpu(&mut self) {
        let Some(sessao) = &mut self.sessao else {
            return;
        };

        if !sessao.cpu_turno || sessao.board.is_over() {
            return;
        }

        sessao.cpu_turno = false;

        let nivel = sessao.config.nivel_cpu;
        if let Some((quad, cell)) = best_move(&sessao.board, nivel) {
            let resultado = sessao.board.make_move(quad, cell);

            if let Some(resultado) = resultado {
                self.registrar_resultado(resultado);
            }
        }
    }

    /// Registra o resultado de uma partida no banco e atualiza o placar.
    fn registrar_resultado(&mut self, resultado: GameResult) {
        let Some(sessao) = &mut self.sessao else {
            return;
        };

        // Atualiza placar da sessão
        match resultado {
            GameResult::Winner(Player::X) => sessao.placar.pontos_x += 1,
            GameResult::Winner(Player::O) => sessao.placar.pontos_o += 1,
            GameResult::Draw => {}
        }

        // Persiste no banco
        let duração = sessao.inicio.elapsed().as_secs() as i64;
        let result_str = match resultado {
            GameResult::Winner(Player::X) => "x_wins",
            GameResult::Winner(Player::O) => "o_wins",
            GameResult::Draw => "draw",
        };
        let modo_str = sessao.config.modo.label().to_lowercase();
        let nome_x = sessao.config.nome_x.clone();
        let nome_o = match sessao.config.modo {
            GameMode::VsCpu => format!("CPU:{}", sessao.config.nivel_cpu),
            _ => sessao.config.nome_o.clone(),
        };

        if let Some(db) = &self.db {
            let _ = db.save_match(&nome_x, &nome_o, &modo_str, result_str, Some(duração));
            // Recarrega o cache
            self.historico_cache = db.list_matches(50).unwrap_or_default();
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Drena eventos de rede (non-blocking)
        self.processar_eventos_rede();

        // Executa turno da CPU antes de renderizar (evita frame em branco)
        self.tick_cpu();

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
                            MenuAction::IrParaLobby => self.tela_atual = Tela::Lobby,
                            MenuAction::IrParaHistorico => self.tela_atual = Tela::Historico,
                            MenuAction::IrParaPerfil => { /* TODO: tela de perfil */ }
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
                        );
                        match ação {
                            HistoricoAction::Voltar => self.tela_atual = Tela::MenuPrincipal,
                            HistoricoAction::Nenhuma => {}
                        }
                    }
                }
            });

        // Repaint contínuo quando CPU ou rede estão ativos
        let precisa_repaint = self
            .sessao
            .as_ref()
            .map(|s| s.cpu_turno || s.aguardando_peer)
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
