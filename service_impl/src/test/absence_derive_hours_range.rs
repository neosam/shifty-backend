//! Stub-Tests fuer `AbsenceService::derive_hours_for_range` (REP-01).
//!
//! Wave 1 implementiert die Bodies; aktuell `#[ignore]` damit `cargo test`
//! gruen bleibt. Der Body ruft die Fixtures aus
//! `crate::test::reporting_phase2_fixtures` auf, damit der Compiler die
//! Imports gegen die Phase-2-Fixtures-Datei waehlt — Wave 1 ergaenzt die
//! eigentlichen Mock-Setups und Assertions.

#![allow(dead_code)]
#![allow(unused_imports)]

use crate::test::reporting_phase2_fixtures::{
    fixture_extra_work_entry, fixture_report_range, fixture_sales_person_id,
    fixture_sick_period, fixture_vacation_period, fixture_work_details_8h_mon_fri,
};

#[ignore = "Wave 1: REP-01 derive_hours_for_range basic case"]
#[tokio::test]
async fn test_derive_hours_for_range_basic() {
    // Wave 1 Implementation:
    // 1. AbsenceDependencies mit MockAbsenceDao + MockSpecialDayService
    //    + MockEmployeeWorkDetailsService aufbauen
    // 2. expect_find_by_sales_person -> Vacation+Sick fixtures
    // 3. expect_find_by_sales_person_id (work_details) -> 8h_mon_fri fixture
    // 4. expect_get_by_week (special_day) -> empty Arc<[]>
    // 5. service.derive_hours_for_range(2024-06-03, 2024-06-09, sales_person_id, Auth, None)
    // 6. Erwartung: BTreeMap mit 5 Werktag-Eintraegen
    //    (Mo+Mi Vacation 8h; Di SickLeave 8h; Do leer; Fr leer)
    // 7. assert_eq! pro Tag auf ResolvedAbsence { category, hours }
    let _ = (
        fixture_sales_person_id(),
        fixture_vacation_period(),
        fixture_sick_period(),
        fixture_work_details_8h_mon_fri(),
        fixture_report_range(),
        fixture_extra_work_entry(),
    );
    unimplemented!("Wave 1 implements REP-01");
}

#[ignore = "Wave 1: REP-01 holiday=0 case"]
#[tokio::test]
async fn test_derive_hours_holiday_is_zero() {
    // Wave 1: SpecialDayService liefert Feiertag fuer 2024-06-04
    //         -> derive_hours fuer 2024-06-04 = ResolvedAbsence { hours: 0.0, category: <prio> }
    unimplemented!("Wave 1 implements REP-01 holiday case");
}

#[ignore = "Wave 1: REP-01 contract change mid-range"]
#[tokio::test]
async fn test_derive_hours_contract_change() {
    // Wave 1: 2 Vertraege (8h/Tag bis KW 23, 4h/Tag ab KW 24)
    //         -> 2024-06-03..05 = 8h/Tag, 2024-06-10..14 = 4h/Tag
    unimplemented!("Wave 1 implements REP-01 contract change");
}
