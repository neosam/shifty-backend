use dioxus::prelude::*;

pub use crate::blog::Blog;
pub use crate::home::Home;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}
