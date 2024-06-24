use std::sync::Arc;

use async_trait::async_trait;
use dao::sales_person::SalesPersonEntity;
use service::{
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    sales_person::SalesPerson,
    ServiceError, ValidationFailureItem,
};
use tokio::join;
use uuid::Uuid;

pub struct SalesPersonServiceImpl<SalesPersonDao, PermissionService, ClockService, UuidService>
where
    SalesPersonDao: dao::sales_person::SalesPersonDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub sales_person_dao: Arc<SalesPersonDao>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
}
impl<SalesPersonDao, PermissionService, ClockService, UuidService>
    SalesPersonServiceImpl<SalesPersonDao, PermissionService, ClockService, UuidService>
where
    SalesPersonDao: dao::sales_person::SalesPersonDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    pub fn new(
        sales_person_dao: Arc<SalesPersonDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
    ) -> Self {
        Self {
            sales_person_dao,
            permission_service,
            clock_service,
            uuid_service,
        }
    }
}

const SALES_PERSON_SERVICE_PROCESS: &str = "sales-person-service";

#[async_trait]
impl<SalesPersonDao, PermissionService, ClockService, UuidService>
    service::sales_person::SalesPersonService
    for SalesPersonServiceImpl<SalesPersonDao, PermissionService, ClockService, UuidService>
where
    SalesPersonDao: dao::sales_person::SalesPersonDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
{
    type Context = PermissionService::Context;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::sales_person::SalesPerson]>, service::ServiceError> {
        let (shiftplanner, sales, hr) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales).or(hr)?;
        let mut sales_persons = self
            .sales_person_dao
            .all()
            .await?
            .iter()
            .map(SalesPerson::from)
            .collect::<Box<[SalesPerson]>>();

        // Remove sensitive information if user is not a sales user.
        if self
            .permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await
            .is_err()
        {
            sales_persons.iter_mut().for_each(|sales_person| {
                sales_person.is_paid = None;
            });
        }
        Ok(sales_persons.into())
    }

    async fn get_all_paid(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        Ok(self
            .sales_person_dao
            .all_paid()
            .await?
            .iter()
            .map(SalesPerson::from)
            .collect())
    }

    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<service::sales_person::SalesPerson, service::ServiceError> {
        let (shiftplanner, sales, hr) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales).or(hr)?;
        let mut sales_person = self
            .sales_person_dao
            .find_by_id(id)
            .await?
            .as_ref()
            .map(SalesPerson::from)
            .ok_or(ServiceError::EntityNotFound(id))?;

        let remove_sensitive_data = if self
            .permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await
            .is_err()
        {
            if let (Some(current_user_id), Some(assigned_user)) = (
                self.permission_service
                    .current_user_id(context.clone())
                    .await?,
                self.get_assigned_user(id, Authentication::Full).await?,
            ) {
                current_user_id != assigned_user
            } else {
                true
            }
        } else {
            false
        };

        if remove_sensitive_data {
            sales_person.is_paid = None;
        }

        Ok(sales_person)
    }

    async fn exists(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
    ) -> Result<bool, ServiceError> {
        Ok(self
            .sales_person_dao
            .find_by_id(id)
            .await
            .map(|x| x.is_some())?)
    }

    async fn create(
        &self,
        sales_person: &SalesPerson,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPerson, service::ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        if sales_person.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if sales_person.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        let sales_person = SalesPerson {
            id: self.uuid_service.new_uuid("sales-person-id"),
            version: self.uuid_service.new_uuid("sales-person-version"),
            ..sales_person.clone()
        };
        self.sales_person_dao
            .create(
                &SalesPersonEntity::from(&sales_person),
                SALES_PERSON_SERVICE_PROCESS,
            )
            .await?;
        Ok(sales_person)
    }

    async fn update(
        &self,
        sales_person: &SalesPerson,
        context: Authentication<Self::Context>,
    ) -> Result<SalesPerson, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let sales_person_entity = self
            .sales_person_dao
            .find_by_id(sales_person.id)
            .await?
            .as_ref()
            .map(SalesPerson::from)
            .ok_or_else(move || ServiceError::EntityNotFound(sales_person.id))?;

        if sales_person.version != sales_person_entity.version {
            return Err(ServiceError::EntityConflicts(
                sales_person.id,
                sales_person_entity.version,
                sales_person.version,
            ));
        }

        if sales_person.deleted != sales_person_entity.deleted {
            return Err(ServiceError::ValidationError(
                [ValidationFailureItem::ModificationNotAllowed(
                    "deleted".into(),
                )]
                .into(),
            ));
        }

        let sales_person = SalesPerson {
            version: self.uuid_service.new_uuid("sales-person-version"),
            ..sales_person.clone()
        };

        self.sales_person_dao
            .update(
                &SalesPersonEntity::from(&sales_person),
                SALES_PERSON_SERVICE_PROCESS,
            )
            .await?;
        Ok(sales_person)
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let mut sales_person_entity = self
            .sales_person_dao
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        sales_person_entity.deleted = Some(self.clock_service.date_time_now());
        sales_person_entity.version = self.uuid_service.new_uuid("sales-person-version");
        self.sales_person_dao
            .update(&sales_person_entity, SALES_PERSON_SERVICE_PROCESS)
            .await?;
        Ok(())
    }

    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Arc<str>>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        Ok(self
            .sales_person_dao
            .get_assigned_user(sales_person_id)
            .await?)
    }

    async fn set_user(
        &self,
        sales_person_id: Uuid,
        user_id: Option<Arc<str>>,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        self.sales_person_dao
            .discard_assigned_user(sales_person_id)
            .await?;
        if let Some(user) = user_id {
            self.sales_person_dao
                .assign_to_user(sales_person_id, user.as_ref(), SALES_PERSON_SERVICE_PROCESS)
                .await?;
        }
        Ok(())
    }

    async fn get_sales_person_for_user(
        &self,
        user_id: Arc<str>,
        context: Authentication<Self::Context>,
    ) -> Result<Option<SalesPerson>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        Ok(self
            .sales_person_dao
            .find_sales_person_by_user_id(&user_id)
            .await?
            .as_ref()
            .map(SalesPerson::from))
    }

    async fn get_sales_person_current_user(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Option<SalesPerson>, ServiceError> {
        let current_user = if let Some(current_user) = self
            .permission_service
            .current_user_id(context.clone())
            .await?
        {
            current_user
        } else {
            return Ok(None);
        };
        Ok(self
            .get_sales_person_for_user(current_user, Authentication::Full)
            .await?)
    }
}
