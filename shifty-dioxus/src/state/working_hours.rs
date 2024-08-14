use uuid::Uuid;

use crate::base_types::ImStr;

#[derive(PartialEq, Clone)]
pub struct WorkingHoursMini {
    pub sales_person_id: Uuid,
    pub sales_person_name: ImStr,
    pub expected_hours: f32,
    pub actual_hours: f32,
}
