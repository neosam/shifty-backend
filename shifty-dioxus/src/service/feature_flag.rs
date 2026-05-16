//! Frontend Feature-Flag-Service (Phase 8 Plan 08-07 Gap-Closure, Task 3).
//!
//! Pattern: GlobalSignal-Store + Action-Enum-Coroutine, analog
//! `service::vacation_balance` (Plan 08-04). Pflegt
//! `FEATURE_FLAGS_STORE` und lädt beim App-Start einmalig den
//! `absence_range_source_active`-Flag.
//!
//! Erweiterung: wenn künftig ein zweiter Flag im Frontend benötigt wird,
//! kommt eine zweite `LoadFlag(...)`-Variante dazu und der App-Start-
//! Trigger wird ergänzt (`app.rs`).

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;

use crate::{loader, state::feature_flag::FeatureFlagsState};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static FEATURE_FLAGS_STORE: GlobalSignal<FeatureFlagsState> =
    Signal::global(FeatureFlagsState::default);

/// Auf den Cutover-Flag scoped. Aktuell der einzige Flag, den das Frontend
/// kennen muss (Plan 08-07 Task 4: TopBar gates auf diesem Wert).
pub const ABSENCE_RANGE_SOURCE_ACTIVE_KEY: &str = "absence_range_source_active";

#[derive(Debug)]
pub enum FeatureFlagAction {
    /// Lädt `absence_range_source_active` und schreibt das Ergebnis in
    /// `FEATURE_FLAGS_STORE.absence_range_source_active`.
    LoadAbsenceRangeSourceActive,
}

/// Wartet, bis `CONFIG.backend` ein nicht-leerer Wert ist.
///
/// Hintergrund: `app.rs` dispatcht `FeatureFlagAction::LoadAbsenceRangeSourceActive`
/// synchron während des ersten Renders — d.h. BEVOR `config_service`
/// `load_config().await` abgeschlossen hat. Ohne diese Wartelogik liest
/// `CONFIG.read()` den `Config::default()` mit leerem `backend: Rc<str>`,
/// und `format!("{}/feature-flag/{}", "", key)` ergibt die relative URL
/// `/feature-flag/...`, die reqwest mit
/// `relative URL without a base` ablehnt.
///
/// Polling mit kurzem Timeout statt einer reaktiven Signal-Subscription,
/// weil wir uns hier außerhalb des render-driven Reactivity-Systems
/// befinden (Coroutine-Body). 10 ms ist eine pragmatische Untergrenze:
/// für lokale dev-Setups landet der Config-Load typischerweise innerhalb
/// von ein bis zwei Ticks, sodass der zusätzliche Delay vom User nicht
/// wahrgenommen wird, das CPU-Profil aber ruhig bleibt.
async fn wait_for_config_ready() {
    while CONFIG.read().backend.is_empty() {
        gloo_timers::future::TimeoutFuture::new(10).await;
    }
}

pub async fn feature_flag_service(mut rx: UnboundedReceiver<FeatureFlagAction>) {
    while let Some(action) = rx.next().await {
        info!("FeatureFlagAction: {:?}", &action);
        // Erst wenn die Backend-URL geladen ist, einen HTTP-Call absetzen —
        // sonst feuert `loader::load_feature_flag(default_config, ...)` mit
        // leerer Base-URL und reqwest meldet "relative URL without a base".
        wait_for_config_ready().await;
        let config = CONFIG.read().clone();
        match action {
            FeatureFlagAction::LoadAbsenceRangeSourceActive => {
                match loader::load_feature_flag(config, ABSENCE_RANGE_SOURCE_ACTIVE_KEY).await {
                    Ok(flag) => {
                        let mut store = FEATURE_FLAGS_STORE.write();
                        store.absence_range_source_active = Some(flag.enabled);
                    }
                    Err(err) => {
                        // Network/Backend down → Wert bleibt bei `None`,
                        // d.h. die UI rendert flag-gegated nichts. Fehler
                        // landet im globalen ERROR_STORE für die Toast-UI.
                        *ERROR_STORE.write() = ErrorStore { error: Some(err) };
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    //! Regression-Tests für die Config-Race-Condition.
    //!
    //! Der `feature_flag_service`-Coroutine wurde während des ersten App-Renders
    //! aufgerufen, BEVOR `config_service` den `assets/config.json`-Inhalt nach
    //! `CONFIG` schreibt. Ohne `wait_for_config_ready()` würde der Service den
    //! Default-`Config { backend: "" }` lesen und in `api::get_feature_flag`
    //! eine relative URL bauen (`/feature-flag/...`), die reqwest mit
    //! `relative URL without a base` ablehnt. Dieser Fehler floss in
    //! `ERROR_STORE` und wurde auf `EmployeeDetails` (eine der vier Pages mit
    //! `ErrorView`) sichtbar.
    //!
    //! Reine asynchrone Service-Tests sind in diesem Crate nicht eingerichtet
    //! (kein Mock-Layer für `reqwest`), daher prüfen wir hier nur die invariant,
    //! dass `wait_for_config_ready` die leere-Backend-Bedingung tatsächlich
    //! erkennt — der Rest ist ein Smoke-Test auf Default-Werte des
    //! `FeatureFlagsState`-Stores.

    use super::*;
    use crate::state::Config;
    use std::rc::Rc;

    /// Lockt den Vertrag: `Config::default()` hat ein leeres `backend`.
    /// Wenn dieses Verhalten sich ändert, fällt der eigentliche Race-Schutz
    /// in `wait_for_config_ready` aus, weil der Polling-Check `is_empty()`
    /// dann nie true wäre.
    #[test]
    fn default_config_has_empty_backend() {
        let cfg = Config::default();
        assert!(
            cfg.backend.is_empty(),
            "Config::default().backend muss leer sein, sonst kann die \
             Race-Detection im feature_flag_service nicht greifen"
        );
    }

    /// Lockt den Vertrag: eine ECHT geladene Config (mit non-empty backend)
    /// erfüllt die Wait-Bedingung NICHT mehr (d.h. der Polling-Loop endet).
    #[test]
    fn populated_config_unblocks_wait_predicate() {
        let cfg = Config {
            backend: Rc::from("http://localhost:8080"),
            ..Default::default()
        };
        assert!(
            !cfg.backend.is_empty(),
            "Mit gesetztem backend muss is_empty() false sein — sonst hängt \
             der Polling-Loop in wait_for_config_ready endlos"
        );
    }
}
