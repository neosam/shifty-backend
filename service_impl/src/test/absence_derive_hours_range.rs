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

use dao::absence::{AbsenceCategoryEntity, AbsencePeriodEntity, DayFractionEntity};
use service::absence::{AbsenceCategory, AbsencePeriod, AbsenceService, DayFraction, ResolvedAbsence};
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
        day_fraction: match period.day_fraction {
            DayFraction::Full => DayFractionEntity::Full,
            DayFraction::Half => DayFractionEntity::Half,
        },
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
        !result.contains_key(&date!(2024 - 06 - 06)),
        "Thursday has no absence"
    );
    // Fr (06-07): keine Absence aktiv -> kein Eintrag
    assert!(
        !result.contains_key(&date!(2024 - 06 - 07)),
        "Friday has no absence"
    );
    // Sa (06-08): kein Werktag -> kein Eintrag
    assert!(
        !result.contains_key(&date!(2024 - 06 - 08)),
        "Saturday is not a workday in the contract"
    );
    // So (06-09): kein Werktag -> kein Eintrag
    assert!(
        !result.contains_key(&date!(2024 - 06 - 09)),
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
        !result.contains_key(&date!(2024 - 06 - 04)),
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
        day_fraction: DayFraction::Full,
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
        day_fraction: DayFraction::Full,
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
    assert!(!result.contains_key(&date!(2024 - 06 - 06)));
    assert!(!result.contains_key(&date!(2024 - 06 - 07)));
    assert_eq!(result.len(), 8, "3 days @ 8h + 5 days @ 4h = 8 entries");
}

// ---------- Phase 8.3 (Plan 04) — Halbtag-Tests ----------
//
// Verifizieren die D-08.3-04-Entscheidung (CONTEXT.md OQ-4): wenn die
// dominante Absence `day_fraction = Half` traegt, halbiert
// `derive_hours_for_range` die effektive Soll-Stundenzahl uniform fuer ALLE
// Tage der Range. Halbierung ist kategorie-unabhaengig (Vacation/SickLeave/
// UnpaidLeave). Aufruferseite (reporting.rs) bleibt unveraendert; die
// halbierten Stunden propagieren automatisch in vacation_hours / vacation_days
// / BillingPeriod-Aggregation, was den Snapshot-Bump 3 -> 4 begruendet.

#[tokio::test]
async fn test_derive_hours_half_day_single_full_day_contract() {
    // Vacation 1 Tag (Mo 2024-06-03) + day_fraction = Half + 8h-Vertrag
    // → 4h erwartet (D-08.3-04).
    let mut deps = build_dependencies();

    let mut vac_entity = period_to_entity(&fixture_vacation_period());
    vac_entity.from_date = date!(2024 - 06 - 03);
    vac_entity.to_date = date!(2024 - 06 - 03);
    vac_entity.day_fraction = DayFractionEntity::Half;
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![vac_entity]);
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
            date!(2024 - 06 - 03),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 4.0,
        }),
        "Half-day vacation must produce hours = contract_hours * 0.5"
    );
    assert_eq!(result.len(), 1, "exactly 1 resolved day expected");
}

#[tokio::test]
async fn test_derive_hours_half_day_two_day_range_uniform_halving() {
    // Vacation Mo+Di + day_fraction = Half + 8h-Vertrag.
    // Erwartung: jeder Tag 4h, Gesamtsumme 8h (D-08.3-04 uniform halving).
    let mut deps = build_dependencies();

    let mut vac_entity = period_to_entity(&fixture_vacation_period());
    vac_entity.from_date = date!(2024 - 06 - 03); // Monday
    vac_entity.to_date = date!(2024 - 06 - 04); // Tuesday
    vac_entity.day_fraction = DayFractionEntity::Half;
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![vac_entity]);
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
            date!(2024 - 06 - 04),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 4.0,
        }),
        "Monday must be halved to 4h"
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 4.0,
        }),
        "Tuesday must also be halved to 4h (uniform per-day halving)"
    );

    let total_hours: f32 = result.values().map(|a| a.hours).sum();
    assert!(
        (total_hours - 8.0).abs() < 1e-6,
        "Sum across 2-day half-range must be 8h, got {}",
        total_hours
    );
}

#[tokio::test]
async fn test_derive_hours_half_day_sick_leave_also_halved() {
    // Verifiziert dass die Halbierung kategorie-unabhaengig in
    // derive_hours_for_range greift — nicht hardcoded auf Vacation.
    // SickLeave + Half + 8h-Vertrag → 4h.
    let mut deps = build_dependencies();

    let mut sick_entity = period_to_entity(&fixture_sick_period());
    sick_entity.from_date = date!(2024 - 06 - 05); // Wednesday
    sick_entity.to_date = date!(2024 - 06 - 05);
    sick_entity.day_fraction = DayFractionEntity::Half;
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![sick_entity]);
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
            date!(2024 - 06 - 05),
            date!(2024 - 06 - 05),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    assert_eq!(
        result.get(&date!(2024 - 06 - 05)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 4.0,
        }),
        "Half-day SickLeave must produce hours = contract_hours * 0.5 \
         (category-agnostic halving)"
    );
}

// ----------------------------------------------------------------------------
// REPRODUCER: Cutover-Drift bug class — `derive_hours_for_range` vs.
// `detect_weekly_lump_sum` divergence on holidays.
//
// User-Convention: "Feiertage kommen oben drauf" — 20h Vacation-Eintrag bei
// 20h-Vertrag bleibt 20h, auch wenn ein Workday in der Woche Feiertag ist.
//
// Code-Realität (this test pins the current behavior):
//   - `detect_weekly_lump_sum` (in cutover.rs) iteriert Mo-So, summiert
//     `contract.hours_per_day()` für jeden Workday → 4 × 5h = 20h Target.
//     Holidays werden NICHT abgezogen. → Heuristik akzeptiert 20h, migriert
//     als `absence_period(Mo, So)`.
//   - `derive_hours_for_range` (this function) skipt Holidays → 3 × 5h = 15h.
//
// Result in `compute_gate`:
//   legacy_sum = 20h (from extra_hours row)
//   derived_sum = 15h (from absence_period via derive)
//   drift = 5h → erscheint im Drift-Report
//
// Dieser Test fixt das aktuelle Verhalten von `derive_hours_for_range` und
// dient als executable specification: solange dieser Test mit `derived = 15h`
// passt UND die Heuristik in cutover.rs Holidays ignoriert, entsteht ein
// systematischer Drift bei Wochenpauschalen mit Feiertag.
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_lump_sum_vacation_period_with_holiday_emits_short_derived_sum() {
    // Setup matching the live INT scenario:
    //   - 20h/Woche-Vertrag, Workdays Di-Fr → hours_per_day = 20 / 4 = 5h
    //   - 1× extra_hours-Row mit 20h auf Mo 2024-06-03 (→ Heuristik-Match)
    //   - Migration würde absence_period(Mo 06-03, So 06-09) schreiben
    //   - Holiday auf Mi 2024-06-05 (Fronleichnam-ähnlich, Workday)
    let mut deps = build_dependencies();

    // Vacation-Period Mo-So der KW 23/2024 (so wie die Heuristik sie schreiben
    // würde, wenn die Quelle ein 20h-Eintrag auf Mo gewesen wäre).
    let vacation = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_DDDD_0001),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 09),
        description: Arc::from("lump-sum migrated vacation"),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    };
    let absence_entities: Arc<[AbsencePeriodEntity]> =
        Arc::from(vec![period_to_entity(&vacation)]);
    deps.absence_dao
        .expect_find_by_sales_person()
        .returning(move |_, _| Ok(absence_entities.clone()));

    // 20h-Vertrag, Workdays Di-Fr (Mo + Sa + So = non-workday).
    let mut wd = fixture_work_details_8h_mon_fri();
    wd.expected_hours = 20.0;
    wd.workdays_per_week = 4;
    wd.monday = false;
    wd.tuesday = true;
    wd.wednesday = true;
    wd.thursday = true;
    wd.friday = true;
    wd.saturday = false;
    wd.sunday = false;
    let wd_arc: Arc<[_]> = Arc::from(vec![wd]);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(wd_arc.clone()));

    // Holiday auf Mi 2024-06-05.
    let holiday = SpecialDay {
        id: Uuid::from_u128(0xFEEE_0000_0000_0000_0000_0000_DDDD_0001),
        year: 2024,
        calendar_week: 23,
        day_of_week: DayOfWeek::Wednesday,
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

    // Mo non-workday → kein Eintrag.
    assert!(!result.contains_key(&date!(2024 - 06 - 03)));
    // Di workday, kein Holiday → Vacation 5h.
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
        })
    );
    // Mi workday, ABER Holiday → kein Eintrag.
    assert!(
        !result.contains_key(&date!(2024 - 06 - 05)),
        "Mi ist Holiday — derive_hours_for_range skipt den Tag, obwohl die \
         Vacation-Period ihn umfasst (Mismatch zur lump-sum-Heuristik)"
    );
    // Do/Fr workdays, kein Holiday → Vacation 5h.
    assert_eq!(
        result.get(&date!(2024 - 06 - 06)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
        })
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 07)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
        })
    );

    // Aggregierte Drift-Diagnose: derived_sum sollte 15h sein, während die
    // Heuristik im Cutover-Pfad die row mit target_sum = 4 × 5 = 20h
    // akzeptieren würde. Daraus folgt drift = 5h in compute_gate.
    let derived_sum: f32 = result
        .values()
        .filter(|r| r.category == AbsenceCategory::Vacation)
        .map(|r| r.hours)
        .sum();
    assert!(
        (derived_sum - 15.0).abs() < 0.01,
        "derived_sum für Mo-So-Vacation mit Holiday auf Mi muss 15h sein \
         (aktuelles Verhalten); Heuristik hätte 20h erwartet → drift = 5h \
         im compute_gate. Beobachtet: {derived_sum}"
    );
}
