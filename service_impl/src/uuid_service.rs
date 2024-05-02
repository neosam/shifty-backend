use uuid::Uuid;

pub struct UuidServiceImpl;

impl service::uuid_service::UuidService for UuidServiceImpl {
    fn new_uuid(&self, _usage: &str) -> Uuid {
        Uuid::new_v4()
    }
}
