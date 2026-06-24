use std::sync::Arc;

use rest_types::WeeklySummaryTO;

#[derive(Debug, Clone, PartialEq)]
pub struct SalesPersonAbsence {
    pub name: Arc<str>,
    pub absence_hours: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeeklySummary {
    pub week: u8,
    pub year: u32,
    pub available_hours: f32,
    pub required_hours: f32,
    pub paid_hours: f32,
    pub volunteer_hours: f32,
    pub committed_voluntary_hours: f32,
    pub monday_available_hours: f32,
    pub tuesday_available_hours: f32,
    pub wednesday_available_hours: f32,
    pub thursday_available_hours: f32,
    pub friday_available_hours: f32,
    pub saturday_available_hours: f32,
    pub sunday_available_hours: f32,
    pub sales_person_absences: Vec<SalesPersonAbsence>,
}

impl From<&WeeklySummaryTO> for WeeklySummary {
    fn from(summary: &WeeklySummaryTO) -> Self {
        Self {
            week: summary.week,
            year: summary.year,
            available_hours: summary.overall_available_hours,
            required_hours: summary.required_hours,
            paid_hours: summary.paid_hours,
            volunteer_hours: summary.volunteer_hours,
            committed_voluntary_hours: summary.committed_voluntary_hours,
            monday_available_hours: summary.monday_available_hours,
            tuesday_available_hours: summary.tuesday_available_hours,
            wednesday_available_hours: summary.wednesday_available_hours,
            thursday_available_hours: summary.thursday_available_hours,
            friday_available_hours: summary.friday_available_hours,
            saturday_available_hours: summary.saturday_available_hours,
            sunday_available_hours: summary.sunday_available_hours,
            sales_person_absences: summary
                .working_hours_per_sales_person
                .iter()
                .filter_map(|sp| {
                    let effective_absence =
                        sp.absence_hours - sp.holiday_hours + sp.unavailable_hours;
                    if effective_absence >= 0.1 {
                        Some(SalesPersonAbsence {
                            name: sp.sales_person_name.clone(),
                            absence_hours: effective_absence,
                        })
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

impl WeeklySummary {
    pub fn monday_date(&self) -> time::Date {
        time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Monday).unwrap()
    }
    pub fn sunday_date(&self) -> time::Date {
        time::Date::from_iso_week_date(self.year as i32, self.week, time::Weekday::Sunday).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a fully-populated `WeeklySummaryTO` for From-mapping tests.
    /// `committed` flows into the field under test; the remaining fields use
    /// distinct dummy values so an accidental field-swap would be caught.
    fn make_to(committed: f32) -> WeeklySummaryTO {
        WeeklySummaryTO {
            year: 2026,
            week: 12,
            overall_available_hours: 42.0,
            required_hours: 30.0,
            paid_hours: 20.0,
            volunteer_hours: 7.0,
            committed_voluntary_hours: committed,
            monday_available_hours: 1.0,
            tuesday_available_hours: 2.0,
            wednesday_available_hours: 3.0,
            thursday_available_hours: 4.0,
            friday_available_hours: 5.0,
            saturday_available_hours: 6.0,
            sunday_available_hours: 7.0,
            working_hours_per_sales_person: Vec::new().into(),
        }
    }

    /// CVC-07c (Pitfall 1 — Omission-Lücke): the From-mapping must carry
    /// `committed_voluntary_hours` 1:1 from the TO, NOT `Default::default()`.
    /// A `0.0` default would compile silently and the UI would show `0.00`.
    #[test]
    fn committed_voluntary_hours_maps_from_to() {
        let to = make_to(5.0);
        let ws = WeeklySummary::from(&to);
        assert!(
            (ws.committed_voluntary_hours - 5.0).abs() < f32::EPSILON,
            "committed_voluntary_hours must map to 5.0 from the TO (got {})",
            ws.committed_voluntary_hours
        );
    }

    /// D-01: `available_hours` is sourced from `overall_available_hours`, which
    /// already carries the committed band via the backend (no frontend extra logic).
    #[test]
    fn available_hours_maps_from_overall_available_hours() {
        let to = make_to(5.0);
        let ws = WeeklySummary::from(&to);
        assert!(
            (ws.available_hours - to.overall_available_hours).abs() < f32::EPSILON,
            "available_hours must equal overall_available_hours (got {} vs {})",
            ws.available_hours,
            to.overall_available_hours
        );
    }
}
