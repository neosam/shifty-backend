use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub backend: Rc<str>,
    pub show_my_time: Option<bool>,
}
