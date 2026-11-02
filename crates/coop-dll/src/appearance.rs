//! Product-level remote player look (`remote_player_appearance`).
//!
//! Spec: docs/superpowers/specs/2026-11-02-lrm-baseline-and-remote-appearance-design.md (rev2).
//! When the new ini key is absent, callers must keep legacy `native_solid` / `ghost_solid`.

use std::sync::atomic::{AtomicU8, Ordering};

/// User-facing modes for LRM.1 (`subtle` is intentionally not exposed).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceMode {
    Normal = 0,
    Phantom = 1,
}

/// Whether product look comes from the new key or pre-LRM knobs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppearanceSource {
    Legacy = 0,
    New = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppearanceControl {
    pub source: AppearanceSource,
    /// Meaningful when `source == New`. Ignored for apply decisions under Legacy.
    pub mode: AppearanceMode,
}

/// 0 = legacy, 1 = new.
pub static APPEARANCE_SOURCE: AtomicU8 = AtomicU8::new(0);
/// 0 = normal, 1 = phantom (only when source is new).
pub static APPEARANCE_MODE: AtomicU8 = AtomicU8::new(0);

pub fn appearance_source() -> AppearanceSource {
    match APPEARANCE_SOURCE.load(Ordering::Relaxed) {
        1 => AppearanceSource::New,
        _ => AppearanceSource::Legacy,
    }
}

pub fn appearance_mode() -> AppearanceMode {
    match APPEARANCE_MODE.load(Ordering::Relaxed) {
        1 => AppearanceMode::Phantom,
        _ => AppearanceMode::Normal,
    }
}

pub fn appearance_control() -> AppearanceControl {
    AppearanceControl {
        source: appearance_source(),
        mode: appearance_mode(),
    }
}

pub fn appearance_source_label() -> &'static str {
    match appearance_source() {
        AppearanceSource::New => "new",
        AppearanceSource::Legacy => "legacy",
    }
}

pub fn store_control(c: AppearanceControl) {
    APPEARANCE_SOURCE.store(c.source as u8, Ordering::Relaxed);
    APPEARANCE_MODE.store(c.mode as u8, Ordering::Relaxed);
}

/// `remote_player_appearance`: `None` = key absent; `Some(v)` = key present (even if empty).
/// `native_solid` / `ghost_solid` are ignored for product mode when the new key is present;
/// they remain available for Legacy callers.
pub fn resolve_appearance(
    remote_player_appearance: Option<&str>,
    _native_solid: bool,
    _ghost_solid: u8,
) -> AppearanceControl {
    let Some(raw) = remote_player_appearance else {
        return AppearanceControl {
            source: AppearanceSource::Legacy,
            mode: AppearanceMode::Normal,
        };
    };
    let v = raw.trim();
    let mode = if v.eq_ignore_ascii_case("phantom") {
        AppearanceMode::Phantom
    } else {
        // empty / normal / invalid → Normal (settings layer adds the warn for invalid)
        AppearanceMode::Normal
    };
    AppearanceControl {
        source: AppearanceSource::New,
        mode,
    }
}

/// Classify a PlayerIns object vptr for the solid-vtable state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VptrClass {
    Stock,
    Solid,
    Other,
}

pub fn classify_player_ins_vptr(vptr: usize, stock: usize, solid: usize) -> VptrClass {
    if vptr == 0 || stock == 0 {
        return VptrClass::Other;
    }
    if vptr == stock {
        VptrClass::Stock
    } else if solid != 0 && vptr == solid {
        VptrClass::Solid
    } else {
        VptrClass::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_key_is_legacy() {
        let c = resolve_appearance(None, true, 1);
        assert_eq!(c.source, AppearanceSource::Legacy);
    }

    #[test]
    fn normal_key_is_new_normal() {
        let c = resolve_appearance(Some("normal"), true, 1);
        assert_eq!(c, AppearanceControl { source: AppearanceSource::New, mode: AppearanceMode::Normal });
    }

    #[test]
    fn phantom_key() {
        let c = resolve_appearance(Some("phantom"), true, 1);
        assert_eq!(c, AppearanceControl { source: AppearanceSource::New, mode: AppearanceMode::Phantom });
    }

    #[test]
    fn invalid_key_falls_back_to_normal() {
        let c = resolve_appearance(Some("subtle"), true, 1);
        assert_eq!(c.source, AppearanceSource::New);
        assert_eq!(c.mode, AppearanceMode::Normal);
        let c2 = resolve_appearance(Some("nope"), false, 0);
        assert_eq!(c2.mode, AppearanceMode::Normal);
    }

    #[test]
    fn new_key_ignores_native_solid_false() {
        let c = resolve_appearance(Some("normal"), false, 0);
        assert_eq!(c.source, AppearanceSource::New);
        assert_eq!(c.mode, AppearanceMode::Normal);
    }

    #[test]
    fn classify_vptr_stock_solid_other() {
        assert_eq!(classify_player_ins_vptr(0x1000, 0x1000, 0x2000), VptrClass::Stock);
        assert_eq!(classify_player_ins_vptr(0x2000, 0x1000, 0x2000), VptrClass::Solid);
        assert_eq!(classify_player_ins_vptr(0x3000, 0x1000, 0x2000), VptrClass::Other);
        assert_eq!(classify_player_ins_vptr(0, 0x1000, 0x2000), VptrClass::Other);
    }

    #[test]
    fn source_label() {
        store_control(AppearanceControl { source: AppearanceSource::New, mode: AppearanceMode::Normal });
        assert_eq!(appearance_source_label(), "new");
        store_control(AppearanceControl { source: AppearanceSource::Legacy, mode: AppearanceMode::Normal });
        assert_eq!(appearance_source_label(), "legacy");
    }
}
