# Phase 9: Booking-Flow Reverse-Warnings (+ Copy-Week Cleanup) - Pattern Map

**Mapped:** 2026-06-12
**Files analyzed:** 8 new/modified files
**Analogs found:** 8 / 8

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `shifty-dioxus/src/component/warning_list.rs` | component | request-response | `shifty-dioxus/src/page/absences.rs:163–244` | exact (move + extend) |
| `shifty-dioxus/src/component/mod.rs` | config | — | `shifty-dioxus/src/component/mod.rs` (itself) | exact (add two lines) |
| `shifty-dioxus/src/api.rs` | utility | request-response | `shifty-dioxus/src/api.rs:197–234` | exact (add function, remove one) |
| `shifty-dioxus/src/loader.rs` | utility | request-response | `shifty-dioxus/src/loader.rs:286–326` | exact (modify + delete) |
| `shifty-dioxus/src/page/shiftplan.rs` | component | event-driven | `shifty-dioxus/src/page/shiftplan.rs:402–533` | exact (extend coroutine match arm) |
| `shifty-dioxus/src/i18n/mod.rs` | config | — | `shifty-dioxus/src/i18n/mod.rs:480–487` (AbsenceWarning* keys) | exact (add Key variants + tests) |
| `shifty-dioxus/src/i18n/de.rs` | config | — | `shifty-dioxus/src/i18n/de.rs:846–866` (AbsenceWarning* translations) | exact |
| `shifty-dioxus/src/i18n/en.rs` + `cs.rs` | config | — | corresponding AbsenceWarning* blocks in each file | exact |

---

## Pattern Assignments

### `shifty-dioxus/src/component/warning_list.rs` (component, request-response)

**Analog:** `shifty-dioxus/src/page/absences.rs` lines 163–244

This is a direct extraction: the `WarningsList` newtype, `WarningListProps`, and `WarningList` component all move verbatim from `absences.rs` into this new file. The only changes are:
1. Remove the `AbsenceOverlapsBooking` and `AbsenceOverlapsManualUnavailable` match arms (they stay in `absences.rs`' local consumer or are kept — see D-10).
2. Add `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded` match arms.
3. Add a `person_name: Option<ImStr>` prop for booking-path callers.
4. Add a `suppress_header: bool` prop (default `false`) so callers that already provide a header (e.g. the booking `Dialog { title }`) can skip the internal header row, avoiding a double-header. Absence-path callers keep the default (internal header shown).

**Imports pattern** (copy from `absences.rs:19–26`):
```rust
use std::rc::Rc;
use dioxus::prelude::*;
use rest_types::WarningTO;
use crate::{
    base_types::ImStr,
    i18n::{generate, Key, Locale, I18N},
};
```

**WarningsList newtype + PartialEq pattern** (from `absences.rs:163–188`):
```rust
#[derive(Clone, Debug)]
pub struct WarningsList(pub Rc<[WarningTO]>);

impl PartialEq for WarningsList {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl WarningsList {
    pub fn empty() -> Self { Self(Rc::new([])) }
    pub fn len(&self) -> usize { self.0.len() }
    pub fn is_empty(&self) -> bool { self.0.is_empty() }
}
```

**Props struct pattern** (from `absences.rs:190–195`):
```rust
#[derive(Props, Clone, PartialEq)]
pub struct WarningListProps {
    pub warnings: WarningsList,
    #[props(default = false)]
    pub dense: bool,
    // New for Phase 9 — None for absence-path callers, Some(name) for booking-path:
    #[props(default = None)]
    pub person_name: Option<ImStr>,
    // New for Phase 9 — when true, the internal header row is NOT rendered.
    // The booking Dialog passes `suppress_header: true` because Dialog.title already
    // provides the "Hinweis · N Konflikt(e)" header; absences.rs keeps the default.
    #[props(default = false)]
    pub suppress_header: bool,
}
```

**Core render pattern** (from `absences.rs:197–244`; header logic is identical except guarded by `suppress_header`, only the match arms differ):
```rust
#[component]
pub fn WarningList(props: WarningListProps) -> Element {
    let i18n = I18N.read().clone();
    let count = props.warnings.len();
    if count == 0 {
        return rsx! {};
    }
    let header_text = if count == 1 {
        i18n.t(Key::AbsenceWarningHeaderSingular).to_string()
        // Phase 9: replace with Key::BookingWarningDialogHeaderSingular for booking context
    } else {
        i18n.t(Key::AbsenceWarningHeaderPlural)
            .as_ref()
            .replace("{count}", &count.to_string())
    };
    let pad_class = if props.dense { "p-2.5" } else { "p-3" };
    rsx! {
        div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md {pad_class} flex flex-col gap-2",
            // Phase 9: only render the internal header when NOT suppressed.
            if !props.suppress_header {
                div { class: "text-micro text-warn font-semibold uppercase", "{header_text}" }
            }
            ul { class: "list-disc pl-4 text-body text-ink",
                for warning in props.warnings.0.iter() {
                    li {
                        match warning {
                            // existing arms stay for absences.rs consumers:
                            WarningTO::AbsenceOverlapsBooking { date, .. } => { … }
                            WarningTO::AbsenceOverlapsManualUnavailable { .. } => { … }
                            // Phase 9 new arms:
                            WarningTO::BookingOnAbsenceDay { date, category, .. } => {
                                let body = i18n.t(Key::BookingWarningOnAbsenceDay)
                                    .as_ref()
                                    .replace("{person}", person)
                                    .replace("{date}", &date.to_string())
                                    .replace("{category}", &i18n.t(category_key(category)).to_string());
                                rsx! { "{body}" }
                            }
                            WarningTO::BookingOnUnavailableDay { week, year, day_of_week, .. } => {
                                let body = i18n.t(Key::BookingWarningOnUnavailableDay)
                                    .as_ref()
                                    .replace("{person}", person)
                                    .replace("{week}", &week.to_string())
                                    .replace("{year}", &year.to_string())
                                    .replace("{day}", &i18n.t(day_of_week_key(day_of_week)).to_string());
                                rsx! { "{body}" }
                            }
                            WarningTO::PaidEmployeeLimitExceeded { current_paid_count, max_paid_employees, .. } => {
                                let body = i18n.t(Key::BookingWarningPaidLimitExceeded)
                                    .as_ref()
                                    .replace("{current}", &current_paid_count.to_string())
                                    .replace("{max}", &max_paid_employees.to_string());
                                rsx! { "{body}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

`person` above is resolved from the `person_name` prop, falling back to `"–"` when `None`:
```rust
let person = props.person_name.as_deref().unwrap_or("–");
```

**SSR test pattern** (from `absences.rs:1989–2001`; replicate verbatim in the `#[cfg(test)]` module of `warning_list.rs`):
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
fn pin_de_locale() {
    use_hook(|| { *I18N.write() = generate(Locale::De); });
}
```

---

### `shifty-dioxus/src/component/mod.rs` (config)

**Analog:** `shifty-dioxus/src/component/mod.rs` (itself, lines 1–51)

Two lines to add following the existing `pub use` pattern (lines 37–51 show the style):
```rust
// Add to the module list (alphabetical with others):
pub mod warning_list;

// Add to the pub use re-exports:
pub use warning_list::{WarningList, WarningsList};
```

---

### `shifty-dioxus/src/api.rs` (utility, request-response)

**Analog:** `shifty-dioxus/src/api.rs:197–251` — the existing `add_booking`, `remove_booking`, `copy_week` functions.

**Imports addition** (from `api.rs:3–13`; add `BookingCreateResultTO` and `WarningTO`):
```rust
use rest_types::{
    // … existing types …
    BookingCreateResultTO, WarningTO,   // ADD THESE
    BookingTO, …
};
```

**New function pattern** — copy `add_booking` structure, change URL and return type:
```rust
// Source: api.rs:197–225 (existing add_booking to replace or supplement)
pub async fn book_slot_with_conflict_check(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<BookingCreateResultTO, reqwest::Error> {
    info!(
        "Booking slot (conflict-check) for user {sales_person_id}, slot {slot_id}, week {week}/{year}"
    );
    let url: String = format!("{}/shiftplan-edit/booking", config.backend);
    let booking_to = BookingTO {
        id: Uuid::nil(),
        sales_person_id,
        slot_id,
        calendar_week: week as i32,
        year,
        created: None,
        deleted: None,
        created_by: None,
        deleted_by: None,
        version: Uuid::nil(),
    };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    let result: BookingCreateResultTO = response.json().await?;
    info!("Booked");
    Ok(result)
}
```

**`remove_booking` stays unchanged** (api.rs:227–235 — already correct for rollback use).

**`copy_week` is deleted** (api.rs:237–251 — D-06).

---

### `shifty-dioxus/src/loader.rs` (utility, request-response)

**Analog:** `shifty-dioxus/src/loader.rs:286–326` — `register_user_to_slot` and `copy_from_previous_week`.

**Modified function pattern** (copy `register_user_to_slot` structure, change return type):
```rust
// Source: loader.rs:286–296 (existing, to replace)
pub async fn register_user_to_slot_with_conflict_check(
    config: Config,
    slot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    week: u8,
    year: u32,
) -> Result<(uuid::Uuid, Vec<rest_types::WarningTO>), ShiftyError> {
    info!("Add booking (conflict-check)");
    let result = api::book_slot_with_conflict_check(config, user_id, slot_id, week, year).await?;
    Ok((result.booking.id, result.warnings))
}
```

**`copy_from_previous_week` is deleted** (loader.rs:318–326 — D-06).

---

### `shifty-dioxus/src/page/shiftplan.rs` (component, event-driven)

**Analog:** `shifty-dioxus/src/page/shiftplan.rs:402–533` — the existing coroutine match arms.

**Action enum change** (from `shiftplan.rs:55–72`):
- Remove `CopyFromPreviousWeek` (line 66) — D-06.
- Add new variant for rollback (following `RemoveUserFromSlot` pattern at lines 55–62):
```rust
// Add after existing variants, before CopyFromPreviousWeek (which is deleted):
RollbackBooking(Uuid),  // booking_id passed directly (Pitfall 5 guard)
```

**Updated `AddUserToSlot` match arm** (replaces `shiftplan.rs:402–429`; preserves 403 silent-swallow from lines 419–423):
```rust
ShiftPlanAction::AddUserToSlot { slot_id, sales_person_id, week, year } => {
    info!("Registering user to slot");
    match loader::register_user_to_slot_with_conflict_check(
        config.to_owned(), slot_id, sales_person_id, week, year,
    ).await {
        Ok((booking_id, warnings)) if !warnings.is_empty() => {
            pending_rollback_id.set(Some(booking_id));
            pending_warnings.set(WarningsList(Rc::from(warnings.as_slice())));
            // Do NOT call update_shiftplan() yet — dialog still open
        }
        Ok(_) => {
            update_shiftplan();
        }
        Err(crate::error::ShiftyError::Reqwest(ref e))
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) => {
            // D-13: silently ignore 403
            update_shiftplan();
        }
        Err(e) => {
            crate::error::error_handler(e);
            update_shiftplan();
        }
    }
}
```

**New `RollbackBooking` match arm** (follow `RemoveUserFromSlot` pattern at lines 430–447):
```rust
ShiftPlanAction::RollbackBooking(booking_id) => {
    // D-04: surface error AND reload even on failure
    if let Err(e) = api::remove_booking(config.to_owned(), booking_id).await {
        crate::error::error_handler(e);
    }
    pending_rollback_id.set(None);
    pending_warnings.set(WarningsList::empty());
    update_shiftplan();
}
```

**Dialog RSX wiring** (pattern from `component/shiftplan_tab_bar.rs:201–243`, adapt for rollback). The `Dialog.title` provides the header, so `WarningList` is rendered with `suppress_header: true` to avoid a double-header:
```rust
// Add in the component's rsx! body, alongside other conditional Dialogs:
if !pending_warnings.read().is_empty() {
    let header_text = /* singular/plural from i18n, see absences.rs:204–211 */;
    let cancel_label = i18n.t(Key::BookingWarningDialogCancel).to_string();
    let confirm_label = i18n.t(Key::BookingWarningDialogConfirm).to_string();
    let warnings_snap = pending_warnings.read().clone();
    let rollback_id = pending_rollback_id.read().expect("rollback id set when warnings non-empty");
    let footer = rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            on_click: move |_| {
                cr.send(ShiftPlanAction::RollbackBooking(rollback_id));
            },
            "{cancel_label}"
        }
        Btn {
            variant: BtnVariant::Primary,
            on_click: move |_| {
                pending_rollback_id.set(None);
                pending_warnings.set(WarningsList::empty());
                update_shiftplan();
            },
            "{confirm_label}"
        }
    };
    rsx! {
        Dialog {
            open: true,
            on_close: move |_| {
                // All close paths (X, ESC, backdrop) → rollback (Pitfall 2 guard)
                cr.send(ShiftPlanAction::RollbackBooking(rollback_id));
            },
            title: ImStr::from(header_text.as_str()),
            footer: Some(footer),
            variant: DialogVariant::Auto,
            // suppress_header: Dialog.title already renders the header — avoid double-header
            WarningList { warnings: warnings_snap, person_name, suppress_header: true }
        }
    }
}
```

**`CopyFromPreviousWeek` match arm deleted** (shiftplan.rs:523–533 — D-06).

---

### `shifty-dioxus/src/i18n/mod.rs` (config)

**Analog:** `shifty-dioxus/src/i18n/mod.rs:480–487` — existing `AbsenceWarning*` keys in the `Key` enum; `mod.rs:660–708` — `i18n_absence_keys_present_in_all_locales` test; `mod.rs:710–724` — `i18n_absence_keys_match_german_reference` test.

**Key enum additions** (after `AbsenceWarningOverlapsManual` at ~line 487, following the naming convention):
```rust
// In the Key enum, add after AbsenceWarningOverlapsManual:
BookingWarningDialogHeaderSingular,
BookingWarningDialogHeaderPlural,
BookingWarningDialogConfirm,
BookingWarningDialogCancel,
BookingWarningOnAbsenceDay,
BookingWarningOnUnavailableDay,
BookingWarningPaidLimitExceeded,
```

**Key enum deletion:** Remove `ShiftplanTakeLastWeek` (line 79 — D-06).

**Parity test addition** (copy structure from `i18n_absence_keys_present_in_all_locales`, lines 666–708):
```rust
#[test]
fn i18n_booking_warning_keys_present_in_all_locales() {
    for locale in [Locale::En, Locale::De, Locale::Cs] {
        let i18n = generate(locale);
        for key in [
            Key::BookingWarningDialogHeaderSingular,
            Key::BookingWarningDialogHeaderPlural,
            Key::BookingWarningDialogConfirm,
            Key::BookingWarningDialogCancel,
            Key::BookingWarningOnAbsenceDay,
            Key::BookingWarningOnUnavailableDay,
            Key::BookingWarningPaidLimitExceeded,
        ] {
            let value = i18n.t(key);
            assert!(
                !value.is_empty() && value.as_ref() != "??",
                "missing translation for {:?} in {:?}: got `{}`",
                key, locale, value
            );
        }
    }
}
```

**Reference-matcher test addition** (copy structure from `i18n_absence_keys_match_german_reference`, lines 710–724):
```rust
#[test]
fn i18n_booking_warning_keys_match_german_reference() {
    // Pitfall-1 guard: de.rs must use Locale::De, not Locale::En
    let i18n = generate(Locale::De);
    assert_eq!(i18n.t(Key::BookingWarningDialogConfirm).as_ref(), "Trotzdem buchen");
    assert_eq!(i18n.t(Key::BookingWarningDialogCancel).as_ref(), "Abbrechen");
    assert_eq!(
        i18n.t(Key::BookingWarningDialogHeaderSingular).as_ref(),
        "Hinweis · 1 Konflikt"
    );
}
```

---

### `shifty-dioxus/src/i18n/de.rs` (config)

**Analog:** `shifty-dioxus/src/i18n/de.rs:846–866` — `AbsenceWarning*` translation block.

**Translations to add** (after existing AbsenceWarning block; use `Locale::De` throughout — Pitfall 1 guard). The per-item booking strings MUST use the `{person}` placeholder (canonical UI-SPEC §Copywriting) so the booked person's name is rendered, not a hardcoded "Mitarbeiter":
```rust
i18n.add_text(Locale::De, Key::BookingWarningDialogHeaderSingular, "Hinweis · 1 Konflikt");
i18n.add_text(Locale::De, Key::BookingWarningDialogHeaderPlural, "Hinweis · {count} Konflikte");
i18n.add_text(Locale::De, Key::BookingWarningDialogConfirm, "Trotzdem buchen");
i18n.add_text(Locale::De, Key::BookingWarningDialogCancel, "Abbrechen");
i18n.add_text(
    Locale::De,
    Key::BookingWarningOnAbsenceDay,
    "{person} ist am {date} als {category} abwesend.",
);
i18n.add_text(
    Locale::De,
    Key::BookingWarningOnUnavailableDay,
    "{person} ist in KW {week}/{year} ({day}) als nicht verfügbar markiert.",
);
i18n.add_text(
    Locale::De,
    Key::BookingWarningPaidLimitExceeded,
    "Bezahlt-Limit überschritten ({current}/{max}).",
);
```

**Translation to remove:** `ShiftplanTakeLastWeek` block (lines ~32–36 — D-06):
```rust
// DELETE THIS BLOCK:
i18n.add_text(Locale::De, Key::ShiftplanTakeLastWeek, "Letzte Woche hinzufügen");
```

---

### `shifty-dioxus/src/i18n/en.rs` (config)

**Analog:** Same pattern as `de.rs` but with `Locale::En`. Per-item booking strings use the `{person}` placeholder per the canonical UI-SPEC.

**Translations to add** (after existing AbsenceWarning block):
```rust
i18n.add_text(Locale::En, Key::BookingWarningDialogHeaderSingular, "Notice · 1 conflict");
i18n.add_text(Locale::En, Key::BookingWarningDialogHeaderPlural, "Notice · {count} conflicts");
i18n.add_text(Locale::En, Key::BookingWarningDialogConfirm, "Book anyway");
i18n.add_text(Locale::En, Key::BookingWarningDialogCancel, "Cancel");
i18n.add_text(Locale::En, Key::BookingWarningOnAbsenceDay, "{person} is absent on {date} as {category}.");
i18n.add_text(Locale::En, Key::BookingWarningOnUnavailableDay, "{person} is marked as unavailable in week {week}/{year} ({day}).");
i18n.add_text(Locale::En, Key::BookingWarningPaidLimitExceeded, "Paid employee limit exceeded ({current}/{max}).");
```

**Translation to remove:** `i18n.add_text(Locale::En, Key::ShiftplanTakeLastWeek, "Add last week");` (line ~32 — D-06).

---

### `shifty-dioxus/src/i18n/cs.rs` (config)

**Analog:** Same pattern as `de.rs` but with `Locale::Cs`. Per-item booking strings use the `{person}` placeholder per the canonical UI-SPEC.

**Translations to add:**
```rust
i18n.add_text(Locale::Cs, Key::BookingWarningDialogHeaderSingular, "Upozornění · 1 konflikt");
i18n.add_text(Locale::Cs, Key::BookingWarningDialogHeaderPlural, "Upozornění · {count} konflikty");
i18n.add_text(Locale::Cs, Key::BookingWarningDialogConfirm, "Zarezervovat stejně");
i18n.add_text(Locale::Cs, Key::BookingWarningDialogCancel, "Zrušit");
i18n.add_text(Locale::Cs, Key::BookingWarningOnAbsenceDay, "{person} je dne {date} nepřítomen jako {category}.");
i18n.add_text(Locale::Cs, Key::BookingWarningOnUnavailableDay, "{person} je v týdnu {week}/{year} ({day}) označen jako nedostupný.");
i18n.add_text(Locale::Cs, Key::BookingWarningPaidLimitExceeded, "Překročen limit placených zaměstnanců ({current}/{max}).");
```

**Translation to remove:** `ShiftplanTakeLastWeek` Cs block (line ~32–36 — D-06).

---

## Shared Patterns

### 403 Silent-Swallow
**Source:** `shifty-dioxus/src/page/shiftplan.rs:419–423`
**Apply to:** `AddUserToSlot` handler (D-13)
```rust
Err(crate::error::ShiftyError::Reqwest(ref e))
    if e.status() == Some(reqwest::StatusCode::FORBIDDEN) => {
    // Silently ignore forbidden booking errors
}
```

### Error Surfacing
**Source:** `shifty-dioxus/src/error.rs:62` (`error_handler`) and `:71` (`result_handler`)
**Apply to:** Rollback DELETE failure path (D-04), 422 responses from booking endpoint
```rust
crate::error::error_handler(e);
// followed unconditionally by:
update_shiftplan();
```

### Dialog + Footer Btn Pattern
**Source:** `shifty-dioxus/src/component/shiftplan_tab_bar.rs:201–243`
**Apply to:** Booking-warning confirm dialog in `shiftplan.rs`
```rust
let footer = rsx! {
    Btn { variant: BtnVariant::Secondary, on_click: move |_| { … }, "{cancel_label}" }
    Btn { variant: BtnVariant::Primary,   on_click: move |_| { … }, "{confirm_label}" }
};
Dialog {
    open: true,
    on_close: move |_| { /* ALL close paths route here */ },
    title: …,
    footer: Some(footer),
    variant: DialogVariant::Auto,
    …body…
}
```

### WarningsList Newtype (PartialEq on Props)
**Source:** `shifty-dioxus/src/page/absences.rs:163–188`
**Apply to:** `WarningList` props and any signal holding `Rc<[WarningTO]>` (Pitfall 3 guard)
```rust
impl PartialEq for WarningsList {
    fn eq(&self, other: &Self) -> bool { Rc::ptr_eq(&self.0, &other.0) }
}
```

### SSR Snapshot Test Fixture
**Source:** `shifty-dioxus/src/page/absences.rs:1989–2001`
**Apply to:** `#[cfg(test)]` module in `component/warning_list.rs`
```rust
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
fn pin_de_locale() {
    use_hook(|| { *I18N.write() = generate(Locale::De); });
}
```

### i18n Parity + Reference-Matcher Test Pattern
**Source:** `shifty-dioxus/src/i18n/mod.rs:666–724`
**Apply to:** New `i18n_booking_warning_keys_*` test functions in `i18n/mod.rs`
- Parity test: iterate all 3 locales, iterate all new keys, assert non-empty and `!= "??"`.
- Reference-matcher test: `generate(Locale::De)` + `assert_eq!(i18n.t(key).as_ref(), "…german…")` for at least 2–3 keys.

### Static Tailwind Class Pattern
**Source:** `shifty-dioxus/src/page/absences.rs:215` (warning box styling)
**Apply to:** All new styled divs in `warning_list.rs`
```rust
// Use literal &'static str class names — no runtime string building for class attributes
div { class: "border-l-[3px] border-warn bg-warn-soft rounded-md {pad_class} flex flex-col gap-2", … }
// pad_class is computed separately from a match → &'static str, then interpolated once
let pad_class = if props.dense { "p-2.5" } else { "p-3" };
```

---

## No Analog Found

None — every file in Phase 9 has a direct codebase analog.

---

## Metadata

**Analog search scope:** `shifty-dioxus/src/` (all subdirectories)
**Files scanned:** 12 source files read directly; Dead-code inventory verified in RESEARCH.md
**Pattern extraction date:** 2026-06-12
