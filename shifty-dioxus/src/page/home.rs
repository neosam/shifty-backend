use dioxus::prelude::*;

use crate::component::TopBar;
use crate::i18n::Key as K;
use crate::router::Route;
use crate::service::AUTH;
use crate::service::I18N;

#[component]
pub fn Home() -> Element {
    let Some(auth_info) = AUTH.read().auth_info.clone() else {
        return rsx! {
            div { "Loading auth info..." }
        };
    };
    let nav = navigator();
    let i18n = I18N.read().clone();
    let title_str = i18n.t(K::WelcomeTitle);
    let choose_str = i18n.t(K::PleaseChoose);

    if auth_info.has_privilege("sales") || auth_info.has_privilege("shiftplanner") {
        nav.push(Route::ShiftPlan {});
    } else if auth_info.privileges.len() == 1 && auth_info.has_privilege("hr") {
        nav.push(Route::Employees {});
    }

    rsx! {
        TopBar {}
        div { class: "flex place-content-center mt-16",
            div {
                h1 { class: "text-6xl font-bold", "{title_str}" }
                p { class: "mt-8 mb-8", "{choose_str}" }
                img { src: "/shifty.webp" }
            }
        }
    }
}
