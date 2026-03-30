//! Design system: paleta de cores, tipografia e espaçamentos.
//!
//! Responsabilidade única: centralizar todas as constantes visuais.
//! Nenhuma cor ou espaçamento deve ser hardcoded em componentes de UI.
//! Qualquer alteração visual deve começar aqui.

use egui::Color32;

/// Paleta de cores do Velha 2.0.
///
/// Tema dark espacial com acentos vibrantes para X e O.
pub mod cores {
    use egui::Color32;

    /// Fundo principal — quase preto azulado.
    pub const FUNDO_ESCURO: Color32 = Color32::from_rgb(10, 10, 15);
    /// Superfície de cards e painéis — cinza azulado escuro.
    pub const SUPERFICIE: Color32 = Color32::from_rgb(18, 18, 26);
    /// Superfície elevada — levemente mais clara que SUPERFICIE.
    pub const SUPERFICIE_ELEVADA: Color32 = Color32::from_rgb(26, 26, 38);
    /// Borda padrão — cinza sutil.
    pub const BORDA: Color32 = Color32::from_rgb(42, 42, 58);
    /// Borda do quadrante ativo — azul elétrico vibrante.
    pub const BORDA_ATIVA: Color32 = Color32::from_rgb(91, 91, 255);
    /// Glow do quadrante ativo (semi-transparente).
    pub const GLOW_ATIVO: Color32 = Color32::from_rgba_premultiplied(91, 91, 255, 30);

    /// Cor de X — vermelho rosado vibrante.
    pub const JOGADOR_X: Color32 = Color32::from_rgb(255, 77, 109);
    /// Cor de O — verde-água vibrante.
    pub const JOGADOR_O: Color32 = Color32::from_rgb(77, 255, 180);

    /// Texto principal — branco levemente azulado.
    pub const TEXTO_PRIMARIO: Color32 = Color32::from_rgb(232, 232, 240);
    /// Texto secundário — cinza médio.
    pub const TEXTO_SECUNDARIO: Color32 = Color32::from_rgb(150, 150, 170);
    /// Texto mudo — cinza escuro para informações de baixa prioridade.
    pub const TEXTO_MUDO: Color32 = Color32::from_rgb(85, 85, 112);

    /// Overlay de vitória no quadrante — semi-transparente.
    pub const OVERLAY_VITORIA_X: Color32 = Color32::from_rgba_premultiplied(255, 77, 109, 40);
    pub const OVERLAY_VITORIA_O: Color32 = Color32::from_rgba_premultiplied(77, 255, 180, 40);
    pub const OVERLAY_EMPATE: Color32 = Color32::from_rgba_premultiplied(85, 85, 112, 40);

    /// Botão primário — gradiente simulado via cor sólida.
    pub const BOTAO_PRIMARIO: Color32 = Color32::from_rgb(91, 91, 255);
    pub const BOTAO_PRIMARIO_HOVER: Color32 = Color32::from_rgb(110, 110, 255);
    pub const BOTAO_TEXTO: Color32 = Color32::from_rgb(255, 255, 255);

    /// Cor de acento dourado para destaque especial (ex: nível Killer).
    pub const ACENTO_DOURADO: Color32 = Color32::from_rgb(255, 200, 70);
}

/// Espaçamentos e tamanhos do tabuleiro.
pub mod espacamentos {
    /// Tamanho de cada célula individual em pixels.
    pub const TAMANHO_CELULA: f32 = 36.0;
    /// Gap entre quadrantes macro do tabuleiro.
    pub const GAP_QUADRANTE: f32 = 8.0;
    /// Gap entre células dentro de um quadrante.
    pub const GAP_CELULA: f32 = 3.0;
    /// Raio de borda padrão para elementos arredondados.
    pub const RAIO_BORDA: f32 = 8.0;
    /// Espessura da borda do quadrante ativo.
    pub const ESPESSURA_BORDA_ATIVA: f32 = 2.5;
    /// Padding interno padrão para painéis.
    pub const PADDING_PAINEL: f32 = 16.0;
}

/// Tamanhos de fonte para a hierarquia tipográfica.
pub mod tipografia {
    /// Título da tela — grande e impactante.
    pub const TITULO: f32 = 32.0;
    /// Subtítulo — médio.
    pub const SUBTITULO: f32 = 18.0;
    /// Texto de corpo padrão.
    pub const CORPO: f32 = 14.0;
    /// Texto pequeno / labels.
    pub const PEQUENO: f32 = 12.0;
    /// Símbolo X/O no tabuleiro.
    pub const SIMBOLO_TABULEIRO: f32 = 22.0;
    /// Símbolo X/O no overlay de vitória do quadrante.
    pub const SIMBOLO_OVERLAY: f32 = 40.0;
}

/// Configura o estilo visual global do egui com o design system do Velha 2.0.
pub fn aplicar_tema(ctx: &egui::Context) {
    let mut estilo = (*ctx.style()).clone();

    // Cores de fundo
    estilo.visuals.panel_fill = cores::SUPERFICIE;
    estilo.visuals.window_fill = cores::SUPERFICIE;
    estilo.visuals.extreme_bg_color = cores::FUNDO_ESCURO;
    estilo.visuals.faint_bg_color = cores::SUPERFICIE_ELEVADA;

    // Texto
    estilo.visuals.override_text_color = Some(cores::TEXTO_PRIMARIO);

    // Widgets
    estilo.visuals.widgets.inactive.bg_fill = cores::SUPERFICIE_ELEVADA;
    estilo.visuals.widgets.inactive.bg_stroke =
        egui::Stroke::new(1.0, cores::BORDA);
    estilo.visuals.widgets.hovered.bg_fill = cores::BOTAO_PRIMARIO_HOVER;
    estilo.visuals.widgets.active.bg_fill = cores::BOTAO_PRIMARIO;

    // Bordas arredondadas
    estilo.visuals.window_rounding = egui::Rounding::same(espacamentos::RAIO_BORDA);

    // Espaçamentos
    estilo.spacing.item_spacing = egui::vec2(8.0, 8.0);
    estilo.spacing.button_padding = egui::vec2(12.0, 6.0);

    ctx.set_style(estilo);
}

/// Retorna a cor do jogador X ou O.
pub fn cor_jogador(player: &crate::game::Player) -> Color32 {
    match player {
        crate::game::Player::X => cores::JOGADOR_X,
        crate::game::Player::O => cores::JOGADOR_O,
    }
}
