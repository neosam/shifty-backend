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
        // Union-Merge (D-53-04): Bezahlten-Loop UNVERAENDERT (Regression-Lock
        // VAA-03 #3), dann Freiwilligen-extend aus neuem DTO-Feld
        // sales_person_absences, anschliessend case-insensitive Name-Sort.
        let sales_person_absences = {
            // (1) Bezahlten-Loop — inhaltlich unveraendert (Regression-Lock VAA-03 #3).
            let mut v: Vec<SalesPersonAbsence> = summary
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
                .collect();
            // (2) Freiwilligen-extend — Filter >= 0.1 faengt Zusage=0 ab (D-53-04).
            v.extend(
                summary
                    .sales_person_absences
                    .iter()
                    .filter(|a| a.hours >= 0.1)
                    .map(|a| SalesPersonAbsence {
                        name: a.name.clone(),
                        absence_hours: a.hours,
                    }),
            );
            // (3) Sortierung — case-insensitive; Vec::sort_by_key ist stable,
            // Namens-Duplikate behalten Insertion-Order (bezahlt zuerst).
            v.sort_by_key(|x| x.name.to_lowercase());
            v
        };
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
            sales_person_absences,
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
    use rest_types::{SalesPersonAbsenceTO, WorkingHoursPerSalesPersonTO};
    use uuid::Uuid;

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
            sales_person_absences: Vec::new().into(),
        }
    }

    /// Test-Fixture fuer Union-Merge-Szenario: baut ein WeeklySummaryTO mit
    /// - einem Bezahlten (via working_hours_per_sales_person, effective_absence >= 0.1)
    /// - einem Freiwilligen (via neuem DTO-Feld sales_person_absences, hours >= 0.1)
    fn make_to_with_paid_and_volunteer(
        paid_name: &str,
        paid_absence: f32,
        volunteer_name: &str,
        volunteer_hours: f32,
    ) -> WeeklySummaryTO {
        let paid_row = WorkingHoursPerSalesPersonTO {
            sales_person_id: Uuid::nil(),
            sales_person_name: paid_name.into(),
            available_hours: 40.0,
            absence_hours: paid_absence,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unavailable_hours: 0.0,
            custom_absence_hours: Vec::new().into(),
        };
        let volunteer_row = SalesPersonAbsenceTO {
            sales_person_id: Uuid::nil(),
            name: volunteer_name.into(),
            hours: volunteer_hours,
        };
        WeeklySummaryTO {
            year: 2026,
            week: 12,
            overall_available_hours: 42.0,
            required_hours: 30.0,
            paid_hours: 20.0,
            volunteer_hours: 7.0,
            committed_voluntary_hours: 0.0,
            monday_available_hours: 1.0,
            tuesday_available_hours: 2.0,
            wednesday_available_hours: 3.0,
            thursday_available_hours: 4.0,
            friday_available_hours: 5.0,
            saturday_available_hours: 6.0,
            sunday_available_hours: 7.0,
            working_hours_per_sales_person: vec![paid_row].into(),
            sales_person_absences: vec![volunteer_row].into(),
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

    /// VAA-01 + D-53-04: Die From-mapping baut sales_person_absences als
    /// Union aus (a) bezahlten-Loop ueber working_hours_per_sales_person +
    /// (b) Freiwilligen-Iteration ueber neues DTO-Feld sales_person_absences,
    /// sortiert case-insensitive nach Name. Bezahlter "Anna" (absence 8.0) und
    /// Freiwilliger "Bob" (hours 5.0) muessen beide erscheinen — Anna vor Bob.
    #[test]
    fn sales_person_absences_union_merges_paid_and_volunteers_sorted_by_name() {
        let to = make_to_with_paid_and_volunteer("Anna", 8.0, "Bob", 5.0);
        let ws = WeeklySummary::from(&to);
        assert_eq!(
            ws.sales_person_absences.len(),
            2,
            "Union muss beide Quellen enthalten (bezahlt + freiwillig)"
        );
        assert_eq!(
            ws.sales_person_absences[0].name.as_ref(),
            "Anna",
            "Anna muss vor Bob stehen (case-insensitive Name-Sort)"
        );
        assert!(
            (ws.sales_person_absences[0].absence_hours - 8.0).abs() < f32::EPSILON,
            "Bezahlten-Zeile muss effective_absence 8.0 tragen (got {})",
            ws.sales_person_absences[0].absence_hours
        );
        assert_eq!(
            ws.sales_person_absences[1].name.as_ref(),
            "Bob",
            "Bob steht nach Anna (zweiter Eintrag)"
        );
        assert!(
            (ws.sales_person_absences[1].absence_hours - 5.0).abs() < f32::EPSILON,
            "Freiwilligen-Zeile muss hours=5.0 tragen (got {})",
            ws.sales_person_absences[1].absence_hours
        );
    }

    /// Regression-Lock VAA-03 #3 (FE-Seite): Bezahlter bleibt via
    /// working_hours_per_sales_person-Pfad sichtbar, auch wenn das neue
    /// DTO-Feld sales_person_absences leer ist. Filter effective_absence
    /// = absence_hours - holiday_hours + unavailable_hours >= 0.1 unveraendert.
    #[test]
    fn bezahlter_bleibt_via_working_hours_pfad_sichtbar() {
        let paid_row = WorkingHoursPerSalesPersonTO {
            sales_person_id: Uuid::nil(),
            sales_person_name: "Anna".into(),
            available_hours: 40.0,
            absence_hours: 8.0,
            vacation_hours: 0.0,
            sick_leave_hours: 0.0,
            holiday_hours: 0.0,
            unavailable_hours: 0.0,
            custom_absence_hours: Vec::new().into(),
        };
        let mut to = make_to(0.0);
        to.working_hours_per_sales_person = vec![paid_row].into();
        let ws = WeeklySummary::from(&to);
        assert_eq!(
            ws.sales_person_absences.len(),
            1,
            "Nur der Bezahlte (via working_hours) muss auftauchen"
        );
        assert_eq!(ws.sales_person_absences[0].name.as_ref(), "Anna");
        assert!(
            (ws.sales_person_absences[0].absence_hours - 8.0).abs() < f32::EPSILON,
            "effective_absence muss weiterhin 8.0 sein"
        );
    }
}
