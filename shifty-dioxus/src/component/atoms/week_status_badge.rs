//! `WeekStatusBadge` — read-only, color-coded pill showing the calendar-week
//! status (KW-Status). Pure display atom: no state, no API calls.
//!
//! ## Visibility invariant (D-39-05, WST-02)
//!
//! The badge is **only** rendered for a set status. At [`WeekStatus::Unset`] a
//! non-shiftplaner sees *nothing* — no empty/grey pill. The caller enforces this
//! via [`should_show_badge`]; the class helper therefore never has to produce a
//! neutral badge (Unset is `unreachable!()` inside [`week_status_badge_class`]).
//!
//! ## Color semantics (D-39-08)
//!
//! Locked = bad (red), Planned = good (green), InPlanning = warn (amber). Only
//! static Tailwind design-token classes per match arm — no `format!()`, no raw
//! palette literals (Tailwind-detect + legacy-class gate); the token classes
//! (`bg-bad-soft`, `bg-good-soft`, `bg-warn-soft`) carry the semantic colors.

use dioxus::prelude::*;

use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::week_status::WeekStatus;

/// Whether the badge should be rendered at all. `Unset` renders nothing
/// (D-39-05, WST-02); every set status is shown.
pub(crate) fn should_show_badge(status: &WeekStatus) -> bool {
    !matches!(status, WeekStatus::Unset)
}

/// Static Tailwind design-token class string for the badge, one per set status
/// (D-39-08). `Unset` is `unreachable!()` — the badge is never rendered for it
/// (the caller gates via [`should_show_badge`]).
pub(crate) fn week_status_badge_class(status: &WeekStatus) -> &'static str {
    match status {
        WeekStatus::Locked => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-bad-soft border border-bad text-bad"
        }
        WeekStatus::Planned => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-good-soft border border-good text-good"
        }
        WeekStatus::InPlanning => {
            "inline-flex items-center px-2 py-0.5 rounded-sm text-small font-medium bg-warn-soft border border-warn text-warn"
        }
        WeekStatus::Unset => unreachable!("Badge wird nie fuer Unset gerendert"),
    }
}

/// i18n label key for a set status. `Unset` has no badge label.
pub(crate) fn week_status_label_key(status: &WeekStatus) -> Key {
    match status {
        WeekStatus::InPlanning => Key::WeekStatusInPlanning,
        WeekStatus::Planned => Key::WeekStatusPlanned,
        WeekStatus::Locked => Key::WeekStatusLocked,
        WeekStatus::Unset => Key::WeekStatusUnset,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct WeekStatusBadgeProps {
    /// The status to display. Only called for a set status; `Unset` must be
    /// gated out by the caller (D-39-05).
    pub status: WeekStatus,
}

/// Read-only color-coded status pill. Renders a `span` with the token class and
/// the translated label.
#[component]
pub fn WeekStatusBadge(props: WeekStatusBadgeProps) -> Element {
    let label = I18N.read().t(week_status_label_key(&props.status));
    rsx! {
        span {
            class: week_status_badge_class(&props.status),
            "{label}"
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::atoms::week_status_badge::{should_show_badge, week_status_badge_class};
    use crate::state::week_status::WeekStatus;

    #[test]
    fn should_show_badge_is_false_for_unset() {
        // D-39-05: a non-shiftplaner sees no element at Unset.
        assert!(!should_show_badge(&WeekStatus::Unset));
    }

    #[test]
    fn should_show_badge_is_true_for_set_states() {
        assert!(should_show_badge(&WeekStatus::InPlanning));
        assert!(should_show_badge(&WeekStatus::Planned));
        assert!(should_show_badge(&WeekStatus::Locked));
    }

    #[test]
    fn class_uses_bad_token_for_locked() {
        let c = week_status_badge_class(&WeekStatus::Locked);
        assert!(c.contains("bg-bad-soft"), "missing bg-bad-soft: {c}");
        assert!(c.contains("border-bad"), "missing border-bad: {c}");
        assert!(c.contains("text-bad"), "missing text-bad: {c}");
    }

    #[test]
    fn class_uses_good_token_for_planned() {
        let c = week_status_badge_class(&WeekStatus::Planned);
        assert!(c.contains("bg-good-soft"), "missing bg-good-soft: {c}");
        assert!(c.contains("border-good"), "missing border-good: {c}");
        assert!(c.contains("text-good"), "missing text-good: {c}");
    }

    #[test]
    fn class_uses_warn_token_for_in_planning() {
        let c = week_status_badge_class(&WeekStatus::InPlanning);
        assert!(c.contains("bg-warn-soft"), "missing bg-warn-soft: {c}");
        assert!(c.contains("border-warn"), "missing border-warn: {c}");
        assert!(c.contains("text-warn"), "missing text-warn: {c}");
    }

    #[test]
    fn class_carries_shared_shape_classes() {
        for status in [WeekStatus::InPlanning, WeekStatus::Planned, WeekStatus::Locked] {
            let c = week_status_badge_class(&status);
            assert!(c.contains("inline-flex"), "missing inline-flex: {c}");
            assert!(c.contains("rounded-sm"), "missing rounded-sm: {c}");
            assert!(c.contains("text-small"), "missing text-small: {c}");
            assert!(c.contains("font-medium"), "missing font-medium: {c}");
        }
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("week_status_badge.rs");
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
