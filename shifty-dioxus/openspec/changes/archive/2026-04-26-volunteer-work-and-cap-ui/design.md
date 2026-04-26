## Context

The frontend `WorkingHoursCategory` enum mirrors the backend `ExtraHoursCategoryTO`/`ExtraHoursReportCategoryTO` enums, with a domestic identifier-string round-trip used by the add-extra-hours form's `<select>` element. The state struct `EmployeeWorkDetails` mirrors `EmployeeWorkDetailsTO` directly; conversions are `TryFrom`-based and propagate every field. The employee report views (`EmployeeView`) render category-level aggregates as a flat list of `<TupleView>` items.

The backend change `weekly-planned-hours-cap` has shipped the following on the wire:

- `EmployeeWorkDetailsTO.cap_planned_hours_to_expected: bool` (default `false`).
- `ExtraHoursCategoryTO::VolunteerWork` and `ExtraHoursReportCategoryTO::VolunteerWork` (parameterless variants).
- `volunteer_hours: f32` on `ShortEmployeeReportTO`, `EmployeeReportTO`, `WorkingHoursReportTO`, and per-week `GroupedReportHours` (within the `WorkingHoursReportTO`).
- A new persisted billing-period `value_type` of `"volunteer"` carrying the same `value_delta` / `value_ytd_*` / `value_full_year` fields as every other value type.

The existing precedent `unpaid-leave-category` shipped a comparable feature (new category, new aggregate field, new dropdown option, new view row, new translations) and provides the architectural template for this change.

The frontend has no business-logic ownership over the cap or volunteer semantics; both live in the backend reporting service. The UI's job is plumbing, surfacing, and translating.

## Goals / Non-Goals

**Goals:**

- Expose the cap flag as an editable boolean on the work-details form with a localised helper text that explains the user-visible consequence.
- Treat `VolunteerWork` as a first-class category alongside `ExtraWork`, `Vacation`, `SickLeave`, etc. — same shape of plumbing, same shape of view rendering.
- Render `volunteer_hours` consistently as a labelled line item next to the existing per-category aggregates, on both the per-week and per-period sections of `EmployeeView`.
- Localise the new persisted `"VOLUNTEER"` billing-period `value_type` to the same category label used everywhere else.
- Keep the change additive: no existing call sites require behavioural modification beyond the compiler-driven extra match arms and field additions.

**Non-Goals:**

- Any custom UI for the cap-induced negative-balance edge case (planner books fewer than expected hours on a capped contract). Per backend spec, this is intentional and shows up as a normal negative balance; no special framing.
- Validation that prevents a planner from manually booking `VolunteerWork` against a non-capped person. The backend accepts this combination by design (see backend `volunteer-work-hours` Req 4).
- Any change to the `AddExtraHoursChoice` quick-pick component (separate code path, not currently the primary entry point and out of scope).
- Any backend or `rest-types` modification — the consumed wire fields all exist already.
- A separate "auto-attributed vs manual" split in the volunteer-hours display. The backend exposes a single combined figure and that is what the UI renders.

## Decisions

### 1. Cap flag lives on the work-details form, with helper text

Add a new `FormPair` to `EmployeeWorkDetailsForm` next to the existing `dynamic` toggle. The control is a `Checkbox`, disabled in `ReadOnly` mode, editable in `New` and `Edit`. A helper text rendered below the checkbox (or as a `<span class="text-xs text-gray-500">` sibling) describes the behaviour: "Hours beyond the expected weekly amount are recorded as volunteer work and do not affect the balance."

**Why a helper text and not just a label:** The cap inverts an otherwise universal rule (overtime is credited). Discovering the consequence by experiment would be painful for planners; the cost of one short translated sentence is negligible. Alternative considered: documentation-only (no helper). Rejected — same argument as for other behaviour-changing toggles where a hint is cheap and avoids tickets.

### 2. `WorkingHoursCategory::VolunteerWork` is parameterless

Variant added without payload (parallel to `Vacation`, `SickLeave`, `Holiday`, `Unavailable`, `UnpaidLeave`). Identifier is `"volunteer_work"`; i18n key is `CategoryVolunteerWork`; helper is `is_volunteer_work()`. All three TO conversion sites (`From<&ExtraHoursReportCategoryTO>`, `From<&ExtraHoursCategoryTO>`, `From<&WorkingHoursCategory> for ExtraHoursCategoryTO`) and the `Display` impl get a new arm.

**Why parallel to UnpaidLeave rather than to ExtraWork:** Although volunteer work *semantically* resembles `ExtraWork` (presence, work performed), structurally `ExtraWork` carries a `Reason: Rc<str>` payload because it is reason-tracked in the database. `VolunteerWork` is parameterless on the backend; matching that shape in the frontend keeps conversions straightforward and avoids a fake reason field that the backend would discard.

### 3. `from_identifier` becomes the only failure surface; mitigated by the compiler

`WorkingHoursCategory::from_identifier` currently `panic!`s on unknown strings. The new variant must be wired in here, otherwise a `<select>` round-trip would crash the frontend. All other touch points (the `From` impls, the `Display`, the `to_i18n_key` mapping) are exhaustive `match` expressions where the compiler will refuse to build until each is updated — no risk of forgotten arms.

**Alternative considered:** Migrate `from_identifier` to return `Result<…>` or `Option<…>`. Rejected as out-of-scope churn; the panic is defensive (only reachable if a programmer-supplied string is wrong) and the new arm is a one-line fix consistent with how every prior variant was added.

### 4. Volunteer hours rendered as one line, always shown

In both the per-week and per-period sections of `EmployeeView`, add one new `<li><TupleView label=… value=…/></li>` between the existing extra-work and vacation rows. The line is rendered unconditionally (including `0.00`), matching the convention used by every other category aggregate. Backend already returns `0.0` when no volunteer hours exist.

**Why always shown:** The other category aggregates (`vacation_hours`, `sick_leave_hours`, `unpaid_leave_hours`, etc.) are unconditionally rendered. Hiding only this one would either (a) make discovery of the feature dependent on a planner already knowing it exists, or (b) introduce a special-case predicate that has to be re-justified later. Symmetric rendering wins on simplicity.

### 5. Dropdown placement near `Extra Work`

In `AddExtraHoursForm`'s `<select>`, the `volunteer_work` `<option>` sits directly under `extra_work` (the first option in the list). Both express "the person was present and working"; this neighbour-grouping reads more naturally than placing it in the absences cluster.

**Alternative considered:** Place it in the absence cluster (vacation, sick leave, etc.) or below the `vacation` separator. Rejected as semantically misleading — volunteer work is presence, not absence.

### 6. Billing-period `value_type` mapping is a single new match arm

`BillingPeriodDetails` (`src/page/billing_period_details.rs:442–448`) already contains an `uppercase().as_str()` match that translates known `value_type` strings to localised labels. Adding `"VOLUNTEER" => i18n.t(Key::CategoryVolunteerWork).to_string()` reuses the existing category translation key — no new translation required for this surface specifically.

**Why reuse `CategoryVolunteerWork`:** A separate "billing-period header" key would diverge from how other types are presented. The backend persists "volunteer" as the type identifier, parallel to "vacation" / "extra_work"; rendering it under the same translation as the category itself keeps the surface consistent across views.

### 7. Translations: full coverage in all three locales

i18n is doctrinal in this project (CLAUDE.md: "When adding text, add translations to all three locales (En, De, Cs)"). New keys needed:

- `CategoryVolunteerWork`
- `CapPlannedHoursLabel` — the form label
- `CapPlannedHoursHelp` — the helper text below the checkbox

Suggested copy:

| Key | En | De | Cs |
|---|---|---|---|
| `CategoryVolunteerWork` | Volunteer Work | Ehrenamt | Dobrovolnictví |
| `CapPlannedHoursLabel` | Cap planned hours at expected | Geplante Stunden auf Soll deckeln | Omezit plánované hodiny na očekávané |
| `CapPlannedHoursHelp` | Hours beyond expected are recorded as volunteer work and do not affect the balance. | Stunden über dem Soll werden als Ehrenamt verbucht und beeinflussen das Stundenkonto nicht. | Hodiny nad rámec očekávaných se zaznamenávají jako dobrovolnictví a neovlivňují bilanci. |

Czech translations should be confirmed by a native speaker before merging. Suggested copy ships with the change so the translator has a starting point.

## Risks / Trade-offs

- **`WorkingHoursCategory::from_identifier` is a panic on unknowns** → mitigation: add the `volunteer_work` arm in the same pass as the variant; covered by an explicit unit test asserting the round-trip.
- **Translation rot in Czech locale** → mitigation: ship suggested copy from this design doc; flag for native-speaker review in the PR description. Not unique to this change.
- **Volunteer hours rendered as `0.00` may add visual clutter** → accepted: symmetric with all existing category aggregates. Reversible at zero cost if rejected later.
- **Helper text translation drift** → mitigation: keep helper text short (one sentence) so all three locales fit in the same visual footprint.
- **Cap toggle on a dynamic-hours contract** → not specifically prevented; backend accepts the combination. The cap simply does nothing useful when `expected_hours` is dynamically zero; documented as benign in the helper text by virtue of "beyond expected" — if expected is dynamic and shrinks to zero, the cap converts everything to volunteer hours, which may or may not be desired but is internally consistent.

## Migration Plan

1. Add the new state fields and enum variant (compiler-driven).
2. Wire conversions for `EmployeeWorkDetailsTO` ↔ `EmployeeWorkDetails` and the three `WorkingHoursCategory` ↔ TO sites.
3. Add i18n keys and translations.
4. Add the form checkbox with helper text.
5. Add the dropdown option.
6. Add the display rows.
7. Add the billing-period match arm.
8. Verify with `cargo check`, `cargo clippy`, `cargo test`, and a manual run against the backend (`cargo run` in `shifty-backend`, `dx serve` in `shifty-dioxus`).

No data migration. No feature flag. No rollback plumbing — feature is additive and non-breaking.

## Open Questions

None blocking. Czech translation copy is a suggestion that the native-speaker reviewer can adjust before merging.
