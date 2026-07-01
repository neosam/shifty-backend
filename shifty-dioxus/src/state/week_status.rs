//! Frontend domain enum for the calendar-week status (KW-Status).
//!
//! Mirrors the backend `service::week_status::WeekStatus` (four variants), but
//! lives entirely on the FE side. `Unset` is the default: a missing row / a
//! missing server value both mean `Unset` (D-39-04, D-39-03 — the variant is
//! named `Unset`, never `None`).

use rest_types::{WeekStatusKindTO, WeekStatusTO};

#[derive(Clone, PartialEq, Debug, Default)]
pub enum WeekStatus {
    #[default]
    Unset,
    InPlanning,
    Planned,
    Locked,
}

impl From<&WeekStatusKindTO> for WeekStatus {
    fn from(kind: &WeekStatusKindTO) -> Self {
        match kind {
            WeekStatusKindTO::Unset => WeekStatus::Unset,
            WeekStatusKindTO::InPlanning => WeekStatus::InPlanning,
            WeekStatusKindTO::Planned => WeekStatus::Planned,
            WeekStatusKindTO::Locked => WeekStatus::Locked,
        }
    }
}

impl From<&WeekStatus> for WeekStatusKindTO {
    fn from(status: &WeekStatus) -> Self {
        match status {
            WeekStatus::Unset => WeekStatusKindTO::Unset,
            WeekStatus::InPlanning => WeekStatusKindTO::InPlanning,
            WeekStatus::Planned => WeekStatusKindTO::Planned,
            WeekStatus::Locked => WeekStatusKindTO::Locked,
        }
    }
}

impl From<&WeekStatusTO> for WeekStatus {
    fn from(to: &WeekStatusTO) -> Self {
        (&to.status).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_unset() {
        assert_eq!(WeekStatus::default(), WeekStatus::Unset);
    }

    #[test]
    fn maps_all_four_kinds_from_to() {
        assert_eq!(WeekStatus::from(&WeekStatusKindTO::Unset), WeekStatus::Unset);
        assert_eq!(
            WeekStatus::from(&WeekStatusKindTO::InPlanning),
            WeekStatus::InPlanning
        );
        assert_eq!(
            WeekStatus::from(&WeekStatusKindTO::Planned),
            WeekStatus::Planned
        );
        assert_eq!(
            WeekStatus::from(&WeekStatusKindTO::Locked),
            WeekStatus::Locked
        );
    }

    #[test]
    fn round_trips_through_kind_to() {
        for status in [
            WeekStatus::Unset,
            WeekStatus::InPlanning,
            WeekStatus::Planned,
            WeekStatus::Locked,
        ] {
            let kind: WeekStatusKindTO = (&status).into();
            assert_eq!(WeekStatus::from(&kind), status);
        }
    }

    #[test]
    fn maps_from_week_status_to() {
        let to = WeekStatusTO {
            year: 2026,
            calendar_week: 27,
            status: WeekStatusKindTO::Planned,
        };
        assert_eq!(WeekStatus::from(&to), WeekStatus::Planned);
    }
}
