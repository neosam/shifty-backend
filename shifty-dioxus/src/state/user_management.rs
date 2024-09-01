use rest_types::UserTO;

use crate::base_types::ImStr;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct User {
    pub username: ImStr,
}

impl From<&UserTO> for User {
    fn from(user_to: &UserTO) -> Self {
        User {
            username: ImStr::from(user_to.name.clone()),
        }
    }
}
