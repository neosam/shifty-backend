## Context

`WeekCellSlot` (`src/component/week_view.rs:1029-1167`) renders one absolutely positioned box per slot inside its day column. The box's height is a direct function of the slot's time extent: `height = (slot.to_hour() - slot.from_hour()) * SCALING`. The current inner DOM is a single `flex flex-wrap items-start gap-1` container that holds, in flow order, the n/m pill, every `WeekCellChip`, and finally the +/- (or `…` dropdown) button. The action button is then re-rendered as `position: absolute; top: 6; right: 6` via `cell_button_classes` — the `flex-wrap` container is what reserves the right padding (`padding: 6px 32px 6px 8px`).

When the number of bookings × chip width exceeds the slot's height, the chips wrap to a second/third line and visually leak into the next hour's slot because the outer container has no `overflow` rule. The shifty-design reference (`shifty-design/project/Shifty Preview.html` lines 494-548) solves this with a fixed-height outer cell that crops with `overflow: hidden`, and an inner chip layer that owns the scroll. The min-resources pill and action button live as sibling absolute elements outside the scroll area, so they remain visible regardless of chip count.

Constraints:
- The slot box height is **already** fixed (driven by time math), so we do not need to introduce a fixed pixel value — the existing `height: <px>` style stays. We only need to crop and rearrange the interior.
- The action button is shared with a dropdown variant (`CellButton::Dropdown`) which carries its own absolute wrapper. Both variants must coexist with the new layout.
- The `PersonChip` atom is consumed in many other contexts (toolbar, my-shifts, employee detail, etc.); changes to its shape classes ripple wider than the shiftplan.

## Goals / Non-Goals

**Goals:**
- Chips never paint outside their slot box, regardless of booking count or slot height.
- The n/m pill stays visible at the slot's top-left even when the chip area scrolls.
- The +/- / `…` button stays visible and clickable at the top-right under the same condition.
- Single-name chips don't wrap mid-word; multi-chip layouts wrap as whole chips.
- No regressions for the dropdown-button variant or the `discourage` warn-soft tinting.

**Non-Goals:**
- Changing slot heights or the time-to-pixel math.
- Replacing the absolute-slot model with a Stunden-Grid (the reference's macro layout). That would be a much larger redesign and is out of scope.
- Adding a "show more" affordance, expand-on-hover, or any UX beyond a native scrollbar. Plain `overflow-y: auto` is sufficient.
- Touch gesture support for the chip scroll beyond what the browser provides natively.
- Updating `column_view.rs::ColumnViewSlot` (the older absolute-positioned slot used in non-shiftplan contexts) — that path is unchanged.

## Decisions

### 1. Three-zone absolute layout inside the slot box

The slot's outer `<div>` keeps its current absolute positioning and computed height, but gains `overflow: hidden`. Inside it, three siblings:

```
┌─ slot box (height: time-derived, overflow: hidden) ──────┐
│ ┌──┐                                              ┌────┐ │
│ │2/3│  ┌── chip area ─────────────────────────┐   │ +/− │ │
│ │  │  │ inset: 6 32 6 38                      │   └────┘ │
│ └──┘  │ flex flex-wrap                        │          │
│       │ overflow-y: auto                      │          │
│       │ overflow-x: hidden                    │          │
│       │ alignContent: flex-start              │          │
│       │  [chip] [chip] [chip] [chip] [chip]   │          │
│       └───────────────────────────────────────┘          │
└──────────────────────────────────────────────────────────┘
```

- **Why three siblings, not nested:** the n/m pill and the action button must remain visible and clickable when the chip area scrolls. Putting them as siblings of the scroll container (not inside it) means they participate in the slot box's layout, not the inner scrollbox's. This matches the reference exactly.
- **Why `inset` instead of `top/right/bottom/left`:** `inset` is one shorthand and reads clearly — `inset: 6 32 6 38` reserves 38px on the left for the n/m pill (`left: 8` + ~24px pill width + ~6px gap) and 32px on the right for the 20px button (`right: 6` + 20px width + ~6px gap).
- **Alternative considered:** keep the single flex-wrap container, just add `overflow: hidden` outside and `overflow: auto` inside. Rejected because the n/m pill and the action button would scroll away with the chips.

### 2. Outer slot keeps `overflow: hidden`, not `overflow: clip`

`overflow: hidden` is universally supported and provides the same crop. `clip` (which forbids any scrolling, even programmatic) is functionally equivalent here but has no benefit and slightly worse browser support in older WASM hosts. We pick `hidden`.

### 3. Min-resources pill becomes absolutely positioned

Currently the pill is the first flex-child of the wrap container. In the new layout it must be a sibling of the chip area, positioned `absolute; top: 6; left: 8` with `pointer-events: none` so it can't swallow chip clicks. The pill's existing `bg-warn-soft text-warn` (understaffed) vs. neutral (fully-staffed) tinting is preserved verbatim — only positioning changes.

### 4. `PersonChip` gains `whitespace-nowrap`

Adding `whitespace-nowrap` to `SHAPE_CLASSES` (`src/component/atoms/person_chip.rs:27`) makes every chip atomic. This is a behavior the reference asserts via inline style (`whiteSpace: 'nowrap'` on the pill). Without it, a chip with a long name (e.g. `"Alexandra-Sophie M."`) could break mid-word and produce an oddly tall chip; with it, the chip stays one line and the chip-area's `flex-wrap: wrap` decides where to break the row.

This is technically a wider change than just the shiftplan, but it's the right invariant for the chip atom: a "name pill" should never break mid-name, in any consumer.

### 5. Use Tailwind utility classes, not inline styles

The current `WeekCellSlot` mixes `class:` token utilities and `style:` inline rules. The new layout follows the same pattern: positions and inset can be inline (because `inset: 6px 32px 6px 38px` doesn't have a Tailwind shorthand and pulling in arbitrary `inset-[...]` is uglier than `style="inset: 6px 32px 6px 38px"`), while overflow / display / flex come via `class:` (`overflow-hidden`, `flex flex-wrap`, `overflow-y-auto`, `overflow-x-hidden`, `content-start`).

**Alternative considered:** push everything to Tailwind arbitrary values (`inset-[6px_32px_6px_38px]`). Rejected — the arbitrary-value syntax is less readable than plain inline `style` for one-off pixel values, and the rest of `week_view.rs` already mixes the two.

### 6. SSR tests stay structural, not pixel-based

We assert class fingerprints (`overflow-hidden` on the slot, `overflow-y-auto` on the chip area, `whitespace-nowrap` on the chip) and DOM structure (one chip area sibling, n/m pill is a sibling of the chip area not a child) — not computed pixel layouts. This matches the existing test style in `person_chip.rs::tests`.

## Risks / Trade-offs

- **Native scrollbar appearance varies by browser/OS** → in extreme cases the scroll area shows a chunky scrollbar that competes visually with chips. Mitigation: rely on the existing global `::-webkit-scrollbar { width: 10px }` rule from `input.css`; if it ever proves too loud here we can add a `no-scrollbar` utility class on the chip area later.
- **`whitespace-nowrap` on every PersonChip ripples beyond the shiftplan** → other consumers (employee detail, my-shifts) might rely on names wrapping. Mitigation: audit those call sites (small list — see tasks) and confirm visually that nowrap is the correct behavior. If a single consumer needs wrapping, we add a `wrap` opt-in prop to `PersonChip` rather than reverting the default.
- **Chip area's left inset (38px) is a magic number** → it depends on the n/m pill's rendered width (currently `2/2`-ish, ≈ 24px). A locale that produces a wider format string (e.g. `12/15`) could make the pill overlap the leftmost chip. Mitigation: 38px is large enough for the worst realistic case (`99/99` ≈ 36px in JetBrains Mono at 11px); if a future format breaks this, switch to `padding-left` on the chip area derived from the pill's rendered width via a CSS variable.
- **The chip area inherits no `tabindex`** → keyboard users can't scroll it without a focusable child. Mitigation: chips already wrap focusable elements (tooltip handlers); native browser scroll-on-tab-into-view will keep the active chip visible. No additional ARIA needed.
- **Tests can only assert class strings, not actual scroll behavior** → a future regression that drops `overflow: hidden` on the outer would slip past unit tests. Mitigation: the proposal adds a structural assertion that matches the exact class fingerprint, so any accidental removal flips the test red.
