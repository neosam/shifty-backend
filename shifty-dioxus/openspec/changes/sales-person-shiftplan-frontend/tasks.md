## 1. API Layer

- [x] 1.1 Add `get_shiftplan_assignments(config, sales_person_id) -> Vec<Uuid>` to `src/api.rs` (GET `/sales-person-shiftplan/{id}/shiftplans`)
- [x] 1.2 Add `set_shiftplan_assignments(config, sales_person_id, shiftplan_ids)` to `src/api.rs` (PUT `/sales-person-shiftplan/{id}/shiftplans`)
- [x] 1.3 Add `get_bookable_sales_persons(config, shiftplan_id) -> Vec<SalesPersonTO>` to `src/api.rs` (GET `/sales-person-shiftplan/by-shiftplan/{shiftplan_id}`)
- [x] 1.4 Add proxy entry for `/sales-person-shiftplan` in `Dioxus.toml` if not already covered

## 2. i18n

- [x] 2.1 Add i18n keys to `src/i18n/mod.rs`: ShiftplanAssignments, ShiftplanAssignmentsInfo (permissive model explanation), BookingForbidden
- [x] 2.2 Add English translations in `src/i18n/en.rs`
- [x] 2.3 Add German translations in `src/i18n/de.rs`
- [x] 2.4 Add Czech translations in `src/i18n/cs.rs`

## 3. State & Service for Assignment Management

- [x] 3.1 Extend `SelectedSalesPerson` in `src/state/user_management.rs` with `shiftplan_assignments: Vec<Uuid>` field
- [x] 3.2 Add `UserManagementAction` variants: `LoadShiftplanCatalog`, `UpdateShiftplanAssignments(Vec<Uuid>)`, `LoadShiftplanAssignments(Uuid)`
- [x] 3.3 Implement action handlers in `src/service/user_management.rs` — load catalog, load assignments, update local state
- [x] 3.4 Extend the `SaveSalesPerson` / `SaveSalesPersonAndNavigate` handlers to also save shiftplan assignments after saving the sales person

## 4. Shiftplan Assignment UI Component

- [x] 4.1 Add shiftplan catalog state to `UserManagementStore` (e.g., `shiftplan_catalog: Rc<[ShiftplanTO]>`)
- [x] 4.2 Create shiftplan assignment section in `src/page/sales_person_details.rs` — checkbox list with info message, between Settings and action buttons
- [x] 4.3 Wire checkbox changes to `UserManagementAction::UpdateShiftplanAssignments`
- [x] 4.4 Load catalog and assignments on page load (extend use_effect)

## 5. Booking Eligibility Filter

- [x] 5.1 Add loader function `load_bookable_sales_persons(config, shiftplan_id)` in `src/loader.rs`
- [x] 5.2 Replace the all-sales-persons list in the shiftplan booking dropdown with `get_bookable_sales_persons` filtered by current shiftplan ID
- [x] 5.3 Handle 403 Forbidden response on booking creation — display user-friendly error message using i18n key

## 6. Testing & Verification

- [x] 6.1 Verify `cargo check` passes with all changes
- [x] 6.2 Verify `cargo clippy` passes
- [x] 6.3 Verify `cargo test` passes (if applicable frontend tests exist)
