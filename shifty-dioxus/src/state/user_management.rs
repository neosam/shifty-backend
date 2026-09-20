use rest_types::UserTO;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::base_types::ImStr;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ShiftplanAssignment {
    pub shiftplan_id: Uuid,
    #[serde(default = "default_permission_level")]
    pub permission_level: String,
}

fn default_permission_level() -> String {
    "available".to_string()
}

#[allow(dead_code)] // reason: Phase 62 loest den N+1-Ladepfad der User-Management-Seite ab; Phase 63 braucht diesen Pfad fuer den Kandidaten-Picker wieder.
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
