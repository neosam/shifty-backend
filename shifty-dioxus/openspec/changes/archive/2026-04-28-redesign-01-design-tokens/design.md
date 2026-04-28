## Context

`shifty-dioxus` currently has minimal styling infrastructure: `input.css` defines only scrollbar utilities and three zoom helpers, while `tailwind.config.js` adds only `missingColor` and `blockedColor` to the default Tailwind palette. There is no theming, no shared color system, and no consistent typography across pages.

The reference design in `design_handoff_shifty/` defines a complete token system (surfaces, borders, ink, accent, semantic colors, radii) with light and dark variants resolved through CSS variables. The design also calls for `Inter` (sans) and `JetBrains Mono` (numerics) as the type pair, and three theme modes (`Light`, `Dark`, `System`) with `localStorage` persistence under key `shifty-theme`.

This change is the foundation for the entire redesign series (`02`–`09`). It must land first because every later change consumes tokens or the theme service, but it must also leave existing pages working unchanged — they keep their current hardcoded Tailwind classes until their respective per-page redesign change.

## Goals / Non-Goals

**Goals:**
- Provide a complete CSS-variable token library covering surfaces, borders, ink, accent, semantic states, and radii
- Alias all tokens in Tailwind so components written later can use idiomatic class names like `bg-surface`, `text-ink-soft`, `border-border-strong`
- Load `Inter` and `JetBrains Mono` and expose them via Tailwind `font-sans` / `font-mono`
- Implement a `Light` / `Dark` / `System` theme service with `localStorage` persistence and live response to OS theme changes when in `System`
- Apply the resolved theme to `<html data-theme="...">` **before** the WASM bundle paints anything, to avoid a flash of the wrong theme
- Keep existing pages and components working — no behavior change

**Non-Goals:**
- Adding the TopBar theme-toggle button (this lives in change `03`)
- Migrating existing pages to use tokens (per-page in `05`–`09`)
- Self-hosting fonts (use Google Fonts initially; revisit later if privacy or offline-mode become requirements)
- Theming `print` stylesheets (current print rules continue to assume a light context)

## Decisions

### 1. CSS-variables + Tailwind aliases (not two class sets)

The token values live in CSS variables under `:root` and `[data-theme="dark"]`. Tailwind aliases reference the variables, not the literal hex values:

```js
// tailwind.config.js
colors: {
  surface: 'var(--surface)',
  ink: 'var(--ink)',
  // ...
}
```

**Why:** Theme-switching at runtime requires only flipping `<html data-theme>`; no rebuild, no class swap. The alternative (defining `light:bg-x` / `dark:bg-x` for every utility) doubles the class surface and forces a global re-render on theme change.

**Trade-off:** Tailwind's autocomplete and color preview in IDEs may display the variable name instead of the actual color. Acceptable.

### 2. `<html data-theme="…">`, not `<body>`

The theme attribute lives on `<html>` so that styles cascading from `:root` and `[data-theme="dark"]` resolve correctly without media queries. Using `<body>` would require slightly different selectors and a tiny FOUC window before body parses.

### 3. Pre-paint theme application via inline script

A small inline script in `index.html` runs before the WASM bundle:

```html
<script>
  (function() {
    var stored = null;
    try { stored = localStorage.getItem('shifty-theme'); } catch (e) {}
    var resolved;
    if (stored === 'dark' || stored === 'light') {
      resolved = stored;
    } else {
      resolved = matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    document.documentElement.setAttribute('data-theme', resolved);
  })();
</script>
```

**Why:** WASM initialization takes hundreds of milliseconds. Without this script, a dark-mode user sees a flash of light theme. The script is plain ES5, runs synchronously, and sets the attribute before any paint.

**Trade-off:** Stored value is `light`/`dark`/`system`, but applied attribute is always resolved (`light` or `dark`). The Rust side reads `localStorage` to know the **mode** (incl. `system`) and the DOM attribute to know the resolved value.

### 4. Storage and mediaquery via `web_sys` directly

Use `web_sys::window().local_storage()` and `web_sys::window().match_media("(prefers-color-scheme: dark)")` directly. No new crate dependency; `web_sys` is already in the tree.

**Alternative considered:** `gloo-storage` for ergonomics. Rejected because the surface area used is tiny (one read/write of a string key) and adding a dep for it is not worth it.

### 5. Theme service shape

```rust
// src/service/theme.rs
#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode { Light, Dark, System }

#[derive(Clone, Copy, PartialEq)]
pub enum ResolvedTheme { Light, Dark }

pub static THEME_MODE: GlobalSignal<ThemeMode> = ...;
pub static RESOLVED_THEME: GlobalSignal<ResolvedTheme> = ...;

pub enum ThemeAction {
    SetMode(ThemeMode), // user picks Light/Dark/System
    SystemThemeChanged(ResolvedTheme), // mediaquery callback
}

pub async fn theme_service(rx: UnboundedReceiver<ThemeAction>);
```

The service:
1. On startup, reads `shifty-theme` from `localStorage` (default: `System`), sets `THEME_MODE`
2. Resolves to `RESOLVED_THEME` (mediaquery if `System`, else literal)
3. Writes `data-theme` to `<html>`
4. Subscribes to `prefers-color-scheme` mediaquery; on change while in `System`, dispatches `SystemThemeChanged`
5. On `SetMode`: persists, recomputes resolved, updates DOM

### 6. Fonts via Google Fonts in `index.html`

Add a single `<link>` to `index.html`:

```html
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600;700&display=swap" rel="stylesheet">
```

Then in `tailwind.config.js`:

```js
fontFamily: {
  sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
  mono: ['"JetBrains Mono"', 'ui-monospace', 'Menlo', 'monospace'],
}
```

`display=swap` ensures text renders with fallback fonts during font-load and swaps in once the webfonts arrive — acceptable visual jitter for first paint.

### 7. Backwards compatibility

Existing custom colors `missingColor` and `blockedColor` stay defined for now. They are referenced by current components (e.g. `bg-missingColor` in `week_view.rs`). The week-view redesign in change `09` will replace them with `bg-warn-soft` and `bg-bad-soft` respectively, after which they can be removed.

## Risks / Trade-offs

**[Google Fonts dependency]** — Privacy and offline use. Mitigation: Document as a known limitation; revisit with self-hosting if it becomes a problem. No PII is sent.

**[Tailwind safelist gaps]** — Classes constructed dynamically (e.g. `format!("bg-{}-soft", state)`) won't be picked up by Tailwind's content-scanner and need manual safelist entries. Risk: future contributors add new dynamic class patterns without updating safelist; CSS misses; debugging is non-obvious. Mitigation: Document the convention in a comment in `tailwind.config.js` and prefer static class strings via `if` / `match` over `format!`.

**[Pre-paint script and CSP]** — The inline `<script>` requires `'unsafe-inline'` in CSP. Project does not currently set CSP headers, so non-issue today; flag if CSP gets added later.

**[Existing print stylesheets]** — Current `print:bg-white` etc. assume light theme. After redesign, dark-mode users printing will now correctly print as light, because `<html data-theme>` is set per-session, but `@media print` rules in `input.css` and components were not designed with theming in mind. Mitigation: Out of scope here; address per-page where print matters.
