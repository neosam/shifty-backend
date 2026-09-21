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
