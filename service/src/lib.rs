use std::sync::Arc;
use thiserror::Error;
use time::Date;
use time::Time;
use uuid::Uuid;

pub mod booking;
pub mod clock;
pub mod permission;
pub mod sales_person;
pub mod slot;
pub mod user_service;
pub mod uuid_service;

pub use permission::MockPermissionService;
pub use permission::PermissionService;
pub use permission::Privilege;
pub use permission::Role;
pub use permission::User;

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationFailureItem {
    ModificationNotAllowed(Arc<str>),
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] dao::DaoError),

    #[error("Forbidden")]
    Forbidden,

    #[error("Entity {0} aready exists")]
    EntityAlreadyExists(Uuid),

    #[error("Entity {0} not found")]
    EntityNotFound(Uuid),

    #[error("Entity {0} conflicts, expected version {1} but got {2}")]
    EntityConflicts(Uuid, Uuid, Uuid),

    #[error("Validation error: {0:?}")]
    ValidationError(Arc<[ValidationFailureItem]>),

    #[error("ID cannot be set on create")]
    IdSetOnCreate,

    #[error("Version cannot be set on create")]
    VersionSetOnCreate,

    #[error("Overlapping time range")]
    OverlappingTimeRange,

    #[error("Time order wrong. {0} must is not smaller or equal to {1}")]
    TimeOrderWrong(Time, Time),

    #[error("Date order wrong. {0} must is not smaller or equal to {1}")]
    DateOrderWrong(Date, Date),
}
