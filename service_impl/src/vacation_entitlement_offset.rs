use crate::gen_service_impl;

use async_trait::async_trait;
use dao::{vacation_entitlement_offset::VacationEntitlementOffsetDao, TransactionDao};
use service::{
    clock::ClockService,
    permission::{Authentication, HR_PRIVILEGE},
    uuid_service::UuidService,
    vacation_entitlement_offset::{VacationEntitlementOffset, VacationEntitlementOffsetService},
    PermissionService, ServiceError,
};
use uuid::Uuid;

gen_service_impl! {
    struct VacationEntitlementOffsetServiceImpl: VacationEntitlementOffsetService = VacationEntitlementOffsetServiceDeps {
        VacationEntitlementOffsetDao: VacationEntitlementOffsetDao<Transaction = Self::Transaction> = vacation_entitlement_offset_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: VacationEntitlementOffsetServiceDeps> VacationEntitlementOffsetService
    for VacationEntitlementOffsetServiceImpl<Deps>
{
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<VacationEntitlementOffset>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let result = self
            .vacation_entitlement_offset_dao
            .find_by_sales_person_id_and_year(sales_person_id, year, tx.clone())
            .await?
            .as_ref()
            .map(VacationEntitlementOffset::from);

        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn set(
        &self,
        sales_person_id: Uuid,
        year: u32,
        offset_days: i32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VacationEntitlementOffset, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let existing = self
            .vacation_entitlement_offset_dao
            .find_by_sales_person_id_and_year(sales_person_id, year, tx.clone())
            .await?;

        let entity = if let Some(existing) = existing {
            let entity = dao::vacation_entitlement_offset::VacationEntitlementOffsetEntity {
                offset_days,
                version: self
                    .uuid_service
                    .new_uuid("vacation-entitlement-offset-service::update version"),
                ..existing
            };
            self.vacation_entitlement_offset_dao
                .update(&entity, "vacation-entitlement-offset-service::update", tx.clone())
                .await?;
            entity
        } else {
            let entity = dao::vacation_entitlement_offset::VacationEntitlementOffsetEntity {
                id: self
                    .uuid_service
                    .new_uuid("vacation-entitlement-offset-service::create id"),
                sales_person_id,
                year,
                offset_days,
                created: self.clock_service.date_time_now(),
                deleted: None,
                version: self
                    .uuid_service
                    .new_uuid("vacation-entitlement-offset-service::create version"),
            };
            self.vacation_entitlement_offset_dao
                .create(&entity, "vacation-entitlement-offset-service::create", tx.clone())
                .await?;
            entity
        };

        self.transaction_dao.commit(tx).await?;
        Ok((&entity).into())
    }

    async fn delete(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let existing = self
            .vacation_entitlement_offset_dao
            .find_by_sales_person_id_and_year(sales_person_id, year, tx.clone())
            .await?
            .ok_or_else(|| {
                ServiceError::EntityNotFoundGeneric(
                    format!("vacation_entitlement_offset for {sales_person_id}/{year}").into(),
                )
            })?;

        let entity = dao::vacation_entitlement_offset::VacationEntitlementOffsetEntity {
            deleted: Some(self.clock_service.date_time_now()),
            version: self
                .uuid_service
                .new_uuid("vacation-entitlement-offset-service::delete version"),
            ..existing
        };
        self.vacation_entitlement_offset_dao
            .update(&entity, "vacation-entitlement-offset-service::delete", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
