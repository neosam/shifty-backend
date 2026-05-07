//! Browser-local UI preferences backed by `localStorage`.
//!
//! Mirrors the shape of `service::theme` for a second per-browser preference:
//! the layout (cards or table) of the working-hours mini overview rendered
//! below the shift plan.

#[allow(dead_code)]
pub const WORKING_HOURS_LAYOUT_KEY: &str = "shifty.ui.workingHoursLayout";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum WorkingHoursLayout {
    #[default]
    Cards,
    Table,
}

impl WorkingHoursLayout {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            WorkingHoursLayout::Cards => "cards",
            WorkingHoursLayout::Table => "table",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cards" => Some(WorkingHoursLayout::Cards),
            "table" => Some(WorkingHoursLayout::Table),
            _ => None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub fn get_working_hours_layout() -> WorkingHoursLayout {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(WORKING_HOURS_LAYOUT_KEY).ok().flatten())
        .and_then(|v| WorkingHoursLayout::from_str(&v))
        .unwrap_or_default()
}

#[cfg(target_arch = "wasm32")]
pub fn set_working_hours_layout(layout: WorkingHoursLayout) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(WORKING_HOURS_LAYOUT_KEY, layout.as_str());
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_working_hours_layout() -> WorkingHoursLayout {
    WorkingHoursLayout::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn set_working_hours_layout(_layout: WorkingHoursLayout) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cards() {
        assert_eq!(
            WorkingHoursLayout::from_str("cards"),
            Some(WorkingHoursLayout::Cards)
        );
    }

    #[test]
    fn parses_table() {
        assert_eq!(
            WorkingHoursLayout::from_str("table"),
            Some(WorkingHoursLayout::Table)
        );
    }

    #[test]
    fn unknown_value_returns_none() {
        assert!(WorkingHoursLayout::from_str("gallery").is_none());
        assert!(WorkingHoursLayout::from_str("").is_none());
        assert!(WorkingHoursLayout::from_str("CARDS").is_none());
    }

    #[test]
    fn round_trip_via_as_str() {
        for layout in [WorkingHoursLayout::Cards, WorkingHoursLayout::Table] {
            assert_eq!(WorkingHoursLayout::from_str(layout.as_str()), Some(layout));
        }
    }

    #[test]
    fn default_is_cards() {
        assert_eq!(WorkingHoursLayout::default(), WorkingHoursLayout::Cards);
    }

    #[test]
    fn storage_key_is_namespaced() {
        assert!(WORKING_HOURS_LAYOUT_KEY.starts_with("shifty.ui."));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_get_returns_default() {
        assert_eq!(get_working_hours_layout(), WorkingHoursLayout::Cards);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn non_wasm_set_does_not_panic() {
        set_working_hours_layout(WorkingHoursLayout::Table);
        // No panic and no observable side effect.
        assert_eq!(get_working_hours_layout(), WorkingHoursLayout::Cards);
    }
}
