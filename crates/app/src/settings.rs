//! Preferências do usuário (tema, larguras dos painéis), em texto
//! simples `chave=valor` no diretório de configuração do sistema.

use crate::glass::{EDITOR_W, INSPECTOR_W, Mode, SIDEBAR_W};
use std::path::PathBuf;

#[derive(Clone, Copy)]
pub struct Settings {
    pub mode: Option<Mode>,
    pub sidebar_w: f32,
    pub editor_w: f32,
    pub inspector_w: f32,
    pub translucent_cards: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mode: None,
            sidebar_w: f32::from(SIDEBAR_W),
            editor_w: f32::from(EDITOR_W),
            inspector_w: f32::from(INSPECTOR_W),
            translucent_cards: false,
        }
    }
}

fn path() -> Option<PathBuf> {
    let base = std::env::var_os("APPDATA")
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME"))
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("db-modeling").join("settings.txt"))
}

/// Larguras fora de faixa (arquivo editado à mão/corrompido) voltam ao padrão.
fn width(v: &str, default: f32) -> f32 {
    v.trim()
        .parse::<f32>()
        .ok()
        .filter(|w| (120.0..=1600.0).contains(w))
        .unwrap_or(default)
}

pub fn load() -> Settings {
    let mut s = Settings::default();
    let Some(text) = path().and_then(|p| std::fs::read_to_string(p).ok()) else {
        return s;
    };
    let d = Settings::default();
    for line in text.lines() {
        match line.split_once('=') {
            Some(("mode", "dark")) => s.mode = Some(Mode::Dark),
            Some(("mode", "light")) => s.mode = Some(Mode::Light),
            Some(("sidebar_w", v)) => s.sidebar_w = width(v, d.sidebar_w),
            Some(("editor_w", v)) => s.editor_w = width(v, d.editor_w),
            Some(("inspector_w", v)) => s.inspector_w = width(v, d.inspector_w),
            Some(("translucent_cards", v)) => s.translucent_cards = v.trim() == "true",
            _ => {}
        }
    }
    s
}

/// Falha ao salvar só perde a preferência; não interrompe o app.
pub fn save(s: &Settings) {
    let Some(p) = path() else { return };
    let mode = match s.mode {
        Some(Mode::Light) => "light",
        _ => "dark",
    };
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(
        p,
        format!(
            "mode={mode}\nsidebar_w={}\neditor_w={}\ninspector_w={}\ntranslucent_cards={}\n",
            s.sidebar_w.round(),
            s.editor_w.round(),
            s.inspector_w.round(),
            s.translucent_cards
        ),
    );
}
