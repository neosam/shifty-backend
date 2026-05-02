//! Stub-Test fuer Flag=off Bit-Identitaet (REP-02). Wave 2 implementiert.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::test::reporting_phase2_fixtures::{
    fixture_extra_work_entry, fixture_report_range, fixture_sales_person_id,
    fixture_work_details_8h_mon_fri,
};

#[ignore = "Wave 2: REP-02 bit identity flag-off"]
#[tokio::test]
async fn test_flag_off_produces_identical_values_to_pre_phase2() {
    // Wave 2 Implementation:
    // 1. ReportingService-Mocks aufsetzen mit MockFeatureFlagService::expect_is_enabled
    //    returning Ok(false)
    // 2. MockAbsenceService::expect_derive_hours_for_range MUSS .times(0) sein
    //    (darf nicht aufgerufen werden)
    // 3. ExtraHours-Mocks liefern Vacation 8h, SickLeave 4h, UnpaidLeave 2h
    // 4. service.get_report_for_employee_range(...).await
    // 5. Vergleiche NUR report.vacation_hours, sick_leave_hours, unpaid_leave_hours
    //    -- NICHT snapshot_schema_version
    // 6. Erwartet: vacation_hours=8.0, sick_leave_hours=4.0, unpaid_leave_hours=2.0
    //    (identisch zu pre-Phase-2-Verhalten)
    let _ = (
        fixture_sales_person_id(),
        fixture_work_details_8h_mon_fri(),
        fixture_report_range(),
        fixture_extra_work_entry(),
    );
    unimplemented!("Wave 2 implements REP-02");
}
