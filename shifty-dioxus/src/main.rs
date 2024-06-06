#![allow(non_snake_case)]

use std::{rc::Rc, sync::Arc};

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{info, Level};

mod i18n;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub backend: Arc<str>,
}

#[derive(Clone, Debug)]
pub struct State {
    config: Config,
    i18n: Arc<i18n::I18n<i18n::Key, i18n::Locale>>,
}

pub async fn load_config() -> Result<Config, reqwest::Error> {
    info!("Loading config.json");
    let url = web_sys::window()
        .expect("no window")
        .location()
        .href()
        .expect("no href");
    info!("URL: {url}");
    let res = reqwest::get(format!("{}/config.json", url))
        .await?
        .json()
        .await?;
    info!("Loaded");
    Ok(res)
}

fn App() -> Element {
    let config = use_resource(|| load_config());
    match &*config.read_unchecked() {
        Some(Ok(config)) => {
            use_context_provider(|| State {
                config: config.clone(),
                i18n: i18n::generate(i18n::Locale::De).into(),
            });
            rsx! {
                Router::<Route> {}
            }
        }
        Some(Err(err)) => {
            rsx! {
                div { "Error while loading technical configuration: {err}" }
            }
        }
        None => {
            rsx! {
                div { "Loading technical configuration..." }
            }
        }
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user: Arc<str>,
    pub privileges: Arc<[Arc<str>]>,
}

pub async fn fetch_auth_info(backend_url: Arc<str>) -> Result<AuthInfo, reqwest::Error> {
    info!("Fetching username");
    let res = reqwest::get(format!("{}/auth-info", backend_url))
        .await?
        .json()
        .await?;
    info!("Fetched");
    Ok(res)
}

#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let state = use_context::<State>();
    let backend = state.config.backend;

    let auth_info = {
        let backend = backend.clone();
        use_resource(move || fetch_auth_info(backend.clone()))
    };

    rsx! {
        Link {
            to: Route::Blog {
                id: count()
            },
            "Go to blog"
        }
        div {
            h1 { "High-Five counter: {count} {backend}" }
            {
                match &*auth_info.read_unchecked() {
                    Some(Ok(auth_info)) => {
                            rsx! { div { "Logged in as {auth_info.user}" } }
                    }
                    Some(Err(err)) => {
                            rsx! { div { "Error while fetching username: {err}" } }
                    }
                    None => {
                            rsx! { div { "Fetching username..." } }
                    }
                }
            }
            button { onclick: move |_| count += 1, "Up high!" }
            button { onclick: move |_| count -= 1, "Down low!" }
        }
    }
}
