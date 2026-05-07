## Context

The redesign series (`redesign-01`–`redesign-09`) ported the application onto the design tokens and component patterns from the original design handoff. After internal feedback that text in the new design is too small to read comfortably, the design team produced an updated handoff bundle in `shifty-design/`. The primary preview file `Shifty Preview.html` and the `kontor` direction in `tokens.jsx` carry an enlarged typography scale relative to the older "kompakt" preview that the existing implementation followed.

Current state of the implementation:

- `input.css` sets no explicit `body { font-size }` — falls back to the browser default of 16 px.
- `tailwind.config.js` does not extend `theme.fontSize` — Tailwind's defaults apply.
- Components heavily use `text-xs` (12 px) for body content and table data.
- Some components use inline `style: "font-size: 10px"` (e.g., `working_hours_mini_overview.rs:139`).
- Page headlines are typically `text-xl` (20 px); modal titles are typically `text-lg` (18 px).
- No mobile breakpoint shrinks body text the way the design specifies.

The design's `kontor` `typeScale` and the explicit sizes in `Shifty Preview.html` together define the target. The `Shifty Preview.html` body is 16 px, mobile body is 15 px, and component sizes cluster on 11 / 12 / 14 / 16 / 18 / 22 / 32.

This change introduces typography as a first-class concept in the design-token system: a named scale, exposed via Tailwind utilities, with a specified element-to-token binding. After this change, font sizes are expressed via `text-<token>` and inline `font-size` is forbidden.

## Goals / Non-Goals

**Goals:**

- Match `shifty-design/project/Shifty Preview.html` exactly: token values, baseline, mobile breakpoint, and per-element usage.
- Make the typography scale explicit and named (`display`, `h1`, `h2`, `lg`, `body`, `small`, `micro`) so future code defaults to the canonical tokens rather than raw Tailwind sizes.
- Eliminate ad-hoc `font-size` declarations from `src/` so the scale has a single source of truth.
- Keep the token names stable across the three design directions defined in `tokens.jsx` (`kontor`, `tagblatt`, `plotter`) — only the values bound to each token differ per direction. The active direction in this change is `kontor`.

**Non-Goals:**

- Switching design directions away from `kontor`. The existing colors and radii are already on `kontor`; this change only addresses typography.
- Self-hosting the web fonts. They are still loaded via Google Fonts from `index.html`, as established in `redesign-01-design-tokens`.
- Per-page density toggles (`compact` / `cozy` / `comfortable` from `design-canvas.jsx`). The application uses a single density that matches the preview.
- Touching weights or letter-spacing of `Inter` / `JetBrains Mono` themselves; only token bindings change.
- Visual changes outside typography (colors, spacing, borders, shadows).
- Backend or `rest-types` changes.

## Decisions

### Decision: Use named tokens (`text-h1`, `text-body`, …) instead of raw px values

Tailwind's defaults (`text-xs`, `text-sm`, `text-base`, `text-lg`, `text-xl`, …) are scale-by-position and tell the reader nothing about *role*. The design specifies role-based sizes (display, h1, h2, body, small, micro). Naming the tokens by role makes intent obvious in the markup and lets us swap the underlying value when the design evolves without renaming every call site.

**Alternatives considered:**

- *Keep raw Tailwind names and only override their values via `theme.extend.fontSize`*: e.g., make `text-xl` = 22 px instead of 20 px. Rejected — collides with reader expectations of the Tailwind scale and silently changes the meaning of every existing `text-xl` site. Easier to introduce role-named tokens and migrate explicitly.
- *Custom CSS classes in `input.css` (`.t-body`, `.t-h1`)*: Rejected — Tailwind utilities compose well with the rest of the codebase's idiom; introducing a parallel CSS class system splits the styling story.

### Decision: Keep the legacy `text-xs` / `text-sm` / `text-lg` / `text-xl` Tailwind classes available

We do not remove or override Tailwind's built-in font-size classes. Doing so risks unintentional regressions in code paths the sweep misses. Instead, we add the new tokens and migrate call sites to them; legacy classes that survive the sweep are dead-code candidates rather than load-bearing.

The proposal lists this as **BREAKING** in the conventional sense (idiom changes), but the runtime behavior of legacy classes is unchanged.

### Decision: `text-lg` token = 16 px (matches design's modal titles)

The design uses 16 px for modal titles. The natural Tailwind name is `lg`. Binding `text-lg` to 16 px is intentional even though Tailwind's default `text-lg` is 18 px. This is the only token where the new value is *smaller* than what existing code currently produces (some modals today use `text-lg` = 18 px, design wants 16 px). Migration must therefore touch modal titles too — they are not no-ops.

**Alternatives considered:**

- *Skip a `text-lg` token, use `text-h2` for modal titles*: Rejected — the design's `h2` is 18 px / 600, not 16 px. The values diverge.
- *Bind modal-title tokens to `text-modal`*: Rejected — too narrow. The 16 px / 600 token is general-purpose for prominent inline labels, not just modals.

### Decision: Body baseline 16 px in `input.css`, mobile 15 px via media query

The design defines this explicitly. `input.css` is the right place because Tailwind's preflight does not set a body font-size and we want the cascade default to match the design.

The media query mirrors `Shifty Preview.html` line 88–89 verbatim (max-width 720 px → 15 px).

### Decision: Migration sweep order — tokens first, then call sites top-down

1. Add `theme.extend.fontSize` tokens in `tailwind.config.js`.
2. Add body baseline + mobile media query in `input.css`.
3. Migrate call sites page by page: TopBar → WeekView → Employees → MyShifts → Overview → BillingPeriods → modals → settings → mini-overviews.
4. After each page, do a visual diff against the corresponding section of `Shifty Preview.html` (read the source, do not run the prototype).

This order means the build keeps compiling and the app stays usable between commits — old classes coexist with new tokens until the sweep finishes.

### Decision: Forbid inline `font-size:` and `text-[…]` arbitrary values

`No ad-hoc font-size declarations` is a normative requirement in the spec. We enforce it by hand in this change (grep at the end), not via a lint rule, because Tailwind/Dioxus tooling does not give us a clean automated check today. A future repo-level CI check is desirable but out of scope for this change.

## Risks / Trade-offs

- **Visual regression risk** → The design is fully specified in HTML/CSS in `shifty-design/`, so each page can be diff'd by reading both side by side. The compile + test suite verify *correctness*, not *appearance*. The `dx serve --hot-reload` workflow should be used for the actual visual check.
- **`text-lg` semantic flip (18 px → 16 px)** → A subset of existing modal titles will visually shrink rather than grow. We must explicitly review every `text-lg` site — the sweep cannot blindly keep them. The `tasks.md` enumerates these.
- **Mixed typography during the migration window** → While the page-by-page sweep is in progress, the app shows a mix of old and new sizes. Acceptable because the change ships in a single PR; no intermediate state is released.
- **Token names diverge from Tailwind convention** → Readers who expect `text-xl > text-lg > text-base > text-sm > text-xs` lose that intuition; `text-h1 > text-h2 > text-lg > text-body > text-small > text-micro` is role-named, not size-ordered. We mitigate by documenting the table in the spec and by including the size in the token name where helpful.
- **Mobile breakpoint at 720 px is design-driven, not user-research-driven** → If the breakpoint turns out to be wrong for production traffic, it is a one-line CSS change. Acceptable.
