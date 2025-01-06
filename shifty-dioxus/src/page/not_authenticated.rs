use dioxus::prelude::*;

use crate::i18n::Key as K;
use crate::service::i18n::I18N;

#[component]
pub fn NotAuthenticated() -> Element {
    let i18n = I18N.read().clone();
    let title_str = i18n.t(K::WelcomeTitle);
    let login_str = i18n.t(K::PleaseLogin);

    rsx! {
        div { class: "flex place-content-center mt-16",
            div {
                h1 { class: "text-6xl font-bold", "{title_str}" }
                p { class: "mt-8 mb-8 underline",
                    a { href: "/authenticate", "{login_str}" }
                }
                img { src: asset!("/assets/shifty.webp") }
            }
        }
    }
}
