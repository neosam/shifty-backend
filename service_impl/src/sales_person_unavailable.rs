use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{sales_person_unavailable::SalesPersonUnavailableDao, TransactionDao};
use service::{
    clock::ClockService,
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    sales_person::SalesPersonService,
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct SalesPersonUnavailableServiceImpl: SalesPersonUnavailableService = SalesPersonUnavailableServiceDeps {
        SalesPersonUnavailableDao: SalesPersonUnavailableDao<Transaction = Self::Transaction> = sales_person_unavailable_dao,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}
#[async_trait]
impl<Deps: SalesPersonUnavailableServiceDeps> SalesPersonUnavailableService
    for SalesPersonUnavailableServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_all_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
        );
        shiftplanner_permission.or(is_sales_person)?;

        let ret = self
            .sales_person_unavailable_dao
            .find_all_by_sales_person_id(sales_person_id, tx.clone())
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect();

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn get_by_week_for_sales_person(
        &self,
        sales_person_id: Uuid,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
        );
        shiftplanner_permission.or(is_sales_person)?;

        let ret = self
            .sales_person_unavailable_dao
            .find_by_week_and_sales_person_id(
                sales_person_id,
                year,
                calendar_week,
                tx.clone().into(),
            )
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect();

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn get_by_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPersonUnavailable]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let ret = self
            .sales_person_unavailable_dao
            .find_by_week(year, calendar_week, tx.clone())
            .await?
            .iter()
            .map(|entity| Ok(SalesPersonUnavailable::from(entity)))
            .collect();

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn create(
        &self,
        entity: &SalesPersonUnavailable,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<SalesPersonUnavailable, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                entity.sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
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
                tx.clone(),
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
            .create(&entity, "SalesPersonUnavailableService::create", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok((&entity).into())
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self
            .sales_person_unavailable_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;

        let (shiftplanner_permission, is_sales_person) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                entity.sales_person_id,
                context.clone(),
                tx.clone().into()
            ),
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
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
