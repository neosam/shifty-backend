use service::{permission::Authentication, ValidationFailureItem};
use time::{Date, Month, PrimitiveDateTime, Time};
use uuid::Uuid;

pub fn test_forbidden<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::Forbidden) = result {
        // All good
    } else {
        panic!("Expected forbidden error");
    }
}

pub fn test_not_found<T>(result: &Result<T, service::ServiceError>, target_id: &Uuid) {
    if let Err(service::ServiceError::EntityNotFound(id)) = result {
        assert_eq!(
            id, target_id,
            "Expected entity {} not found but got {}",
            target_id, id
        );
    } else {
        panic!("Expected entity {} not found error", target_id);
    }
}

pub fn test_zero_id_error<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::IdSetOnCreate) = result {
    } else {
        panic!("Expected id set on create error");
    }
}

pub fn test_zero_version_error<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::VersionSetOnCreate) = result {
    } else {
        panic!("Expected version set on create error");
    }
}

pub fn test_overlapping_time_range_error<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::OverlappingTimeRange) = result {
    } else {
        panic!("Expected overlapping time range error");
    }
}

pub fn test_time_order_wrong<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::TimeOrderWrong(_from, _to)) = result {
    } else {
        panic!("Expected time order failure");
    }
}

pub fn test_date_order_wrong<T>(result: &Result<T, service::ServiceError>) {
    if let Err(service::ServiceError::DateOrderWrong(_from, _to)) = result {
    } else {
        panic!("Expected date order failure");
    }
}

pub fn test_conflicts<T>(
    result: &Result<T, service::ServiceError>,
    target_id: &Uuid,
    expected_version: &Uuid,
    actual_version: &Uuid,
) {
    if let Err(service::ServiceError::EntityConflicts(
        err_id,
        err_expected_version,
        err_actual_version,
    )) = result
    {
        assert_eq!(
            err_id, target_id,
            "Expected entity {} conflicts but got {}",
            target_id, err_id
        );

        assert_eq!(
            expected_version, err_expected_version,
            "Expected expected version {} but got {}",
            expected_version, err_expected_version
        );
        assert_eq!(
            actual_version, err_actual_version,
            "Expected actual version {} but got {}",
            actual_version, err_actual_version
        );
    } else {
        panic!("Expected entity {} conflicts error", target_id);
    }
}

pub fn test_validation_error<T>(
    result: &Result<T, service::ServiceError>,
    validation_failure: &ValidationFailureItem,
    fail_count: usize,
) {
    if let Err(service::ServiceError::ValidationError(validation_failure_items)) = result {
        if !validation_failure_items.contains(validation_failure) {
            panic!(
                "Validation failure not found: {:?} in {:?}",
                validation_failure, validation_failure_items
            );
        }
        assert_eq!(fail_count, validation_failure_items.len());
    } else {
        panic!("Expected validation error");
    }
}

pub fn generate_default_datetime() -> PrimitiveDateTime {
    PrimitiveDateTime::new(
        Date::from_calendar_date(2063, Month::April, 5).unwrap(),
        Time::from_hms(23, 42, 0).unwrap(),
    )
}

pub trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }

}
