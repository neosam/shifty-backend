//! Tests fuer `AbsenceService::derive_hours_for_range` (REP-01, Wave 1).
//!
//! Fixtures stammen aus `crate::test::reporting_phase2_fixtures`. Mock-Setup
//! teilt sich `AbsenceDependencies` aus `crate::test::absence` (8 Mock-Deps
//! seit Plan 02-02 Task 1.2).
//!
//! Verifizierte Fixture-Erwartungen (Range Mo 2024-06-03 bis So 2024-06-09,
//! 8h-Vertrag Mo-Fr):
//! - basic: Mo+Mi Vacation 8h, Di SickLeave (BUrlG §9 vor Vacation), Do/Fr/Sa/So leer
//! - holiday: Di als Holiday -> KEIN Eintrag fuer Di (auch nicht SickLeave)
//! - contract-change: 8h-Vertrag KW22-23, 4h-Vertrag KW24-25 -> 06-03 = 8h, 06-10 = 4h

#![allow(dead_code)]

use std::sync::Arc;

use dao::absence::{AbsenceCategoryEntity, AbsencePeriodEntity};
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService, ResolvedAbsence};
use service::permission::Authentication;
use service::special_days::{SpecialDay, SpecialDayType};
use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use uuid::Uuid;

use crate::test::absence::build_dependencies;
use crate::test::reporting_phase2_fixtures::{
    fixture_sales_person_id, fixture_sick_period, fixture_vacation_period,
    fixture_work_details_8h_mon_fri,
};

/// Hilfsfunktion: konvertiert ein Domain-`AbsencePeriod`-Fixture in ein
/// `AbsencePeriodEntity`, damit `MockAbsenceDao::expect_find_by_sales_person`
/// es zurueckliefern kann (der Service-Body ruft das DAO direkt).
fn period_to_entity(period: &AbsencePeriod) -> AbsencePeriodEntity {
    AbsencePeriodEntity {
        id: period.id,
        logical_id: period.id,
        sales_person_id: period.sales_person_id,
        category: match period.category {
            AbsenceCategory::Vacation => AbsenceCategoryEntity::Vacation,
            AbsenceCategory::SickLeave => AbsenceCategoryEntity::SickLeave,
            AbsenceCategory::UnpaidLeave => AbsenceCategoryEntity::UnpaidLeave,
        },
        from_date: period.from_date,
        to_date: period.to_date,
        description: period.description.clone(),
        created: period
            .created
            .unwrap_or(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: period.deleted,
        version: period.version,
    }
}

#[tokio::test]
async fn test_derive_hours_for_range_basic() {
    // Setup: Mock-DAO liefert Vacation (Mo-Mi) + Sick (Di), WorkDetails 8h
    // Mo-Fr, keine Holidays. Range Mo-So.
    let mut deps = build_dependencies();

    let vacation = period_to_entity(&fixture_vacation_period());
    let sick = period_to_entity(&fixture_sick_period());
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![vacation, sick]);
    deps.absence_dao
        .expect_find_by_sales_person()
        .returning(move |_, _| Ok(absence_entities.clone()));

    let work_details = fixture_work_details_8h_mon_fri();
    let work_details_arc: Arc<[_]> = Arc::from(vec![work_details]);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(work_details_arc.clone()));

    deps.special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<SpecialDay>::new())));

    let service = deps.build_service();

    let result = service
        .derive_hours_for_range(
            date!(2024 - 06 - 03),
            date!(2024 - 06 - 09),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // Mo (06-03): Vacation, 8h
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        }),
        "Monday must resolve to Vacation/8h"
    );
    // Di (06-04): SickLeave gewinnt (BUrlG §9), 8h
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
        }),
        "Tuesday must resolve to SickLeave (overrides Vacation per BUrlG §9)"
    );
    // Mi (06-05): Vacation, 8h
    assert_eq!(
        result.get(&date!(2024 - 06 - 05)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        }),
        "Wednesday must resolve to Vacation/8h"
    );
    // Do (06-06): keine Absence aktiv -> kein Eintrag
    assert!(
        result.get(&date!(2024 - 06 - 06)).is_none(),
        "Thursday has no absence"
    );
    // Fr (06-07): keine Absence aktiv -> kein Eintrag
    assert!(
        result.get(&date!(2024 - 06 - 07)).is_none(),
        "Friday has no absence"
    );
    // Sa (06-08): kein Werktag -> kein Eintrag
    assert!(
        result.get(&date!(2024 - 06 - 08)).is_none(),
        "Saturday is not a workday in the contract"
    );
    // So (06-09): kein Werktag -> kein Eintrag
    assert!(
        result.get(&date!(2024 - 06 - 09)).is_none(),
        "Sunday is not a workday in the contract"
    );
    assert_eq!(result.len(), 3, "exactly 3 resolved days expected");
}

#[tokio::test]
async fn test_derive_hours_holiday_is_zero() {
    // Setup: SpecialDayService liefert Di 2024-06-04 als Holiday. Trotz
    // Sick+Vacation am Di darf die Map fuer den Tag KEINEN Eintrag haben
    // (Feiertag verbraucht keine Urlaubs- oder Krankheitsstunden).
    let mut deps = build_dependencies();

    let vacation = period_to_entity(&fixture_vacation_period());
    let sick = period_to_entity(&fixture_sick_period());
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![vacation, sick]);
    deps.absence_dao
        .expect_find_by_sales_person()
        .returning(move |_, _| Ok(absence_entities.clone()));

    let work_details = fixture_work_details_8h_mon_fri();
    let work_details_arc: Arc<[_]> = Arc::from(vec![work_details]);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(work_details_arc.clone()));

    // Holiday Di 2024-06-04 (KW 23 / 2024). 06-03..06-09 ist KW 23.
    let holiday = SpecialDay {
        id: Uuid::from_u128(0xFEEE_0000_0000_0000_0000_0000_0000_0001),
        year: 2024,
        calendar_week: 23,
        day_of_week: DayOfWeek::Tuesday,
        day_type: SpecialDayType::Holiday,
        time_of_day: None,
        created: Some(datetime!(2024 - 01 - 01 00:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };
    let holidays: Arc<[SpecialDay]> = Arc::from(vec![holiday]);
    deps.special_day_service
        .expect_get_by_week()
        .returning(move |_, _, _| Ok(holidays.clone()));

    let service = deps.build_service();

    let result = service
        .derive_hours_for_range(
            date!(2024 - 06 - 03),
            date!(2024 - 06 - 09),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // Di 2024-06-04 = Holiday -> kein Eintrag (auch nicht SickLeave)
    assert!(
        result.get(&date!(2024 - 06 - 04)).is_none(),
        "Tuesday is a holiday — no entry expected"
    );
    // Mo + Mi unveraendert: Vacation 8h
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        }),
        "Monday still Vacation/8h"
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 05)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
        }),
        "Wednesday still Vacation/8h"
    );
    assert_eq!(
        result.len(),
        2,
        "only Monday and Wednesday remain after holiday filter"
    );
}

#[tokio::test]
async fn test_derive_hours_contract_change() {
    // Setup: 2 Vertraege:
    //   - 8h/Tag, Mo-Fr, KW 22/2024 (Mo 2024-05-27) bis KW 23/2024 (So 2024-06-09)
    //   - 4h/Tag, Mo-Fr, KW 24/2024 (Mo 2024-06-10) bis KW 25/2024 (So 2024-06-23)
    // 2 Vacation-Periods:
    //   - 2024-06-03..05 (Mo-Mi der KW 23 -> 8h-Vertrag)
    //   - 2024-06-10..14 (Mo-Fr der KW 24 -> 4h-Vertrag)
    let mut deps = build_dependencies();

    let v1 = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_AAAA_0001),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 05),
        description: Arc::from("v1"),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };
    let v2 = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_AAAA_0002),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 10),
        to_date: date!(2024 - 06 - 14),
        description: Arc::from("v2"),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    };
    let entities: Arc<[AbsencePeriodEntity]> =
        Arc::from(vec![period_to_entity(&v1), period_to_entity(&v2)]);
    deps.absence_dao
        .expect_find_by_sales_person()
        .returning(move |_, _| Ok(entities.clone()));

    let mut wd_8h = fixture_work_details_8h_mon_fri();
    wd_8h.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BBBB_0001);
    wd_8h.from_year = 2024;
    wd_8h.from_calendar_week = 22;
    wd_8h.from_day_of_week = DayOfWeek::Monday;
    wd_8h.to_year = 2024;
    wd_8h.to_calendar_week = 23;
    wd_8h.to_day_of_week = DayOfWeek::Sunday;
    wd_8h.expected_hours = 40.0;
    wd_8h.workdays_per_week = 5;

    let mut wd_4h = fixture_work_details_8h_mon_fri();
    wd_4h.id = Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_BBBB_0002);
    wd_4h.from_year = 2024;
    wd_4h.from_calendar_week = 24;
    wd_4h.from_day_of_week = DayOfWeek::Monday;
    wd_4h.to_year = 2024;
    wd_4h.to_calendar_week = 25;
    wd_4h.to_day_of_week = DayOfWeek::Sunday;
    wd_4h.expected_hours = 20.0;
    wd_4h.workdays_per_week = 5;

    let wd_arc: Arc<[_]> = Arc::from(vec![wd_8h, wd_4h]);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(wd_arc.clone()));

    deps.special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<SpecialDay>::new())));

    let service = deps.build_service();

    let result = service
        .derive_hours_for_range(
            date!(2024 - 06 - 03),
            date!(2024 - 06 - 14),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // KW 23 (8h-Vertrag): Mo-Mi (06-03..05) je 8h Vacation
    for d in [
        date!(2024 - 06 - 03),
        date!(2024 - 06 - 04),
        date!(2024 - 06 - 05),
    ] {
        assert_eq!(
            result.get(&d),
            Some(&ResolvedAbsence {
                category: AbsenceCategory::Vacation,
                hours: 8.0,
            }),
            "{:?} should be Vacation/8h (KW 23, 8h-Vertrag)",
            d
        );
    }
    // KW 24 (4h-Vertrag): Mo-Fr (06-10..14) je 4h Vacation
    for d in [
        date!(2024 - 06 - 10),
        date!(2024 - 06 - 11),
        date!(2024 - 06 - 12),
        date!(2024 - 06 - 13),
        date!(2024 - 06 - 14),
    ] {
        assert_eq!(
            result.get(&d),
            Some(&ResolvedAbsence {
                category: AbsenceCategory::Vacation,
                hours: 4.0,
            }),
            "{:?} should be Vacation/4h (KW 24, 4h-Vertrag)",
            d
        );
    }
    // Do/Fr KW 23 (06-06, 06-07): keine Absence -> kein Eintrag
    assert!(result.get(&date!(2024 - 06 - 06)).is_none());
    assert!(result.get(&date!(2024 - 06 - 07)).is_none());
    assert_eq!(result.len(), 8, "3 days @ 8h + 5 days @ 4h = 8 entries");
}
