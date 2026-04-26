use dioxus::prelude::*;
use futures_util::StreamExt;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

const STORAGE_KEY: &str = "shifty-theme";
const DARK_MEDIA_QUERY: &str = "(prefers-color-scheme: dark)";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
            "system" => Some(ThemeMode::System),
            _ => None,
        }
    }
}

impl ResolvedTheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ResolvedTheme::Light => "light",
            ResolvedTheme::Dark => "dark",
        }
    }
}

pub enum ThemeAction {
    SetMode(ThemeMode),
    SystemThemeChanged(ResolvedTheme),
}

pub static THEME_MODE: GlobalSignal<ThemeMode> = Signal::global(|| ThemeMode::System);
pub static RESOLVED_THEME: GlobalSignal<ResolvedTheme> = Signal::global(|| ResolvedTheme::Light);

pub fn cycle_theme(current: ThemeMode) -> ThemeMode {
    match current {
        ThemeMode::Light => ThemeMode::Dark,
        ThemeMode::Dark => ThemeMode::System,
        ThemeMode::System => ThemeMode::Light,
    }
}

pub fn resolve_theme(mode: ThemeMode, system_prefers_dark: bool) -> ResolvedTheme {
    match mode {
        ThemeMode::Light => ResolvedTheme::Light,
        ThemeMode::Dark => ResolvedTheme::Dark,
        ThemeMode::System => {
            if system_prefers_dark {
                ResolvedTheme::Dark
            } else {
                ResolvedTheme::Light
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_stored_mode() -> ThemeMode {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .and_then(|v| ThemeMode::from_str(&v))
        .unwrap_or(ThemeMode::System)
}

#[cfg(target_arch = "wasm32")]
fn store_mode(mode: ThemeMode) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, mode.as_str());
    }
}

#[cfg(target_arch = "wasm32")]
fn system_prefers_dark() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media(DARK_MEDIA_QUERY).ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

#[cfg(target_arch = "wasm32")]
fn apply_resolved_to_dom(resolved: ResolvedTheme) {
    if let Some(html) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = html.set_attribute("data-theme", resolved.as_str());
    }
}

#[cfg(target_arch = "wasm32")]
fn subscribe_system_theme() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(mql)) = window.match_media(DARK_MEDIA_QUERY) else {
        return;
    };

    let closure = Closure::wrap(Box::new(move |event: web_sys::MediaQueryListEvent| {
        let resolved = if event.matches() {
            ResolvedTheme::Dark
        } else {
            ResolvedTheme::Light
        };
        handle_system_theme_change(resolved);
    }) as Box<dyn FnMut(web_sys::MediaQueryListEvent)>);

    let _ = mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
    // Leak the closure so it lives for the duration of the app session.
    closure.forget();
}

#[cfg(not(target_arch = "wasm32"))]
fn load_stored_mode() -> ThemeMode {
    ThemeMode::System
}

#[cfg(not(target_arch = "wasm32"))]
fn store_mode(_mode: ThemeMode) {}

#[cfg(not(target_arch = "wasm32"))]
fn system_prefers_dark() -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn apply_resolved_to_dom(_resolved: ResolvedTheme) {}

#[cfg(not(target_arch = "wasm32"))]
fn subscribe_system_theme() {}

/// Apply a `SetMode` action: persist the mode, recompute the resolved theme,
/// and update the DOM. Used by the service loop and re-usable from a toggle.
pub fn apply_set_mode(mode: ThemeMode) {
    store_mode(mode);
    let resolved = resolve_theme(mode, system_prefers_dark());
    *THEME_MODE.write() = mode;
    if *RESOLVED_THEME.read() != resolved {
        *RESOLVED_THEME.write() = resolved;
    }
    apply_resolved_to_dom(resolved);
}

/// Apply a system-theme mediaquery change. Only effective while the current
/// `THEME_MODE` is `System`; other modes ignore OS-level theme switches.
pub fn handle_system_theme_change(resolved: ResolvedTheme) {
    if *THEME_MODE.read() != ThemeMode::System {
        return;
    }
    if *RESOLVED_THEME.read() != resolved {
        *RESOLVED_THEME.write() = resolved;
        apply_resolved_to_dom(resolved);
    }
}

pub async fn theme_service(mut rx: UnboundedReceiver<ThemeAction>) {
    let initial_mode = load_stored_mode();
    let initial_resolved = resolve_theme(initial_mode, system_prefers_dark());
    *THEME_MODE.write() = initial_mode;
    *RESOLVED_THEME.write() = initial_resolved;
    apply_resolved_to_dom(initial_resolved);

    subscribe_system_theme();

    while let Some(action) = rx.next().await {
        match action {
            ThemeAction::SetMode(mode) => apply_set_mode(mode),
            ThemeAction::SystemThemeChanged(resolved) => handle_system_theme_change(resolved),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_theme_transitions() {
        assert_eq!(cycle_theme(ThemeMode::Light), ThemeMode::Dark);
        assert_eq!(cycle_theme(ThemeMode::Dark), ThemeMode::System);
        assert_eq!(cycle_theme(ThemeMode::System), ThemeMode::Light);
    }

    #[test]
    fn resolve_theme_light_mode() {
        assert_eq!(resolve_theme(ThemeMode::Light, false), ResolvedTheme::Light);
        assert_eq!(resolve_theme(ThemeMode::Light, true), ResolvedTheme::Light);
    }

    #[test]
    fn resolve_theme_dark_mode() {
        assert_eq!(resolve_theme(ThemeMode::Dark, false), ResolvedTheme::Dark);
        assert_eq!(resolve_theme(ThemeMode::Dark, true), ResolvedTheme::Dark);
    }

    #[test]
    fn resolve_theme_system_mode() {
        assert_eq!(
            resolve_theme(ThemeMode::System, false),
            ResolvedTheme::Light
        );
        assert_eq!(resolve_theme(ThemeMode::System, true), ResolvedTheme::Dark);
    }

    #[test]
    fn theme_mode_round_trip() {
        for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
            assert_eq!(ThemeMode::from_str(mode.as_str()), Some(mode));
        }
        assert_eq!(ThemeMode::from_str("nonsense"), None);
    }
}
