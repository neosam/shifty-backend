use std::sync::Arc;

use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{
    custom_extra_hours::{CustomExtraHoursDao, CustomExtraHoursEntity},
    TransactionDao,
};
use service::{
    clock::ClockService,
    custom_extra_hours::{CustomExtraHours, CustomExtraHoursService},
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

const CARRYOVER_SERVICE_PROCESS: &str = "custom-extra-hours-service";

gen_service_impl! {
    struct CustomExtraHoursServiceImpl: CustomExtraHoursService = CustomExtraHoursDeps {
        CustomExtraHoursDao: CustomExtraHoursDao<Transaction = Self::Transaction> = custom_extra_hours_dao,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        UuidService: UuidService = uuid_service,
        ClockService: ClockService = clock_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: CustomExtraHoursDeps> CustomExtraHoursService for CustomExtraHoursServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[CustomExtraHours]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;
        let res = Ok(self
            .custom_extra_hours_dao
            .find_all(tx.clone())
            .await?
            .iter()
            .filter(|entity| entity.deleted.is_none())
            .map(CustomExtraHours::from)
            .collect());

        self.transaction_dao.commit(tx).await?;
        res
    }

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;
        let entity = self
            .custom_extra_hours_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let res = Ok(CustomExtraHours::from(&entity));

        self.transaction_dao.commit(tx).await?;
        res
    }

    async fn get_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[CustomExtraHours]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context.clone(),
                tx.clone().into()
            )
        );
        hr_permission.or(sales_person_permission)?;

        let entity = self
            .custom_extra_hours_dao
            .find_by_sales_person_id(sales_person_id, tx.clone())
            .await?
            .iter()
            .filter(|entity| entity.deleted.is_none())
            .map(CustomExtraHours::from)
            .collect();
        let res = Ok(entity);

        self.transaction_dao.commit(tx).await?;
        res
    }

    async fn create(
        &self,
        extra_hours: &CustomExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        if !extra_hours.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !extra_hours.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        if extra_hours.deleted.is_some() {
            return Err(ServiceError::DeletedSetOnCreate);
        }
        if extra_hours.created.is_some() {
            return Err(ServiceError::CreatedSetOnCreate);
        }

        let mut extra_hours = extra_hours.clone();
        extra_hours.created = Some(self.clock_service.date_time_now());
        let mut entity: CustomExtraHoursEntity = extra_hours.try_into()?;
        entity.id = self.uuid_service.new_uuid("create-id");
        entity.version = self.uuid_service.new_uuid("create-version");

        self.custom_extra_hours_dao
            .create(&entity, CARRYOVER_SERVICE_PROCESS, tx.clone())
            .await?;
        let res = Ok((&entity).into());

        self.transaction_dao.commit(tx).await?;
        res
    }

    async fn update(
        &self,
        custom_extra_hours: &CustomExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<CustomExtraHours, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let mut entity = self
            .custom_extra_hours_dao
            .find_by_id(custom_extra_hours.id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(custom_extra_hours.id))?;
        if entity.version != custom_extra_hours.version {
            return Err(ServiceError::EntityConflicts(
                custom_extra_hours.id,
                custom_extra_hours.version,
                entity.version,
            ));
        }
        entity.version = self.uuid_service.new_uuid("update-version");

        self.custom_extra_hours_dao
            .update(
                custom_extra_hours.try_into()?,
                CARRYOVER_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;

        let res = Ok(entity.into());

        self.transaction_dao.commit(tx).await?;
        res
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let mut entity = self
            .custom_extra_hours_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;

        entity.deleted = Some(self.clock_service.date_time_now());

        self.custom_extra_hours_dao
            .update(entity, CARRYOVER_SERVICE_PROCESS, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
