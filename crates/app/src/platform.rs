//! Fundo da janela por plataforma (vidro só onde o sistema desfoca de verdade).

use gpui_kit::WindowBackgroundAppearance;

/// Número de build do Windows (ex.: 22631), via `RtlGetVersion` (não mente por compatibilidade).
#[cfg(windows)]
fn windows_build() -> u32 {
    #[repr(C)]
    struct OsVersionInfoW {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        csd: [u16; 128],
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
    }
    let mut info = OsVersionInfoW {
        size: size_of::<OsVersionInfoW>() as u32,
        major: 0,
        minor: 0,
        build: 0,
        platform: 0,
        csd: [0; 128],
    };
    // SAFETY: estrutura com o tamanho declarado em `size`, como a API exige.
    if unsafe { RtlGetVersion(&mut info) } == 0 {
        info.build
    } else {
        0
    }
}

/// (fundo da janela, chrome translúcido?)
/// - Windows 11 (build >= 22621): Mica Alt — segue o papel de parede, leve.
/// - Windows 10: acrílico (`Blurred`).
/// - Linux: opaco. O GPUI só desfoca no KDE Wayland; no GNOME/X11 alfa sem blur fica "sujo".
pub fn window_background() -> (WindowBackgroundAppearance, bool) {
    #[cfg(windows)]
    {
        if windows_build() >= 22621 {
            (WindowBackgroundAppearance::MicaAltBackdrop, true)
        } else {
            (WindowBackgroundAppearance::Blurred, true)
        }
    }
    #[cfg(not(windows))]
    {
        (WindowBackgroundAppearance::Opaque, false)
    }
}

/// Fonte dos glifos dos botões de janela no Windows (Fluent no 11, MDL2 no 10).
#[cfg(windows)]
pub fn caption_font() -> &'static str {
    if windows_build() >= 22000 {
        "Segoe Fluent Icons"
    } else {
        "Segoe MDL2 Assets"
    }
}
