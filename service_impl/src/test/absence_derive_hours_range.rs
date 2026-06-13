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

    // Vertrag: expected=40, workdays_per_week=5, verfügbar Mo-Fr (Felder
    // stimmen überein) → keine Wochen-Deckelung greift (3 < 5 Tage), days=1.0
    // pro Tag, hours = 40/5 = 8h.
    // Mo (06-03): Vacation, 8h, 1 Tag
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        }),
        "Monday must resolve to Vacation/8h/1 day"
    );
    // Di (06-04): SickLeave gewinnt (BUrlG §9), 8h, 1 Tag
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::SickLeave,
            hours: 8.0,
            days: 1.0,
        }),
        "Tuesday must resolve to SickLeave (overrides Vacation per BUrlG §9)"
    );
    // Mi (06-05): Vacation, 8h, 1 Tag
    assert_eq!(
        result.get(&date!(2024 - 06 - 05)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        }),
        "Wednesday must resolve to Vacation/8h/1 day"
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
    // Mo + Mi unveraendert: Vacation 8h, 1 Tag
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        }),
        "Monday still Vacation/8h/1 day"
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 05)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 8.0,
            days: 1.0,
        }),
        "Wednesday still Vacation/8h/1 day"
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
                days: 1.0,
            }),
            "{:?} should be Vacation/8h/1 day (KW 23, 8h-Vertrag)",
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
                days: 1.0,
            }),
            "{:?} should be Vacation/4h/1 day (KW 24, 4h-Vertrag)",
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
            days: 0.5,
        }),
        "Half-day vacation must produce hours = hours_per_day * 0.5, days = 0.5"
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
            days: 0.5,
        }),
        "Monday must be halved to 4h / 0.5 day"
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 4.0,
            days: 0.5,
        }),
        "Tuesday must also be halved to 4h / 0.5 day (uniform per-day halving)"
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
            days: 0.5,
        }),
        "Half-day SickLeave must produce hours = hours_per_day * 0.5, days = 0.5 \
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
    // Di workday, kein Holiday → Vacation 5h / 1 Tag.
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        })
    );
    // Mi workday, ABER Holiday → kein Eintrag.
    assert!(
        !result.contains_key(&date!(2024 - 06 - 05)),
        "Mi ist Holiday — derive_hours_for_range skipt den Tag, obwohl die \
         Vacation-Period ihn umfasst (Mismatch zur lump-sum-Heuristik)"
    );
    // Do/Fr workdays, kein Holiday → Vacation 5h / 1 Tag.
    assert_eq!(
        result.get(&date!(2024 - 06 - 06)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        })
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 07)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
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

// ----------------------------------------------------------------------------
// REPRODUCER: vacation-hours-overcounted
//
// User-Report: 10h-Wochenvertrag, Urlaub eine ganze Woche (Mo-So) → erwartet
// werden die Wochenstunden (10h), berechnet wurden ABER viel zu viele.
//
// Domänenmodell (vom User bestätigt): `workdays_per_week` ist die maßgebliche
// Zahl der Arbeitstage pro Woche; die angehakten Wochentag-Booleans sind nur
// Verfügbarkeit. `derive_hours_for_range` deckelt daher pro ISO-Woche auf
// `workdays_per_week` Tage und zählt je `hours_per_day = expected_hours /
// workdays_per_week`.
//
// Hier: expected_hours=10, workdays_per_week=5, alle 7 Tage verfügbar. Die
// Wochen-Deckelung lässt nur 5 Tage zählen (Mo-Fr, je 10/5 = 2h), Sa+So
// bekommen keinen Eintrag mehr → Summe = exakt 10h (nicht 14h wie ohne
// Deckelung).
// ----------------------------------------------------------------------------
#[tokio::test]
async fn test_repro_weekly_contract_overcounts_when_workdays_per_week_diverges_from_booleans() {
    let mut deps = build_dependencies();

    // Vacation Mo-So der KW 23/2024 (eine volle Kalenderwoche).
    let vacation = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_CCCC_0001),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 09),
        description: Arc::from("full-week vacation"),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
        day_fraction: DayFraction::Full,
    };
    let absence_entities: Arc<[AbsencePeriodEntity]> = Arc::from(vec![period_to_entity(&vacation)]);
    deps.absence_dao
        .expect_find_by_sales_person()
        .returning(move |_, _| Ok(absence_entities.clone()));

    // 10h-Wochenvertrag. workdays_per_week = 5, ABER alle 7 Wochentag-Booleans
    // sind true (divergierende Konfiguration, wie sie das Formular zulässt).
    let mut wd = fixture_work_details_8h_mon_fri();
    wd.expected_hours = 10.0;
    wd.workdays_per_week = 5;
    wd.monday = true;
    wd.tuesday = true;
    wd.wednesday = true;
    wd.thursday = true;
    wd.friday = true;
    wd.saturday = true;
    wd.sunday = true;
    let wd_arc: Arc<[_]> = Arc::from(vec![wd]);
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
            date!(2024 - 06 - 09),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // Wochen-Deckelung: genau 5 Tage (Mo-Fr) zählen, je 2h / 1 Tag; Sa+So
    // (über workdays_per_week=5 hinaus) bekommen keinen Eintrag.
    for d in [
        date!(2024 - 06 - 03),
        date!(2024 - 06 - 04),
        date!(2024 - 06 - 05),
        date!(2024 - 06 - 06),
        date!(2024 - 06 - 07),
    ] {
        assert_eq!(
            result.get(&d),
            Some(&ResolvedAbsence {
                category: AbsenceCategory::Vacation,
                hours: 2.0,
                days: 1.0,
            }),
            "{:?} muss Vacation/2h/1 Tag sein (10/5)",
            d
        );
    }
    assert!(
        !result.contains_key(&date!(2024 - 06 - 08)),
        "Sa über Wochen-Deckelung (5 Tage) hinaus → kein Eintrag"
    );
    assert!(
        !result.contains_key(&date!(2024 - 06 - 09)),
        "So über Wochen-Deckelung (5 Tage) hinaus → kein Eintrag"
    );
    assert_eq!(result.len(), 5, "genau 5 gedeckelte Tage (Mo-Fr)");

    let total: f32 = result.values().map(|r| r.hours).sum();
    let total_days: f32 = result.values().map(|r| r.days).sum();

    // Wochen-Deckelung auf workdays_per_week=5 → Summe = expected_hours = 10h,
    // 5 Tage; egal wie viele Wochentag-Booleans gesetzt sind.
    assert!(
        (total - 10.0).abs() < 0.01,
        "Eine volle Urlaubswoche darf nie mehr als die Wochenstunden \
         (10h) ergeben, egal wie workdays_per_week vs. Booleans gesetzt \
         sind. Beobachtet: {total}h"
    );
    assert!(
        (total_days - 5.0).abs() < 0.01,
        "Eine volle Urlaubswoche ergibt genau workdays_per_week=5 Tage. \
         Beobachtet: {total_days}"
    );
}

// ----------------------------------------------------------------------------
// USER-SZENARIO (vom User bestätigt): Vertrag expected_hours=10,
// workdays_per_week=2, verfügbar Mo-So.
//   - 1 Tag Urlaub  → 1 Tag, 5h
//   - Mo-Mi (3 verfügbare Tage) → gedeckelt auf 2 Tage, 10h (Mi kein Eintrag)
//   - volle Woche Mo-So → 2 Tage, 10h
// hours_per_day = 10 / 2 = 5h.
// ----------------------------------------------------------------------------

/// 10h-Wochenvertrag, workdays_per_week=2, verfügbar Mo-So. hours_per_day = 5h.
fn wd_10h_2workdays_all_available() -> service::employee_work_details::EmployeeWorkDetails {
    let mut wd = fixture_work_details_8h_mon_fri();
    wd.expected_hours = 10.0;
    wd.workdays_per_week = 2;
    wd.monday = true;
    wd.tuesday = true;
    wd.wednesday = true;
    wd.thursday = true;
    wd.friday = true;
    wd.saturday = true;
    wd.sunday = true;
    wd
}

async fn run_vacation_range(
    from: time::Date,
    to: time::Date,
) -> std::collections::BTreeMap<time::Date, ResolvedAbsence> {
    let mut deps = build_dependencies();

    let vacation = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_EEEE_0001),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: from,
        to_date: to,
        description: Arc::from("user-scenario vacation"),
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

    let wd_arc: Arc<[_]> = Arc::from(vec![wd_10h_2workdays_all_available()]);
    deps.employee_work_details_service
        .expect_find_by_sales_person_id()
        .returning(move |_, _, _| Ok(wd_arc.clone()));

    deps.special_day_service
        .expect_get_by_week()
        .returning(|_, _, _| Ok(Arc::from(Vec::<SpecialDay>::new())));

    let service = deps.build_service();
    service
        .derive_hours_for_range(from, to, fixture_sales_person_id(), Authentication::Full, None)
        .await
        .expect("derive_hours_for_range should succeed")
}

#[tokio::test]
async fn test_user_scenario_single_day_is_five_hours() {
    // 1 verfügbarer Tag (Mo 2024-06-03) → 1 Tag, 5h.
    let result = run_vacation_range(date!(2024 - 06 - 03), date!(2024 - 06 - 03)).await;
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        }),
        "1 Tag Urlaub = 1 Tag / 5h"
    );
    assert_eq!(result.len(), 1);
}

#[tokio::test]
async fn test_user_scenario_mon_to_wed_caps_at_two_days_ten_hours() {
    // Mo-Mi (3 verfügbare Tage) → gedeckelt auf 2 Tage, 10h; Mi kein Eintrag.
    let result = run_vacation_range(date!(2024 - 06 - 03), date!(2024 - 06 - 05)).await;
    assert_eq!(
        result.get(&date!(2024 - 06 - 03)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        }),
        "Mo zählt (1. von 2 Workdays)"
    );
    assert_eq!(
        result.get(&date!(2024 - 06 - 04)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        }),
        "Di zählt (2. von 2 Workdays)"
    );
    assert!(
        !result.contains_key(&date!(2024 - 06 - 05)),
        "Mi über Wochen-Deckelung (workdays_per_week=2) → kein Eintrag"
    );
    assert_eq!(result.len(), 2);
    let total: f32 = result.values().map(|r| r.hours).sum();
    let total_days: f32 = result.values().map(|r| r.days).sum();
    assert!((total - 10.0).abs() < 0.01, "Mo-Mi = 10h (gedeckelt), got {total}");
    assert!((total_days - 2.0).abs() < 0.01, "Mo-Mi = 2 Tage (gedeckelt)");
}

#[tokio::test]
async fn test_user_scenario_full_week_is_two_days_ten_hours() {
    // volle Woche Mo-So → 2 Tage, 10h (gedeckelt auf workdays_per_week=2).
    let result = run_vacation_range(date!(2024 - 06 - 03), date!(2024 - 06 - 09)).await;
    assert_eq!(result.len(), 2, "nur 2 gezählte Tage trotz 7 verfügbarer Tage");
    let total: f32 = result.values().map(|r| r.hours).sum();
    let total_days: f32 = result.values().map(|r| r.days).sum();
    assert!((total - 10.0).abs() < 0.01, "volle Woche = 10h, got {total}");
    assert!((total_days - 2.0).abs() < 0.01, "volle Woche = 2 Tage");
}

#[tokio::test]
async fn test_user_scenario_three_full_weeks_accumulate_per_week() {
    // 3 volle Wochen Mo 2024-06-03 .. So 2024-06-23 → 6 Tage, 30h
    // (pro Woche gedeckelt auf 2 Tage / 10h).
    let result = run_vacation_range(date!(2024 - 06 - 03), date!(2024 - 06 - 23)).await;
    assert_eq!(result.len(), 6, "3 Wochen × 2 gedeckelte Tage = 6 Einträge");
    let total: f32 = result.values().map(|r| r.hours).sum();
    let total_days: f32 = result.values().map(|r| r.days).sum();
    assert!((total - 30.0).abs() < 0.01, "3 Wochen = 30h, got {total}");
    assert!((total_days - 6.0).abs() < 0.01, "3 Wochen = 6 Tage");
}

// ----------------------------------------------------------------------------
// REGRESSION (vacation-hours-overcounted, human-verify follow-up):
// Multi-week + partial-week Absicherung — REALES User-Szenario.
//
// User-Klarstellung: 10h-Wochenvertrag mit workdays_per_week=2 (Mo+Di
// verfügbar) ergibt pro voller Urlaubswoche genau 10h (= 2 Tage × 5h).
// hours_per_day = expected_hours / workdays_per_week = 10 / 2 = 5h; die
// Wochen-Deckelung auf workdays_per_week=2 zählt höchstens 2 Tage/Woche.
//
// Diese Tests sperren das korrekte Verhalten über mehrere Wochen und über eine
// mitten in der Woche endende (partielle) Range ein.
// ----------------------------------------------------------------------------

/// Baut einen 10h-Wochenvertrag mit workdays_per_week=2 und genau 2 verfügbaren
/// Tagen (Mo+Di). Gültig KW22-25/2024 (geerbt aus fixture_work_details_8h_mon_fri).
/// hours_per_day = 10 / 2 = 5h.
fn wd_10h_mon_tue_divergent() -> service::employee_work_details::EmployeeWorkDetails {
    let mut wd = fixture_work_details_8h_mon_fri();
    wd.expected_hours = 10.0;
    wd.workdays_per_week = 2;
    wd.monday = true;
    wd.tuesday = true;
    wd.wednesday = false;
    wd.thursday = false;
    wd.friday = false;
    wd.saturday = false;
    wd.sunday = false;
    wd
}

#[tokio::test]
async fn test_multi_week_vacation_counts_only_active_days_per_week() {
    // Urlaub über 3 volle Kalenderwochen: Mo 2024-06-03 (KW23) bis So
    // 2024-06-23 (KW25) = 21 Kalendertage. Vertrag: 10h/Woche,
    // workdays_per_week=2, verfügbar Mo+Di. hours_per_day = 10/2 = 5h.
    //
    // Erwartung: pro Woche zählen NUR Mo+Di (je 5h) = 10h/Woche.
    // Aktive Tage: 06-03, 06-04, 06-10, 06-11, 06-17, 06-18 = 6 Einträge.
    // Gesamtsumme = 3 × 10h = 30h. Niemals mehr als 2 Tage/Woche oder 10h/Woche.
    let mut deps = build_dependencies();

    let vacation = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_CCCC_0002),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 23),
        description: Arc::from("three-week vacation"),
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

    let wd_arc: Arc<[_]> = Arc::from(vec![wd_10h_mon_tue_divergent()]);
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
            date!(2024 - 06 - 23),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // Genau die 6 aktiven Tage (Mo+Di der 3 Wochen), je 5h Vacation.
    let active_days = [
        date!(2024 - 06 - 03), // KW23 Mo
        date!(2024 - 06 - 04), // KW23 Di
        date!(2024 - 06 - 10), // KW24 Mo
        date!(2024 - 06 - 11), // KW24 Di
        date!(2024 - 06 - 17), // KW25 Mo
        date!(2024 - 06 - 18), // KW25 Di
    ];
    for d in active_days {
        assert_eq!(
            result.get(&d),
            Some(&ResolvedAbsence {
                category: AbsenceCategory::Vacation,
                hours: 5.0,
                days: 1.0,
            }),
            "{:?} muss Vacation/5h/1 Tag sein (10h / workdays_per_week=2)",
            d
        );
    }
    assert_eq!(
        result.len(),
        6,
        "exakt 6 aktive Tage über 3 Wochen (2 Tage/Woche), keine Wochenend-/inaktiven Tage"
    );

    // Pro-Woche-Summe: je genau 10h, nie mehr.
    let week_sum = |days: &[time::Date]| -> f32 {
        days.iter()
            .filter_map(|d| result.get(d))
            .map(|r| r.hours)
            .sum()
    };
    for (label, days) in [
        ("KW23", &[date!(2024 - 06 - 03), date!(2024 - 06 - 04)][..]),
        ("KW24", &[date!(2024 - 06 - 10), date!(2024 - 06 - 11)][..]),
        ("KW25", &[date!(2024 - 06 - 17), date!(2024 - 06 - 18)][..]),
    ] {
        let s = week_sum(days);
        assert!(
            (s - 10.0).abs() < 0.01,
            "{label} muss exakt 10h ergeben (2 aktive Tage × 5h), nie mehr. Beobachtet: {s}h"
        );
    }

    // Gesamtsumme = 30h.
    let total: f32 = result.values().map(|r| r.hours).sum();
    assert!(
        (total - 30.0).abs() < 0.01,
        "3 volle Urlaubswochen müssen 3 × 10h = 30h ergeben, niemals mehr \
         (Wochen-Deckelung auf workdays_per_week=2). Beobachtet: {total}h"
    );
}

#[tokio::test]
async fn test_multi_week_vacation_partial_trailing_week_counts_only_its_active_days() {
    // Partial-week edge: Range endet MITTEN in der dritten Woche, auf Montag
    // 2024-06-17 (KW25 Mo). Damit umfasst die abschließende (partielle) Woche
    // KW25 nur den Montag — NICHT den Dienstag — also genau 1 aktiven Tag.
    //
    // Erwartung:
    //   KW23 (voll):    Mo+Di = 10h
    //   KW24 (voll):    Mo+Di = 10h
    //   KW25 (partiell): nur Mo = 5h  (Di liegt außerhalb der Range)
    //   Gesamt = 25h.
    // Die partielle Woche darf NICHT als volle Woche (10h) gezählt werden.
    let mut deps = build_dependencies();

    let vacation = AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_CCCC_0003),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 17),
        description: Arc::from("two-and-partial-week vacation"),
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

    let wd_arc: Arc<[_]> = Arc::from(vec![wd_10h_mon_tue_divergent()]);
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
            date!(2024 - 06 - 17),
            fixture_sales_person_id(),
            Authentication::Full,
            None,
        )
        .await
        .expect("derive_hours_for_range should succeed");

    // KW23 + KW24: Mo+Di je 5h.
    for d in [
        date!(2024 - 06 - 03),
        date!(2024 - 06 - 04),
        date!(2024 - 06 - 10),
        date!(2024 - 06 - 11),
    ] {
        assert_eq!(
            result.get(&d),
            Some(&ResolvedAbsence {
                category: AbsenceCategory::Vacation,
                hours: 5.0,
                days: 1.0,
            }),
            "{:?} muss Vacation/5h/1 Tag sein",
            d
        );
    }
    // KW25 partiell: nur Mo 06-17 aktiv (5h).
    assert_eq!(
        result.get(&date!(2024 - 06 - 17)),
        Some(&ResolvedAbsence {
            category: AbsenceCategory::Vacation,
            hours: 5.0,
            days: 1.0,
        }),
        "KW25 Mo muss als einziger aktiver Tag der partiellen Woche 5h sein"
    );
    // Di KW25 (06-18) liegt außerhalb der Range → kein Eintrag.
    assert!(
        !result.contains_key(&date!(2024 - 06 - 18)),
        "Di KW25 liegt außerhalb der Range — kein Eintrag (partielle Woche)"
    );

    assert_eq!(
        result.len(),
        5,
        "2 volle Wochen (4 Tage) + 1 partielle Woche (1 Tag) = 5 Einträge"
    );

    let total: f32 = result.values().map(|r| r.hours).sum();
    assert!(
        (total - 25.0).abs() < 0.01,
        "Die abschließende partielle Woche darf nur ihre aktiven Tage zählen \
         (KW25 = nur Mo = 5h), nicht eine volle Woche. Erwartet 10+10+5 = 25h. \
         Beobachtet: {total}h"
    );
}
