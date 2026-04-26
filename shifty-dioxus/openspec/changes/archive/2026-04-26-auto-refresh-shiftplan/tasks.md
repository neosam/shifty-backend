## 1. Add Global Refresh Signal

- [x] 1.1 Add `SHIFTPLAN_REFRESH: GlobalSignal<u64>` to `src/service/slot_edit.rs`
- [x] 1.2 Create a helper function `trigger_shiftplan_refresh()` that increments the counter

## 2. Trigger Refresh on Mutations

- [x] 2.1 Call `trigger_shiftplan_refresh()` in `save_slot_edit()` after successful save (both edit and new)
- [x] 2.2 Call `trigger_shiftplan_refresh()` in `delete_slot_edit()` after successful delete

## 3. React to Refresh in Shiftplan Page

- [x] 3.1 Read `SHIFTPLAN_REFRESH` inside the `shift_plan_context` `use_resource` closure in `src/page/shiftplan.rs` to register it as a reactive dependency

## 4. Testing

- [ ] 4.1 Verify new slot appears immediately in week view after creation
- [ ] 4.2 Verify edited slot updates immediately in week view after save
- [ ] 4.3 Verify deleted slot disappears immediately from week view
- [ ] 4.4 Verify cancel does not trigger a reload
