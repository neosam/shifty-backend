## 1. i18n Keys

- [x] 1.1 Add `Key::TopBarYouAreLabel` to `src/i18n/mod.rs` with translations: De `"Du bist"`, En `"You are"`, Cs `"Jsi"` (verify final wording with neosam)
- [x] 1.2 Verify whether a shorter `Key::Logout` is needed for the dropdown entry; if so, add to all three locales

## 2. TopBar Skeleton

- [x] 2.1 Replace contents of `src/component/top_bar.rs` with new structure: outer `<header class="sticky top-0 h-14 bg-surface text-ink border-b border-border z-40 print:hidden flex items-center px-4 gap-4">`
- [x] 2.2 Brand: `<span class="font-semibold text-base tracking-tight">Shifty<span class="text-accent">.</span></span>`; on non-prod append the `env_short_description` span
- [x] 2.3 Mobile burger button visible only `<md`, toggles `visible` signal as today

## 3. Privilege-Gated Nav

- [x] 3.1 Compute `show_*` flags exactly as today (preserve all 7 rules verbatim)
- [x] 3.2 Render nav items in a `<nav>` with `flex items-center gap-1` on `>=md`; collapsed in mobile dropdown when `visible`
- [x] 3.3 Each nav item is a `<Link>` with class derived from `is_active(target)`:
  - active: `px-3 py-1.5 rounded-md bg-accent-soft text-accent font-semibold`
  - inactive: `px-3 py-1.5 rounded-md text-ink-soft hover:bg-surface-alt`
- [x] 3.4 Implement `is_active` via `use_route::<Route>()` matching variants (use `matches!` for parameterised routes)

## 4. Theme Toggle

- [x] 4.1 Add `NavBtn` (from `02`) on the right side with glyph matching current `THEME_MODE`
- [x] 4.2 `on_click` dispatches `ThemeAction::SetMode(cycle_theme(*THEME_MODE.read()))` to the theme service coroutine
- [x] 4.3 Set `aria_label` to `format!("Theme: {}", mode)` based on current mode (i18n-able later if desired)

## 5. Identity Pill + Account Dropdown

- [x] 5.1 When `auth_info.is_some()`: render a `DropdownTrigger` whose visible button is a pill: `<button class="flex items-center gap-1 px-3 h-8 bg-surface-alt rounded-full text-[13px]">{i18n.t(Key::TopBarYouAreLabel)} <span class="font-medium">{auth_info.user}</span></button>`
- [x] 5.2 Dropdown entries:
  - `Logout` → navigates to `format!("{}/logout", backend_url)` (preserve current behavior)
  - (Future entries — leave room but no other entries in this change)
- [x] 5.3 When `auth_info.is_none()`: render a single `<a href="/authenticate">Login</a>` link styled as a ghost button

## 6. Mobile Dropdown Restyling

- [x] 6.1 Replace inline `style` attribute with proper class-based dropdown panel: `absolute top-14 left-2 right-2 bg-surface border border-border rounded-md shadow-md z-50 p-2 flex flex-col gap-1` when `visible`
- [x] 6.2 Burger glyph swaps between `☰` (closed) and `✕` (open)
- [x] 6.3 Auto-close panel on route change: `use_effect(move || { let _ = use_route::<Route>(); visible.set(false); })`

## 7. Non-Prod Warning Banner

- [x] 7.1 Restyle banner: `bg-warn-soft text-warn` (replace `bg-yellow-200 text-yellow-800`)
- [x] 7.2 Preserve `title` attribute with detail i18n string
- [x] 7.3 Verify banner is hidden on print (`print:hidden`)

## 8. Tests

- [x] 8.1 Render test: matrix of privilege combinations × expected nav-item visibility
- [x] 8.2 Render test: each route highlights the correct nav item (including parameterised routes like `EmployeeDetails` highlighting `Mitarbeiter`)
- [x] 8.3 Render test: theme toggle glyph matches current `THEME_MODE`
- [x] 8.4 Render test: identity dropdown contains a Logout entry pointing to `{backend_url}/logout`
- [x] 8.5 Render test: when `auth_info.is_none()`, only the Login link renders on the right side
- [x] 8.6 Render test: mobile burger toggles `visible`; route change resets `visible` to `false`
- [x] 8.7 Render test: non-prod banner appears only when `!config.is_prod`
