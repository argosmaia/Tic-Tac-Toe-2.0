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
    cpu_turno: bool, // true quando a CPU deve jogar neste frame
}

/// Estado global da aplicação.
pub struct AppState {
    tela_atual: Tela,
    lobby_state: LobbyState,
    sessao: Option<SessaoJogo>,
    db: Option<Database>,
    historico_cache: Vec<crate::storage::MatchRecord>,
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
        let cpu_joga_primeiro = config.modo == GameMode::VsCpu
            && Player::X == Player::O; // CPU sempre joga como O → nunca é o primeiro turno dela

        self.sessao = Some(SessaoJogo {
            board: Board::new(),
            config,
            placar: Placar::default(),
            inicio: Instant::now(),
            cpu_turno: false,
        });

        self.tela_atual = Tela::Jogo;
        let _ = cpu_joga_primeiro;
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
                            crate::ui::screens::lobby::render_lobby(ui, &mut self.lobby_state);
                        match ação {
                            LobbyAction::IniciarPartida(config) => self.iniciar_jogo(config),
                            LobbyAction::Voltar => self.tela_atual = Tela::MenuPrincipal,
                            LobbyAction::Nenhuma => {}
                        }
                    }

                    Tela::Jogo => {
                        if let Some(sessao) = &self.sessao {
                            let interativo = !sessao.cpu_turno && !sessao.board.is_over();
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
                                    ctx.request_repaint(); // Garante repaint para CPU jogar
                                }
                                GameScreenAction::Desistir => {
                                    self.sessao = None;
                                    self.tela_atual = Tela::MenuPrincipal;
                                }
                                GameScreenAction::NovaPartida => {
                                    if let Some(sess) = &self.sessao {
                                        let config = sess.config.clone();
                                        self.iniciar_jogo(config);
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

        // Repaint contínuo quando CPU está pensando
        if let Some(sess) = &self.sessao {
            if sess.cpu_turno {
                ctx.request_repaint();
            }
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
