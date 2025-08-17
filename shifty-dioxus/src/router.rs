use dioxus::prelude::*;

pub use crate::page::BillingPeriodDetails;
pub use crate::page::CustomExtraHoursManagement;
pub use crate::page::EmployeeDetails;
pub use crate::page::Employees;
pub use crate::page::Home;
pub use crate::page::MyEmployeeDetails;
pub use crate::page::SalesPersonDetails;
pub use crate::page::ShiftPlan;
pub use crate::page::ShiftPlanDeep;
pub use crate::page::TextTemplateManagement;
pub use crate::page::UserDetails;
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
    #[route("/employees/:employee_id/")]
    EmployeeDetails { employee_id: String },
    #[route("/my_employee_details/")]
    MyEmployeeDetails {},
    #[route("/user_management/")]
    UserManagementPage {},
    #[route("/user_details/:user_id/")]
    UserDetails { user_id: String },
    #[route("/sales_person_details/:sales_person_id/")]
    SalesPersonDetails { sales_person_id: String },
    #[route("/custom_extra_hours/")]
    CustomExtraHoursManagement {},
    #[route("/text_templates/")]
    TextTemplateManagement {},
    #[route("/billing_period/:billing_period_id/")]
    BillingPeriodDetails { billing_period_id: String },
}
