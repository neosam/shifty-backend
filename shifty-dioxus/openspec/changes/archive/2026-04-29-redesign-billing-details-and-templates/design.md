## Context

The redesign series (archived as `2026-04-28-redesign-01-design-tokens` through `…-99-cleanup`) introduced a CSS-variable design-token palette in `input.css` and exposed it as Tailwind utility aliases in `tailwind.config.js`. Each `:root` token has a `[data-theme="dark"]` counterpart, so any class name expressed in tokens (e.g. `bg-surface`, `text-ink`) renders correctly under both themes without per-component code paths.

The series migrated every primary page to this palette **except two**:

- `src/page/billing_period_details.rs` (~500 lines) — uses `bg-white`, `bg-gray-50`, `bg-gray-100`, `text-gray-{400,500,600,700,900}`, `border-gray-{200,300}`, `bg-{green,red,blue}-{100,500,600,700}`, `text-{green,red,blue}-{600,800}`, ad-hoc `<button>`s with hard-coded blue/green/red backgrounds, native `<input>`/`<select>`/`<textarea>`/`<input type="checkbox">` styled inline.
- `src/page/text_template_management.rs` (~270 lines) — same legacy palette, plus a full HTML `<table>` styled with `bg-white`, `bg-gray-50` (header row), `divide-gray-200`, and four ad-hoc colored `<button>`s (Add / Save / Cancel / Edit / Delete).

Both pages render acceptably in light mode and degrade severely in dark mode (white card surfaces against `--bg: #0e1014`, low-contrast `text-gray-700` on dark surface, status pills lose hue distinction).

The migrated reference page in this codebase is `src/page/billing_periods.rs` (also includes a `no_legacy_classes_in_source` test at lines 294–321). Its mapping conventions are the de-facto playbook for this change.

## Goals / Non-Goals

**Goals:**
- Both pages render correctly under `[data-theme="dark"]` without any per-page conditional CSS.
- Both pages share the same atom set already used by the rest of the UI (`Btn`, `TextInput`, `Field`, `.form-input`).
- A `cargo test` run catches regressions if a future contributor reintroduces a legacy palette class on either file.

**Non-Goals:**
- No behavioral changes (data loading, filtering, action dispatch, routing, copy-to-clipboard, report generation all remain bit-for-bit identical).
- No new translation keys.
- No introduction of a "good"/"success" Btn variant. Save buttons that were green become `Primary` (accent-colored).
- No accessibility-driven structural changes (keep existing `<table>`, `<select>`, `<textarea>`, etc.).
- No change to `tailwind.config.js` safelist. The token classes are statically present in the migrated source.
- The `BillingPeriods` list page (`src/page/billing_periods.rs`) is already migrated and is **out of scope**.

## Decisions

### Decision 1: Reuse the `bg-surface` / `bg-surface-alt` / `bg-surface-2` triplet for layered surfaces

Cards on white backgrounds (`bg-white shadow rounded-lg`) become `bg-surface border border-border rounded-lg` (no shadow). Recessed surfaces inside a card (e.g. the `bg-gray-50` value tiles in the sales-person section, the form-card `bg-gray-100` background in template management) become `bg-surface-alt`. The Custom Report `<pre>` output gets `bg-surface-2` to give it the slightly recessed code-block look.

**Rationale**: This is exactly the layering already used by `src/page/billing_periods.rs:222` for list rows (`rounded-md border border-border bg-surface`) and matches the mapping table from the explore session.

**Alternatives considered**: Keeping `shadow` on cards. Rejected — the redesign series consistently dropped shadows in favor of `border border-border` because shadows look heavy under the dark palette.

### Decision 2: Every styled button becomes a `Btn` atom

The Add / Save / Cancel / Edit / Delete buttons in `text_template_management.rs` and the Generate-Report / Copy-to-Clipboard buttons in `billing_period_details.rs` are replaced with the existing `Btn` atom from `src/component/atoms/`. The mapping is:

| Current button                 | New `BtnVariant`           |
|--------------------------------|----------------------------|
| `bg-blue-500` (Add, Edit, Generate) | `Primary`             |
| `bg-green-500` (Save, Copy-to-Clipboard) | `Primary`        |
| `bg-gray-500` / `bg-gray-400` (Cancel, disabled) | `Secondary` (with `disabled` prop where applicable) |
| `bg-red-500` (Delete)          | `Danger`                   |

**Rationale**: The `Btn` atom already encodes accent/secondary/danger styling against tokens, so dark-mode behavior is automatic. Save and Copy-to-Clipboard are semantically primary affirmations, not "good/success" states — they belong on the accent color, not on `--good`. We do not introduce a `Good` Btn variant for one button.

**Alternatives considered**: Adding a `Good` variant to `Btn` to preserve the green Save button. Rejected — the redesign series never introduced a "success-action" button; Primary already carries that semantic in the atom contract. Adding a variant just to preserve old hue is gratuitous.

### Decision 3: Status pills use `*-soft` background tokens

The active/deleted pills currently expressed as `bg-green-100 text-green-800` and `bg-red-100 text-red-800` become `bg-good-soft text-good` and `bg-bad-soft text-bad`. The CSS tokens already define both variants for both themes, so the pills retain a hue distinction in dark mode while remaining legible.

**Rationale**: This is the same swap already performed in `src/page/billing_periods.rs:262-268`.

### Decision 4: Form inputs use the `form-input` focus class

Bare `<input type="text">`, `<input type="checkbox">`, `<select>`, and `<textarea>` elements that currently style their own border/focus (`border-gray-300 focus:ring-blue-500`) are migrated to either:
- the `TextInput` / `Field` atom pair where they fit (the filter text input in billing-period-details, the form fields in text-template-management), or
- a bare element with the `form-input` class added so that `:focus` is driven by `--accent` / `--accent-soft` from `input.css:155-159`.

For the `<select>` and `<textarea>` elements, the second path is used because we have no dedicated select/textarea atom yet — keeping these elements inline with the `form-input` class is the smallest move that yields a token-driven focus ring.

**Rationale**: The `.form-input:focus` rule is the established global focus convention (introduced in the redesign-02 atoms change). It works on any input/select/textarea without requiring a new atom.

**Alternatives considered**: Building new `Select` and `Textarea` atoms in this change. Rejected — out of scope; would expand the change beyond a presentation migration.

### Decision 5: Source-level guard tests on both pages

Both pages get a `#[cfg(test)] mod tests { fn no_legacy_classes_in_source() }` modeled exactly on `src/page/billing_periods.rs:294-321`. The test reads its own source via `include_str!`, slices off the test module, and asserts the prefix contains none of the legacy palette substrings: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`.

**Rationale**: Without the guard, the same drift that left these pages behind in the first place can recur. The pattern is already proven in this codebase.

**Alternatives considered**: A workspace-wide grep test. Rejected — slower, harder to localize failures, and the project convention is per-file guards.

### Decision 6: Bundle both pages in one change

Both pages get the same migration treatment, share the same token mapping, and depend on the same atom set. Splitting them into two changes would double the OpenSpec overhead without any benefit — they would be implemented and reviewed together regardless.

**Rationale**: The user explicitly asked for both pages in one ask. The redesign-series precedent (`redesign-05-page-myshifts`, `redesign-06-page-overview`, etc.) is one page per change, but those changes also reshaped layout and added requirements. This change is purely a token swap and has no per-page design decisions.

## Risks / Trade-offs

- **[Risk] Visual regression in light mode**: Card shadows disappear, status-pill hue saturation drops. → **Mitigation**: Visually inspect both pages in light mode before merging. The redesign series treated this kind of saturation drop as intended; users have already accepted it on every other migrated page.
- **[Risk] Filter text input loses its strong blue focus ring**: The current ring is `focus:ring-2 focus:ring-blue-500`; the migrated ring is `--accent` / `--accent-soft` (more muted, especially in light mode). → **Mitigation**: This is consistent with every other input on every other page. Acceptable.
- **[Risk] `<select>` elements still render with native browser chrome that does not honor the dark theme on some browsers**: This is not regressed by this change, but it remains a known limitation. → **Mitigation**: Out of scope; same situation exists on already-migrated pages.
- **[Risk] Removing `shadow rounded-lg` changes vertical rhythm**: Shadows currently add a few pixels of optical separation. → **Mitigation**: Switch to `border border-border` which is the codebase convention. Spacing utilities (`mb-6`, `p-6`, `gap-4`) stay unchanged.
- **[Risk] Save button changes from green to accent (blue)**: Some users may rely on the green-Save mental model. → **Mitigation**: Accept the change — the redesign series already established Primary as the affirmation color across every form on every other page.
