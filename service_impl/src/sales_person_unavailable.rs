use std::sync::Arc;

use async_trait::async_trait;
use service::{
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct SalesPersonUnavailableServiceImpl<
    SalesPersonUnavailableDao,
    SalesPersonService,
    PermissionService,
    ClockService,
    UuidService,
> where
    SalesPersonUnavailableDao:
        dao::sales_person_unavailable::SalesPersonUnavailableDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub sales_person_unavailable_dao: Arc<SalesPersonUnavailableDao>,
    pub sales_person_service: Arc<SalesPersonService>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}

impl<
        SalesPersonUnavailableDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
    SalesPersonUnavailableServiceImpl<
        SalesPersonUnavailableDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    SalesPersonUnavailableDao:
        dao::sales_person_unavailable::SalesPersonUnavailableDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub fn new(
        sales_person_unavailable_dao: Arc<SalesPersonUnavailableDao>,
        sales_person_service: Arc<SalesPersonService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            sales_person_unavailable_dao,
            sales_person_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        SalesPersonUnavailableDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    > SalesPersonUnavailableService
    for SalesPersonUnavailableServiceImpl<
        SalesPersonUnavailableDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    SalesPersonUnavailableDao:
        dao::sales_person_unavailable::SalesPersonUnavailableDao + Send + Sync,
    SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
        + Send
        + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_all_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context.clone()),
        );
        shiftplanner_permission.or(is_sales_person)?;

        self.sales_person_unavailable_dao
            .find_all_by_sales_person_id(sales_person_id)
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect()
    }

    async fn get_by_week_for_sales_person(
        &self,
        sales_person_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context.clone()),
        );
        shiftplanner_permission.or(is_sales_person)?;

        self.sales_person_unavailable_dao
            .find_by_week_and_sales_person_id(sales_person_id, year, calendar_week)
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect()
    }

    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        self.sales_person_unavailable_dao
            .find_by_week(year, calendar_week)
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect()
    }

    async fn create(
        &self,
        entity: &SalesPersonUnavailable,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPersonUnavailable, ServiceError> {
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(entity.sales_person_id, context.clone()),
        );
        shiftplanner_permission.or(is_sales_person)?;

        if entity.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if entity.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        if entity.deleted.is_some() {
            return Err(ServiceError::DeletedSetOnCreate);
        }
        if entity.created.is_some() {
            return Err(ServiceError::CreatedSetOnCreate);
        }
        if let Some(entity) = self
            .sales_person_unavailable_dao
            .find_by_week_and_sales_person_id(
                entity.sales_person_id,
                entity.year,
                entity.calendar_week,
            )
            .await?
            .iter()
            .find(|e| e.day_of_week == entity.day_of_week.into())
        {
            return Err(ServiceError::EntityAlreadyExists(entity.id));
        }

        let entity = dao::sales_person_unavailable::SalesPersonUnavailableEntity::try_from(
            &SalesPersonUnavailable {
                id: self
                    .uuid_service
                    .new_uuid("SalesPersonUnavailableService::create id"),
                version: self
                    .uuid_service
                    .new_uuid("SalesPersonUnavailableService::create version"),
                created: Some(self.clock_service.date_time_now()),
                ..entity.clone()
            },
        )?;
        self.sales_person_unavailable_dao
            .create(&entity, "SalesPersonUnavailableService::create")
            .await?;

        Ok((&entity).into())
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        let entity = self
            .sales_person_unavailable_dao
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;

        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(entity.sales_person_id, context.clone()),
        );
        shiftplanner_permission.or(is_sales_person)?;

        self.sales_person_unavailable_dao
            .update(
                &dao::sales_person_unavailable::SalesPersonUnavailableEntity {
                    deleted: Some(self.clock_service.date_time_now()),
                    version: self
                        .uuid_service
                        .new_uuid("SalesPersonUnavailableService::delete version"),
                    ..entity
                },
                "SalesPersonUnavailableService::delete",
            )
            .await?;

        Ok(())
    }
}
