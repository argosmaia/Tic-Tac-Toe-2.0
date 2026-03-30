//! Tela inicial do Velha 2.0.
//!
//! Apresenta as opções principais: Jogar, Perfil, Histórico e Sair.

use egui::Ui;

use crate::ui::theme::{cores, espacamentos, tipografia};

/// Ação solicitada pelo usuário no menu principal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuAction {
    /// Navegar para a tela de lobby (seleção de modo de jogo).
    IrParaLobby,
    /// Navegar para a tela de gerenciamento de perfil.
    IrParaPerfil,
    /// Navegar para o histórico de partidas.
    IrParaHistorico,
    /// Encerrar a aplicação.
    Sair,
    /// Nenhuma ação — aguardando input.
    Nenhuma,
}

/// Renderiza a tela do menu principal.
///
/// Retorna a ação solicitada pelo usuário, ou `MenuAction::Nenhuma` se nenhum botão foi clicado.
pub fn render_main_menu(ui: &mut Ui) -> MenuAction {
    let mut ação = MenuAction::Nenhuma;

    ui.with_layout(
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.add_space(60.0);

            // Logo / Título
            ui.label(
                egui::RichText::new("VELHA")
                    .size(tipografia::TITULO * 2.0)
                    .color(cores::BOTAO_PRIMARIO)
                    .strong(),
            );
            ui.label(
                egui::RichText::new("2.0")
                    .size(tipografia::TITULO)
                    .color(cores::TEXTO_MUDO),
            );

            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Ultimate Tic-Tac-Toe")
                    .size(tipografia::SUBTITULO)
                    .color(cores::TEXTO_SECUNDARIO),
            );

            ui.add_space(48.0);

            // Botões de navegação
            let largura_botao = 200.0;

            if botao_menu(ui, "▶  Jogar", largura_botao).clicked() {
                ação = MenuAction::IrParaLobby;
            }

            ui.add_space(8.0);

            if botao_menu(ui, "👤  Perfil", largura_botao).clicked() {
                ação = MenuAction::IrParaPerfil;
            }

            ui.add_space(8.0);

            if botao_menu(ui, "📋  Histórico", largura_botao).clicked() {
                ação = MenuAction::IrParaHistorico;
            }

            ui.add_space(24.0);

            if botao_secundario(ui, "Sair", largura_botao).clicked() {
                ação = MenuAction::Sair;
            }

            ui.add_space(40.0);

            // Versão
            ui.label(
                egui::RichText::new("v0.1.0 — HappyCode Productions")
                    .size(tipografia::PEQUENO)
                    .color(cores::TEXTO_MUDO),
            );
        },
    );

    ação
}

/// Renderiza um botão de ação primária do menu.
fn botao_menu(ui: &mut Ui, texto: &str, largura: f32) -> egui::Response {
    let cor_texto = cores::BOTAO_TEXTO;
    let cor_fundo = cores::BOTAO_PRIMARIO;

    ui.add_sized(
        [largura, 44.0],
        egui::Button::new(
            egui::RichText::new(texto)
                .size(tipografia::CORPO)
                .color(cor_texto),
        )
        .fill(cor_fundo)
        .rounding(espacamentos::RAIO_BORDA),
    )
}

/// Renderiza um botão secundário (transparente com borda).
fn botao_secundario(ui: &mut Ui, texto: &str, largura: f32) -> egui::Response {
    ui.add_sized(
        [largura, 36.0],
        egui::Button::new(
            egui::RichText::new(texto)
                .size(tipografia::CORPO)
                .color(cores::TEXTO_MUDO),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(1.0, cores::BORDA))
        .rounding(espacamentos::RAIO_BORDA),
    )
}
