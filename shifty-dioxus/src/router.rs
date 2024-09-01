use dioxus::prelude::*;

pub use crate::page::EmployeeDetails;
pub use crate::page::Employees;
pub use crate::page::Home;
pub use crate::page::MyEmployeeDetails;
pub use crate::page::SalesPersonDetails;
pub use crate::page::ShiftPlan;
pub use crate::page::UserDetails;
pub use crate::page::UserManagementPage;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/shiftplan/")]
    ShiftPlan {},
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
}
