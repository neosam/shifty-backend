## Context

`src/component/top_bar.rs` (after `redesign-03-topbar-layout`) renders the eight privilege-gated nav items as a flat list inside a single `<nav class="hidden md:flex items-center gap-0.5 flex-1 min-w-0">`. The reference design (`shifty-design/project/Shifty Preview.html` lines 167–311) groups admin items under a `Verwaltung` dropdown trigger to keep the bar narrow on a laptop. The grouping was overlooked in `redesign-03`'s scope.

Existing infrastructure to lean on:

- `nav_item_class(active)` already returns the correct active/inactive class strings (active uses `bg-accent-soft text-accent font-semibold`).
- `is_active_for(NavTarget, &Route)` already maps each route to its nav target, including parameterised routes (e.g. `Route::EmployeeDetails { .. }` → `NavTarget::Employees`).
- `DropdownTrigger` / `DropdownBase` exist (`src/component/dropdown_base.rs`) and are already used for the account-pill logout.

What does NOT yet exist:

- A way to anchor a dropdown panel directly under a specific trigger button. The current `DropdownBase` reads click-page coordinates (`e.data().page_coordinates()`) and renders a fullscreen overlay at those absolute coordinates — fine for an account pill, wrong for a nav-bar trigger that must align under the trigger's bounding box and stay there.
- An "active state" for `DropdownTrigger`'s visible button. The trigger button must adopt the active-pill style when any of its grouped items is the active route.
- A label-substitution rule for the trigger.

The reference's mock implements this with `position: fixed` + `getBoundingClientRect` on the trigger (lines 265–273). The mock also handles outside-click via a document `mousedown` listener installed on the next animation frame, plus an `Escape` key handler (lines 198–218). On mobile, it deliberately *bypasses* the dropdown and inlines the items under a section header (lines 299–310).

## Goals / Non-Goals

**Goals:**

- Faithful re-creation of the `Verwaltung` grouping in the desktop TopBar — same layout, same trigger label substitution, same active-state propagation, same chevron, same panel position and styling, same outside-click + Escape handling.
- Faithful re-creation of the mobile flat-section behavior — no nested dropdown on mobile.
- Trigger and group hidden when no admin item is visible to the current user.
- Reuse `nav_item_class`, `is_active_for`, and `NavTarget` so privilege gating, route matching, and visual styling remain in one place.
- All three locales updated for the new label.

**Non-Goals:**

- Changing privilege rules, route names, or which items each privilege unlocks.
- Refactoring `DropdownBase` to support trigger-anchored panels generically (out of scope; we build the group-specific dropdown inline).
- Adding sub-grouping beyond `Verwaltung` (e.g. a "My data" submenu). The design only specifies one group.
- Keyboard arrow-key navigation inside the dropdown panel (not in the reference).

## Decisions

### 1. New i18n key `Key::TopBarAdminGroupLabel`

Add a single new key:

| Locale | Value |
|---|---|
| `De` | `Verwaltung` |
| `En` | `Administration` |
| `Cs` | `Správa` |

Reusing `Key::UserManagement` (which means "Benutzerverwaltung") would conflict — they are different concepts. The mock uses `Verwaltung` literally; the new key carries that.

### 2. Group composition follows "admin-only" rule

The reference comment says: *"Admin-only items are grouped under 'Verwaltung'"*. Today's `nav_visibility` flags map cleanly:

| Item | Top-level or grouped | Visibility flag |
|---|---|---|
| Schichtplan | top-level | `visibility.shiftplan` |
| Meine Schichten | top-level | `visibility.my_shifts` |
| Meine Zeit | top-level | `visibility.my_time` |
| Jahresübersicht | top-level | `visibility.year_overview` |
| Mitarbeiter | grouped | `visibility.employees` |
| Abrechnungszeiträume | grouped | `visibility.billing_periods` |
| Benutzerverwaltung | grouped | `visibility.user_management` |
| Textvorlagen | grouped | `visibility.templates` |

The reference mock only listed three grouped items because it doesn't model `Textvorlagen`. The proposal extends the rule consistently: anything gated by `admin` (templates, user_management) plus `hr` admin views (employees, billing_periods) belongs under `Verwaltung`. This keeps the mental model simple: "everything in Verwaltung is administrative work the average user does not see."

### 3. Inline rendering helper instead of new generic component

Two options were considered:

| Option | How | Tradeoff |
|---|---|---|
| A — Generalize `DropdownBase` to support trigger-anchored panels | Add a new mode that uses a passed-in trigger ref / bounding rect | Larger blast radius; touches every existing dropdown user; out of scope per Non-Goals |
| B — Build a small inline `AdminGroup` rendering inside `top_bar.rs` | A `use_signal::<bool>` for open state, a `use_node_ref` (or web-sys `getBoundingClientRect`) for the trigger, a fixed-position panel | Localized; copies the mock's pattern verbatim; no impact on other dropdowns |

**Chosen: B.** The grouping is one trigger in one component; an inline implementation matches the design without spilling into the rest of the dropdown system. If a second grouped trigger appears later, we generalize then.

### 4. Trigger anchoring — use `getBoundingClientRect` + `position: fixed`

This is what the reference does. In Dioxus we get the trigger DOM node via `MountedData` (the `onmounted` event) or by reading via `web_sys::window().document().get_element_by_id(...)`. Using a stable `id` on the trigger button is simplest and avoids touching `MountedData` mid-render:

- Trigger button: `id="top-bar-admin-trigger"`
- Open handler reads `getBoundingClientRect` and stores `{ top, left }` in a `Signal<Option<(f64, f64)>>`
- Panel renders with `style="position: fixed; top: {top + 4}px; left: {left}px"`

The bounding-rect read happens in the same click handler that toggles `open`, so it is fresh whenever the panel renders.

### 5. Outside-click + Escape closing

The reference uses a `requestAnimationFrame` arming trick (lines 198–218) so the same click that opens the panel cannot immediately close it. In Dioxus we replicate this with:

- A `Signal<bool>` `armed`, set to `true` after one tick (`use_effect` reading `open`, scheduling via `gloo-timers` or via a `requestAnimationFrame` web-sys call).
- A `mousedown` listener installed on `document` while the panel is open (`use_effect` adds the listener and returns a cleanup that removes it).
- The listener checks: if `armed && !trigger.contains(target) && !panel.contains(target)`, set `open` to false.
- A `keydown` listener on `document` for `Escape`, same lifecycle.

If `gloo-timers` or web-sys event listeners are awkward in this codebase, an acceptable fallback: place a transparent fullscreen `div` behind the panel (already done in `DropdownBase`) and let it handle the close via `onclick`. The trade-off is that the trigger button must `stopPropagation` so its click doesn't bubble to the overlay.

**Chosen: the document-listener approach** (matches the mock; no overlay div, so the panel does not block clicks on the rest of the page during the same event tick). Helper kept inside `top_bar.rs`.

### 6. Active-state propagation on the trigger

Two pieces of state to derive:

```rust
let admin_targets = [
    (NavTarget::Employees,       visibility.employees,        Route::Employees {},                    Key::Employees),
    (NavTarget::BillingPeriods,  visibility.billing_periods,  Route::BillingPeriods {},               Key::BillingPeriods),
    (NavTarget::UserManagement,  visibility.user_management,  Route::UserManagementPage {},           Key::UserManagement),
    (NavTarget::Templates,       visibility.templates,        Route::TextTemplateManagement {},       Key::TextTemplateManagement),
];

let admin_visible_count = admin_targets.iter().filter(|(_, v, _, _)| *v).count();
let active_admin = admin_targets.iter().find(|(t, v, _, _)| *v && is_active_for(*t, &route));
```

If `admin_visible_count == 0`, the trigger is omitted. Otherwise:

- Trigger label = `active_admin.map(|(_,_,_,k)| i18n.t(k)).unwrap_or(i18n.t(Key::TopBarAdminGroupLabel))`
- Trigger class = `nav_item_class(active_admin.is_some())`

### 7. Mobile flat section reuses the existing burger panel

The mobile burger panel (the `mobile_panel_visible` block at `top_bar.rs:308–318`) currently lists every nav item. We split that loop into two sections:

```rust
for (target, target_route, label) in top_level_items.iter().cloned() { Link { … } }

if admin_visible_count > 0 {
    div { class: "mt-1 pt-3 px-3.5 pb-1 text-[11px] font-bold uppercase tracking-[0.06em] text-ink-muted border-t border-border",
        "{admin_group_label}"
    }
    for (target, target_route, label) in admin_items.iter().cloned() { Link { … } }
}
```

Tailwind classes match the reference's inline styles exactly: `font-size: 11px` → `text-[11px]`; `font-weight: 700` → `font-bold`; `text-transform: uppercase` → `uppercase`; `letter-spacing: 0.06em` → `tracking-[0.06em]`; `color: var(--ink-muted)` → `text-ink-muted`; `padding: 12px 14px 4px` → `pt-3 px-3.5 pb-1`; `border-top: 1px solid var(--border)` → `border-t border-border`; `margin-top: 4px` → `mt-1`.

### 8. Test approach

Unit-test the pure data slicing:

- `partition_nav_items(items, &visibility)` returns `(Vec<top_level>, Vec<admin>)` — testable without rendering.
- `admin_active_label(admin_items, &route, fallback)` returns either the active item's label or the fallback — testable without rendering.

Render tests (using existing patterns from `redesign-03`) cover:

- Trigger hidden when `admin_visible_count == 0` (sales-only user).
- Trigger present and labelled `Verwaltung` when no admin route active.
- Trigger labelled with the active admin item when on an admin route.
- Trigger has active classes when on an admin route.
- Mobile section header renders with the correct text/classes when `admin_visible_count > 0` and is hidden otherwise.

The position-and-listener wiring is covered manually (browser smoke test) since DOM measurement is hard to unit-test in Dioxus without a runtime.

## Risks / Trade-offs

**[Two routes to one item — mismatch potential]** The redesign-03 spec stipulates `is_active_for` already handles parameterised routes (e.g. `EmployeeDetails`). When the user is on `EmployeeDetails`, the trigger label flips to "Mitarbeiter" — confirmed correct by the design (`activeAdminItem ? activeAdminItem.label : ADMIN_GROUP.label`, line 262 of the mock). If both `Mitarbeiter` and `Benutzerverwaltung` somehow matched simultaneously, `find` would prefer the first declared one — order matters; declare in the same order as the mock.

**[Document listener leakage]** If the cleanup function from the `use_effect` hook fails to remove the `mousedown` / `keydown` listener, every reopen accumulates handlers. Mitigation: write the cleanup carefully and add a Drop-style assertion in the test by counting handlers via a wrapper (or rely on browser smoke test).

**[Bounding rect stale on resize]** If the user resizes the window while the panel is open, the panel stays at the old position. The reference behavior is the same (no `resize` listener). We follow the reference verbatim — out of scope to fix.

**[Mobile section header without items]** If `admin_visible_count == 0` we must skip both the header and the items, otherwise an empty labelled section appears. Covered by an explicit conditional in the rendering loop.

**[Trigger button focus ring inconsistent with `Link`]** The trigger is a `button` whereas the other nav entries are `<Link>` components rendering as `<a>`. Browser focus styles may differ. Mitigation: apply identical Tailwind focus classes (`focus:outline-none focus:ring-2 focus:ring-accent-soft` if present elsewhere) — verify against the existing nav links during implementation.

## Open Questions

None — the design is fully prescribed by the reference. Any ambiguity (e.g. which items count as "admin") is resolved in Decision 2 above. If the user later wants different items in the group, that is a follow-up change.
