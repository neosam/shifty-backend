## Context

The current TopBar is implemented in `src/component/top_bar.rs` as a single `#[component]` with inline logic for privilege checks and conditional rendering. It uses `bg-gray-800 text-white` (a fixed dark theme regardless of system theme) and has no sticky positioning. The mobile menu uses an inline `style` attribute that absolutely positions the nav under the bar when `visible` is true.

Seven nav items are gated by these privilege rules (preserved):

| Item | Required privileges |
|---|---|
| Schichtplan | `sales` OR `shiftplanner` |
| Meine Schichten | `sales` |
| Meine Zeit (`MyEmployeeDetails`) | `is_paid && !hr` (note: actually `is_paid && !show_reports`, where `show_reports = has_privilege(hr)`) |
| Jahresübersicht | `shiftplanner` OR `sales` |
| Mitarbeiter | `hr` |
| Benutzerverwaltung | `admin` |
| Textvorlagen | `admin` |

The reference design shows only five nav items. The two extras (`Meine Zeit`, `Textvorlagen`) are privilege-gated and project-specific; the redesign master plan does not call for their removal. They stay.

Logout is currently a plain `<a href="{backend_url}/logout">` containing the username. The new design moves identity into a `Du bist [name]` text-only pill on the right side. Logout has no obvious slot in the new design — a decision is needed.

Below the TopBar, a `bg-yellow-200` warning banner shows non-production environment info. It must continue to function but should be restyled to use the warn token.

## Goals / Non-Goals

**Goals:**
- Faithful 56 px sticky TopBar matching the reference design
- Theme-aware via tokens — works in light, dark, and system modes
- Active-route highlight using `use_route::<Route>()` matching
- Theme toggle wired to the service from `01`
- Mobile burger preserved with restyled dropdown
- Non-prod warning banner preserved
- Privilege rules unchanged

**Non-Goals:**
- Reducing the nav set to match the reference (the project has more routes than the prototype)
- Implementing keyboard shortcuts for nav
- Adding settings or user-preferences UI in the dropdown

## Decisions

### 1. Logout lives in a dropdown opened from the identity pill

The reference design puts identity as a text-only pill with no obvious affordance for logout. Three options were considered:

| Option | How | Tradeoff |
|---|---|---|
| A — Pill is itself the logout link | Click pill → navigate to `/logout` | Discoverable but irreversible; user might click accidentally |
| B — Pill opens a dropdown with Logout entry | Click pill → small panel with Logout button (and room for future Settings, etc.) | Slightly more clicks; matches common app patterns |
| C — Separate logout button next to pill | Two elements | Increases TopBar density, conflicts with reference visual |

**Chosen: B.** The dropdown matches user mental model (identity area = account menu), leaves room for future entries (e.g. theme submenu, language switch), and avoids accidental logout.

The dropdown reuses the existing `DropdownTrigger` component from `src/component/dropdown_base.rs` (used elsewhere in shiftplan and slot-edit). No new infra needed.

When `auth_info` is `None` (logged out), the right-side area shows a single `Login` link instead of pill+dropdown.

### 2. Active route detection via `use_route::<Route>()`

`dioxus_router` exposes the current `Route` value. The TopBar matches against the variant to decide which nav item is active:

```rust
let route = use_route::<Route>();
let is_active = |target: Route| route == target;
```

For routes with parameters (e.g. `EmployeeDetails { id }`), the comparison checks variant only — so `Mitarbeiter` stays highlighted even on a detail page. Pattern matching:

```rust
let is_employees_active = matches!(route, Route::Employees | Route::EmployeeDetails { .. });
```

### 3. TopBar layout uses CSS Grid for the right side, not flex stacks

The right side has three elements (theme toggle, identity pill, optional dropdown trigger) with different widths and one optional. Grid lets us define the layout once with consistent gaps:

```rust
class: "ml-auto flex items-center gap-2"
```

Flex with `gap-2` is sufficient — the elements are simple enough that grid would be overkill. Dropping grid in favor of flex.

### 4. Theme toggle uses `NavBtn`, not a new variant on `Btn`

A 28×28 square icon button is exactly what `NavBtn` from `02` provides. Reusing `NavBtn` keeps the TopBar implementation slim and the toggle visually consistent with prev/next buttons elsewhere in the app.

The glyph cycles based on `THEME_MODE.read()`:

```rust
let glyph = match *THEME_MODE.read() {
    ThemeMode::Light => "☀",
    ThemeMode::Dark => "☾",
    ThemeMode::System => "⌬",
};
```

`aria-label` reflects the current mode (`"Theme: light"`, etc.) and is announced when focus lands on the toggle.

### 5. Mobile burger keeps current trigger pattern, restyles container

The current code uses `visible: Signal<bool>` to toggle nav visibility on mobile. We keep this pattern — it works. What changes:

- Container: `bg-surface border border-border rounded-md shadow` instead of inline absolute styling
- Position: `absolute top-14 left-0 right-0 mx-2` for proper drop-down placement under the bar
- Burger glyph: `☰` when closed, `✕` when open (matches reference)
- Auto-close on route change via a `use_effect` that watches `use_route::<Route>()` and clears `visible`

### 6. Non-prod warning banner restyle, no relocation

Currently rendered as a sibling of the TopBar in the same component. Stays there; just changes classes from `bg-yellow-200 text-yellow-800` to `bg-warn-soft text-warn`. The detail tooltip (`title` attribute) is preserved.

### 7. `Du bist` is i18n'd

Currently the logout anchor uses `i18n.t(Key::LogoutUser).replace("{user}", ...)`. The new identity pill needs a `Du bist` prefix label. Add a new i18n key `Key::TopBarYouAreLabel` with translations for `En`, `De`, `Cs`. Keep `Key::LogoutUser` for the dropdown entry (its current value is the full sentence, may need a shorter `Key::Logout` — TBD during implementation).

## Risks / Trade-offs

**[Sticky positioning interferes with print]** — A sticky header with `print:hidden` is the current behavior, preserved. No change.

**[Auth-info loading state]** — `AUTH.read().auth_info` is `Option<AuthInfo>`. When loading, the right side renders nothing or shows a placeholder; current code shows the login link as fallback. Mitigation: keep the same fallback (`Login` link when `auth_info.is_none()`).

**[Mobile dropdown z-index conflicts]** — The page content below sometimes has its own z-index (e.g. shiftplan zoom selector at `z-50`). Mitigation: TopBar dropdown gets `z-50` or higher; verify no conflict on shiftplan during implementation.

**[Glyphs don't render on all platforms]** — `☀ ☾ ⌬ ☰ ✕` are unicode glyphs without webfont fallback. They're widely supported but not guaranteed. Mitigation: aria-label provides accessible label; visual fallback is acceptable.
