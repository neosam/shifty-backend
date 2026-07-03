//! Frontend Feature-Flag-Service (Phase 8 Plan 08-07 Gap-Closure, Task 3).
//!
//! Phase 8.6 Cutover-Abriss (D-02): `FeatureFlagAction` und
//! `feature_flag_service` wurden auf leere Shells reduziert. Das Modul
//! bleibt erhalten, damit `app.rs` weiterhin kompiliert (kein
//! `use_coroutine`-Call mehr, aber das Modul selbst ist noch importiert).
//!
//! `FEATURE_FLAGS_STORE` bleibt als GlobalSignal, damit allfällige
//! Downstream-Reads (falls noch vorhanden) nicht brechen.
//!
//! Phase 8.6 D-02 (Option A): Das gesamte konservierte Flag-Modul ist mit
//! `#![allow(dead_code)]` markiert — der generische Mechanismus bleibt für
//! künftige Flags erhalten und kompiliert warnungsfrei, obwohl er nach dem
//! Cutover-Abriss aktuell keinen Konsumenten mehr hat.
#![allow(dead_code)]

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::state::feature_flag::FeatureFlagsState;

pub static FEATURE_FLAGS_STORE: GlobalSignal<FeatureFlagsState> =
    Signal::global(FeatureFlagsState::default);

/// Phase 8.6 D-02: Leere Action-Enum-Shell. Kein Variant mehr aktiv.
#[allow(dead_code)]
#[derive(Debug)]
pub enum FeatureFlagAction {}

/// Phase 8.6 D-02: Leere Coroutine-Shell.
/// Konsumiert den Receiver, tut aber nichts — kein Backend-Call mehr.
pub async fn feature_flag_service(mut rx: UnboundedReceiver<FeatureFlagAction>) {
    while let Some(_action) = rx.next().await {
        // Kein aktiver Flag mehr — Body intentionally leer.
    }
}

#[cfg(test)]
mod tests {
    use crate::state::Config;
    use std::rc::Rc;

    /// Lockt den Vertrag: `Config::default()` hat ein leeres `backend`.
    #[test]
    fn default_config_has_empty_backend() {
        let cfg = Config::default();
        assert!(
            cfg.backend.is_empty(),
            "Config::default().backend muss leer sein"
        );
    }

    /// Lockt den Vertrag: eine ECHT geladene Config (mit non-empty backend)
    /// ergibt ein nicht-leeres backend.
    #[test]
    fn populated_config_unblocks_wait_predicate() {
        let cfg = Config {
            backend: Rc::from("http://localhost:8080"),
            ..Default::default()
        };
        assert!(
            !cfg.backend.is_empty(),
            "Mit gesetztem backend muss is_empty() false sein"
        );
    }
}
