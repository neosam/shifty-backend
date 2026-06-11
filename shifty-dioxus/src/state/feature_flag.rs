//! Frontend state-types for backend feature flags (Phase 8 Plan 08-07
//! Gap-Closure, Task 3).
//!
//! Phase 8.6 Cutover-Abriss (D-02): `absence_range_source_active` Field und
//! Convenience-Lookup wurden entfernt. Das Struct bleibt als leere Shell
//! bestehen, damit `service::feature_flag` und Downstream-Code kompilieren.

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
/// Phase 8.6 D-02: Leere Shell — kein aktiv genutzter Flag-Slot mehr.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FeatureFlagsState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_to_preserves_all_fields() {
        let to = FeatureFlagTO {
            key: "some_flag".into(),
            enabled: true,
            description: Some("desc".into()),
        };
        let f: FeatureFlag = (&to).into();
        assert_eq!(f.key, "some_flag");
        assert!(f.enabled);
        assert_eq!(f.description.as_deref(), Some("desc"));
    }

    #[test]
    fn default_state_is_empty() {
        let _s = FeatureFlagsState::default();
        // Shell-State: kein Feld, kein Panic.
    }
}
