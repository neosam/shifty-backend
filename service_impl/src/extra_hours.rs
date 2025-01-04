use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    extra_hours::{self, ExtraHoursDao},
    TransactionDao,
};
use service::{
    clock::ClockService,
    extra_hours::{ExtraHours, ExtraHoursService},
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct ExtraHoursServiceImpl: ExtraHoursService = ExtraHoursServiceDeps {
        ExtraHoursDao: ExtraHoursDao<Transaction = Self::Transaction> = extra_hours_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: ExtraHoursServiceDeps> ExtraHoursService for ExtraHoursServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn find_by_sales_person_id_and_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        until_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_permission.or(sales_person_permission)?;

        let extra_hours_entities = self
            .extra_hours_dao
            .find_by_sales_person_id_and_year(sales_person_id, year, tx.clone())
            .await?;
        let extra_hours = extra_hours_entities
            .iter()
            .filter(|extra_hours| extra_hours.date_time.iso_week() <= until_week)
            .map(ExtraHours::from)
            .collect::<Vec<ExtraHours>>();
        self.transaction_dao.commit(tx).await?;
        Ok(extra_hours.into())
    }

    async fn find_by_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ExtraHours]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_only_full_authentication(context)
            .await?;
        let ret = Ok(self
            .extra_hours_dao
            .find_by_week(week, year, tx.clone())
            .await?
            .iter()
            .map(ExtraHours::from)
            .collect());

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn create(
        &self,
        extra_hours: &ExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ExtraHours, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                extra_hours.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_permission.or(sales_person_permission)?;

        let mut extra_hours = extra_hours.to_owned();
        if !extra_hours.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !extra_hours.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        extra_hours.id = self.uuid_service.new_uuid("extra_hours_service::create id");
        extra_hours.version = self
            .uuid_service
            .new_uuid("extra_hours_service::create version");
        extra_hours.created = Some(self.clock_service.date_time_now());

        let extra_hours_entity = extra_hours::ExtraHoursEntity::try_from(&extra_hours)?;
        self.extra_hours_dao
            .create(
                &extra_hours_entity,
                "extra_hours_service::create",
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(extra_hours.into())
    }
    async fn update(
        &self,
        _entity: &ExtraHours,
        _context: Authentication<Self::Context>,
    ) -> Result<ExtraHours, ServiceError> {
        unimplemented!()
    }

    async fn delete(
        &self,
        extra_hours_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
        );
        hr_permission.or(sales_person_permission)?;

        let mut extra_hours_entity = self
            .extra_hours_dao
            .find_by_id(extra_hours_id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(extra_hours_id))?;

        let (hr_permission, user_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                extra_hours_entity.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_permission.or(user_permission)?;

        extra_hours_entity.deleted = Some(self.clock_service.date_time_now());

        self.extra_hours_dao
            .update(
                &extra_hours_entity,
                "extra_hours_service::delete",
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
