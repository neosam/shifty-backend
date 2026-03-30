## 1. Extend REST types

- [x] 1.1 Add `ShiftplanTO` struct to `rest-types/src/lib.rs` (id, name, is_planning, deleted, version)
- [x] 1.2 Add `shiftplan_id: Option<Uuid>` field to `SlotTO` in `rest-types/src/lib.rs`

## 2. API functions

- [x] 2.1 Create `get_all_shiftplans(config)` API function in `src/api.rs` (GET /shiftplan)
- [x] 2.2 Add `shiftplan_id` parameter to `get_shiftplan_week` in `src/api.rs` (GET /shiftplan-info/{shiftplan_id}/{year}/{week})
- [x] 2.3 Add `shiftplan_id` parameter to `get_slots` in `src/api.rs` (GET /slot/week/{year}/{week}/{shiftplan_id})

## 3. State and loader

- [x] 3.1 Add `shiftplan_id: Option<Uuid>` to `SlotEditItem` in `src/state/slot_edit.rs`
- [x] 3.2 Update `SlotEditItem::from<SlotTO>` and `SlotTO::from<SlotEditItem>` conversions
- [x] 3.3 Add `shiftplan_id` parameter to `loader::load_shift_plan`
- [x] 3.4 Log warning and filter out slots with `shiftplan_id == None` in loader
- [x] 3.5 Create `load_shiftplan_catalog(config)` loader function

## 4. Tab bar component

- [x] 4.1 Create `ShiftplanTabBar` component in `src/component/` (props: shiftplans, selected_id, on_select)

## 5. Integrate into shiftplan page

- [x] 5.1 Load shiftplan catalog via `use_resource` in `src/page/shiftplan.rs`
- [x] 5.2 Add `Signal<Option<Uuid>>` for active shiftplan tab, auto-select first one
- [x] 5.3 Add tab bar to the page (between navigation and WeekView)
- [x] 5.4 Pass `shiftplan_id` to `load_shift_plan` call
- [x] 5.5 Set `shiftplan_id` from active tab when creating new slots

## 6. Tests

- [x] 6.1 Write tests for new/changed API functions and type conversions
