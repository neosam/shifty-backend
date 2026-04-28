## ADDED Requirements

### Requirement: Canonical type scale

The application SHALL expose a canonical typography scale that matches `shifty-design/project/Shifty Preview.html` (the design handoff bundle's primary preview file). The scale SHALL be the single source of truth for font sizes, line heights, weights, and letter-spacing in the frontend.

The scale tokens SHALL be:

| Token       | Size  | Line height | Weight   | Tracking      | Intended use                                                  |
|-------------|-------|-------------|----------|---------------|---------------------------------------------------------------|
| `display`   | 32 px | 38 px       | 700      | -0.02em       | Empty-state hero text, marketing-style headers (rare).        |
| `h1`        | 22 px | 28 px       | 600      | -0.01em       | Page-level headlines (e.g., "Meine Schichten").              |
| `h2`        | 18 px | 24 px       | 600      | -0.005em      | Section headlines inside a page.                              |
| `lg`        | 16 px | 22 px       | 600      | 0             | Modal titles and prominent inline labels.                     |
| `body`      | 14 px | 20 px       | 400      | 0             | Default body text, table cells, list items, form fields.      |
| `small`     | 12 px | 16 px       | 500      | 0             | Sub-body, muted captions, hints, secondary table metadata.    |
| `micro`     | 11 px | 14 px       | 600      | 0.06em        | Uppercase eyebrow labels, badge content, column headers.      |

The scale SHALL be exposed as Tailwind utility classes via `theme.extend.fontSize` in `tailwind.config.js`, so each token is available as `text-<token>` (e.g., `text-h1`, `text-body`, `text-micro`).

#### Scenario: Tailwind exposes every token as a utility class
- **WHEN** a developer writes `class: "text-h1"`, `class: "text-body"`, `class: "text-small"`, `class: "text-micro"`, `class: "text-lg"`, `class: "text-h2"`, or `class: "text-display"` in a Dioxus component
- **THEN** the resulting CSS sets `font-size`, `line-height`, `font-weight`, and `letter-spacing` to the values defined in the table above

#### Scenario: Token sizes match the design handoff
- **WHEN** the rendered font-size of a typography token is measured against the same element in `shifty-design/project/Shifty Preview.html`
- **THEN** the values are identical (no off-by-one drift)

### Requirement: Body baseline and mobile breakpoint

`input.css` SHALL set the document body to the design's baseline so that elements which inherit (rather than override) the font size render at the design's intended size.

- Default: `body { font-size: 16px; line-height: 1.5; }`
- Mobile breakpoint: `@media (max-width: 720px) { body { font-size: 15px; } }`

#### Scenario: Default body inherits 16 px
- **WHEN** a Dioxus element renders text without an explicit `text-*` class
- **THEN** its computed font-size is 16 px on viewports wider than 720 px

#### Scenario: Mobile body inherits 15 px
- **WHEN** the same element renders on a viewport at most 720 px wide
- **THEN** its computed font-size is 15 px

### Requirement: No ad-hoc font-size declarations

The `src/` tree SHALL NOT contain inline `style: "font-size: …"` declarations or arbitrary Tailwind font-size classes (e.g., `text-[10px]`). All font sizes SHALL be expressed via the canonical token classes from this spec, so the scale stays consistent and the design tokens stay the single source of truth.

#### Scenario: Sweep finds no inline font-size declarations
- **WHEN** the codebase is grepped for `font-size:` and `text-\[` inside `src/**/*.rs`
- **THEN** no matches are returned (or any matches are explicitly justified in a code comment referencing this spec)

### Requirement: Token-to-element binding

Each element role SHALL use the token specified below, matching the design:

- Page headline (`h1` element or top-of-page heading): `text-h1`
- Section heading (`h2`, "Section header"): `text-h2`
- Modal title: `text-lg` (the 16 px token)
- Body text in tables, lists, detail views, form labels: `text-body`
- Sub-body, muted captions, table column metadata: `text-small`
- Uppercase eyebrow labels, badges, very small mono numerics: `text-micro`

#### Scenario: Page headline uses h1 token
- **WHEN** a page renders its top-level heading
- **THEN** the heading element has class `text-h1` (or equivalent token usage) and renders at 22 px / 28 px line-height

#### Scenario: Modal title uses lg token
- **WHEN** a dialog renders its title bar
- **THEN** the title text has class `text-lg` and renders at 16 px

#### Scenario: Eyebrow label uses micro token
- **WHEN** a section renders an uppercase eyebrow label (e.g., "ZUGEORDNETE PERSONEN")
- **THEN** the label has class `text-micro` and renders at 11 px with 0.06em letter-spacing
