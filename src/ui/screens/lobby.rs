//! Tela de lobby: seleção de modo de jogo e configuração de partida.

use egui::Ui;

use crate::ai::AiLevel;
use crate::game::GameMode;
use crate::ui::theme::{cores, espacamentos, tipografia};

/// Configuração completa de uma partida montada no lobby.
#[derive(Debug, Clone)]
pub struct LobbyConfig {
    pub modo: GameMode,
    pub nome_x: String,
    pub nome_o: String,
    pub nivel_cpu: AiLevel,
    pub session_id_entrada: String, // Para modo P2P guest
}

impl Default for LobbyConfig {
    fn default() -> Self {
        Self {
            modo: GameMode::Local,
            nome_x: String::from("Jogador X"),
            nome_o: String::from("Jogador O"),
            nivel_cpu: AiLevel::Jogadora,
            session_id_entrada: String::new(),
        }
    }
}

/// Ação solicitada na tela de lobby.
#[derive(Debug, Clone)]
pub enum LobbyAction {
    /// Iniciar uma partida com a configuração atual.
    IniciarPartida(LobbyConfig),
    /// Voltar ao menu principal.
    Voltar,
    /// Nenhuma ação — aguardando input.
    Nenhuma,
}

/// Estado interno da tela de lobby.
pub struct LobbyState {
    pub config: LobbyConfig,
}

impl Default for LobbyState {
    fn default() -> Self {
        Self {
            config: LobbyConfig::default(),
        }
    }
}

/// Renderiza a tela de lobby.
pub fn render_lobby(
    ui: &mut Ui,
    estado: &mut LobbyState,
    ticket_p2p: Option<&str>,
    status_rede: Option<&str>,
) -> LobbyAction {
    let mut ação = LobbyAction::Nenhuma;

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(32.0);

        ui.label(
            egui::RichText::new("Nova Partida")
                .size(tipografia::TITULO)
                .color(cores::TEXTO_PRIMARIO)
                .strong(),
        );

        ui.add_space(24.0);

        // Seleção de modo
        ui.label(
            egui::RichText::new("Modo de jogo")
                .size(tipografia::SUBTITULO)
                .color(cores::TEXTO_SECUNDARIO),
        );

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            for modo in [GameMode::Local, GameMode::VsCpu, GameMode::P2P] {
                let selecionado = estado.config.modo == modo;
                let cor_fundo = if selecionado {
                    cores::BOTAO_PRIMARIO
                } else {
                    cores::SUPERFICIE_ELEVADA
                };

                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(modo.label())
                                .color(cores::BOTAO_TEXTO)
                                .size(tipografia::CORPO),
                        )
                        .fill(cor_fundo)
                        .rounding(espacamentos::RAIO_BORDA),
                    )
                    .clicked()
                {
                    estado.config.modo = modo;
                }
            }
        });

        ui.add_space(20.0);

        // Campos de nome conforme o modo
        match estado.config.modo {
            GameMode::Local => {
                campo_nome(ui, "Jogador X", &mut estado.config.nome_x, cores::JOGADOR_X);
                ui.add_space(8.0);
                campo_nome(ui, "Jogador O", &mut estado.config.nome_o, cores::JOGADOR_O);
            }
            GameMode::VsCpu => {
                campo_nome(ui, "Seu nome", &mut estado.config.nome_x, cores::JOGADOR_X);
                ui.add_space(8.0);

                ui.label(
                    egui::RichText::new("Dificuldade da CPU")
                        .size(tipografia::CORPO)
                        .color(cores::TEXTO_SECUNDARIO),
                );
                ui.horizontal(|ui| {
                    for nivel in [
                        AiLevel::Noob,
                        AiLevel::Jogadora,
                        AiLevel::Master,
                        AiLevel::Killer,
                    ] {
                        let selecionado = estado.config.nivel_cpu == nivel;
                        let cor_fundo = if selecionado {
                            cores::BOTAO_PRIMARIO
                        } else {
                            cores::SUPERFICIE_ELEVADA
                        };
                        let cor_texto = if nivel == AiLevel::Killer {
                            cores::ACENTO_DOURADO
                        } else {
                            cores::BOTAO_TEXTO
                        };

                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new(nivel.label())
                                        .size(tipografia::PEQUENO)
                                        .color(cor_texto),
                                )
                                .fill(cor_fundo)
                                .rounding(espacamentos::RAIO_BORDA),
                            )
                            .clicked()
                        {
                            estado.config.nivel_cpu = nivel;
                        }
                    }
                });
            }
            GameMode::P2P => {
                campo_nome(ui, "Seu nome", &mut estado.config.nome_x, cores::JOGADOR_X);
                ui.add_space(8.0);

                // Exibe ticket se já foi gerado (modo host aguardando peer)
                if let Some(ticket) = ticket_p2p {
                    ui.label(
                        egui::RichText::new("🎫 Ticket — envie ao seu amigo:")
                            .size(tipografia::CORPO)
                            .color(cores::TEXTO_SECUNDARIO),
                    );
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut ticket.to_owned())
                                .desired_width(240.0)
                                .interactive(false),
                        );
                        if ui.button("📋 Copiar").clicked() {
                            ui.ctx().copy_text(ticket.to_owned());
                        }
                    });
                } else {
                    ui.label(
                        egui::RichText::new("Ticket do host (deixe vazio para hospedar)")
                            .size(tipografia::CORPO)
                            .color(cores::TEXTO_SECUNDARIO),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut estado.config.session_id_entrada)
                            .hint_text("Cole o ticket do seu amigo aqui...")
                            .desired_width(280.0),
                    );
                }

                // Status de rede
                if let Some(status) = status_rede {
                    ui.add_space(8.0);
                    let cor_status = if status.starts_with('❌') {
                        cores::JOGADOR_X
                    } else if status.starts_with('⚠') {
                        cores::ACENTO_DOURADO
                    } else {
                        cores::JOGADOR_O
                    };
                    ui.label(
                        egui::RichText::new(status)
                            .size(tipografia::CORPO)
                            .color(cor_status),
                    );
                }
            }
        }

        ui.add_space(32.0);

        // Botão iniciar
        if ui
            .add_sized(
                [200.0, 44.0],
                egui::Button::new(
                    egui::RichText::new("▶  Iniciar")
                        .size(tipografia::CORPO)
                        .color(cores::BOTAO_TEXTO),
                )
                .fill(cores::BOTAO_PRIMARIO)
                .rounding(espacamentos::RAIO_BORDA),
            )
            .clicked()
        {
            ação = LobbyAction::IniciarPartida(estado.config.clone());
        }

        ui.add_space(12.0);

        if ui
            .add(
                egui::Button::new(
                    egui::RichText::new("← Voltar")
                        .size(tipografia::CORPO)
                        .color(cores::TEXTO_MUDO),
                )
                .fill(egui::Color32::TRANSPARENT),
            )
            .clicked()
        {
            ação = LobbyAction::Voltar;
        }
    });

    ação
}

/// Renderiza um campo de entrada de nome com label colorida.
fn campo_nome(ui: &mut Ui, label: &str, valor: &mut String, cor_label: egui::Color32) {
    ui.label(
        egui::RichText::new(label)
            .size(tipografia::CORPO)
            .color(cor_label),
    );
    ui.add(egui::TextEdit::singleline(valor).desired_width(280.0));
}
