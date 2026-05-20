//! Tela de gerenciamento de perfis de jogador.
//!
//! Permite criar, deletar e visualizar estatísticas (V/D/E) por perfil.
//! Consome os dados de `Database` via referência — sem lógica de domínio aqui.

use egui::Ui;

use crate::storage::{Database, Profile, ProfileStats};
use crate::ui::theme::{cores, espacamentos, tipografia};

/// Ação solicitada na tela de perfil.
#[derive(Debug, Clone)]
pub enum PerfilAction {
    /// Voltar ao menu principal.
    Voltar,
    /// Nenhuma ação — frame normal.
    Nenhuma,
}

/// Estado interno da tela de perfil.
pub struct PerfilState {
    /// Nome digitado no campo de criação de perfil.
    pub nome_novo: String,
    /// Mensagem de feedback (erro ou sucesso) da última operação.
    pub mensagem_feedback: Option<(String, bool)>, // (texto, é_erro)
    /// Cache de perfis com suas estatísticas (recarregado sob demanda).
    pub perfis: Vec<(Profile, ProfileStats)>,
}

impl Default for PerfilState {
    fn default() -> Self {
        Self {
            nome_novo: String::new(),
            mensagem_feedback: None,
            perfis: Vec::new(),
        }
    }
}

impl PerfilState {
    /// Recarrega a lista de perfis e estatísticas do banco.
    pub fn recarregar(&mut self, db: &Database) {
        self.perfis = db
            .list_profiles()
            .unwrap_or_default()
            .into_iter()
            .map(|perfil| {
                let stats = db
                    .get_stats_for_profile(&perfil.name)
                    .unwrap_or_default();
                (perfil, stats)
            })
            .collect();
    }
}

/// Renderiza a tela de gerenciamento de perfis.
///
/// Recebe referência ao banco para operações CRUD e ao estado local de UI.
pub fn render_perfil(
    ui: &mut Ui,
    estado: &mut PerfilState,
    db: Option<&Database>,
) -> PerfilAction {
    let mut ação = PerfilAction::Nenhuma;

    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        ui.add_space(32.0);

        // Título
        ui.label(
            egui::RichText::new("👤  Perfis")
                .size(tipografia::TITULO)
                .color(cores::TEXTO_PRIMARIO)
                .strong(),
        );

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Gerencie seus perfis e veja suas estatísticas")
                .size(tipografia::CORPO)
                .color(cores::TEXTO_SECUNDARIO),
        );

        ui.add_space(24.0);

        // ── Criação de novo perfil ─────────────────────────────────
        ui.group(|ui| {
            ui.set_width(360.0);
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("Novo Perfil")
                        .size(tipografia::SUBTITULO)
                        .color(cores::TEXTO_PRIMARIO),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut estado.nome_novo)
                            .hint_text("Nome do jogador...")
                            .desired_width(230.0),
                    );

                    let pode_criar = !estado.nome_novo.trim().is_empty();
                    let botao_criar = ui.add_enabled(
                        pode_criar,
                        egui::Button::new(
                            egui::RichText::new("✚ Criar")
                                .size(tipografia::PEQUENO)
                                .color(cores::BOTAO_TEXTO),
                        )
                        .fill(if pode_criar {
                            cores::BOTAO_PRIMARIO
                        } else {
                            cores::SUPERFICIE_ELEVADA
                        })
                        .rounding(espacamentos::RAIO_BORDA),
                    );

                    if botao_criar.clicked() {
                        if let Some(banco) = db {
                            let nome = estado.nome_novo.trim().to_owned();
                            match banco.create_profile(&nome) {
                                Ok(_) => {
                                    estado.mensagem_feedback =
                                        Some((format!("Perfil \"{}\" criado! ✨", nome), false));
                                    estado.nome_novo.clear();
                                    estado.recarregar(banco);
                                }
                                Err(e) => {
                                    // Nome duplicado é o erro mais comum
                                    let msg = if e.to_string().contains("UNIQUE") {
                                        format!("Já existe um perfil com o nome \"{}\".", nome)
                                    } else {
                                        format!("Erro ao criar perfil: {e}")
                                    };
                                    estado.mensagem_feedback = Some((msg, true));
                                }
                            }
                        } else {
                            estado.mensagem_feedback =
                                Some(("Banco de dados não disponível.".to_owned(), true));
                        }
                    }
                });

                // Feedback de operação
                if let Some((msg, é_erro)) = &estado.mensagem_feedback {
                    ui.add_space(6.0);
                    let cor = if *é_erro {
                        cores::JOGADOR_X
                    } else {
                        cores::JOGADOR_O
                    };
                    ui.label(
                        egui::RichText::new(msg.as_str())
                            .size(tipografia::PEQUENO)
                            .color(cor),
                    );
                }
            });
        });

        ui.add_space(20.0);

        // ── Lista de perfis existentes ─────────────────────────────
        if estado.perfis.is_empty() {
            ui.label(
                egui::RichText::new("Nenhum perfil cadastrado ainda.")
                    .size(tipografia::CORPO)
                    .color(cores::TEXTO_MUDO),
            );
        } else {
            ui.label(
                egui::RichText::new("Perfis cadastrados")
                    .size(tipografia::SUBTITULO)
                    .color(cores::TEXTO_PRIMARIO),
            );
            ui.add_space(8.0);

            let mut id_para_deletar: Option<i64> = None;

            egui::ScrollArea::vertical()
                .max_height(280.0)
                .show(ui, |ui| {
                    for (perfil, stats) in &estado.perfis {
                        ui.group(|ui| {
                            ui.set_width(360.0);
                            ui.horizontal(|ui| {
                                // Nome e stats
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&perfil.name)
                                            .size(tipografia::CORPO)
                                            .color(cores::TEXTO_PRIMARIO)
                                            .strong(),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "🏆 {}V  💔 {}D  🤝 {}E  |  {} partidas",
                                            stats.wins,
                                            stats.losses,
                                            stats.draws,
                                            stats.total
                                        ))
                                        .size(tipografia::PEQUENO)
                                        .color(cores::TEXTO_SECUNDARIO),
                                    );
                                });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    egui::RichText::new("🗑")
                                                        .size(tipografia::CORPO)
                                                        .color(cores::JOGADOR_X),
                                                )
                                                .fill(egui::Color32::TRANSPARENT),
                                            )
                                            .on_hover_text("Deletar perfil")
                                            .clicked()
                                        {
                                            id_para_deletar = Some(perfil.id);
                                        }
                                    },
                                );
                            });
                        });
                        ui.add_space(4.0);
                    }
                });

            // Processa deleção fora do loop de renderização (evita borrow mutável duplo)
            if let Some(id) = id_para_deletar {
                if let Some(banco) = db {
                    match banco.delete_profile(id) {
                        Ok(_) => {
                            estado.mensagem_feedback =
                                Some(("Perfil removido. 🥀".to_owned(), false));
                            estado.recarregar(banco);
                        }
                        Err(e) => {
                            estado.mensagem_feedback =
                                Some((format!("Erro ao deletar: {e}"), true));
                        }
                    }
                }
            }
        }

        ui.add_space(24.0);

        // Botão de voltar
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
            ação = PerfilAction::Voltar;
        }
    });

    ação
}
