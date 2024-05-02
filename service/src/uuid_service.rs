use mockall::automock;
use uuid::Uuid;

#[automock]
pub trait UuidService {
    fn new_uuid(&self, usage: &str) -> Uuid;
}
