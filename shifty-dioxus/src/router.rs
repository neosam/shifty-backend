use dioxus::prelude::*;

pub use crate::page::AbsencesFor;
// `dioxus_router`'s `#[derive(Routable)]` looks up a component by the exact
// variant name, so the `Absences {}` route below needs an item called
// `Absences` in scope. We alias the page component (which is named
// `AbsencesPage` per the Plan-05 component-inventory contract) so both
// the descriptive name and the route lookup work.
pub use crate::page::AbsencesPage as Absences;
pub use crate::page::BillingPeriodDetails;
pub use crate::page::BillingPeriods;
pub use crate::page::CustomExtraHoursManagement;
pub use crate::page::EmployeeDetails;
pub use crate::page::Employees;
pub use crate::page::Home;
pub use crate::page::MyEmployeeDetails;
pub use crate::page::MyShifts;
pub use crate::page::PersonDetail;
pub use crate::page::SettingsPage as Settings;
pub use crate::page::ShiftPlan;
pub use crate::page::ShiftPlanDeep;
pub use crate::page::TextTemplateManagement;
pub use crate::page::UserManagementPage;
pub use crate::page::WeeklyOverview;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/shiftplan/")]
    ShiftPlan {},
    #[route("/shiftplan/:year/:week")]
    ShiftPlanDeep { year: u32, week: u8 },
    #[route("/weekly_overview/")]
    WeeklyOverview {},
    #[route("/employees/")]
    Employees {},
    #[route("/billing-periods/")]
    BillingPeriods {},
    #[route("/employees/:employee_id/")]
    EmployeeDetails { employee_id: String },
    #[route("/my_employee_details/")]
    MyEmployeeDetails {},
    #[route("/user_management/")]
    UserManagementPage {},
    /// Phase 63 (D-63-04): die EINE Personen-Detailroute fuer alle drei
    /// Auspraegungen. Die beiden bisherigen Detailrouten sind mit Phase 63
    /// ersatzlos entfallen — bewusst ohne Weiterleitungs-Huellen, damit es
    /// je Sache genau einen Pfad gibt. Ein gespeichertes Lesezeichen auf
    /// einen der alten Pfade laeuft ins Leere; das Nachruesten einer
    /// Weiterleitung bleibt moeglich, ist aber keine stille Schuld.
    /// Der Pfad heisst `/person-detail/`, nicht `/person/`: der Dev-Proxy in
    /// `Dioxus.toml` leitet das Segment `/person` vollstaendig ans Backend
    /// weiter (die sieben REST-Routen aus 63-06). Unter `/person/:person_id/`
    /// hat der Browser deshalb 404 vom Backend bekommen statt der SPA — die
    /// Seite war ueber ihre eigene URL nicht erreichbar. Das Matching des
    /// Proxys ist segmentweise, `/person-detail` ist damit kollisionsfrei.
    #[route("/person-detail/:person_id/")]
    PersonDetail { person_id: String },
    #[route("/custom_extra_hours/")]
    CustomExtraHoursManagement {},
    #[route("/text_templates/")]
    TextTemplateManagement {},
    #[route("/billing_period/:billing_period_id/")]
    BillingPeriodDetails { billing_period_id: String },
    #[route("/my-shifts/")]
    MyShifts {},
    #[route("/absences/")]
    Absences {},
    #[route("/absences/:employee_id/")]
    AbsencesFor { employee_id: String },
    #[route("/settings/")]
    Settings {},
}

#[cfg(test)]
mod tests {
    use super::Route;
    use dioxus::prelude::{Routable, SegmentType};
    use std::collections::BTreeSet;

    /// Erstsegmente, die der Dev-Proxy in `Dioxus.toml` ans Backend abgibt.
    fn proxied_first_segments() -> BTreeSet<String> {
        include_str!("../Dioxus.toml")
            .lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("backend = \""))
            .filter_map(|rest| rest.strip_suffix('"'))
            // "http://localhost:3000/admin/cutover" -> "admin/cutover"
            .filter_map(|url| url.split_once("://"))
            .filter_map(|(_, host_and_path)| host_and_path.split_once('/'))
            .map(|(_, path)| path)
            .filter_map(|path| path.split('/').next())
            .filter(|seg| !seg.is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// Erstsegmente, unter denen die SPA eigene Seiten fuehrt.
    fn spa_first_segments() -> BTreeSet<String> {
        Route::SITE_MAP
            .iter()
            .filter_map(|segment| match segment.segment_type {
                SegmentType::Static(s) if !s.is_empty() => Some(s.to_owned()),
                _ => None,
            })
            .collect()
    }

    /// Vorbestehend und bewusst geduldet: `/shiftplan/` (SPA) und
    /// `.nest("/shiftplan", ...)` (Backend) teilen sich das Segment schon
    /// seit vor Phase 63. Heute kollidiert keine konkrete Route, weil die
    /// Backend-Pfade vier Segmente tragen und die SPA-Pfade zwei bis drei.
    /// Das ist Zufall, keine Konstruktion — dokumentiert als Altlast, damit
    /// dieser Test nicht an ihr scheitert und sie trotzdem sichtbar bleibt.
    const KNOWN_PREEXISTING: &[&str] = &["shiftplan"];

    /// Der Dev-Proxy leitet ein Pfad-Segment vollstaendig ans Backend weiter.
    /// Liegt eine SPA-Route unter demselben Erstsegment, bekommt der Browser
    /// die Backend-Antwort (oder deren 404) statt der Anwendung — die Seite
    /// ist ueber ihre eigene URL nicht erreichbar.
    ///
    /// Genau das ist in Phase 63 passiert: `/person/:person_id/` gegen
    /// `.nest("/person", ...)`. Keine Testsuite hat es bemerkt, weil SSR-Tests
    /// die Komponente direkt rendern und das Routing nie anfassen. Dieser Test
    /// schliesst die Luecke fuer jede kuenftige Route.
    #[test]
    fn keine_spa_route_liegt_im_proxy_namensraum_des_backends() {
        let proxied = proxied_first_segments();
        assert!(
            proxied.contains("person"),
            "Parser defekt: `/person` muss unter den Proxy-Eintraegen sein"
        );

        let kollisionen: Vec<String> = spa_first_segments()
            .into_iter()
            .filter(|seg| proxied.contains(seg))
            .filter(|seg| !KNOWN_PREEXISTING.contains(&seg.as_str()))
            .collect();

        assert!(
            kollisionen.is_empty(),
            "Diese SPA-Erstsegmente gibt der Dev-Proxy ans Backend ab — die \
             Seiten sind unter ihrer eigenen URL nicht erreichbar: {kollisionen:?}. \
             Entweder die Route umbenennen (das Proxy-Matching ist segmentweise, \
             `/person-detail` statt `/person` genuegt) oder den Backend-Praefix \
             verschieben."
        );
    }

    /// Haelt den Fix aus Phase 63 fest: die Detailroute darf nicht zurueck
    /// unter `/person/` wandern.
    #[test]
    fn person_detail_route_liegt_ausserhalb_des_person_praefix() {
        let pfad = Route::PersonDetail {
            person_id: "836f3408-d276-7972-eaea-7102df9b7903".into(),
        }
        .to_string();
        assert!(
            pfad.starts_with("/person-detail/"),
            "Detailroute ist nach `{pfad}` gewandert — `/person/` gehoert dem \
             Backend (sieben REST-Routen aus 63-06)"
        );
    }
}
