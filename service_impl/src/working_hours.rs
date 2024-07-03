use std::sync::Arc;

use async_trait::async_trait;
use dao::{sales_person, working_hours::WorkingHoursEntity};
use service::{
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    working_hours::WorkingHours,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

pub struct WorkingHoursServiceImpl<
    WorkingHoursDao: dao::working_hours::WorkingHoursDao,
    SalesPersonService: service::sales_person::SalesPersonService,
    PermissionService: service::PermissionService,
    ClockService: service::clock::ClockService,
    UuidService: service::uuid_service::UuidService,
> {
    working_hours_dao: Arc<WorkingHoursDao>,
    sales_person_service: Arc<SalesPersonService>,
    permission_service: Arc<PermissionService>,
    clock_service: Arc<ClockService>,
    uuid_service: Arc<UuidService>,
}

impl<WorkingHoursDao, SalesPersonService, PermissionService, ClockService, UuidService>
    WorkingHoursServiceImpl<
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
where
    WorkingHoursDao: dao::working_hours::WorkingHoursDao + Sync + Send,
    SalesPersonService: service::sales_person::SalesPersonService + Sync + Send,
    PermissionService: service::PermissionService + Sync + Send,
    ClockService: service::clock::ClockService + Sync + Send,
    UuidService: service::uuid_service::UuidService + Sync + Send,
{
    pub fn new(
        working_hours_dao: Arc<WorkingHoursDao>,
        sales_person_service: Arc<SalesPersonService>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            working_hours_dao,
            sales_person_service,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

#[async_trait]
impl<
        WorkingHoursDao: dao::working_hours::WorkingHoursDao + Sync + Send,
        SalesPersonService: service::sales_person::SalesPersonService<Context = PermissionService::Context>
            + Sync
            + Send,
        PermissionService: service::PermissionService + Sync + Send,
        ClockService: service::clock::ClockService + Sync + Send,
        UuidService: service::uuid_service::UuidService + Sync + Send,
    > service::working_hours::WorkingHoursService
    for WorkingHoursServiceImpl<
        WorkingHoursDao,
        SalesPersonService,
        PermissionService,
        ClockService,
        UuidService,
    >
{
    type Context = PermissionService::Context;

    async fn all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[WorkingHours]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let working_hours: Arc<[WorkingHours]> = self
            .working_hours_dao
            .all()
            .await?
            .iter()
            .map(WorkingHours::from)
            .collect::<Vec<WorkingHours>>()
            .into();
        Ok(working_hours)
    }
    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[WorkingHours]>, ServiceError> {
        let (hr_privilege, user_privilege) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service
                .verify_user_is_sales_person(sales_person_id, context),
        );
        hr_privilege.or(user_privilege)?;

        let working_hours: Arc<[WorkingHours]> = self
            .working_hours_dao
            .find_by_sales_person_id(sales_person_id)
            .await?
            .iter()
            .map(WorkingHours::from)
            .collect::<Vec<WorkingHours>>()
            .into();
        Ok(working_hours)
    }
    async fn create(
        &self,
        working_hours: &WorkingHours,
        context: Authentication<Self::Context>,
    ) -> Result<WorkingHours, ServiceError> {
        let mut working_hours = working_hours.to_owned();
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        working_hours.created = Some(self.clock_service.date_time_now());
        let mut entity: WorkingHoursEntity = (&working_hours).try_into()?;

        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        entity.id = self
            .uuid_service
            .new_uuid("working-hours-service::create id");
        entity.version = self
            .uuid_service
            .new_uuid("working-hours-service::create version");
        self.working_hours_dao
            .create(&entity, "working-hours-service::create")
            .await?;

        Ok(WorkingHours::from(&entity))
    }
    async fn update(
        &self,
        _entity: &WorkingHours,
        _context: Authentication<Self::Context>,
    ) -> Result<WorkingHours, ServiceError> {
        unimplemented!()
    }
}
