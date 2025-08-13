use std::sync::Arc;
use thiserror::Error;
use time::Date;
use time::Time;
use uuid::Uuid;

pub mod billing_period;
pub mod billing_period_report;
pub mod block;
pub mod booking;
pub mod booking_information;
pub mod carryover;
pub mod clock;
pub mod config;
pub mod custom_extra_hours;
pub mod datetime_utils;
pub mod employee_work_details;
pub mod extra_hours;
pub mod ical;
pub mod permission;
pub mod reporting;
pub mod sales_person;
pub mod sales_person_unavailable;
pub mod scheduler;
pub mod session;
pub mod shiftplan;
pub mod shiftplan_edit;
pub mod shiftplan_report;
pub mod slot;
pub mod special_days;
pub mod user_service;
pub mod uuid_service;
pub mod week_message;

pub use permission::MockPermissionService;
pub use permission::PermissionService;
pub use permission::Privilege;
pub use permission::Role;
pub use permission::User;

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationFailureItem {
    ModificationNotAllowed(Arc<str>),
    InvalidValue(Arc<str>),
    IdDoesNotExist(Arc<str>, Uuid),
    Duplicate,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Database query error: {0}")]
    DatabaseQueryError(#[from] dao::DaoError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Entity {0} aready exists")]
    EntityAlreadyExists(Uuid),

    #[error("Entity {0} not found")]
    EntityNotFound(Uuid),

    #[error("Entity {0} not found")]
    EntityNotFoundGeneric(Arc<str>),

    #[error("Entity {0} conflicts, expected version {1} but got {2}")]
    EntityConflicts(Uuid, Uuid, Uuid),

    #[error("Validation error: {0:?}")]
    ValidationError(Arc<[ValidationFailureItem]>),

    #[error("ID cannot be set on create")]
    IdSetOnCreate,

    #[error("Version cannot be set on create")]
    VersionSetOnCreate,

    #[error("Created cannot bet set on create")]
    CreatedSetOnCreate,

    #[error("Deleted cannot bet set on create")]
    DeletedSetOnCreate,

    #[error("Overlapping time range")]
    OverlappingTimeRange,

    #[error("Time order wrong. {0} must is not smaller or equal to {1}")]
    TimeOrderWrong(Time, Time),

    #[error("Date order wrong. {0} must is not smaller or equal to {1}")]
    DateOrderWrong(Date, Date),

    #[error("Time component range error: {0}")]
    TimeComponentRangeError(#[from] time::error::ComponentRange),

    #[error("Time format error: {0}")]
    TimeFormatError(#[from] time::error::Format),

    #[error("ShiftyDateUtils error: {0}")]
    ShiftyDateUtilsError(#[from] shifty_utils::ShiftyDateUtilsError),

    #[error("Internal error")]
    InternalError,
}
