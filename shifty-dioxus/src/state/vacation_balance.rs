//! Frontend state-type for `VacationBalanceTO` (Phase 8 Wave 4).
//!
//! Pure 1:1 mapping — no side-join. The aggregate is computed by the backend
//! and consumed by `VacationEntitlementCard` / `VacationPerPersonList` in
//! Plan 08-05.

use rest_types::VacationBalanceTO;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct VacationBalance {
    pub sales_person_id: Uuid,
    pub year: u32,
    pub entitled_days: f32,
    pub carryover_days: i32,
    pub used_days: f32,
    pub planned_days: f32,
    pub remaining_days: f32,
}

impl From<&VacationBalanceTO> for VacationBalance {
    fn from(t: &VacationBalanceTO) -> Self {
        Self {
            sales_person_id: t.sales_person_id,
            year: t.year,
            entitled_days: t.entitled_days,
            carryover_days: t.carryover_days,
            used_days: t.used_days,
            planned_days: t.planned_days,
            remaining_days: t.remaining_days,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vacation_balance_from_to_preserves_all_fields() {
        let sp = Uuid::from_u128(42);
        let to = VacationBalanceTO {
            sales_person_id: sp,
            year: 2026,
            entitled_days: 30.0,
            carryover_days: 3,
            used_days: 5.0,
            planned_days: 2.0,
            remaining_days: 26.0,
        };
        let state: VacationBalance = (&to).into();
        assert_eq!(state.sales_person_id, sp);
        assert_eq!(state.year, 2026);
        assert_eq!(state.entitled_days, 30.0);
        assert_eq!(state.carryover_days, 3);
        assert_eq!(state.used_days, 5.0);
        assert_eq!(state.planned_days, 2.0);
        assert_eq!(state.remaining_days, 26.0);
    }
}
