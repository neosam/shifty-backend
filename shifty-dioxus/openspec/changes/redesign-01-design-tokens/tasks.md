## 1. CSS Variables and Tailwind Configuration

- [x] 1.1 Add `:root` block in `input.css` with all light-theme tokens (surfaces, borders, ink, accent, semantic, radii, modal veil)
- [x] 1.2 Add `[data-theme="dark"]` block in `input.css` with the dark-theme overrides
- [x] 1.3 Extend `tailwind.config.js` `theme.extend.colors` with aliases referencing `var(--…)` for each token
- [x] 1.4 Extend `tailwind.config.js` `theme.extend.borderRadius` with `sm` / `md` / `lg` aliases for `var(--r-sm/-md/-lg)`
- [x] 1.5 Extend `tailwind.config.js` `safelist` with `bg-{bad,warn,accent}-soft`, `text-{bad,warn,good}`, `border-{bad,warn,accent}`
- [x] 1.6 Add a comment in `tailwind.config.js` documenting that dynamic class strings must use static `if`/`match` branches or be added to the safelist

## 2. Font Loading

- [x] 2.1 Add Google Fonts `<link>` for Inter (400/500/600/700) and JetBrains Mono (400/500/600/700) in `index.html`
- [x] 2.2 Extend `tailwind.config.js` `theme.extend.fontFamily.sans` with `['Inter', 'system-ui', '-apple-system', 'sans-serif']`
- [x] 2.3 Extend `theme.extend.fontFamily.mono` with `['"JetBrains Mono"', 'ui-monospace', 'Menlo', 'monospace']`

## 3. Pre-paint Theme Application

- [x] 3.1 Add inline `<script>` in `index.html` that reads `shifty-theme` from `localStorage`, resolves `system` via `matchMedia`, and writes the resolved value to `<html data-theme="…">` synchronously before the WASM bundle loads
- [x] 3.2 Verify the script is plain ES5 (no `let`, no `const`, no arrow functions) so it runs on every supported browser without transpilation

## 4. Theme Service

- [x] 4.1 Create `src/service/theme.rs` with `ThemeMode` (`Light`/`Dark`/`System`), `ResolvedTheme` (`Light`/`Dark`), `ThemeAction` enum, and `THEME_MODE` + `RESOLVED_THEME` global signals
- [x] 4.2 Implement `theme_service` async coroutine: load mode from `localStorage` on start, resolve, sync DOM
- [x] 4.3 Implement system-theme mediaquery subscription using `web_sys::MediaQueryList` and `prefers-color-scheme: dark`
- [x] 4.4 Implement `ThemeAction::SetMode(mode)` handler: write `localStorage`, recompute resolved theme, update `<html data-theme>`
- [x] 4.5 Implement `ThemeAction::SystemThemeChanged(resolved)` handler: only effective while `THEME_MODE == System`
- [x] 4.6 Helper functions: `pub fn cycle_theme(current: ThemeMode) -> ThemeMode` (Light → Dark → System → Light) for use by the toggle in change `03`
- [x] 4.7 Re-export from `src/service/mod.rs`

## 5. Application Integration

- [x] 5.1 Spawn `theme_service` coroutine in `src/app.rs` or wherever other services are launched
- [x] 5.2 Confirm the theme service runs once globally, not per-page

## 6. Tests

- [x] 6.1 Unit test `cycle_theme`: covers all three transitions (Light → Dark → System → Light)
- [x] 6.2 Unit test the resolution logic: given `(mode, system_pref)`, returns expected `ResolvedTheme`
- [x] 6.3 Browser-level smoke check (manual or scripted): set `localStorage.shifty-theme = 'dark'`, reload, observe `<html data-theme="dark">` before WASM mounts
- [x] 6.4 Browser-level check: with mode = System, toggling OS theme flips `<html data-theme>` without page reload
