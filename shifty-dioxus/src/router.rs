use dioxus::prelude::*;

pub use crate::page::EmployeeDetails;
pub use crate::page::Employees;
pub use crate::page::Home;
pub use crate::page::MyEmployeeDetails;
pub use crate::page::ShiftPlan;

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
}
