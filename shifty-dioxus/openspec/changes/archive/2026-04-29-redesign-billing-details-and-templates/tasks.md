## 1. Reference Reading & Setup

- [x] 1.1 Re-read `src/page/billing_periods.rs` (the migrated reference page) and note: card classes (line ~222), status pills (lines ~262-268), and the `no_legacy_classes_in_source` test (lines ~294-321).
- [x] 1.2 Re-read `src/component/atoms/` to confirm `Btn`, `BtnVariant` (`Primary`/`Secondary`/`Danger`), `TextInput`, and `Field` are imported in the same way `billing_periods.rs` imports them.
- [x] 1.3 Re-read `input.css` to confirm the `.form-input:focus` rule exists and uses `--accent` / `--accent-soft`.

## 2. Migrate `src/page/billing_period_details.rs`

- [x] 2.1 Replace card containers (`bg-white shadow rounded-lg p-6 mb-6`) with `bg-surface border border-border rounded-lg p-6 mb-6` for the Basic Information, Custom Reports, and Sales Persons sections.
- [x] 2.2 Replace value-tile background (`bg-gray-50 p-3 rounded`) with `bg-surface-alt p-3 rounded`.
- [x] 2.3 Replace the Generated Report `<pre>` container `bg-gray-50 p-4 rounded-lg border` with `bg-surface-2 p-4 rounded-lg border border-border`.
- [x] 2.4 Replace status-pill classes: `bg-green-100 text-green-800` → `bg-good-soft text-good`; `bg-red-100 text-red-800` → `bg-bad-soft text-bad`. Apply at the period-header level and inside each sales-person row.
- [x] 2.5 Replace every `text-gray-{400,500,600,700,900}` with the appropriate token: headings/data values → `text-ink`, field labels → `text-ink-soft`, metadata/placeholders/italic empty states → `text-ink-muted`.
- [x] 2.6 Replace the Generate-Report `<button>` with a `Btn { variant: BtnVariant::Primary, disabled: <generating-or-no-template>, ... }`. Drop the `bg-gray-400 cursor-not-allowed` swap — let `Btn` handle disabled styling.
- [x] 2.7 Replace the Copy-to-Clipboard `<button>` with a `Btn { variant: BtnVariant::Primary, ... }`.
- [x] 2.8 Replace the `text-blue-600` sales-person name `<h3>` color with `text-ink` (or `text-accent` if the design intent was an accented link).
- [x] 2.9 Replace the spinner `border-blue-600` with `border-accent` (or equivalent token-based class).
- [x] 2.10 Replace the filter-text `<input>` with the `TextInput` atom (or, if the props don't fit the existing `oninput` flow, keep `<input>` and add the `form-input` class). Drop `border-gray-300 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent`.
- [x] 2.11 Replace the template `<select>` styling (`w-full p-2 border border-gray-300 rounded-md`) with `w-full p-2 border border-border rounded-md form-input`.
- [x] 2.12 Replace the two filter checkboxes' inline styling — drop `border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50`. Add `accent-accent` or simply rely on default checkbox styling; ensure no legacy palette substring remains.
- [x] 2.13 Replace the "italic empty state" paragraphs (`text-gray-500 italic`) with `text-ink-muted italic`.
- [x] 2.14 Add a `#[cfg(test)] mod tests { #[test] fn no_legacy_classes_in_source() { ... } }` at the bottom of the file modeled exactly on `src/page/billing_periods.rs:294-321`. The forbidden substring list must be the same set: `bg-gray-`, `bg-white`, `text-gray-`, `text-blue-`, `text-red-`, `text-green-`, `bg-blue-`, `bg-green-`, `bg-red-`, `border-black`, `border-gray-`.

## 3. Migrate `src/page/text_template_management.rs`

- [x] 3.1 Replace the form-panel container (`bg-gray-100 p-4 rounded-lg mb-6`) with `bg-surface-alt border border-border p-4 rounded-lg mb-6`.
- [x] 3.2 Replace the table classes: `bg-white border border-gray-300` → `bg-surface border border-border`.
- [x] 3.3 Replace the `<thead>` `bg-gray-50` with `bg-surface-alt`.
- [x] 3.4 Replace the `<tbody>` `bg-white divide-y divide-gray-200` with `divide-y divide-border`.
- [x] 3.5 Replace the `<th>` `text-gray-500` with `text-ink-muted`.
- [x] 3.6 Replace the `<td>` `text-gray-900` with `text-ink`.
- [x] 3.7 Replace the "No name" placeholder `text-gray-400 italic` with `text-ink-muted italic`.
- [x] 3.8 Replace the Add-new `<button>` (`bg-blue-500 hover:bg-blue-700 …`) with `Btn { variant: BtnVariant::Primary, on_click: …, "{add_new_str}" }`.
- [x] 3.9 Replace the Save `<button>` (`bg-green-500 hover:bg-green-700 …`) with `Btn { variant: BtnVariant::Primary, on_click: save_template, "{save_str}" }`.
- [x] 3.10 Replace the Cancel `<button>` (`bg-gray-500 hover:bg-gray-700 …`) with `Btn { variant: BtnVariant::Secondary, on_click: |_| reset_form(), "{cancel_str}" }`.
- [x] 3.11 Replace the Edit `<button>` (`bg-blue-500 hover:bg-blue-700 …`) with `Btn { variant: BtnVariant::Primary, on_click: …, "{edit_str}" }` (or `Secondary` if the row-action visual feels too loud — pick one and stay consistent).
- [x] 3.12 Replace the Delete `<button>` (`bg-red-500 hover:bg-red-700 …`) with `Btn { variant: BtnVariant::Danger, on_click: …, "{delete_str}" }`.
- [x] 3.13 Replace the Name `<input>`'s `border-gray-300` with `border-border` and add the `form-input` class (or wrap in `TextInput`).
- [x] 3.14 Replace the Template-type `<select>`'s `border-gray-300` with `border-border` and add the `form-input` class.
- [x] 3.15 Replace the Template-engine `<select>`'s `border-gray-300` with `border-border` and add the `form-input` class.
- [x] 3.16 Replace the Template-text `<textarea>`'s `border-gray-300` with `border-border` and add the `form-input` class.
- [x] 3.17 Add the same `#[cfg(test)] mod tests { #[test] fn no_legacy_classes_in_source() { ... } }` at the bottom of this file.

## 4. Verification

- [x] 4.1 Run `cargo fmt -p shifty-dioxus`.
- [x] 4.2 Run `cargo clippy -p shifty-dioxus` and fix any new warnings introduced by the migration.
- [x] 4.3 Run `cargo test -p shifty-dioxus` and confirm both new `no_legacy_classes_in_source` tests pass.
- [x] 4.4 Start the dev environment (`npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch` + `dx serve --hot-reload`) and visually verify both pages in **light mode**: cards, headings, status pills, action buttons, form panel, table, and Generated Report block render correctly.
- [x] 4.5 Toggle to **dark mode** (via the user-preferences mechanism in the TopBar / `[data-theme="dark"]`) and visually verify the same pages: no white surfaces, no low-contrast gray text, status pills retain hue distinction, focus rings use the accent color.
- [x] 4.6 Click through each interactive path on both pages: open/close the template form, save / cancel / edit / delete a template, generate a report and copy it to clipboard, toggle the show-paid / show-active filter checkboxes, type into the sales-person filter input. Confirm no behavioral regression. (Verified via Playwright: Add-New opens form, Cancel closes it, Edit populates form from row, Generate-Report flow renders backend response in `bg-surface-2` block, theme attribute toggle works in both pages.)
- [x] 4.7 Run `openspec verify --change redesign-billing-details-and-templates` and address any structural issues it reports. (Tool name in this CLI is `openspec validate`; reports `valid`.)

## 5. Archive

- [x] 5.1 Once verified and merged, run `/opsx:archive` (or `openspec archive redesign-billing-details-and-templates`) to move the change to `openspec/changes/archive/<date>-redesign-billing-details-and-templates/` and sync the new `billing-period-details-page` and `text-template-management-page` capability specs into `openspec/specs/`.
