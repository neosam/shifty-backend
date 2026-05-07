# design-tokens Specification

## Purpose
TBD - created by archiving change redesign-01-design-tokens. Update Purpose after archive.
## Requirements
### Requirement: Design token library
The application SHALL define a complete set of design tokens as CSS variables, covering surfaces, borders, ink, accent, semantic states, modal veil, and border radii. Tokens SHALL be defined for both light and dark themes.

#### Scenario: Light-theme tokens are available
- **WHEN** the application is loaded with `<html data-theme="light">`
- **THEN** the CSS variables `--bg`, `--surface`, `--surface-alt`, `--surface-2`, `--border`, `--border-strong`, `--ink`, `--ink-soft`, `--ink-muted`, `--accent`, `--accent-ink`, `--accent-soft`, `--good`, `--good-soft`, `--warn`, `--warn-soft`, `--bad`, `--bad-soft`, `--modal-veil`, `--r-sm`, `--r-md`, `--r-lg` SHALL resolve to the values specified in `design_handoff_shifty/README.md` for light theme

#### Scenario: Dark-theme tokens override light values
- **WHEN** the application is loaded with `<html data-theme="dark">`
- **THEN** the same CSS variables SHALL resolve to the values specified for dark theme

### Requirement: Tailwind aliases reference tokens
The Tailwind configuration SHALL alias every token under `theme.extend.colors` and `theme.extend.borderRadius` so utility classes like `bg-surface`, `text-ink-soft`, `border-border-strong`, `rounded-md` resolve through the CSS variable system.

#### Scenario: Color utility resolves through token
- **WHEN** a component uses `class="bg-accent text-accent-ink"`
- **THEN** the rendered element SHALL use `background: var(--accent)` and `color: var(--accent-ink)`, which respond to the active theme

#### Scenario: Radius utility resolves through token
- **WHEN** a component uses `class="rounded-md"`
- **THEN** the rendered element SHALL use `border-radius: var(--r-md)`

### Requirement: Three theme modes with persistence
The system SHALL support three theme modes (`Light`, `Dark`, `System`). The current mode SHALL be persisted in `localStorage` under the key `shifty-theme`.

#### Scenario: User selects Light mode
- **WHEN** the user sets the theme mode to `Light`
- **THEN** `localStorage.shifty-theme` SHALL be `"light"` and `<html data-theme>` SHALL be `"light"`

#### Scenario: User selects Dark mode
- **WHEN** the user sets the theme mode to `Dark`
- **THEN** `localStorage.shifty-theme` SHALL be `"dark"` and `<html data-theme>` SHALL be `"dark"`

#### Scenario: User selects System mode while OS prefers dark
- **WHEN** the user sets the theme mode to `System` and the OS reports `prefers-color-scheme: dark`
- **THEN** `localStorage.shifty-theme` SHALL be `"system"` and `<html data-theme>` SHALL be `"dark"`

#### Scenario: User selects System mode while OS prefers light
- **WHEN** the user sets the theme mode to `System` and the OS reports `prefers-color-scheme: light`
- **THEN** `localStorage.shifty-theme` SHALL be `"system"` and `<html data-theme>` SHALL be `"light"`

### Requirement: Live response to OS theme changes in System mode
While the theme mode is `System`, the application SHALL listen to `prefers-color-scheme` mediaquery changes and update the DOM accordingly without a page reload.

#### Scenario: OS theme flips while in System mode
- **WHEN** the theme mode is `System` and the operating system theme changes from light to dark
- **THEN** `<html data-theme>` SHALL change from `"light"` to `"dark"` automatically

#### Scenario: OS theme change ignored in Light/Dark modes
- **WHEN** the theme mode is `Light` and the operating system theme changes
- **THEN** `<html data-theme>` SHALL remain `"light"`

### Requirement: Pre-paint theme application
The resolved theme SHALL be applied to `<html data-theme>` synchronously before the WASM bundle paints, to prevent a flash of the wrong theme on load.

#### Scenario: Page loads with stored Dark mode
- **WHEN** the page is opened with `localStorage.shifty-theme = "dark"`
- **THEN** `<html>` SHALL already carry `data-theme="dark"` at the moment first paint occurs (before WASM mount)

#### Scenario: Page loads with no stored mode and OS prefers dark
- **WHEN** the page is opened with no `shifty-theme` value stored and OS reports `prefers-color-scheme: dark`
- **THEN** `<html>` SHALL already carry `data-theme="dark"` at the moment first paint occurs

### Requirement: Inter and JetBrains Mono available
The application SHALL load `Inter` and `JetBrains Mono` fonts, and Tailwind SHALL expose them under `font-sans` and `font-mono` respectively.

#### Scenario: Sans utility uses Inter
- **WHEN** a component uses `class="font-sans"`
- **THEN** the computed font-family SHALL be `Inter, system-ui, -apple-system, sans-serif`

#### Scenario: Mono utility uses JetBrains Mono
- **WHEN** a component uses `class="font-mono"`
- **THEN** the computed font-family SHALL be `"JetBrains Mono", ui-monospace, Menlo, monospace`

### Requirement: Dynamic state classes are safelisted
Tailwind utilities used for state-dependent backgrounds and text colors SHALL be present in the build output even when not statically referenced in source files, so dynamically constructed class strings continue to work.

#### Scenario: Dynamic state class is generated
- **WHEN** a component computes a class string like `format!("bg-{}-soft", "warn")` at runtime
- **THEN** the resulting class SHALL produce visible styling because `bg-warn-soft` is in the Tailwind safelist

