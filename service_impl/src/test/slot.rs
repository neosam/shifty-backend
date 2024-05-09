use std::sync::Arc;

use crate::slot::*;
use crate::test::error_test::*;
use dao::slot::{MockSlotDao, SlotEntity};
use mockall::predicate::eq;
use service::{
    clock::MockClockService, slot::*, uuid_service::MockUuidService, MockPermissionService,
    ValidationFailureItem,
};
use time::{Date, Month, PrimitiveDateTime, Time};
use tokio;
use uuid::{uuid, Uuid};

pub fn default_id() -> Uuid {
    uuid!("682DA62E-20CB-49D9-A2A7-3F53C6842405")
}
pub fn default_version() -> Uuid {
    uuid!("86DE856C-D176-4F1F-A4FE-0D9844C02C03")
}
pub fn default_changed_version() -> Uuid {
    uuid!("4A818852-45D2-400F-A02A-755D34FFE815")
}

pub fn generate_default_slot() -> Slot {
    Slot {
        id: default_id(),
        day_of_week: DayOfWeek::Monday,
        from: time::Time::from_hms(10, 0, 0).unwrap(),
        to: time::Time::from_hms(11, 0, 0).unwrap(),
        valid_from: time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: default_version(),
    }
}
pub fn generate_default_slot_entity() -> SlotEntity {
    SlotEntity {
        id: uuid!("682DA62E-20CB-49D9-A2A7-3F53C6842405"),
        day_of_week: dao::slot::DayOfWeek::Monday,
        from: time::Time::from_hms(10, 0, 0).unwrap(),
        to: time::Time::from_hms(11, 0, 0).unwrap(),
        valid_from: time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 1).unwrap(),
        valid_to: None,
        deleted: None,
        version: uuid!("86DE856C-D176-4F1F-A4FE-0D9844C02C03"),
    }
}

pub struct SlotServiceDependencies {
    pub slot_dao: MockSlotDao,
    pub permission_service: MockPermissionService,
    pub clock_service: MockClockService,
    pub uuid_service: MockUuidService,
}
impl SlotServiceDependencies {
    pub fn build_service(
        self,
    ) -> SlotServiceImpl<MockSlotDao, MockPermissionService, MockClockService, MockUuidService>
    {
        SlotServiceImpl::new(
            self.slot_dao.into(),
            self.permission_service.into(),
            self.clock_service.into(),
            self.uuid_service.into(),
        )
    }
}

pub fn build_dependencies(permission: bool, role: &'static str) -> SlotServiceDependencies {
    let slot_dao = MockSlotDao::new();
    let mut permission_service = MockPermissionService::new();
    permission_service
        .expect_check_permission()
        .with(eq(role), eq(().auth()))
        .returning(move |_, _| {
            if permission {
                Ok(())
            } else {
                Err(service::ServiceError::Forbidden)
            }
        });
    permission_service
        .expect_check_permission()
        .returning(move |_, _| Err(service::ServiceError::Forbidden));
    let mut clock_service = MockClockService::new();
    clock_service
        .expect_time_now()
        .returning(|| time::Time::from_hms(23, 42, 0).unwrap());
    clock_service
        .expect_date_now()
        .returning(|| time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap());
    clock_service.expect_date_time_now().returning(|| {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2063, 4.try_into().unwrap(), 5).unwrap(),
            time::Time::from_hms(23, 42, 0).unwrap(),
        )
    });
    let uuid_service = MockUuidService::new();

    SlotServiceDependencies {
        slot_dao,
        permission_service,
        clock_service,
        uuid_service,
    }
}

#[tokio::test]
async fn test_get_slots() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies.slot_dao.expect_get_slots().returning(|| {
        Ok(Arc::new([
            SlotEntity {
                id: uuid!("DA703BC1-F488-4E4F-BA10-0972196639F7"),
                version: uuid!("FAC4FAD9-89AE-4E56-9608-03C56558B192"),
                ..generate_default_slot_entity()
            },
            generate_default_slot_entity(),
        ]))
    });

    let slot_service = dependencies.build_service();

    let result = slot_service.get_slots(().auth()).await;
    assert!(result.is_ok());
    let result = result.unwrap();

    assert_eq!(result.len(), 2);
    assert_eq!(
        result[0],
        Slot {
            id: uuid!("DA703BC1-F488-4E4F-BA10-0972196639F7"),
            version: uuid!("FAC4FAD9-89AE-4E56-9608-03C56558B192"),
            ..generate_default_slot()
        },
    );
    assert_eq!(result[1], generate_default_slot(),);
}

#[tokio::test]
async fn test_get_slots_sales_role() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slots()
        .returning(|| Ok(Arc::new([])));
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slots(().auth()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_slots_no_permission() {
    let mut dependencies = build_dependencies(false, "hr");
    dependencies
        .slot_dao
        .expect_get_slots()
        .returning(|| Ok(Arc::new([])));
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slots(().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_get_slot() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slot(&default_id(), ().auth()).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result, generate_default_slot());
}

#[tokio::test]
async fn test_get_slot_sales_role() {
    let mut dependencies = build_dependencies(true, "sales");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slot(&default_id(), ().auth()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_get_slot_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slot(&default_id(), ().auth()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_get_slot_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let slot_service = dependencies.build_service();
    let result = slot_service.get_slot(&default_id(), ().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_slot() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_create_slot()
        .with(eq(generate_default_slot_entity()), eq("slot-service"))
        .times(1)
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_version());
    dependencies
        .slot_dao
        .expect_get_slots()
        .returning(|| Ok(Arc::new([])));

    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), generate_default_slot());
}

#[tokio::test]
async fn test_create_slot_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(&generate_default_slot(), ().auth())
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_create_slot_non_zero_id() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_version());
    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(
            &Slot {
                version: Uuid::nil(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_zero_id_error(&result);
}

#[tokio::test]
async fn test_create_slot_non_zero_version() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_version());
    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_zero_version_error(&result);
}

#[tokio::test]
async fn test_create_slot_intersects() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies.slot_dao.expect_get_slots().returning(|| {
        Ok(Arc::new([
            generate_default_slot_entity(),
            SlotEntity {
                id: Uuid::new_v4(),
                from: Time::from_hms(12, 0, 0).unwrap(),
                to: Time::from_hms(13, 0, 0).unwrap(),
                ..generate_default_slot_entity()
            },
            SlotEntity {
                id: Uuid::new_v4(),
                day_of_week: DayOfWeek::Wednesday.into(),
                from: Time::from_hms(11, 0, 0).unwrap(),
                to: Time::from_hms(12, 0, 0).unwrap(),
                ..generate_default_slot_entity()
            },
        ]))
    });
    dependencies
        .slot_dao
        .expect_create_slot()
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-id"))
        .returning(|_| default_id());
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_version());
    let slot_service = dependencies.build_service();

    // Test successful case, directly between two existing slots.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(11, 0, 0).unwrap(),
                to: Time::from_hms(12, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    assert!(result.is_ok());

    // Test case where it is exactly on an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(10, 0, 0).unwrap(),
                to: Time::from_hms(11, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_overlapping_time_range_error(&result);

    // Test case where from is inside an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(10, 30, 0).unwrap(),
                to: Time::from_hms(11, 30, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_overlapping_time_range_error(&result);

    // Test case where to is inside an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(11, 30, 0).unwrap(),
                to: Time::from_hms(12, 30, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_overlapping_time_range_error(&result);

    // Test case where is completely inside an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(10, 15, 0).unwrap(),
                to: Time::from_hms(10, 45, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_overlapping_time_range_error(&result);

    // Test case where is completely outside of an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(9, 0, 0).unwrap(),
                to: Time::from_hms(11, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_overlapping_time_range_error(&result);

    // Test case where is would intersect on monday but not on tuesday.
    // Test case where is completely outside of an existing slot.
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                day_of_week: DayOfWeek::Tuesday.into(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_create_slot_time_order() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_create_slot()
        .returning(|_, _| Ok(()));
    dependencies
        .slot_dao
        .expect_get_slots()
        .returning(|| Ok(Arc::new([])));

    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                from: Time::from_hms(12, 00, 0).unwrap(),
                to: Time::from_hms(11, 00, 00).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_time_order_wrong(&result);
}

#[tokio::test]
async fn test_create_slot_date_order() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_create_slot()
        .returning(|_, _| Ok(()));
    dependencies
        .slot_dao
        .expect_get_slots()
        .returning(|| Ok(Arc::new([])));

    let slot_service = dependencies.build_service();
    let result = slot_service
        .create_slot(
            &Slot {
                id: Uuid::nil(),
                version: Uuid::nil(),
                valid_from: Date::from_calendar_date(2022, Month::January, 2).unwrap(),
                valid_to: Some(Date::from_calendar_date(2022, Month::January, 1).unwrap()),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_date_order_wrong(&result);
}

#[tokio::test]
async fn test_delete_slot() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    dependencies
        .slot_dao
        .expect_update_slot()
        .with(
            eq(SlotEntity {
                deleted: Some(PrimitiveDateTime::new(
                    Date::from_calendar_date(2063, time::Month::April, 5).unwrap(),
                    Time::from_hms(23, 42, 0).unwrap(),
                )),
                ..generate_default_slot_entity()
            }),
            eq("slot-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let slot_service = dependencies.build_service();
    let result = slot_service.delete_slot(&default_id(), ().auth()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_slot_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let slot_service = dependencies.build_service();
    let result = slot_service.delete_slot(&default_id(), ().auth()).await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_delete_slot_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let slot_service = dependencies.build_service();
    let result = slot_service.delete_slot(&default_id(), ().auth()).await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_update_slot_no_permission() {
    let dependencies = build_dependencies(false, "hr");
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(&generate_default_slot(), ().auth())
        .await;
    test_forbidden(&result);
}

#[tokio::test]
async fn test_update_slot_not_found() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .times(1)
        .returning(|_| Ok(None));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(&generate_default_slot(), ().auth())
        .await;
    test_not_found(&result, &default_id());
}

#[tokio::test]
async fn test_update_slot_version_mismatch() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                version: uuid!("86DE856C-D176-4F1F-A4FE-0D9844C02C04"),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_conflicts(
        &result,
        &default_id(),
        &default_version(),
        &uuid!("86DE856C-D176-4F1F-A4FE-0D9844C02C04"),
    );
}

#[tokio::test]
async fn test_update_slot_valid_to() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_update_slot()
        .once()
        .with(
            eq(dao::slot::SlotEntity {
                valid_to: Some(
                    time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 10).unwrap(),
                ),
                version: default_changed_version(),
                ..generate_default_slot_entity()
            }),
            eq("slot-service"),
        )
        .times(1)
        .returning(|_, _| Ok(()));
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_changed_version());

    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                valid_to: Some(
                    time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 10).unwrap(),
                ),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    dbg!(&result);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_slot_valid_to_before_valid_from() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));

    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                valid_to: Some(
                    time::Date::from_calendar_date(2021, 1.try_into().unwrap(), 10).unwrap(),
                ),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_date_order_wrong(&result);
}

#[tokio::test]
async fn test_update_slot_deleted() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    dependencies
        .slot_dao
        .expect_update_slot()
        .once()
        .with(
            eq(dao::slot::SlotEntity {
                deleted: Some(time::PrimitiveDateTime::new(
                    Date::from_calendar_date(2022, 1.try_into().unwrap(), 10).unwrap(),
                    Time::from_hms(0, 0, 0).unwrap(),
                )),
                version: default_changed_version(),
                ..generate_default_slot_entity()
            }),
            eq("slot-service"),
        )
        .returning(|_, _| Ok(()));
    dependencies
        .uuid_service
        .expect_new_uuid()
        .with(eq("slot-version"))
        .returning(|_| default_changed_version());

    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &Slot {
                deleted: Some(time::PrimitiveDateTime::new(
                    Date::from_calendar_date(2022, 1.try_into().unwrap(), 10).unwrap(),
                    Time::from_hms(0, 0, 0).unwrap(),
                )),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_slot_day_of_week_forbidden() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                day_of_week: service::slot::DayOfWeek::Friday,
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("day_of_week".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_to_forbidden_when_not_none() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| {
            Ok(Some(SlotEntity {
                valid_to: Some(
                    time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 3).unwrap(),
                ),
                ..generate_default_slot_entity()
            }))
        });
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                valid_to: Some(
                    time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 4).unwrap(),
                ),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("valid_to".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_from_forbidden() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                from: time::Time::from_hms(14, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("from".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_to_forbidden() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                to: time::Time::from_hms(14, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("to".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_valid_from_forbidden() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                valid_from: time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 10)
                    .unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("valid_from".into()),
        1,
    );
}

#[tokio::test]
async fn test_update_valid_multiple_forbidden_changes() {
    let mut dependencies = build_dependencies(true, "hr");
    dependencies
        .slot_dao
        .expect_get_slot()
        .with(eq(default_id()))
        .returning(|_| Ok(Some(generate_default_slot_entity())));
    let slot_service = dependencies.build_service();
    let result = slot_service
        .update_slot(
            &service::slot::Slot {
                valid_from: time::Date::from_calendar_date(2022, 1.try_into().unwrap(), 10)
                    .unwrap(),
                from: time::Time::from_hms(14, 0, 0).unwrap(),
                ..generate_default_slot()
            },
            ().auth(),
        )
        .await;
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("valid_from".into()),
        2,
    );
    test_validation_error(
        &result,
        &ValidationFailureItem::ModificationNotAllowed("from".into()),
        2,
    );
}
