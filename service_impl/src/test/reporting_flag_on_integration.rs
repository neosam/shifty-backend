//! Stub-Test fuer Flag=on Switch-Integration (REP-03). Wave 2 implementiert.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::test::reporting_phase2_fixtures::{
    fixture_extra_work_entry, fixture_report_range, fixture_sales_person_id,
    fixture_sick_period, fixture_vacation_period, fixture_work_details_8h_mon_fri,
};

#[ignore = "Wave 2: REP-03 flag-on uses AbsencePeriod source"]
#[tokio::test]
async fn test_flag_on_uses_absence_source() {
    // Wave 2 Implementation:
    // 1. MockFeatureFlagService::expect_is_enabled returning Ok(true)
    // 2. MockAbsenceService::expect_derive_hours_for_range returning fixture-derived
    //    BTreeMap (Mo: Vacation 8h, Di: SickLeave 8h, Mi: Vacation 8h)
    // 3. ExtraHours-Mocks liefern noch Vacation/Sick/UnpaidLeave-ExtraHours-Eintraege
    //    ABER diese muessen IGNORIERT werden
    // 4. report.vacation_hours = 16.0 (Mo+Mi aus AbsencePeriod),
    //    report.sick_leave_hours = 8.0 (Di),
    //    report.unpaid_leave_hours = 0.0
    // 5. ExtraWork (Do) bleibt aus ExtraHours-Quelle: report.extra_work = 2.0
    let _ = (
        fixture_sales_person_id(),
        fixture_vacation_period(),
        fixture_sick_period(),
        fixture_work_details_8h_mon_fri(),
        fixture_report_range(),
        fixture_extra_work_entry(),
    );
    unimplemented!("Wave 2 implements REP-03");
}
