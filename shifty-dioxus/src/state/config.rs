use std::rc::Rc;

use serde::{Deserialize, Serialize};

fn default_env_short_description() -> Rc<str> {
    "DEV".into()
}

fn default_application_title() -> Rc<str> {
    "Shifty".into()
}

fn default_show_vacation() -> bool {
    false
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub backend: Rc<str>,
    #[serde(default = "default_application_title")]
    pub application_title: Rc<str>,
    #[serde(default)]
    pub is_prod: bool,
    #[serde(default = "default_env_short_description")]
    pub env_short_description: Rc<str>,
    #[serde(default = "default_show_vacation")]
    pub show_vacation: bool,
}
