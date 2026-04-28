## ADDED Requirements

### Requirement: Week grid vertical extent snaps to whole-hour boundaries
The week grid's vertical extent SHALL be computed from a hour-aligned range, not from the raw fractional slot bounds. Specifically, given the per-week minimum slot start hour `day_start_raw` (a `f32`) and maximum slot end hour `day_end_raw`, the WeekView SHALL derive `day_start = day_start_raw.floor()` and `day_end = day_end_raw.ceil()` (both as whole-hour `f32` values) and use these snapped values for: (a) the body height (`(day_end - day_start) * SCALING`), (b) the inclusive lower / exclusive upper bound of the time-column whole-hour label loop, and (c) the height of every day column. Slot placement inside a day column SHALL continue to use absolute `top` / `height` math relative to the snapped `day_start`, so a slot beginning at minute `m` past hour `h` lands at `top = (h + m/60 - day_start) * SCALING`. The day-column height MUST equal the body height exactly (no extra `+ SCALING / 2.0` buffer).

#### Scenario: Body, time column, and day columns share the same height for whole-hour bounds
- **WHEN** the week grid renders for a plan whose earliest slot starts at 09:00 and whose latest slot ends at 18:00
- **THEN** the body height, the time-column inline `height`, and every day-column inline `height` SHALL each equal `9 * SCALING` pixels

#### Scenario: Fractional end hour is rounded up so the labeled hour fits
- **WHEN** the week grid renders for a plan whose earliest slot starts at 09:00 and whose latest slot ends at 19:30
- **THEN** the body height SHALL equal `11 * SCALING` pixels (covering 09:00..20:00) AND the time column SHALL render exactly 11 whole-hour labels from `09:00–10:00` through `19:00–20:00` AND the WeekView wrapper SHALL NOT show a vertical scrollbar

#### Scenario: Fractional start hour is rounded down so the leading hour is labeled
- **WHEN** the week grid renders for a plan whose earliest slot starts at 09:30 and whose latest slot ends at 11:30
- **THEN** the body height SHALL equal `3 * SCALING` pixels (covering 09:00..12:00) AND the time column SHALL render exactly 3 whole-hour labels from `09:00–10:00` through `11:00–12:00` AND the 09:30 slot SHALL be positioned at `top = 0.5 * SCALING` pixels inside its day column

#### Scenario: WeekView wrapper does not scroll vertically for fractional bounds
- **WHEN** the week grid renders for any plan whose slot bounds include fractional minutes
- **THEN** the WeekView wrapper element (the `bg-surface border border-border rounded-lg overflow-auto` container) SHALL satisfy `clientHeight >= scrollHeight`
