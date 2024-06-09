use std::rc::Rc;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthInfo {
    pub user: Rc<str>,
    pub privileges: Rc<[Rc<str>]>,
    #[serde(default)]
    pub authenticated: bool,
}

impl Default for AuthInfo {
    fn default() -> Self {
        Self {
            user: "".into(),
            privileges: Rc::new([]),
            authenticated: false,
        }
    }
}

impl AuthInfo {
    pub fn has_privilege(&self, privilege: &str) -> bool {
        self.privileges.iter().any(|p| p.as_ref() == privilege)
    }
}
