//! Wiederverwendbare deterministische Fixtures fuer Phase-2-Reporting-Tests.
//!
//! Quelle: .planning/phases/02-reporting-integration-snapshot-versioning/02-CONTEXT.md
//! Section <specifics> -- Pin-Map-Fixture-Vorschlag.
//!
//! Test-Setup (gemeinsam fuer Wave 1 und Wave 2):
//! - 1 Sales-Person, Vertrag 8h/Tag Mo-Fr, gueltig KW 22-25/2024
//! - Range Mo 2024-06-03 .. So 2024-06-09 (Kalenderwoche 23/2024)
//! - 1 Vacation 2024-06-03..05 (3 Werktage Mo-Mi)
//! - 1 SickLeave 2024-06-04..04 (1 Tag Di — ueberlappt mit Vacation, BUrlG §9-Test)
//! - 1 ExtraWork +2h am Do 2024-06-06

use std::sync::Arc;

use shifty_utils::DayOfWeek;
use time::macros::{date, datetime};
use uuid::Uuid;

use service::absence::{AbsenceCategory, AbsencePeriod};
use service::employee_work_details::EmployeeWorkDetails;
use service::extra_hours::{ExtraHours, ExtraHoursCategory};
use service::sales_person::SalesPerson;

/// Deterministische SalesPerson-Id fuer alle Phase-2-Tests.
#[allow(dead_code)]
pub fn fixture_sales_person_id() -> Uuid {
    Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0001)
}

/// Sales-Person mit Test-Werten ("Test Person", paid).
#[allow(dead_code)]
pub fn fixture_sales_person() -> SalesPerson {
    SalesPerson {
        id: fixture_sales_person_id(),
        name: Arc::from("Test Person"),
        background_color: Arc::from("#000000"),
        is_paid: Some(true),
        inactive: false,
        deleted: None,
        version: Uuid::nil(),
    }
}

/// 8h/Tag, Mo-Fr, gueltig von KW 22/2024 bis KW 25/2024 (deckt 2024-06-03..09).
#[allow(dead_code)]
pub fn fixture_work_details_8h_mon_fri() -> EmployeeWorkDetails {
    EmployeeWorkDetails {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0010),
        sales_person_id: fixture_sales_person_id(),
        expected_hours: 40.0,
        from_day_of_week: DayOfWeek::Monday,
        from_calendar_week: 22,
        from_year: 2024,
        to_day_of_week: DayOfWeek::Sunday,
        to_calendar_week: 25,
        to_year: 2024,
        workdays_per_week: 5,
        is_dynamic: false,
        cap_planned_hours_to_expected: false,
        monday: true,
        tuesday: true,
        wednesday: true,
        thursday: true,
        friday: true,
        saturday: false,
        sunday: false,
        vacation_days: 30,
        created: Some(datetime!(2024 - 01 - 01 10:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Vacation Mo-Mi (3 Werktage) -- 2024-06-03..05.
#[allow(dead_code)]
pub fn fixture_vacation_period() -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0100),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::Vacation,
        from_date: date!(2024 - 06 - 03),
        to_date: date!(2024 - 06 - 05),
        description: Arc::from("fixture vacation"),
        created: Some(datetime!(2024 - 06 - 01 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// SickLeave Di (ueberlappt mit Vacation -> BUrlG §9 Test).
#[allow(dead_code)]
pub fn fixture_sick_period() -> AbsencePeriod {
    AbsencePeriod {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_0200),
        sales_person_id: fixture_sales_person_id(),
        category: AbsenceCategory::SickLeave,
        from_date: date!(2024 - 06 - 04),
        to_date: date!(2024 - 06 - 04),
        description: Arc::from("fixture sick"),
        created: Some(datetime!(2024 - 06 - 04 08:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// ExtraWork +2h am Do 2024-06-06 09:00.
#[allow(dead_code)]
pub fn fixture_extra_work_entry() -> ExtraHours {
    ExtraHours {
        id: Uuid::from_u128(0x0000_0000_0000_0000_0000_0000_0000_1000),
        sales_person_id: fixture_sales_person_id(),
        amount: 2.0,
        category: ExtraHoursCategory::ExtraWork,
        description: Arc::from("fixture extra work"),
        date_time: datetime!(2024 - 06 - 06 09:00:00),
        created: Some(datetime!(2024 - 06 - 06 09:00:00)),
        deleted: None,
        version: Uuid::nil(),
    }
}

/// Report-Range Mo 2024-06-03 bis So 2024-06-09 (Kalenderwoche 23/2024).
#[allow(dead_code)]
pub fn fixture_report_range() -> (time::Date, time::Date) {
    (date!(2024 - 06 - 03), date!(2024 - 06 - 09))
}
