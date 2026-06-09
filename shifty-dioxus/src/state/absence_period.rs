//! Frontend state-type for `AbsencePeriodTO` (Phase 8 Wave 4).
//!
//! Wraps the wire DTO with side-join fields (`person_name`, `background_color`)
//! that the loader resolves from the SalesPerson list. The wire-level type
//! lives in `rest-types`; this module only adds the rendering-time enrichment.
//!
//! See `.planning/phases/08-absence-crud-page-foundation/08-PATTERNS.md` for
//! the analog `Booking` cross-resolve pattern.

use std::sync::Arc;

use rest_types::{AbsenceCategoryTO, AbsencePeriodTO, ExtraHoursCategoryTO, ExtraHoursMarkerTO};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbsenceCategory {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

impl From<&AbsenceCategoryTO> for AbsenceCategory {
    fn from(c: &AbsenceCategoryTO) -> Self {
        match c {
            AbsenceCategoryTO::Vacation => Self::Vacation,
            AbsenceCategoryTO::SickLeave => Self::SickLeave,
            AbsenceCategoryTO::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}

impl From<&AbsenceCategory> for AbsenceCategoryTO {
    fn from(c: &AbsenceCategory) -> Self {
        match c {
            AbsenceCategory::Vacation => Self::Vacation,
            AbsenceCategory::SickLeave => Self::SickLeave,
            AbsenceCategory::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}

/// Tageshälfte einer Absence (Phase 8.3, D-02 zweiwertig).
///
/// `Full` ist Default; `Half` halbiert die Soll-Stundenzahl pro Tag in der
/// Reporting-Aggregation (Backend Plan 08.3-04). Frontend-Mirror des
/// Backend-`service::absence::DayFraction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum DayFraction {
    #[default]
    Full,
    Half,
}

impl From<&rest_types::DayFractionTO> for DayFraction {
    fn from(f: &rest_types::DayFractionTO) -> Self {
        match f {
            rest_types::DayFractionTO::Full => Self::Full,
            rest_types::DayFractionTO::Half => Self::Half,
        }
    }
}

impl From<&DayFraction> for rest_types::DayFractionTO {
    fn from(f: &DayFraction) -> Self {
        match f {
            DayFraction::Full => Self::Full,
            DayFraction::Half => Self::Half,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriod {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategory,
    pub from_date: time::Date,
    pub to_date: time::Date,
    pub description: Arc<str>,
    pub version: Uuid,
    pub day_fraction: DayFraction,
    /// Side-join field — populated by the loader from the SalesPerson list.
    /// Empty by default; rendering code should treat empty as "unknown".
    pub person_name: Arc<str>,
    /// Side-join field — populated by the loader from the SalesPerson list.
    pub background_color: Arc<str>,
}

impl From<&AbsencePeriodTO> for AbsencePeriod {
    fn from(t: &AbsencePeriodTO) -> Self {
        Self {
            id: t.id,
            sales_person_id: t.sales_person_id,
            category: (&t.category).into(),
            from_date: t.from_date,
            to_date: t.to_date,
            description: t.description.clone(),
            version: t.version,
            day_fraction: (&t.day_fraction).into(),
            person_name: Arc::<str>::from(""),
            background_color: Arc::<str>::from(""),
        }
    }
}

/// Frontend state-Typ für einen noch nicht konvertierten `extra_hours`-Eintrag
/// (Vacation/SickLeave/UnpaidLeave), der als HR-Projektion inline neben den
/// `AbsencePeriod`-Ranges angezeigt wird.
///
/// Analogon zu `AbsencePeriod` — der loader befüllt `person_name` aus dem
/// SalesPerson-Join (bei `LoadAll`) bzw. trägt das Backend-Feld direkt über
/// (bei `LoadForSalesPerson`, wo `ExtraHoursMarkerTO.person_name` schon gesetzt ist).
#[derive(Clone, Debug, PartialEq)]
pub struct ExtraHoursMarker {
    pub extra_hours_id: Uuid,
    pub sales_person_id: Uuid,
    pub when: time::Date,
    pub amount: f32,
    /// Kategorie direkt vom Backend — `ExtraHoursCategoryTO` hat `PartialEq + Clone`.
    pub category: ExtraHoursCategoryTO,
    pub description: Arc<str>,
    pub person_name: Arc<str>,
}

impl From<&ExtraHoursMarkerTO> for ExtraHoursMarker {
    fn from(t: &ExtraHoursMarkerTO) -> Self {
        Self {
            extra_hours_id: t.extra_hours_id,
            sales_person_id: t.sales_person_id,
            when: t.when,
            amount: t.amount,
            category: t.category.clone(),
            description: t.description.clone(),
            person_name: t.person_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::date;

    #[test]
    fn absence_category_roundtrip() {
        for c in [
            AbsenceCategory::Vacation,
            AbsenceCategory::SickLeave,
            AbsenceCategory::UnpaidLeave,
        ] {
            let to: AbsenceCategoryTO = (&c).into();
            let back: AbsenceCategory = (&to).into();
            assert_eq!(back, c);
        }
    }

    #[test]
    fn absence_period_from_to_carries_identity_and_dates() {
        let id = Uuid::from_u128(1);
        let sp = Uuid::from_u128(2);
        let v = Uuid::from_u128(3);
        let to = AbsencePeriodTO {
            id,
            sales_person_id: sp,
            category: AbsenceCategoryTO::Vacation,
            from_date: date!(2026 - 06 - 01),
            to_date: date!(2026 - 06 - 14),
            description: Arc::<str>::from("Italy"),
            created: None,
            deleted: None,
            version: v,
            day_fraction: rest_types::DayFractionTO::Full,
        };
        let state: AbsencePeriod = (&to).into();
        assert_eq!(state.id, id);
        assert_eq!(state.sales_person_id, sp);
        assert_eq!(state.version, v);
        assert_eq!(state.category, AbsenceCategory::Vacation);
        assert_eq!(state.from_date, date!(2026 - 06 - 01));
        assert_eq!(state.to_date, date!(2026 - 06 - 14));
        assert_eq!(state.description.as_ref(), "Italy");
        assert_eq!(state.day_fraction, DayFraction::Full);
        // Side-join fields default empty — loader fills these.
        assert_eq!(state.person_name.as_ref(), "");
        assert_eq!(state.background_color.as_ref(), "");
    }

    #[test]
    fn absence_period_from_to_carries_half_day_fraction() {
        let to = AbsencePeriodTO {
            id: Uuid::from_u128(10),
            sales_person_id: Uuid::from_u128(20),
            category: AbsenceCategoryTO::Vacation,
            from_date: date!(2026 - 12 - 24),
            to_date: date!(2026 - 12 - 24),
            description: Arc::<str>::from("Heiligabend"),
            created: None,
            deleted: None,
            version: Uuid::from_u128(30),
            day_fraction: rest_types::DayFractionTO::Half,
        };
        let state: AbsencePeriod = (&to).into();
        assert_eq!(state.day_fraction, DayFraction::Half);
    }
}
