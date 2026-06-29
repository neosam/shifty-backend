use crate::js;
use dioxus::prelude::*;

/// The currently selected (year, week) — a single shared guard truth for all three
/// week loaders (D-30-02). Initialized lazily to the current ISO week; overwritten
/// synchronously on mount and every week-switch BEFORE any LoadWeek dispatch
/// (D-30-01: guard truth must be ahead of loads).
pub static SELECTED_WEEK: GlobalSignal<(u32, u8)> =
    GlobalSignal::new(|| (js::get_current_year(), js::get_current_week()));

/// Imperative synchronous setter for `SELECTED_WEEK`. Call this BEFORE dispatching
/// any `LoadWeek` action or `reload_unavailable_days` call so the loaders' post-await
/// comparison is against the fresh selection, not the previous one.
pub fn set_selected_week(year: u32, week: u8) {
    *SELECTED_WEEK.write() = (year, week);
}

/// Pure staleness predicate — the one decision point shared by all three week loaders
/// (D-30-02) and the summary-card render-guard (D-30-03).
///
/// Returns `true` iff the `(year, week)` a loader loaded for still equals the
/// currently selected `(year, week)` passed as `selected_yw`. A `true` result
/// allows the store write; `false` means the result is stale and must be silently
/// dropped (no store write, no error, no log — D-30-01 / SC3).
///
/// Accepts plain tuples (no `GlobalSignal` read inside) so it is unit-testable
/// without a Dioxus runtime.
pub fn is_current_selection(result_yw: (u32, u8), selected_yw: (u32, u8)) -> bool {
    result_yw == selected_yw
}

#[cfg(test)]
mod tests {
    use super::is_current_selection;

    /// Match on both year and week → write allowed.
    #[test]
    fn match_same_year_and_week_allows_write() {
        assert!(
            is_current_selection((2026, 27), (2026, 27)),
            "identical (year, week) must return true"
        );
    }

    /// Stale week (same year, different week) → drop write.
    #[test]
    fn mismatch_stale_week_drops_write() {
        assert!(
            !is_current_selection((2026, 26), (2026, 27)),
            "result week 26 must not match selected week 27"
        );
    }

    /// Stale year (same week number, different year) → drop write.
    #[test]
    fn mismatch_stale_year_drops_write() {
        assert!(
            !is_current_selection((2025, 1), (2026, 1)),
            "result year 2025 must not match selected year 2026"
        );
    }

    /// Result week is later than the selection (selection moved backwards) → drop write.
    #[test]
    fn result_ahead_of_selection_drops_write() {
        assert!(
            !is_current_selection((2026, 27), (2026, 26)),
            "result week 27 must not match selected week 26"
        );
    }
}
