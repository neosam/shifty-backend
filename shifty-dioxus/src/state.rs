use std::sync::Arc;

use crate::{i18n, AuthInfo};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub backend: Arc<str>,
}

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub i18n: Arc<i18n::I18n<i18n::Key, i18n::Locale>>,
    pub auth_info: AuthInfo,
}
