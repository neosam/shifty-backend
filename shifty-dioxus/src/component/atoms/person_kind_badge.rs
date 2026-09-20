//! `PersonKindBadge` — read-only pill showing the *Auspraegung* of a person
//! directory row (Phase 62, PADM-01, D-62-01). Pure display atom: no state,
//! no API calls.
//!
//! ## Where the variant comes from (D-62-01, T-62-09)
//!
//! The variant is the backend-supplied `PersonKindTO` enum, never a frontend
//! heuristic over empty `Option`s. This atom only translates that enum into a
//! class string and a label key — it cannot invent an Auspraegung.
//!
//! ## Color semantics (62-UI-SPEC)
//!
//! All three variants recycle tokens that already appear on this exact page —
//! no new palette entry is introduced:
//! - `Linked` → accent, echoing the "complete" semantic the `Paid` pill
//!   already carries here.
//! - `SalesPersonOnly` → warn, echoing the `Volunteer` pill's
//!   incomplete/pending amber ("no login yet").
//! - `UserOnly` → the neutral dashed `PersonChip` no-color fallback. A pure
//!   login account is a normal end state, not a gap, so it must not look
//!   alarming.
//!
//! Only static Tailwind design-token classes per match arm — no `format!()`,
//! no raw palette literals (Tailwind-detect + legacy-class gate).

use dioxus::prelude::*;
use rest_types::PersonKindTO;

use crate::i18n::Key;
use crate::service::i18n::I18N;

/// Static Tailwind design-token class string for the badge, one per
/// Auspraegung. Shared shape classes are identical to `WeekStatusBadge`'s.
pub(crate) fn person_kind_badge_class(kind: &PersonKindTO) -> &'static str {
    match kind {
        PersonKindTO::Linked => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium border bg-accent-soft border-accent text-accent"
        }
        PersonKindTO::SalesPersonOnly => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium border bg-warn-soft border-warn text-warn"
        }
        PersonKindTO::UserOnly => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium border border-dashed border-border-strong text-ink-soft bg-transparent"
        }
    }
}

/// i18n label key for an Auspraegung (the three keys added in Plan 62-02).
pub(crate) fn person_kind_label_key(kind: &PersonKindTO) -> Key {
    match kind {
        PersonKindTO::Linked => Key::PersonKindLinked,
        PersonKindTO::SalesPersonOnly => Key::PersonKindSalesPersonOnly,
        PersonKindTO::UserOnly => Key::PersonKindUserOnly,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PersonKindBadgeProps {
    /// The backend-supplied Auspraegung of the row (D-62-01).
    pub kind: PersonKindTO,
}

/// Read-only Auspraegung pill. Renders a `span` with the token class and the
/// translated label.
#[component]
pub fn PersonKindBadge(props: PersonKindBadgeProps) -> Element {
    let label = I18N.read().t(person_kind_label_key(&props.kind));
    rsx! {
        span {
            class: person_kind_badge_class(&props.kind),
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_KINDS: [PersonKindTO; 3] = [
        PersonKindTO::Linked,
        PersonKindTO::SalesPersonOnly,
        PersonKindTO::UserOnly,
    ];

    #[test]
    fn person_kind_badge_class_ist_statisch_je_variante() {
        // Die drei in 62-UI-SPEC festgelegten Token-Klassenstrings — je
        // Variante genau einer, alle voneinander unterscheidbar.
        let linked = person_kind_badge_class(&PersonKindTO::Linked);
        assert!(linked.contains("bg-accent-soft"), "linked: {linked}");
        assert!(linked.contains("border-accent"), "linked: {linked}");
        assert!(linked.contains("text-accent"), "linked: {linked}");

        let sp_only = person_kind_badge_class(&PersonKindTO::SalesPersonOnly);
        assert!(sp_only.contains("bg-warn-soft"), "sales person only: {sp_only}");
        assert!(sp_only.contains("border-warn"), "sales person only: {sp_only}");
        assert!(sp_only.contains("text-warn"), "sales person only: {sp_only}");

        let user_only = person_kind_badge_class(&PersonKindTO::UserOnly);
        assert!(user_only.contains("border-dashed"), "user only: {user_only}");
        assert!(
            user_only.contains("border-border-strong"),
            "user only: {user_only}"
        );
        assert!(user_only.contains("text-ink-soft"), "user only: {user_only}");
        assert!(user_only.contains("bg-transparent"), "user only: {user_only}");

        // Paarweise verschieden — sonst waeren zwei Auspraegungen visuell
        // ununterscheidbar (PADM-01, Success Criterion 1).
        assert_ne!(linked, sp_only);
        assert_ne!(linked, user_only);
        assert_ne!(sp_only, user_only);
    }

    #[test]
    fn class_carries_shared_shape_classes() {
        for kind in ALL_KINDS {
            let c = person_kind_badge_class(&kind);
            assert!(c.contains("inline-flex"), "missing inline-flex: {c}");
            assert!(c.contains("items-center"), "missing items-center: {c}");
            assert!(c.contains("rounded-sm"), "missing rounded-sm: {c}");
            assert!(c.contains("text-small"), "missing text-small: {c}");
            assert!(c.contains("font-medium"), "missing font-medium: {c}");
            assert!(c.contains("border"), "missing border: {c}");
        }
    }

    #[test]
    fn class_has_no_raw_hex_value() {
        // Laufzeit-Formatierung und Roh-Hexwerte sind verboten: die
        // Tailwind-Erkennung sieht nur statische Klassenstrings.
        for kind in ALL_KINDS {
            let c = person_kind_badge_class(&kind);
            assert!(!c.contains('#'), "raw hex value in class string: {c}");
        }
    }

    #[test]
    fn label_key_is_one_per_variant() {
        assert_eq!(
            person_kind_label_key(&PersonKindTO::Linked),
            Key::PersonKindLinked
        );
        assert_eq!(
            person_kind_label_key(&PersonKindTO::SalesPersonOnly),
            Key::PersonKindSalesPersonOnly
        );
        assert_eq!(
            person_kind_label_key(&PersonKindTO::UserOnly),
            Key::PersonKindUserOnly
        );
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("person_kind_badge.rs");
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-black",
            "border-gray-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }
}
