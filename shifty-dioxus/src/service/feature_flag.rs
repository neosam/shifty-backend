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

pub async fn feature_flag_service(mut rx: UnboundedReceiver<FeatureFlagAction>) {
    while let Some(action) = rx.next().await {
        info!("FeatureFlagAction: {:?}", &action);
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
