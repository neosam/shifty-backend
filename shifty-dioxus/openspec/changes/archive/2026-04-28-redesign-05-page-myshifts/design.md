## Context

`src/page/my_shifts.rs` is a read-only view that lists the current user's upcoming shifts. The current implementation:

- Loads `BlockTO` records via `loader::load_blocks(...)` for the current week through 10 weeks ahead.
- Groups blocks by `(year, week)` using a `BTreeMap`.
- Renders each week as an accordion: a clickable gray header (`bg-gray-100`) with a `▼ / ►` glyph and a body that lists each block on its own row (`weekday, date | from–to | sales person`).
- Uses legacy Tailwind classes (`bg-gray-100`, `bg-white`, `text-gray-600`) instead of the design tokens introduced in change `01`.

The reference design (`design_handoff_shifty/README.md` § 3) reframes this page as a stack of compact week-cards centered on the page (max 760 px), each card showing a header (`KW <n> · <date range>` left, total hours right), then one row per day with the day label, the day's shift items as `time–time` plus a colored area badge, and the day's hour total right-aligned. Days with a warning note get a soft warning background.

This page is the easiest to migrate (read-only, no forms, no dialogs) and intentionally precedes the more complex pages so we validate the token/atom stack end-to-end first.

### Data shape constraints

`BlockTO` carries: `year`, `week`, `day_of_week`, `from`, `to`, `sales_person: Option<SalesPersonTO>`, `bookings`, `slots`. There is **no per-block or per-day note field** today. The reference design's "warning note" path is therefore a forward-looking visual that has no data source yet — see decision 5 below.

`SalesPersonTO` carries `name` and `background_color` — the latter is the pastel hex used by `PersonChip` and is the source for the area-badge color.

The page is named `MyShifts` (route `/my-shifts`); the loaded blocks already contain only the current user's bookings (the backend filters by authenticated principal). No backend or loader change is required.

## Goals / Non-Goals

**Goals:**
- Replace the accordion-on-gray layout with stacked week cards using the new design tokens.
- Card max-width 760 px, centered horizontally on desktop; full-width on mobile.
- Card header with `KW <n> · <date range>` (left) and the week's total hours in mono (right).
- One row per day: day label (mono, fixed width) · shift items · day-total hours (mono, right-aligned).
- Each shift item: `HH:MM–HH:MM` in mono followed by an area badge styled as an `accent-soft` pill containing the sales-person name (current proxy for "area").
- Days with no shifts render a muted "—" row so the seven weekdays are visible at a glance.
- Reuse existing data loading; no changes to `loader.rs`, `api.rs` or `BlockTO`.
- Preserve the "no shifts found" message and the loading and error states.

**Non-Goals:**
- Inline edit/remove of shifts on this page (read-only by design).
- Backend or `BlockTO` changes (e.g. adding a `note` field).
- Replacing the `Modal` legacy component or any dialogs (this page has none).
- Adding filtering or week navigation controls (the existing 10-weeks-ahead range stays).
- Per-week persistence of expand/collapse state (cards are always expanded — see decision 1).

## Decisions

### 1. Drop the accordion; cards are always expanded

Today the user can collapse and re-expand each week (initialized expanded). The reference design renders week cards as a flat stack with no toggle. Three reasons to drop the accordion:

- The new card layout is dense (one row per day, ~7 rows per card) — collapsing buys little vertical space.
- Removing the toggle removes the `expanded_weeks` signal, the `initialized` signal, and the click-handler — all of which were accordion-specific bookkeeping with no other purpose.
- Read-only stack matches the reference and simplifies the test surface.

If users miss the collapse, we can add it back behind a per-week control later. For now, simpler.

### 2. "Area" badge sources its label and color from the sales person

The reference design talks about an "area" (e.g. `Laden`) for each shift item. The current data model has no first-class "area" concept; the closest is the sales person. We render the sales-person name in the badge and use `sales_person.background_color` as the pill background, falling back to the dashed-border `PersonChip` variant when no sales person is attached.

Implementation: reuse the existing `PersonChip` atom for the badge — it already enforces the "dark ink in both themes" invariant via the `.person-pill` rule, handles the dashed-border fallback, and never renders initials. This keeps the visual language consistent with other places that show people (`employee_view`, `slot_edit`, future page changes).

If the project later introduces a dedicated `Area` concept, the badge swaps to that model in a separate change.

### 3. Day rows show all seven weekdays, not only days with bookings

Two layouts considered:

| Option | UX |
|---|---|
| Render only days with shifts | Compact, but visually inconsistent across weeks (some cards have 1 row, others 7) |
| Always render Mon–Sun, "—" for empty days | Consistent rhythm, easier to scan a stack of cards |

**Chosen: always render all seven weekdays.** The reference design shows full-week cards. Empty days render the day label + a muted em-dash in the shifts cell + an empty hour cell (or `0:00`). Total card height is predictable.

### 4. Layout uses a CSS grid per row, not flex stacks

Each day row needs three columns with fixed left/right widths and a flexible middle:

- Desktop: `110px 1fr 60px` (day label · shifts · hours)
- Mobile (<720 px): `80px 1fr 50px`

CSS Grid handles this cleanly with `grid-template-columns`. Inline `style:` keeps the layout obvious in the RSX without polluting Tailwind config with one-off classes.

Multiple shift items inside a single day stack vertically (column flex) inside the middle cell so a day with two shifts grows downward — the day label stays top-aligned (`align-items: start`).

### 5. No note/warning data source yet — warning visuals are deferred

The proposal references a `note` starting with `⚠` triggering `bg-warn-soft`. `BlockTO` has no note field today, and adding one would be a backend change outside this redesign's scope. We design the row-render helper to take an `Option<&str>` note parameter so a future backend change can wire it through, but in this iteration the helper is always called with `None` — no warning rendering occurs and no test exercises the path.

This deviates from the proposal's "warning note path" test bullet. The deviation is recorded here so the next reviewer doesn't expect the path. If someone later wants the visual, a follow-up change adds the note field to `BlockTO`/`SlotTO` and flips the helper to read from it; the layout will already accommodate it.

### 6. Total-hour calculations live in the page module as small pure helpers

Two computations are needed:

- `block_hours(&BlockTO) -> f32` — duration of a single block in hours.
- `sum_hours<I>(blocks: I) -> f32` — sum across an iterator of blocks, used both for day totals and week totals.

These are arithmetic, not domain logic, and have no other consumers today — keeping them as private `fn`s in `my_shifts.rs` avoids premature extraction. If a second page (e.g. `06-page-overview`) needs the same arithmetic, we promote them to a shared module then.

`time::Time::sub` returns a `time::Duration`; convert to hours via `as_seconds_f32() / 3600.0`.

### 7. Hours rendering uses one decimal mono format

The reference design shows hours as `5.5` (one decimal, space-aligned in mono). All hour numbers — per-day, per-week, totals — use `format!("{:.1}", hours)` and the `font-mono tabular-nums` Tailwind classes for alignment. Zero is rendered as `0.0` to maintain column alignment, not as an em-dash; the em-dash is reserved for the shifts cell of an empty day.

### 8. Reuse `TopBar` and the H1 label; drop the page padding wrapper

The existing page wraps everything in `<div class="px-4 py-4 md:px-6">`. The new design centers the card stack at max 760 px. Outer wrapper:

```rsx
TopBar {}
main { class: "mx-auto max-w-[760px] w-full px-4 py-6 md:py-8 space-y-4",
    h1 { class: "text-xl font-semibold text-ink", "{i18n.t(Key::MyShifts)}" }
    // ... cards
}
```

`max-w-[760px]` is a one-off arbitrary value — no need to add a token (other pages have different max-widths).

## Risks / Trade-offs

**[Empty-day rows inflate "no shifts this week" cards]** — A week with zero shifts still renders 7 empty rows. Mitigation: when the entire week is empty (every day's shifts list is empty), skip rendering the card entirely; the existing "no shifts found" message remains for the all-weeks-empty case. Implementation: filter the `grouped` map to include only weeks with at least one block.

**[Sales-person color absent for legacy data]** — `SalesPersonTO::background_color` is a `Arc<str>`; if a backend record has an empty string, `PersonChip` renders the dashed-border fallback, which is acceptable.

**[Dropping accordion is a UX regression for power users]** — Anyone who used collapse/expand to focus on one week loses that affordance. Mitigation: communicate in the changelog; add back behind a control if user feedback demands.

**[`tabular-nums` is not in the default Tailwind safelist]** — `font-mono` plus `tabular-nums` are utility classes that Tailwind v3 emits when present in source. They appear statically in the new RSX, so no `safelist` change is needed. Verified by looking at existing usage of `font-mono` in `tuple_row.rs` and elsewhere.

**[Test coverage of layout]** — SSR tests can verify class strings and the presence of expected text but not visual layout. We accept the same trade-off as changes 02–04: structural tests confirm wiring (correct day labels, hour formatting, empty-day em-dash, badge color binding); visual review confirms the rest.
