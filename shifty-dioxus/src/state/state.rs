use std::rc::Rc;

use crate::{
    i18n::{self, I18n, Key, Locale},
    state::Config,
    AuthInfo,
};
use rest_types::{DayOfWeekTO, SlotTO};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct State {
    pub config: Config,
    pub i18n: Rc<i18n::I18n<i18n::Key, i18n::Locale>>,
    pub auth_info: AuthInfo,
}
