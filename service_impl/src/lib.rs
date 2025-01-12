use std::sync::Arc;

use async_trait::async_trait;

pub mod block;
pub mod booking;
pub mod booking_information;
pub mod carryover;
pub mod clock;
pub mod employee_work_details;
pub mod extra_hours;
pub mod ical;
pub mod macros;
pub mod permission;
pub mod reporting;
pub mod sales_person;
pub mod sales_person_unavailable;
pub mod scheduler;
pub mod session;
pub mod shiftplan_edit;
pub mod shiftplan_report;
pub mod slot;
pub mod special_days;
mod test;
pub mod uuid_service;

pub use permission::PermissionServiceImpl;
use service::permission::MockContext;

pub struct UserServiceDev;

#[async_trait]
impl service::user_service::UserService for UserServiceDev {
    type Context = MockContext;

    async fn current_user(
        &self,
        _context: Self::Context,
    ) -> Result<Arc<str>, service::ServiceError> {
        Ok("DEVUSER".into())

        // Uncomment to test unauthorized response (not logged in)
        //Err(service::ServiceError::Unauthorized)
    }
}

pub struct UserServiceImpl;

#[async_trait]
impl service::user_service::UserService for UserServiceImpl {
    type Context = Option<Arc<str>>;

    async fn current_user(
        &self,
        context: Self::Context,
    ) -> Result<Arc<str>, service::ServiceError> {
        context.ok_or_else(|| service::ServiceError::Unauthorized)
    }
}
