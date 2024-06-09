use dioxus::prelude::*;

pub use crate::page::Home;
pub use crate::page::ShiftPlan;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/shiftplan/")]
    ShiftPlan {},
}
