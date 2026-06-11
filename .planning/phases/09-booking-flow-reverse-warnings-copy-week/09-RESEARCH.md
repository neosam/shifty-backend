# Phase 9: Booking-Flow Reverse-Warnings (+ Copy-Week Cleanup) - Research

**Researched:** 2026-06-11
**Domain:** Dioxus frontend — API call-site switch, confirm-dialog with optimistic rollback, dead-code removal, i18n
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** The single editor booking call-site (`page/shiftplan.rs:409` → `loader::register_user_to_slot` → `api::add_booking`) switches to `POST /shiftplan-edit/booking`. Returns `BookingCreateResultTO { booking, warnings }`. No other booking call-sites exist.
- **D-02:** Old `POST /booking` (`rest/src/booking.rs:100`) stays untouched. Regression-lock via grep-check.
- **D-03:** New endpoint is not a dry-run — it persists immediately (201). Warnings-empty → reload. Warnings-present → Dioxus Dialog. "Abbrechen" → DELETE rollback → reload. "Trotzdem buchen" → keep → reload.
- **D-04:** Rollback DELETE failure: show via `error_handler`/Toast AND call `update_shiftplan()`. No silent-swallow on rollback error.
- **D-05/D-06:** Copy-Week fully out of scope (UI). Dead code to remove: `ShiftPlanAction::CopyFromPreviousWeek` (`:66` + `:523`), `api::copy_week` (`:237`), `loader::copy_from_previous_week` (`:318`), i18n key `ShiftplanTakeLastWeek` (en/de/cs).
- **D-07:** Backend `POST /shiftplan-edit/copy-week` + `CopyWeekResultTO` stay untouched.
- **D-08:** ROADMAP (Phase-9-Titel + SC2) and REQUIREMENTS (FUI-A-06) to be updated as part of this phase.
- **D-09:** `AbsenceWarningDisplay`/`WarningList` component moved from `page/absences.rs` to `component/`. Extended with `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded`. `absences.rs` and `shiftplan.rs` both use the shared component. `WarningsList` newtype-PartialEq wrapper pattern preserved.
- **D-10:** Only booking-path warning variants here: `BookingOnAbsenceDay`, `BookingOnUnavailableDay`, `PaidEmployeeLimitExceeded`. Forward variants (`AbsenceOverlapsBooking`, `AbsenceOverlapsManualUnavailable`) remain in `absences.rs`' consumer.
- **D-11:** Warning texts show Person + Date + Reason. Person name resolved from Action context (the booked person is known at call-site). Side-join pattern from Plan 08-04.
- **D-12:** Buttons: primary "Trotzdem buchen", secondary "Abbrechen". Header Singular/Plural with `{count}` interpolation analog `AbsenceWarningHeaderSingular`/`Plural`. All new texts in De/En/Cs.
- **D-13:** 403 stays silently swallowed (existing `AddUserToSlot` pattern). 422 via `error_handler`.
- **D-14:** Full test set analog Phase 8: SSR snapshots per warning variant, empty-array → no dialog, rollback-action dispatch, Per-Locale-Reference-Matcher tests, i18n parity test group extension.

### Claude's Discretion

- Exact Tailwind styling of the dialog.
- Whether the shared component is named `component/warning_list.rs` or `AbsenceWarningDisplay` renamed/moved.
- Exact i18n key names.
- Whether `api::add_booking` is extended or a new function (e.g. `book_slot_with_conflict_check`) is added; analogous loader-wrapper signature.
- Exact wave/plan split (small phase — likely 1–2 plans).

### Deferred Ideas (OUT OF SCOPE)

- Copy-Week UI reactivation (backend endpoint stays; no frontend consumer).
- Optional migration of `absences.rs` to the shared Warning component beyond what Phase 9 requires.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| FUI-A-05 | Booking-flow switched to `POST /shiftplan-edit/booking`; `BookingCreateResultTO.warnings[]` rendered as non-blocking Dioxus confirm-dialog with Optimistic-Create + Rollback semantics | Endpoint verified in `rest/src/shiftplan_edit.rs:134`; `BookingCreateResultTO` verified in `rest-types/src/lib.rs:1912`; `Dialog` component verified in `component/dialog.rs`; `remove_booking` API verified at `api.rs:227` |
| FUI-A-06 | DROPPED 2026-06-11 — dead Copy-Week frontend code cleanup only | Dead code confirmed at exact file:line references; no new UI required |
</phase_requirements>

---

## Summary

Phase 9 is a **small, self-contained frontend phase** with two tasks: (1) switch the single shiftplan-editor booking call-site to the conflict-aware endpoint and render its warnings in a Dioxus confirm-dialog with rollback semantics, and (2) remove dead Copy-Week frontend code. No backend changes. All required DTOs/endpoints were built in v1.0 Phase 3 and remain unmodified.

The implementation has very few unknowns because the project already contains all reusable assets: the `Dialog` component with footer-slot and `Btn` buttons, the `WarningList` component in `absences.rs` (to be refactored into `component/`), the `WarningsList` newtype wrapper, the `remove_booking` API function, the `error_handler`/`result_handler` sinks, and the complete `WarningTO` enum with all 5 variants verified in rest-types. The Phase 8 SSR-snapshot and Per-Locale-Reference-Matcher test patterns are established and must be replicated.

The **single non-trivial design question** is person-name resolution for `BookingOnUnavailableDay` (which carries `year/week/day_of_week`, not a date or person name). The booked person is available from the existing `sales_persons_resource` in `shiftplan.rs`, so no additional API call is needed — the person-name side-join pattern (Plan 08-04) applies directly. `BookingOnAbsenceDay` carries `date` and `category` so its text is straightforward. `PaidEmployeeLimitExceeded` carries `current_paid_count`/`max_paid_employees` which renders without person-join.

**Primary recommendation:** One plan covering (a) api/loader signature change + shared warning component + dialog integration in `shiftplan.rs`, (b) copy-week dead-code removal, (c) i18n keys + all three locales, and (d) full test suite. Two plans are viable if the shared-component extraction warrants its own wave before the dialog wiring.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Booking endpoint call | Browser / Client (WASM api.rs) | — | HTTP call to backend; already exists as `add_booking`, signature changes to return `BookingCreateResultTO` |
| Warning render (list) | Browser / Client (shared component) | — | Pure presentation of `WarningTO` slice; shared between `absences.rs` and `shiftplan.rs` |
| Confirm-dialog orchestration | Browser / Client (page/shiftplan.rs) | — | Owns the action handler, dialog open-state signal, and rollback dispatch |
| Rollback (DELETE booking) | Browser / Client (api.rs + loader.rs) | — | `remove_booking` already exists; called from shiftplan action handler |
| i18n keys | Browser / Client (i18n/mod.rs + 3 locale files) | — | All new dialog/warning strings must be in en/de/cs simultaneously |
| Dead code removal | Browser / Client (frontend files only) | — | `ShiftPlanAction`, `api::copy_week`, `loader::copy_from_previous_week`, `Key::ShiftplanTakeLastWeek` |

---

## Standard Stack

### Core (all verified in this session)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `dioxus` | 0.6 (from Cargo.toml) | Component framework, RSX, signals, coroutines | The only frontend framework in use [VERIFIED: Cargo.toml] |
| `dioxus-ssr` | 0.6 | SSR rendering for tests | Used in 60+ existing test sites [VERIFIED: grep] |
| `rest_types` | workspace | Shared DTOs, `WarningTO`, `BookingCreateResultTO`, `BookingTO` | Single source of truth [VERIFIED: rest-types/src/lib.rs] |
| `reqwest` | workspace | HTTP client | All API calls use reqwest [VERIFIED: api.rs] |
| `uuid` | workspace | Booking IDs for rollback call | Used throughout [VERIFIED: api.rs] |

### Reusable Assets Already in Codebase

| Asset | Location | Purpose in Phase 9 |
|-------|----------|---------------------|
| `Dialog` component | `component/dialog.rs` | Confirm-dialog shell with footer slot, ESC, backdrop, scroll-lock |
| `Btn` / `BtnVariant` | `component/atoms/btn.rs` | "Trotzdem buchen" (Primary) and "Abbrechen" (Secondary) buttons |
| `WarningList` + `WarningsList` | `page/absences.rs:163–243` | Warning renderer — refactor into `component/`; extend with 3 reverse variants |
| `remove_booking` | `api.rs:227` | DELETE /booking/{id} — rollback path |
| `error_handler` / `result_handler` | `error.rs` | Error sink for rollback failure (D-04) |
| `AbsenceWarningHeaderSingular/Plural` | i18n de/en/cs | Header Singular/Plural pattern with `{count}` interpolation |
| `pin_de_locale()` + SSR render helper | `page/absences.rs:1989–2001` | Test locale fixture — replicate in new component tests |

---

## Architecture Patterns

### System Architecture Diagram

```
User clicks "Mitarbeiter buchen" in WeekView
         │
         ▼
ShiftPlanAction::AddUserToSlot dispatched to coroutine
         │
         ▼
loader::register_user_to_slot(config, slot_id, user_id, week, year)
         │ (currently calls add_booking → POST /booking)
         ▼ (PHASE 9 CHANGE)
api::book_slot_with_conflict_check(config, …)
         │  POST /shiftplan-edit/booking
         │  returns BookingCreateResultTO { booking, warnings }
         ▼
warnings.is_empty()?
    ├─ YES → update_shiftplan() — done
    └─ NO  → store booking_id + WarningsList in signals
              │
              ▼
         Dialog opens (component/dialog.rs)
         WarningList rendered (component/warning_list.rs — NEW)
              │
    ┌─────────┴──────────┐
    │                    │
"Abbrechen"        "Trotzdem buchen"
    │                    │
    ▼                    ▼
api::remove_booking  close dialog
(DELETE rollback)    update_shiftplan()
    │
    ├─ OK  → close dialog → update_shiftplan()
    └─ ERR → error_handler(e) → update_shiftplan()
```

### Recommended Structure Changes

```
shifty-dioxus/src/
├── api.rs                          # add_booking renamed or new fn; returns BookingCreateResultTO
├── loader.rs                       # register_user_to_slot signature update; copy_from_previous_week DELETED
├── component/
│   ├── mod.rs                      # add pub use warning_list::WarningList; pub use warning_list::WarningsList
│   └── warning_list.rs             # NEW — WarningsList newtype + WarningList component (moved + extended)
├── page/
│   └── shiftplan.rs                # CopyFromPreviousWeek DELETED; AddUserToSlot handler updated; dialog state added
└── i18n/
    ├── mod.rs                      # New Key variants for booking warnings + dialog buttons
    ├── en.rs                       # New translations
    ├── de.rs                       # New translations
    └── cs.rs                       # New translations
```

### Pattern 1: Optimistic Create + Rollback

The booking is persisted immediately on POST. Warnings are delivered in the same 201 response. The dialog holds the booking ID for potential rollback.

```rust
// Source: derived from existing AddUserToSlot handler (shiftplan.rs:402–428) + D-03/D-04

// In the coroutine match arm:
ShiftPlanAction::AddUserToSlot { slot_id, sales_person_id, week, year } => {
    match loader::register_user_to_slot_with_conflict_check(
        config.to_owned(), slot_id, sales_person_id, week, year,
    ).await {
        Ok((booking_id, warnings)) if !warnings.is_empty() => {
            // Set state → dialog opens
            pending_rollback_id.set(Some(booking_id));
            pending_warnings.set(WarningsList(Rc::from(warnings.as_slice())));
        }
        Ok(_) => {
            // No warnings — plain success path
        }
        Err(ShiftyError::Reqwest(ref e))
            if e.status() == Some(reqwest::StatusCode::FORBIDDEN) => {
            // D-13: silent-swallow 403
        }
        Err(e) => {
            crate::error::error_handler(e); // D-13: 422 surfaced
        }
    }
    update_shiftplan();
}

// "Abbrechen" path (new action or inline handler):
ShiftPlanAction::RollbackBooking(booking_id) => {
    // D-04: show error AND reload even on failure
    if let Err(e) = api::remove_booking(config.to_owned(), booking_id).await {
        crate::error::error_handler(e);
    }
    pending_rollback_id.set(None);
    pending_warnings.set(WarningsList::empty());
    update_shiftplan();
}
```
[VERIFIED: existing patterns in shiftplan.rs:402–428 and error.rs]

### Pattern 2: WarningList — Extended for Booking Variants

The existing `WarningList` in `absences.rs` only renders `AbsenceOverlapsBooking` and `AbsenceOverlapsManualUnavailable`. The other three variants have a fallback arm (`_ => rsx! { fallback }`). Phase 9 moves this component to `component/warning_list.rs` and fills in the three booking-path variants.

```rust
// Source: verified from absences.rs:197–243 + rest-types/src/lib.rs:1757–1805

// In component/warning_list.rs:
match warning {
    WarningTO::AbsenceOverlapsBooking { date, .. } => {
        // existing: "{date} text"
    }
    WarningTO::AbsenceOverlapsManualUnavailable { .. } => {
        // existing: manual text
    }
    // NEW for Phase 9 (D-09):
    WarningTO::BookingOnAbsenceDay { date, category, .. } => {
        // "Person ist am {date} als {category} eingetragen."
        // date via i18n.format_date(date); category via AbsenceCategory key
        // person_name passed as prop from shiftplan context
    }
    WarningTO::BookingOnUnavailableDay { year, week, day_of_week, .. } => {
        // "Person ist in KW {week}/{year} ({weekday}) als nicht verfügbar markiert."
        // DayOfWeekTO::Monday.. mapped to weekday i18n key
    }
    WarningTO::PaidEmployeeLimitExceeded { current_paid_count, max_paid_employees, .. } => {
        // "Bezahlt-Limit überschritten ({current}/{max})."
        // No person name needed — slot-level warning
    }
}
```
[VERIFIED: WarningTO variants at rest-types/src/lib.rs:1757–1805]

### Pattern 3: Dialog with Footer Buttons (existing)

```rust
// Source: component/dialog.rs — verified in this session

Dialog {
    open: *show_booking_warning_dialog.read(),
    on_close: move |_| {
        // "Abbrechen" triggers rollback (D-03)
        cr.send(ShiftPlanAction::RollbackBooking(*pending_rollback_id.read()
            .expect("rollback id must be set when dialog is open")));
        show_booking_warning_dialog.set(false);
    },
    title: ImStr::from(header_text),
    footer: Some(rsx! {
        Btn {
            variant: BtnVariant::Secondary,
            on_click: move |_| { /* same as on_close */ },
            "Abbrechen"  // Key::BookingWarningDialogCancel
        }
        Btn {
            variant: BtnVariant::Primary,
            on_click: move |_| {
                show_booking_warning_dialog.set(false);
                update_shiftplan();
            },
            "Trotzdem buchen"  // Key::BookingWarningDialogConfirm
        }
    }),
    WarningList {
        warnings: *pending_warnings.read(),
        // person_name injected from current_sales_person context
    }
}
```
[VERIFIED: dialog.rs Props at lines 113–132; Btn at atoms/btn.rs:63–106]

### Anti-Patterns to Avoid

- **`window.confirm`:** Never use browser confirm/alert for warnings. Dioxus `Dialog` component is the standard (established in Phase 8, referenced in CLAUDE.md).
- **Dynamic Tailwind class names:** Use `match` returning `&'static str` literals. The Tailwind config only emits statically-detectable class names. [VERIFIED: CONVENTIONS.md]
- **`Locale::En` instead of `Locale::De` in de.rs:** Historical bug (Pitfall 2 from Plan 08-04). Every new `add_text` in `de.rs` must use `Locale::De`. The Per-Locale-Reference-Matcher test pattern guards against this.
- **Swallowing rollback DELETE errors:** D-04 requires surfacing them via `error_handler` AND calling `update_shiftplan()`.
- **Missing `WarningsList` newtype on props:** `WarningTO` does not implement `PartialEq`. Direct `Rc<[WarningTO]>` props will fail to compile with `#[derive(Props, Clone, PartialEq)]`. Use the `WarningsList` wrapper with `Rc::ptr_eq` equality. [VERIFIED: absences.rs:163–188]

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Modal with ESC + backdrop | Custom overlay div | `component/dialog.rs` | Already handles scroll-lock, WASM/non-WASM split, SSR-safe, tested (21 tests) |
| Warning list rendering | Inline match in shiftplan.rs | `component/warning_list.rs` (refactored from absences.rs) | Single source of truth for both surfaces |
| Rollback DELETE | Custom fetch | `api::remove_booking` (api.rs:227) | Already exists and tested by design |
| Error surfacing | `eprintln!` / JS alert | `error_handler` (error.rs:62) | Routes to browser DevTools via tracing |
| i18n lookups | String literals | `I18N.read().t(Key::…)` / `.t_m(Key::…, map)` | Locale fallback, `??` sentinel, placeholder substitution |

---

## Dead Code Inventory (Copy-Week Cleanup, D-06)

All items verified via grep and direct file inspection:

| Symbol | File | Line(s) | Action |
|--------|------|---------|--------|
| `ShiftPlanAction::CopyFromPreviousWeek` | `page/shiftplan.rs` | 66 (enum variant), 523–533 (match arm) | Delete both |
| `api::copy_week` | `api.rs` | 237–251 | Delete function |
| `loader::copy_from_previous_week` | `loader.rs` | 318–326 | Delete function |
| `Key::ShiftplanTakeLastWeek` | `i18n/mod.rs` | 79 (enum variant) | Delete |
| `ShiftplanTakeLastWeek` en translation | `i18n/en.rs` | ~32 | Delete `i18n.add_text(Locale::En, Key::ShiftplanTakeLastWeek, "Add last week")` |
| `ShiftplanTakeLastWeek` de translation | `i18n/de.rs` | ~32–36 | Delete `i18n.add_text(Locale::De, Key::ShiftplanTakeLastWeek, "Letzte Woche hinzufügen")` |
| `ShiftplanTakeLastWeek` cs translation | `i18n/cs.rs` | ~32–36 | Delete `i18n.add_text(Locale::Cs, Key::ShiftplanTakeLastWeek, "Přidat minulý týden")` |

**Compiler verification:** After removal, `cargo build --target wasm32-unknown-unknown` must pass with zero unused-variable or unreachable-pattern warnings on these symbols.

**Backend items that stay untouched (D-07):**
- `rest/src/shiftplan_edit.rs:173` — `copy_week_with_conflict_check` handler
- `rest-types/src/lib.rs:1929–1942` — `CopyWeekResultTO` struct

---

## Common Pitfalls

### Pitfall 1: Locale::En-instead-of-Locale::De in de.rs (Pitfall 2 from Phase 8)
**What goes wrong:** `add_text(Locale::En, Key::NewKey, "German text")` is written in `de.rs`. Compiles fine. German users see English strings.
**Why it happens:** Copy-paste from `en.rs` without changing the locale argument.
**How to avoid:** Every `add_text` in `de.rs` uses `Locale::De`; every `add_text` in `cs.rs` uses `Locale::Cs`. Write Per-Locale-Reference-Matcher tests that call `generate(Locale::De)` and assert the German text appears.
**Warning signs:** i18n parity test passes but reference-matcher test fails; German UI shows English strings.

### Pitfall 2: Dialog open/close state and rollback ID lifecycle
**What goes wrong:** Closing the dialog via the X button (not "Abbrechen") does not trigger rollback, leaving a phantom booking.
**Why it happens:** The `on_close` handler for X-button and Backdrop fires independently of the "Abbrechen" button.
**How to avoid:** Route ALL close paths (X button, backdrop click, ESC) through the same rollback dispatch. The `Dialog.on_close` callback is the single close point — wire rollback there. "Trotzdem buchen" clears `pending_rollback_id` BEFORE calling `on_close`.
**Warning signs:** Booking appears in shiftplan after clicking X or pressing ESC without confirming.

### Pitfall 3: WarningsList PartialEq on props
**What goes wrong:** Using `Rc<[WarningTO]>` directly in a `#[derive(Props, Clone, PartialEq)]` struct fails to compile because `WarningTO` does not implement `PartialEq`.
**Why it happens:** `WarningTO` contains `time::Date`, `AbsenceCategoryTO`, `Uuid` — all `PartialEq`, but the blanket impl for slices is not generated automatically for all cases across the feature-flag boundary.
**How to avoid:** Wrap in `WarningsList` newtype using `Rc::ptr_eq` as the equality check. This is the established pattern in `absences.rs:163–188`.
**Warning signs:** Compile error "the trait `PartialEq` is not satisfied for …".

### Pitfall 4: api.rs `BookingCreateResultTO` import gap
**What goes wrong:** `api.rs` imports from `rest_types` but `BookingCreateResultTO` is not in the current import list.
**Why it happens:** The import at `api.rs:3–13` lists only the currently-used types. `BookingCreateResultTO` and `WarningTO` are not imported.
**How to avoid:** Add `BookingCreateResultTO` (and `WarningTO` if needed) to the `use rest_types::{…}` block in `api.rs`.
**Warning signs:** Compile error `unresolved import rest_types::BookingCreateResultTO`.

### Pitfall 5: Rollback booking_id from pending signal vs. closed-dialog access
**What goes wrong:** By the time the rollback coroutine action fires, the `pending_rollback_id` signal may have been cleared by an intermediate reload.
**Why it happens:** Signal writes from different async paths can interleave.
**How to avoid:** Pass the `booking_id` directly as a parameter of the rollback action (`ShiftPlanAction::RollbackBooking(booking_id: Uuid)`) rather than reading it from a signal inside the handler. This is the same pattern used for `RemoveUserFromSlot` (passes UUIDs as action data).

### Pitfall 6: `BookingOnUnavailableDay` has no `date` field
**What goes wrong:** Trying to call `i18n.format_date(date)` on `BookingOnUnavailableDay` — it has `year`, `week`, `day_of_week: DayOfWeekTO`, no `time::Date`.
**Why it happens:** Different backend warning variants have different field shapes.
**How to avoid:** Render as "KW {week}/{year} ({weekday})" using `i18n.t(DayOfWeek Key)` for the weekday name. Do not try to reconstruct a `Date` from week/year/day_of_week in the frontend.

---

## Code Examples

### Existing booking call-site (current, to be replaced)

```rust
// Source: api.rs:197–225 (VERIFIED in this session)
pub async fn add_booking(
    config: Config,
    sales_person_id: Uuid,
    slot_id: Uuid,
    week: u8,
    year: u32,
) -> Result<(), reqwest::Error> {
    let url: String = format!("{}/booking", config.backend);
    let booking_to = BookingTO { id: Uuid::nil(), sales_person_id, slot_id,
        calendar_week: week as i32, year, created: None, deleted: None,
        created_by: None, deleted_by: None, version: Uuid::nil() };
    let client = reqwest::Client::new();
    let response = client.post(url).json(&booking_to).send().await?;
    response.error_for_status_ref()?;
    Ok(())
}
```

### Target endpoint (backend contract — verified)

```rust
// Source: rest/src/shiftplan_edit.rs:134–155 (VERIFIED)
// POST /shiftplan-edit/booking
// body: BookingTO (same shape)
// response: 201 BookingCreateResultTO { booking: BookingTO, warnings: Vec<WarningTO> }
// errors: 403 Forbidden, 422 Validation

// rest-types/src/lib.rs:1912–1914 (VERIFIED)
pub struct BookingCreateResultTO {
    pub booking: BookingTO,
    pub warnings: Vec<WarningTO>,
}
```

### WarningList component (current, from absences.rs — to be moved + extended)

```rust
// Source: page/absences.rs:163–243 (VERIFIED in this session)
// WarningsList newtype pattern:
pub struct WarningsList(pub Rc<[WarningTO]>);
impl PartialEq for WarningsList {
    fn eq(&self, other: &Self) -> bool { Rc::ptr_eq(&self.0, &other.0) }
}
// WarningList component reads I18N, renders count-based header, iterates warnings
// Current match arms: AbsenceOverlapsBooking (uses {date}), AbsenceOverlapsManualUnavailable
// Phase 9 adds: BookingOnAbsenceDay, BookingOnUnavailableDay, PaidEmployeeLimitExceeded
```

### pin_de_locale() SSR test fixture (canonical, from absences.rs)

```rust
// Source: page/absences.rs:1989–2001 (VERIFIED in this session)
fn render(comp: fn() -> Element) -> String {
    let mut vdom = VirtualDom::new(comp);
    vdom.rebuild_in_place();
    dioxus_ssr::render(&vdom)
}
fn pin_de_locale() {
    use_hook(|| { *I18N.write() = generate(Locale::De); });
}
```

### Per-Locale-Reference-Matcher pattern (from i18n/mod.rs tests)

```rust
// Source: i18n/mod.rs:711–724 (VERIFIED in this session)
#[test]
fn i18n_booking_warning_keys_match_german_reference() {
    let i18n = generate(Locale::De);
    assert_eq!(
        i18n.t(Key::BookingWarningDialogConfirm).as_ref(),
        "Trotzdem buchen"
    );
    assert_eq!(
        i18n.t(Key::BookingWarningDialogCancel).as_ref(),
        "Abbrechen"
    );
}
```

---

## i18n Key Design

New keys required (exact names are Claude's discretion per D-14, below are recommended names):

| Recommended Key | Purpose | German text (example) |
|---|---|---|
| `BookingWarningDialogHeaderSingular` | Dialog title — 1 warning | "Hinweis · 1 Konflikt" |
| `BookingWarningDialogHeaderPlural` | Dialog title — N warnings (`{count}`) | "Hinweis · {count} Konflikte" |
| `BookingWarningDialogConfirm` | Primary button | "Trotzdem buchen" |
| `BookingWarningDialogCancel` | Secondary button | "Abbrechen" |
| `BookingWarningOnAbsenceDay` | Per-item text (uses `{date}`, `{category}`) | "Mitarbeiter ist am {date} als {category} abwesend." |
| `BookingWarningOnUnavailableDay` | Per-item text (uses `{week}`, `{year}`, `{day}`) | "Mitarbeiter ist in KW {week}/{year} ({day}) als nicht verfügbar markiert." |
| `BookingWarningPaidLimitExceeded` | Per-item text (uses `{current}`, `{max}`) | "Bezahlt-Limit überschritten ({current}/{max})." |

Keys to remove: `Key::ShiftplanTakeLastWeek` (verified present in mod.rs:79, en.rs:~32, de.rs:~34, cs.rs:~34).

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) + `dioxus-ssr` for SSR component tests |
| Config file | None (no `pytest.ini` equivalent) |
| Quick run command | `cargo test --package shifty-dioxus warning` (filter by name) |
| Full suite command | `cargo test --package shifty-dioxus` |
| WASM build gate | `cargo build --target wasm32-unknown-unknown` from `shifty-dioxus/` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| FUI-A-05 | `WarningList` renders `BookingOnAbsenceDay` with date+category text | SSR snapshot | `cargo test warning_list_renders_booking_on_absence_day` | ❌ Wave 0 — new file |
| FUI-A-05 | `WarningList` renders `BookingOnUnavailableDay` with week/year/day text | SSR snapshot | `cargo test warning_list_renders_booking_on_unavailable_day` | ❌ Wave 0 |
| FUI-A-05 | `WarningList` renders `PaidEmployeeLimitExceeded` with count/max text | SSR snapshot | `cargo test warning_list_renders_paid_limit_exceeded` | ❌ Wave 0 |
| FUI-A-05 | Empty warnings array → `WarningList` renders nothing | SSR snapshot | `cargo test warning_list_empty_renders_nothing` | ❌ Wave 0 |
| FUI-A-05 | Dialog header uses Singular text when exactly 1 warning | SSR snapshot | `cargo test booking_warning_dialog_singular_header` | ❌ Wave 0 |
| FUI-A-05 | Dialog header uses Plural text with `{count}` when N>1 warnings | SSR snapshot | `cargo test booking_warning_dialog_plural_header` | ❌ Wave 0 |
| FUI-A-05 | New i18n keys present in all 3 locales | i18n parity | `cargo test i18n_booking_warning_keys_present_in_all_locales` | ❌ Wave 0 |
| FUI-A-05 | German translations use `Locale::De` (Pitfall-2 guard) | i18n reference | `cargo test i18n_booking_warning_keys_match_german_reference` | ❌ Wave 0 |
| FUI-A-06 cleanup | `ShiftplanTakeLastWeek` key removed — compile passes | WASM build | `cargo build --target wasm32-unknown-unknown` | Existing gate |
| FUI-A-06 cleanup | No `copy_week`/`CopyFromPreviousWeek`/`copy_from_previous_week` in frontend sources | Source grep | `cargo test no_copy_week_in_source` (new self-test) | ❌ Wave 0 |
| SC3 (regression lock) | `POST /booking` endpoint still present in backend — handler unmodified | Source grep | `cargo test booking_endpoint_regression_lock` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --package shifty-dioxus` (full frontend suite)
- **Per wave merge:** `cargo test --package shifty-dioxus` + WASM build gate
- **Phase gate:** Full frontend suite green + WASM build exit 0 + backend `cargo test --workspace` (regression check)

### Wave 0 Gaps

- [ ] `shifty-dioxus/src/component/warning_list.rs` — new file; covers `WarningList`, `WarningsList`, SSR tests for all 5 warning variants
- [ ] New i18n keys in `mod.rs`, `en.rs`, `de.rs`, `cs.rs` — presence + reference-matcher tests in `i18n/mod.rs`
- [ ] Source self-test `no_copy_week_in_source` — guards against Copy-Week code re-introduction

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Auth handled by backend; this phase adds no new auth surface |
| V3 Session Management | no | Session management not touched |
| V4 Access Control | no | 403 from backend is silently swallowed (D-13); no new privilege gate |
| V5 Input Validation | yes (minimal) | Booking ID from backend response is a `Uuid` — validated by serde deserialization |
| V6 Cryptography | no | No crypto in this phase |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Phantom booking on rollback failure | Tampering | `update_shiftplan()` called even on DELETE error (D-04) — actual state becomes visible |
| Dialog bypass (ESC/X) leaving phantom booking | Tampering | Route all `on_close` paths through rollback dispatch (Pitfall 2 above) |

This phase has minimal security surface: it is a purely frontend change that adds a confirmation dialog before accepting a backend-persisted booking. All privilege enforcement remains on the backend (403 response).

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `POST /booking` (direct, no conflict info) | `POST /shiftplan-edit/booking` (conflict-aware, returns warnings) | v1.0 Phase 3 (backend); Phase 9 (frontend wiring) | Frontend finally uses the conflict-aware path |
| `window.confirm` for confirmations (JSX mockup) | Dioxus `Dialog` component | Phase 8 (Dialog primitive built) | Accessible, SSR-testable modal |
| `WarningTO` reverse variants rendered as fallback | Explicit per-variant rendering with Person+Date+Reason | Phase 9 | Meaningful warning text for booking conflicts |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `cargo build --target wasm32-unknown-unknown` gates both the start state (clean) and the end state (no new compile errors). The WASM build currently passes after Phase 8.6. | Validation Architecture | If WASM build is already broken entering Phase 9, the gate is unreliable. Executor should verify at start. |
| A2 | The `sales_persons_resource` in `shiftplan.rs` is always available when the dialog is open (it loaded before the booking action was dispatched). No additional API call is needed for person-name in warnings. | Architecture Patterns — Pattern 1 | If person was not yet loaded, name would be `"?"`. Low risk: resource loads on component mount before any booking action is available to the user. |

**If this table is near-empty:** All critical claims were verified directly in source files in this session. The two ASSUMED entries are low-risk operational assumptions.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / Rust toolchain | Build, test | ✓ | NixOS project | — |
| `wasm32-unknown-unknown` target | WASM build gate | ✓ (assumed from Phase 8.6 passing) | — | — |
| `dioxus-ssr` dev-dep | SSR tests | ✓ | 0.6 in Cargo.toml | — |

---

## Open Questions

1. **Name for the shared component file**
   - What we know: D-09 says move to `component/`; Claude's Discretion for exact name.
   - What's unclear: Whether to reuse `WarningList` (already used in absences.rs as the component name) or rename for clarity.
   - Recommendation: Name the file `component/warning_list.rs`, keep the component `WarningList`, keep `WarningsList` newtype. This minimizes changes to `absences.rs` — it just changes its import path.

2. **Whether `api::add_booking` is extended or a new function is created**
   - What we know: Claude's Discretion (D-14 item). Current `add_booking` returns `Result<(), reqwest::Error>`.
   - Recommendation: Create a separate function `api::book_slot_with_conflict_check` returning `Result<BookingCreateResultTO, reqwest::Error>`. Keep the old `add_booking` but it becomes unused — remove it or keep for future use. Adding a clear deprecation comment is sufficient if not removing immediately.

3. **Person name in `BookingOnUnavailableDay` warnings**
   - What we know: The warning carries `year/week/day_of_week` but no person name or `sales_person_id`. The booked person is known from the Action context.
   - Recommendation: Pass `person_name: Option<ImStr>` as a prop to `WarningList` for use in booking-path warnings. Absence-path callers (which don't have a single person context) pass `None`.

---

## Sources

### Primary (HIGH confidence)
- `rest-types/src/lib.rs:1757–1925` — `WarningTO` (5 variants), `BookingCreateResultTO`, `BookingTO` — verified directly
- `rest/src/shiftplan_edit.rs:134–155` — `book_slot_with_conflict_check` endpoint contract — verified directly
- `shifty-dioxus/src/api.rs:197–251` — current `add_booking`, `remove_booking`, `copy_week` — verified directly
- `shifty-dioxus/src/loader.rs:286–326` — `register_user_to_slot`, `copy_from_previous_week` — verified directly
- `shifty-dioxus/src/page/shiftplan.rs:55–73,400–533` — `ShiftPlanAction` enum + handler coroutine — verified directly
- `shifty-dioxus/src/component/dialog.rs` — full Dialog primitive — verified directly
- `shifty-dioxus/src/page/absences.rs:163–243` — `WarningsList`/`WarningList` — verified directly
- `shifty-dioxus/src/i18n/mod.rs:55–536,552–759` — `Key` enum + parity tests — verified directly
- `shifty-dioxus/src/i18n/de.rs:28–36,845–866` — German `ShiftplanTakeLastWeek` + `AbsenceWarning*` translations — verified directly
- `.planning/codebase/frontend/TESTING.md` — SSR test patterns, co-location conventions — verified directly
- `.planning/codebase/frontend/CONVENTIONS.md` — WarningsList, Props, Tailwind, i18n patterns — verified directly

### Secondary (MEDIUM confidence)
- Phase 8 plans (08-04, 08-05) documented via CONTEXT.md canonical_refs — Per-Locale-Reference-Matcher pattern and `WarningsList` wrapper origin confirmed by source inspection

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — All libraries and versions verified from Cargo.toml and source files
- Architecture: HIGH — All call-site chains verified from source; all DTOs verified from rest-types
- Pitfalls: HIGH — Based on established Phase 8 patterns documented in codebase maps and CONTEXT.md
- Dead-code inventory: HIGH — All symbols verified via grep with exact line references

**Research date:** 2026-06-11
**Valid until:** 2026-07-11 (stable domain — no fast-moving dependencies)
