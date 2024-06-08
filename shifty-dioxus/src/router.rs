use dioxus::prelude::*;

pub use crate::blog::Blog;
pub use crate::home::Home;
pub use crate::shiftplan::ShiftPlan;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/shiftplan/")]
    ShiftPlan {},
}
