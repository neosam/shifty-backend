# Handoff: Shifty UI Redesign

## Overview

This handoff covers the redesign of **Shifty** — a shift planning app for retail/sales teams. The bundle contains a high-fidelity, interactive HTML prototype of the entire app: shift planning grid, my-shifts view, year overview, employees admin, and user/SalesPerson administration. It is intended to be implemented in the existing **Dioxus 0.6 + Tailwind v3** codebase (`shifty-dioxus`).

## About the Design Files

The HTML files in this bundle are **design references**, not production code. They are React + inline-style prototypes whose only purpose is to demonstrate the intended layout, color tokens, interactions, and component anatomy.

**Your job is to recreate these designs in the existing `shifty-dioxus` codebase** — Dioxus 0.6 components in Rust, styled with Tailwind utilities, building on the project's existing structure. Do **not** copy HTML/JSX verbatim; translate it idiomatically.

## Fidelity

**High-fidelity (hifi).** Final colors, typography, spacing, radii, and interaction patterns are settled. Recreate pixel-perfectly using Tailwind utilities and CSS variables.

## Files in this bundle

- `Shifty Preview.html` — main interactive prototype (all pages)
- `preview-data.jsx` — sample data shape (people, weeks, year summary, my-shifts)
- `modal.jsx` — slot-edit modal component
- `Shifty Design System.html` — token + component reference page

The Preview is a single-page React app loaded via Babel-standalone. All code lives in `<script type="text/babel">` blocks inside `Shifty Preview.html`.

---

## Recommended migration order

1. **Tokens** — port CSS variables to `input.css`, alias them in `tailwind.config.js`
2. **Atom components** — `Btn`, `PersonChip`, `TupleRow`, `navBtn`
3. **Layout** — `TopBar`, routing
4. **Pages** in this order: `MyShiftsPage` → `OverviewPage` → `EmployeesPage` → `UsersPage` → `SchichtplanPage` (last because it's the most complex)
5. **Modals** — `SlotModal`, contract modal, extra-hours modal

---

## Design Tokens

Ported directly from the prototype's `<style>` block. Define in `input.css` and alias in `tailwind.config.js`.

### CSS variables (light theme)

```css
:root {
  --bg: #fbfbfc;
  --surface: #ffffff;
  --surface-alt: #f4f5f7;
  --surface-2: #eef0f4;
  --border: #e6e8ec;
  --border-strong: #d0d3da;
  --ink: #0e1117;
  --ink-soft: #3a4150;
  --ink-muted: #6b7382;
  --accent: #3a4cd1;
  --accent-ink: #ffffff;
  --accent-soft: #eaecfb;
  --good: #0e7a4d;
  --good-soft: #defaee;
  --warn: #a65a08;
  --warn-soft: #fef0d6;
  --bad: #b8281a;
  --bad-soft: #fde4e1;
  --modal-veil: rgba(14,17,23,0.4);
  --r-sm: 4px; --r-md: 6px; --r-lg: 10px;
}
```

### CSS variables (dark theme)

```css
[data-theme="dark"] {
  --bg: #0e1014;
  --surface: #16191f;
  --surface-alt: #1c2027;
  --surface-2: #232831;
  --border: #2a2f39;
  --border-strong: #3a4151;
  --ink: #eef0f4;
  --ink-soft: #b8bdc7;
  --ink-muted: #7a8290;
  --accent: #8b97ff;
  --accent-ink: #0e1014;
  --accent-soft: #232a4a;
  --good: #4ed59a;
  --good-soft: #16322a;
  --warn: #f0b766;
  --warn-soft: #3a2a14;
  --bad: #ef6a5b;
  --bad-soft: #3a1c18;
  --modal-veil: rgba(0,0,0,0.6);
}
```

### Recommended `tailwind.config.js` extension

```js
theme: {
  extend: {
    colors: {
      bg: 'var(--bg)',
      surface: 'var(--surface)',
      'surface-alt': 'var(--surface-alt)',
      'surface-2': 'var(--surface-2)',
      border: 'var(--border)',
      'border-strong': 'var(--border-strong)',
      ink: 'var(--ink)',
      'ink-soft': 'var(--ink-soft)',
      'ink-muted': 'var(--ink-muted)',
      accent: 'var(--accent)',
      'accent-ink': 'var(--accent-ink)',
      'accent-soft': 'var(--accent-soft)',
      good: 'var(--good)',
      'good-soft': 'var(--good-soft)',
      warn: 'var(--warn)',
      'warn-soft': 'var(--warn-soft)',
      bad: 'var(--bad)',
      'bad-soft': 'var(--bad-soft)',
    },
    borderRadius: {
      sm: 'var(--r-sm)',  // 4px
      md: 'var(--r-md)',  // 6px
      lg: 'var(--r-lg)',  // 10px
    },
    fontFamily: {
      sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
      mono: ['"JetBrains Mono"', 'ui-monospace', 'Menlo', 'monospace'],
    },
  },
}
```

Add to `safelist` (state-dependent classes built dynamically):
`bg-bad-soft`, `bg-warn-soft`, `bg-accent-soft`, `text-bad`, `text-warn`, `text-good`, `border-bad`, `border-warn`, `border-accent`.

### Typography

- **Sans:** Inter (400, 500, 600, 700)
- **Mono:** JetBrains Mono (400, 500, 600, 700) — used for all numeric values (hours, dates, counts)
- Body base: 14px / 1.5
- H1 (page title): 22px / 600 / letter-spacing -0.01em
- H2 (section): 15–16px / 600–700
- Mono cell numbers: 10–13px depending on density

---

## Theming behavior

- Three modes: `light`, `dark`, `system`
- Persisted in `localStorage` under key `shifty-theme`
- Resolved value written to `<html data-theme="...">`
- Toggle button in TopBar cycles light → dark → system
- Icons: ☀ light · ☾ dark · ⌬ system

---

## Component inventory

### `Btn`
4 variants:
- **primary**: `bg-accent text-accent-ink border-accent`
- **secondary**: `bg-surface text-ink border-border-strong`
- **ghost**: transparent, `text-ink-soft`, transparent border
- **danger**: `bg-surface text-bad border-bad`

Padding `6px 12px`, radius `md`, font 13/500, optional mono icon prefix. Disabled: 50% opacity, `not-allowed` cursor.

### `PersonChip`
A pastel name pill. **No initials, no avatar circle inside.**
- Background: `person.color` (predefined pastel hex per SalesPerson) or transparent dashed if no color
- Text: always dark ink (`--chip-ink: #0e1117`) in **both** light and dark themes — pastel colors are designed for dark text. There's a `.person-pill` CSS rule with `!important` to enforce this.
- Padding `1px 4px 1px 7px`, radius `sm`, font 12/500
- Whole-column conflicts are signaled by the cell background, **not** by the chip

### `TupleRow`
Label/value row used in the Mitarbeiter detail page. Label left, mono value right, 1px bottom border, 13px text, optional dim variant for secondary fields.

### `navBtn`
Square 28×28 icon button with `border-strong` border, used for prev/next week and prev/next year navigation.

---

## Screens

### 1. TopBar (sticky header, 56px tall)

- Brand wordmark `Shifty` + accent-colored period (`.`)
- Nav buttons: Schichtplan · Meine Schichten · Jahresübersicht · Mitarbeiter · Benutzerverwaltung
- Active nav: `bg-accent-soft text-accent`, font-weight 600
- Right side: theme toggle + "Du bist [name]" pill
  - The pill is **just text** wrapped in a `surface-alt` rounded-full container. **No avatar circle, no initials.**
- **Mobile (< 720px):** burger menu, nav collapses into dropdown panel below the bar

### 2. Schichtplan (Shift Plan) — the core screen

Layout: toolbar → tabs → week grid → working-hours mini overview → optional booking log

#### Toolbar (`.toolbar-row`)
- Prev/next week buttons + week label (`KW 17 · 20.04 – 26.04`)
- View toggle: `Woche` / `Tag` (segmented control inside `surface-alt` pill)
- "Letzte Woche" copy button + iCal export
- **"Du bearbeitest:" select** — picks the SalesPerson currently being scheduled. This drives the +/− button behavior in cells (see below).

#### Tabs
Categories like `Laden`, `Lager`, `Kasse` etc. — flat underline tabs with accent active state.

#### Week grid
- CSS Grid: `grid-template-columns: 76px repeat(6, minmax(140px, 1fr))`, `min-width: 920px`
- Header row: empty corner cell + 6 day cells (long name + date + day total in mono)
- Body rows: hour label (e.g. `09:00–10:00`) + 6 cells

**Critical behaviors:**

- **Sticky time column.** The 76px hour column is `position: sticky; left: 0; z-index: 2` (3 for the empty corner) so it stays visible during horizontal scroll on mobile.
- **Each cell shows:**
  - Mono `filled/need` count (e.g. `2/2`)
  - Person chips for each assigned SalesPerson
  - **Single +/− button** absolutely positioned `top: 6, right: 6`, 20×20px
- **+/− button logic:** there is exactly one button per cell; it adds or removes the SalesPerson currently selected in the "Du bearbeitest:" dropdown. If that person is already in the cell, the button shows `−` with `bad`-tinted styling; otherwise `+` with neutral styling. **No per-chip × button** — chips are display-only.
- **State backgrounds:**
  - Missing staff (`filled < need`): `bg-warn-soft`
  - Conflict: `bg-bad-soft`, **but only when the entire day column is in conflict** (every staffed cell in that day has a conflict). Per-cell red was rejected as too noisy. Compute `dayConflicts[di]` once per day.
  - Hover: slightly darker variant of state color
- Cell padding: `6px 32px 6px 8px` (right padding reserved for the absolute +/− button)

#### Working-hours mini overview
Below the grid, an auto-fit grid of small cards — one per SalesPerson — showing color dot, name, current/target hours, and a thin progress bar (warn color if under target, good if at/over).

#### Booking log
Toggleable table with rows: day · name · time · created-by · deleted-by. Deleted rows shown at 50% opacity with bad-tinted "Gelöscht" cell.

### 3. Meine Schichten (My Shifts)

Per-week cards (max-width 760px, centered):
- Card header: `KW <n> · <date range>` left, total hours mono right
- One row per day: day label (mono, 110px) · shift items · hours (mono, 60px right-aligned)
- Each shift item: `09:00–13:00` (mono) + colored area badge (`Laden` etc., accent-soft pill)
- Days with `note` starting `⚠` get `bg-warn-soft` background and warning text

### 4. Jahresübersicht (Year Overview)

- Year nav (prev/next + mono year label)
- Stacked-bar chart: 52 thin bars, each = paid + volunteer hours, with dashed `bad`-colored required-hours line per bar. Current week highlighted in full accent; others dimmed.
- Legend chips above the chart
- Table below: 10-week window (KW 16–25), columns: Woche · Bezahlt/Freiwillig · Verfügbar/Benötigt · Differenz. Current week row tinted `accent-soft`.

### 5. Mitarbeiter (Employees)

Two-column layout:
- **Left list (280–360px):** searchable list of people. Each row: color-dot avatar circle (no initials, no text in it — just color), name, hours-vs-target mono. Active row: `bg-accent-soft` + 3px `accent` left border. Mobile: only one column shown at a time, with a back button on detail.
- **Right detail:**
  - Header: large color circle, name, type (Bezahlt/Freiwillig) + target hours, year nav, "Sonstige Stunden" button, "Mehr ▾"
  - Three-column sub-grid (auto-fit, min 280px):
    1. **Gesamtansicht** — TupleRow stacks: Stundenkonto, Gesamt, Soll, then dim breakdown
    2. **Arbeitsverträge + Stunden pro Woche** — clickable contract cards, then a 17-bar histogram (warn color if under target). Clicking a bar opens an inline week-detail panel.
    3. **Zusatzarbeit** — extra-hours list

### 6. Benutzerverwaltung (User Administration)

**Two tabs at the top:** `SalesPerson` and `Benutzer` — both manageable independently.

#### SalesPerson tab
Table columns: SalesPerson (color dot + name) · Typ (Bezahlt/Freiwillig pill) · Verknüpfter Benutzer (mono, login name or `—`) · Edit button.

**Important:** `Sollstunden` does **not** belong here. Working hours are managed via Arbeitsverträge in the Employee detail page.

#### Benutzer tab
Table columns: Benutzer (mono login) · Verknüpfter SalesPerson · Rollen (accent pill chips) · Status (good/muted dot + label) · Edit button.

Roles list: `admin`, `shiftplanner`, `sales`, `hr`.

---

## Interactions & Behavior

- **Theme toggle** cycles light → dark → system; persists in `localStorage`.
- **Schichtplan +/− button** mutates the day's assignments for the SalesPerson selected in "Du bearbeitest:". Single canonical action — no separate add/remove flows.
- **Modal dismissal** — backdrop click + Esc + close button + Cancel; primary action confirms.
- **Hover states** on cells: subtly darker variant of the cell's state color.
- **Mobile nav** opens a dropdown via a burger button at `< 720px`.
- **Pinch-zoom** is allowed (no `maximum-scale`/`user-scalable=no`).

## State Management

For Dioxus, use `use_signal` for local UI state and `use_context` for cross-component values.

Per-page state needed:
- **App-level:** `route` (which page), `theme`, `selectedEmployee`, `slot` (open slot modal), `tweaks` (config)
- **SchichtplanPage:** `tab` (category), `view` (week/day), `editingFor` (SalesPerson id), `showLog`
- **EmployeesPage:** `filter`, plus EmployeeDetail's `year`, `fullYear`, `expandWeeks`, `selectedWeek`, `contractOpen`, `extraOpen`
- **UsersPage:** `tab` (`sales` | `users`)

Server data assumed: People list, week grids, year summary, my-shifts. Shape visible in `preview-data.jsx`.

---

## Responsive behavior

Breakpoint: `720px`. Below that:
- TopBar collapses to burger menu
- Employees grid stacks; list and detail are mutually exclusive (back button shown)
- Week grid scrolls horizontally with **sticky time column**
- Toolbar wraps; dividers and spacers hide
- My-Shifts day rows compact: `80px 1fr 50px`

---

## Assets

No external image or icon assets. Inline glyphs only:
- Theme icons: `☀ ☾ ⌬`
- Burger: `☰` / `✕`
- Nav arrows: `‹ ›`
- Plus/minus: `+ −`

Fonts via Google Fonts: Inter + JetBrains Mono. Replace with self-hosted in production if desired.

---

## Reference: behaviors that came out of design review

These were explicitly resolved during prototyping and should not be re-litigated:

1. **No initials.** No `LB`, `MH` etc. anywhere. Avatar circles are color dots only.
2. **No empty avatar in TopBar.** "Du bist [name]" is text only inside a pill.
3. **Red conflicts only on whole columns**, never on single cells or chips.
4. **Single +/− button per cell**, fixed top-right, drives the currently-edited SalesPerson.
5. **Sticky time column** in the shift grid for mobile horizontal scroll.
6. **Sollstunden lives in Arbeitsverträge**, not in SalesPerson admin.
7. **Benutzerverwaltung has two tabs**: SalesPerson and Benutzer.

---

## Open questions for implementer

- Should `Tag` view (mobile-friendly single-column) be implemented? Currently a UI toggle without logic.
- Should an in-app zoom / density control be added for the shift grid? Currently relies on browser pinch-zoom.
- Day-only conflict detection in the prototype is a simple heuristic (every staffed cell in the column conflicts). Confirm the real backend rule.
