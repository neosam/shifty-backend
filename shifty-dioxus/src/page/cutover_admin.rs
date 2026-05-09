//! `CutoverAdminPage` stub (Phase 8.1 Plan 07).
//!
//! Plan 09 replaces this file with the full Single-File Page-Composition
//! (StageIndicator, ProfileStage, DryRunStage, CommitStage, SuccessPage,
//! DriftGroupSection, DriftEntryRow, EditExtraHoursModal,
//! TypeToConfirmDialog, IdempotenzBanner, StatBox + ssr-snapshot tests).
//!
//! This stub exists so `Route::AdminCutover {}` in router.rs compiles and the
//! TopBar Verwaltung-Submenu entry can land before the page implementation.

use dioxus::prelude::*;

use crate::component::TopBar;

#[component]
pub fn CutoverAdminPage() -> Element {
    rsx! {
        TopBar {}
        div {
            class: "p-md text-ink-muted",
            "Cutover-Migration UI — Plan 09 implements this page."
        }
    }
}
