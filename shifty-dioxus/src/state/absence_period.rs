//! Frontend state-type for `AbsencePeriodTO` (Phase 8 Wave 4).
//!
//! Wraps the wire DTO with side-join fields (`person_name`, `background_color`)
//! that the loader resolves from the SalesPerson list. The wire-level type
//! lives in `rest-types`; this module only adds the rendering-time enrichment.
//!
//! See `.planning/phases/08-absence-crud-page-foundation/08-PATTERNS.md` for
//! the analog `Booking` cross-resolve pattern.

use std::sync::Arc;

use rest_types::{AbsenceCategoryTO, AbsencePeriodTO};
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriod {
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategory,
    pub from_date: time::Date,
    pub to_date: time::Date,
    pub description: Arc<str>,
    pub version: Uuid,
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
            person_name: Arc::<str>::from(""),
            background_color: Arc::<str>::from(""),
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
        };
        let state: AbsencePeriod = (&to).into();
        assert_eq!(state.id, id);
        assert_eq!(state.sales_person_id, sp);
        assert_eq!(state.version, v);
        assert_eq!(state.category, AbsenceCategory::Vacation);
        assert_eq!(state.from_date, date!(2026 - 06 - 01));
        assert_eq!(state.to_date, date!(2026 - 06 - 14));
        assert_eq!(state.description.as_ref(), "Italy");
        // Side-join fields default empty — loader fills these.
        assert_eq!(state.person_name.as_ref(), "");
        assert_eq!(state.background_color.as_ref(), "");
    }
}
