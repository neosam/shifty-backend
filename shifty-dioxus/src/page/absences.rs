//! `AbsencesPage` — Top-Level Route `/absences` für CRUD gegen
//! `/absence-period` (Phase 8 Wave 5).
//!
//! Plan 05 Task 1 legt nur den Compile-Stub an, damit Routing- und TopBar-
//! Wiring (Task 1) kompilieren. Task 2 ersetzt diesen Stub durch die volle
//! UI-Komposition (Modal, WarningList, CategoryBadge, StatusPill,
//! VacationEntitlementCard, VacationPerPersonList, AbsenceList,
//! AbsenceFilterBar, StatsGrid, DeleteConfirmDialog, VersionConflictBanner,
//! SelfOverlapBanner) per `08-UI-SPEC.md`.

use dioxus::prelude::*;

use crate::component::TopBar;

#[component]
pub fn AbsencesPage() -> Element {
    rsx! {
        TopBar {}
        div {
            class: "p-md text-ink-muted",
            "Absences page (Plan 05 Task 2 will fill this in)"
        }
    }
}
