//! Frontend state-types for backend feature flags (Phase 8 Plan 08-07
//! Gap-Closure, Task 3).
//!
//! Mirror des `FeatureFlagTO` aus `rest-types`. Die Frontend-Seite hält für
//! jeden bekannten Flag-Key einen `Option<bool>`-Slot in `FeatureFlagsState`
//! (`None` = noch nicht geladen, `Some(false)` = explizit aus, `Some(true)`
//! = explizit an). Aktuell wird nur `absence_range_source_active` aktiv
//! genutzt — wenn künftig weitere Flags ans Frontend müssen, wird der
//! Struct erweitert.

use rest_types::FeatureFlagTO;

/// Generischer Frontend-Mirror für ein einzelnes Feature-Flag.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureFlag {
    pub key: String,
    pub enabled: bool,
    pub description: Option<String>,
}

impl From<&FeatureFlagTO> for FeatureFlag {
    fn from(t: &FeatureFlagTO) -> Self {
        Self {
            key: t.key.clone(),
            enabled: t.enabled,
            description: t.description.clone(),
        }
    }
}

/// Aggregat-State: Frontend-spezifische Sicht auf bekannte Flags.
/// Wird von `service::feature_flag::feature_flag_service` befüllt; jeder
/// Konsument liest hier reaktiv den aktuellen Wert.
///
/// **Default = `None`-überall**: solange das Service den ersten Load nicht
/// abgeschlossen hat, weiß die UI nicht, ob ein Flag an oder aus ist —
/// sie muss explizit gegen `Some(true)` matchen, um eine flag-gegated UI zu
/// rendern. Damit ist die Rendering-Logik fail-safe: kein Render bei
/// Unwissenheit, nicht "as-if-disabled" aber auch nicht "as-if-enabled".
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FeatureFlagsState {
    /// Cutover-Status: Range-Based Absences sind die Source-of-Truth.
    /// Plan 08-07 nutzt das, um den Absences-Menüeintrag im TopBar
    /// auszublenden, solange noch nicht migriert wurde.
    pub absence_range_source_active: Option<bool>,
}

impl FeatureFlagsState {
    /// Convenience-Lookup: liefert `false` solange unbekannt — d.h.
    /// "noch nicht geladen" und "explizit deaktiviert" sind UI-äquivalent.
    /// Reicht für den Cutover-Gate-Use-Case (Menü-Eintrag erst zeigen,
    /// wenn bekannt UND aktiv).
    pub fn absence_range_source_active(&self) -> bool {
        self.absence_range_source_active.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_to_preserves_all_fields() {
        let to = FeatureFlagTO {
            key: "absence_range_source_active".into(),
            enabled: true,
            description: Some("desc".into()),
        };
        let f: FeatureFlag = (&to).into();
        assert_eq!(f.key, "absence_range_source_active");
        assert!(f.enabled);
        assert_eq!(f.description.as_deref(), Some("desc"));
    }

    #[test]
    fn default_state_returns_false_for_known_flags() {
        let s = FeatureFlagsState::default();
        assert!(!s.absence_range_source_active());
        assert_eq!(s.absence_range_source_active, None);
    }

    #[test]
    fn convenience_lookup_returns_inner_when_some() {
        let s = FeatureFlagsState {
            absence_range_source_active: Some(true),
        };
        assert!(s.absence_range_source_active());
    }
}
