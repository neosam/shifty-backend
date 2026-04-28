## ADDED Requirements

### Requirement: `PersonChip` is atomic and never wraps mid-name
The `PersonChip` component SHALL render as a single, non-wrapping inline-flex pill. Its shape classes SHALL include `whitespace-nowrap` (in addition to the existing `inline-flex px-[4px] pl-[7px] py-px rounded-sm text-body font-medium`) so that the chip's text remains on one line regardless of the chip's container width or the length of the `name` value. Multi-chip layout decisions (when to break a row) SHALL be made by the chip's parent container via flex-wrap, not by the chip wrapping its own text.

#### Scenario: Chip with long name does not wrap mid-word
- **WHEN** a `PersonChip` is rendered with a long name like `"Alexandra-Sophie M."`
- **THEN** the chip's class list SHALL include `whitespace-nowrap`

#### Scenario: Chip without color also carries whitespace-nowrap
- **WHEN** a `PersonChip` is rendered with `color: None`
- **THEN** the chip's class list SHALL include `whitespace-nowrap`

#### Scenario: Bold variant preserves whitespace-nowrap
- **WHEN** a `PersonChip` is rendered with `bold: true`
- **THEN** the chip's class list SHALL include `whitespace-nowrap` AND `font-semibold`
