---
phase: quick-260613-jxe
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - shifty-dioxus/src/page/absences.rs
  - shifty-dioxus/src/i18n/mod.rs
  - shifty-dioxus/src/i18n/de.rs
  - shifty-dioxus/src/i18n/en.rs
  - shifty-dioxus/src/i18n/cs.rs
autonomous: true
requirements: [QUICK-260613-JXE]
must_haves:
  truths:
    - "The per-person sales-person list (VacationPerPersonList) shows ONLY paid & active sales persons"
    - "Inactive or unpaid sales persons never appear in the per-person list, even if they have a vacation balance"
    - "The absences page shows exactly one year at a time; default is the current year"
    - "User can navigate to the previous year via a ◀ button and re-load that year's data"
    - "User can navigate to the next year via a ▶ button and re-load that year's data"
    - "Per displayed year, the per-person list stays sorted by remaining days (existing sort preserved)"
  artifacts:
    - path: "shifty-dioxus/src/page/absences.rs"
      provides: "selectable_balances helper, year-nav state + UI, filtered + year-scoped per-person list"
      contains: "fn selectable_balances"
    - path: "shifty-dioxus/src/i18n/mod.rs"
      provides: "Year-nav aria-label i18n Keys"
      contains: "AbsenceYearNavPrev"
  key_links:
    - from: "AbsencesPage selected_year signal"
      to: "VacationBalanceAction::LoadTeam / LoadSelf"
      via: "use_effect re-dispatch on year change"
      pattern: "LoadTeam\\(.*year"
    - from: "VacationPerPersonList rows"
      to: "selectable_balances(rows, sales_persons)"
      via: "filter join balance.sales_person_id -> is_selectable_employee"
      pattern: "selectable_balances"
---

<objective>
Two changes to the Absences page (`shifty-dioxus/src/page/absences.rs`):

1. **Inactive filter for the per-person listing.** The per-person sales-person
   list (`VacationPerPersonList`, sorted by remaining days) must exclude
   sales persons that are NOT (paid && active) — the exact same predicate the
   recent dropdown filter uses (`is_selectable_employee`, commit 7c2e0a0).

2. **Year navigation.** Instead of showing the current year only (and never
   being able to look at other years), the page must let the user page through
   years one at a time: default = current year, with ◀ / ▶ buttons. Changing
   the year re-loads that year's vacation data (team for HR, self for employee)
   and re-scopes the stats grid. Per year, the per-person list keeps its
   existing remaining-days sort.

Purpose: keep the absences page consistent with the dropdown behavior (no
inactive people) and make historical / future years browsable instead of a
single fixed year.
Output: filtered + year-navigable Absences page, new pure test helpers, i18n
keys for the nav buttons, all three locales translated.
</objective>

<execution_context>
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/workflows/execute-plan.md
@/home/neosam/programming/rust/projects/shifty/shifty-backend/.claude/get-shit-done/templates/summary.md
</execution_context>

<vcs_jj_only>
This repository is **jj (Jujutsu)**, co-located with git. GSD auto-commit is
disabled — the USER controls commits. Do NOT run `git commit` / `git add`.
If you commit at all, use `jj` commands (or the `jj-commit` skill). Prefer to
leave the change uncommitted for the user to review unless explicitly asked.
</vcs_jj_only>

<context>
@.planning/STATE.md

<interfaces>
<!-- Already in absences.rs — use directly, do NOT redefine. -->

Existing pure predicate (commit 7c2e0a0) — REUSE this, do not duplicate the rule:
```rust
// shifty-dioxus/src/page/absences.rs
pub fn is_selectable_employee(sales_person: &SalesPerson) -> bool {
    sales_person.is_paid && !sales_person.inactive
}
```

Per-person list component (the listing to filter). It sorts by remaining_days:
```rust
#[derive(Props, Clone, PartialEq)]
pub struct VacationPerPersonListProps {
    pub rows: Rc<[VacationBalance]>,       // vacation balances
    pub sales_persons: Rc<[SalesPerson]>,  // join target for name + selectable check
}
```
`VacationBalance` has `sales_person_id: Uuid` and `remaining_days: f32`
(see `src/state/vacation_balance.rs`). `SalesPerson` has `id: Uuid`,
`is_paid: bool`, `inactive: bool`.

Year is currently fixed in `AbsencesPage`:
```rust
let year = current_year_for_init();   // u32, current year
```
and dispatched once:
```rust
absence_service.send(AbsenceAction::LoadAll(sales_persons_for_effect.clone()));
vacation_service.send(VacationBalanceAction::LoadTeam(year));        // HR
// or
vacation_service.send(VacationBalanceAction::LoadSelf(sp, year));    // employee
```
`year` is also passed to `VacationEntitlementCard { year }` and
`StatsGrid { year }`.

Year helper (WASM-only; native test build returns 2026):
```rust
fn current_year_for_init() -> u32   // crate::js::get_current_year() on wasm32
```

The per-person list is reachable ONLY in the HR variant
(`VacationEntitlementHrBody` → `VacationPerPersonList`). The employee variant
shows the self hero, no per-person list.
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Inactive filter on the per-person list (pure helper + wire)</name>
  <files>shifty-dioxus/src/page/absences.rs</files>
  <behavior>
    Add a pure, unit-testable helper:
      `fn selectable_balances(rows: &[VacationBalance], sales_persons: &[SalesPerson]) -> Vec<VacationBalance>`
    Semantics: keep a balance ONLY if its `sales_person_id` matches a
    `SalesPerson` in `sales_persons` for which `is_selectable_employee(sp)` is
    true. A balance whose sales person is missing from the list is DROPPED
    (consistent with "only paid & active appear"; unknown == not selectable).
    Tests (native, in the existing `#[cfg(test)] mod tests`):
      - balance for a paid & active person is KEPT
      - balance for a paid & INACTIVE person is DROPPED
      - balance for an UNPAID & active person is DROPPED
      - balance for an id NOT present in sales_persons is DROPPED
      - ordering of the kept balances is preserved (helper does not sort;
        VacationPerPersonList still applies its own remaining-days sort)
    Reuse the existing `make_vacation_balance(id, remaining)` test helper and
    build small `SalesPerson` values via `SalesPerson { is_paid, inactive,
    id, ..Default::default() }` (mirror the existing `sales_person(..)` test
    helper pattern at the `is_selectable_employee` tests).
  </behavior>
  <action>
    1. Add `selectable_balances` next to `is_selectable_employee` (top of file,
       pure-function block). It joins each balance to its sales person and
       filters with `is_selectable_employee`. Keep it allocation-light:
       `rows.iter().filter(|b| sales_persons.iter().any(|sp| sp.id ==
       b.sales_person_id && is_selectable_employee(sp))).cloned().collect()`.
    2. Wire it into `VacationPerPersonList` (the per-person component): at the
       top of the component, replace the direct use of `props.rows` with the
       filtered set, e.g. compute
       `let filtered: Vec<VacationBalance> = selectable_balances(&props.rows, &props.sales_persons);`
       then build the existing `sorted` Vec from `filtered` instead of
       `props.rows`. The existing remaining-days `sort_by` and the
       show-all/`total > 4` logic must operate on the FILTERED set so the
       "Show all (N)" count reflects only selectable persons. Do NOT change the
       props signature — `sales_persons` is already passed in.
    3. Also fix the empty-guard: the early `if props.rows.is_empty()` return
       must become `if filtered.is_empty()` so a list that is non-empty only
       because of inactive persons still collapses to nothing. (Compute
       `filtered` BEFORE that guard.)
    4. Leave the existing `is_selectable_employee` dropdown filters untouched.
  </action>
  <verify>
    <automated>cd shifty-dioxus && cargo test --lib selectable_balances 2>&1 | tail -20</automated>
  </verify>
  <done>
    `selectable_balances` exists, is covered by the 5 listed unit tests, and
    `VacationPerPersonList` renders only paid & active persons; the show-all
    counter and empty-state reflect the filtered set. `cargo test` passes.
  </done>
</task>

<task type="auto">
  <name>Task 2: Year navigation (state, prev/next buttons, per-year reload) + i18n</name>
  <files>shifty-dioxus/src/page/absences.rs, shifty-dioxus/src/i18n/mod.rs, shifty-dioxus/src/i18n/de.rs, shifty-dioxus/src/i18n/en.rs, shifty-dioxus/src/i18n/cs.rs</files>
  <action>
    Goal: show ONE year at a time (default = current year) with ◀ Jahr ▶
    navigation that re-loads that year's data. Locked decision — do NOT show
    all years simultaneously.

    1. i18n keys. In `src/i18n/mod.rs`, add two `Key` variants near the other
       Absence keys (e.g. after `AbsencePageSubtitle` or near the Vacation
       block): `AbsenceYearNavPrev` and `AbsenceYearNavNext` (used as the
       buttons' `aria-label` / `title`). Add `i18n.add_text(...)` entries for
       BOTH keys in all three locales:
         - de.rs: AbsenceYearNavPrev = "Vorheriges Jahr", AbsenceYearNavNext = "Nächstes Jahr"
         - en.rs: AbsenceYearNavPrev = "Previous year",   AbsenceYearNavNext = "Next year"
         - cs.rs: AbsenceYearNavPrev = "Předchozí rok",    AbsenceYearNavNext = "Další rok"
       (Follow the exact `i18n.add_text(Locale::X, Key::Y, "...")` pattern used
       around `VacationPerPersonHeader`.) The visible year NUMBER is rendered
       directly (e.g. `"{selected_year}"`), not via i18n.

    2. State. In `AbsencesPage`, replace the fixed
       `let year = current_year_for_init();` with a signal:
         `let mut selected_year = use_signal(current_year_for_init);`
       and read `let year = *selected_year.read();` where the old `year` was
       used (VacationEntitlementCard, StatsGrid, the load dispatch). Keep the
       name `year` for the local read so downstream rsx stays unchanged.

    3. Reload on change. The existing refresh `use_effect` dispatches the
       vacation load. Make the year a dependency so changing it re-fires:
       capture `let year_for_effect = *selected_year.read();` before the
       `use_effect` (alongside the existing captured signals) and use it inside
       for `VacationBalanceAction::LoadTeam(year_for_effect)` /
       `LoadSelf(sp, year_for_effect)`. Because the effect already reads
       reactive signals, ensure `selected_year` is read so Dioxus subscribes —
       the simplest correct form is to read `*selected_year.read()` INSIDE the
       effect closure and pass that into the actions. Absence list loading
       (`LoadAll` / `LoadForSalesPerson`) is unchanged; only the vacation +
       stats year changes (matches "page through the per-person list per year").

    4. UI. Render a compact year-nav control. Place it on the
       `VacationEntitlementCard` header row or directly above it inside the
       page `div` (HR and employee both get it — both have year-scoped data).
       Structure (STATIC Tailwind, Pitfall 5 — no `format!` for classes):
         ```
         div { class: "flex items-center gap-2",
             button {
                 r#type: "button",
                 class: "px-2 py-1 rounded-md border border-border text-ink hover:bg-surface-alt",
                 title: "{i18n.t(Key::AbsenceYearNavPrev)}",
                 "aria-label": "{i18n.t(Key::AbsenceYearNavPrev)}",
                 onclick: move |_| { let y = *selected_year.read(); selected_year.set(y - 1); },
                 "◀"
             }
             span { class: "text-body font-semibold font-mono text-ink min-w-[3.5rem] text-center", "{year}" }
             button {
                 r#type: "button",
                 class: "px-2 py-1 rounded-md border border-border text-ink hover:bg-surface-alt",
                 title: "{i18n.t(Key::AbsenceYearNavNext)}",
                 "aria-label": "{i18n.t(Key::AbsenceYearNavNext)}",
                 onclick: move |_| { let y = *selected_year.read(); selected_year.set(y + 1); },
                 "▶"
             }
         }
         ```
       Use `selected_year` (the signal) in the onclicks and `year` (the read
       copy) for the displayed number. Guard against underflow with
       `saturating_sub(1)` for the prev button so the year never wraps below 0
       (`selected_year.set(y.saturating_sub(1))`).

    5. Do NOT change `VacationBalanceAction`, the loader, or the per-person
       sort. This is page-state + dispatch wiring only.
  </action>
  <verify>
    <automated>cd shifty-dioxus && cargo test --lib 2>&1 | tail -25</automated>
  </verify>
  <done>
    `selected_year` signal drives a ◀ {year} ▶ control; prev/next mutate it and
    the vacation/stats data re-loads for the new year (default = current year);
    two new i18n keys exist in all three locales; all existing `cargo test`
    snapshots still pass (no key-count / locale-coverage test breaks).
  </done>
</task>

<task type="auto">
  <name>Task 3: WASM build gate + full test gate</name>
  <files>shifty-dioxus/src/page/absences.rs</files>
  <action>
    Project rule: every frontend change must pass the WASM build gate. Run the
    WASM target build and the native test suite from inside `shifty-dioxus/`.
    If the build fails, fix compile errors in absences.rs / i18n files until
    BOTH gates are green. Do not weaken or delete existing tests to pass.
    Note: the year-nav UI is interaction-only (button click → signal → reload);
    it is verified by the WASM build gate (compiles + renders) plus the pure
    helper unit tests from Task 1 and the unchanged snapshot tests. There is no
    headless DOM click test in this suite, which is the existing convention —
    surface this explicitly in the SUMMARY.
  </action>
  <verify>
    <automated>cd shifty-dioxus && cargo build --target wasm32-unknown-unknown 2>&1 | tail -15 && cargo test --lib 2>&1 | tail -15</automated>
  </verify>
  <done>
    `cargo build --target wasm32-unknown-unknown` succeeds AND `cargo test`
    passes from inside `shifty-dioxus/`.
  </done>
</task>

</tasks>

<verification>
- `cd shifty-dioxus && cargo build --target wasm32-unknown-unknown` succeeds (WASM gate).
- `cd shifty-dioxus && cargo test` passes (includes new `selectable_balances` tests + existing snapshots + i18n locale-coverage tests).
- Manual (optional, user): on the absences page the per-person list shows only
  paid & active persons; ◀ / ▶ switches the year and the vacation card +
  stats update; default year is the current year.
</verification>

<success_criteria>
- The per-person sales-person list excludes non-(paid && active) persons,
  reusing the `is_selectable_employee` predicate (consistent with 7c2e0a0).
- The page shows one year at a time (default current year) with working
  ◀ / ▶ navigation that re-loads per-year vacation + stats data.
- Per year, the per-person list keeps its remaining-days sort.
- New pure helper `selectable_balances` is unit-tested (5 cases).
- Two new i18n keys translated in De / En / Cs.
- Both gates green: WASM build + `cargo test`.
</success_criteria>

<output>
After completion, create
`.planning/quick/260613-jxe-abwesenheitsseite-inaktive-sales-persons/260613-jxe-SUMMARY.md`.
In it, explicitly note: (a) that the inactive filter reuses
`is_selectable_employee` (paid && active) and drops balances whose person is
missing/inactive/unpaid; (b) that year-nav is page-state only (no API/loader
change) and is verified via the WASM build gate + pure unit tests since the
suite has no headless click test by convention.
</output>
