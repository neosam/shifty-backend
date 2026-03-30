use async_trait::async_trait;
use dao::{shiftplan::ShiftplanDao, TransactionDao};
use service::{
    clock::ClockService,
    permission::{Authentication, PermissionService, SHIFTPLANNER_PRIVILEGE},
    shiftplan_catalog::{Shiftplan, ShiftplanService},
    uuid_service::UuidService,
    ServiceError,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct ShiftplanServiceImpl: service::shiftplan_catalog::ShiftplanService = ShiftplanServiceDeps {
        ShiftplanDao: dao::shiftplan::ShiftplanDao<Transaction = Self::Transaction> = shiftplan_dao,
        PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        ClockService: service::clock::ClockService = clock_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: ShiftplanServiceDeps> ShiftplanService for ShiftplanServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_all(
        &self,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Shiftplan]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self.shiftplan_dao.all(tx.clone()).await?;
        let result = entities.iter().map(Shiftplan::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn get_by_id(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self
            .shiftplan_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let result = Shiftplan::from(&entity);
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn create(
        &self,
        shiftplan: &Shiftplan,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        if shiftplan.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if shiftplan.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        let new_shiftplan = Shiftplan {
            id: self.uuid_service.new_uuid("shiftplan-id"),
            version: self.uuid_service.new_uuid("shiftplan-version"),
            ..shiftplan.clone()
        };
        let entity = dao::shiftplan::ShiftplanEntity::from(&new_shiftplan);
        self.shiftplan_dao
            .create(&entity, "shiftplan-service", tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(new_shiftplan)
    }

    async fn update(
        &self,
        shiftplan: &Shiftplan,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Shiftplan, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let persisted = self
            .shiftplan_dao
            .find_by_id(shiftplan.id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(shiftplan.id))?;

        if persisted.version != shiftplan.version {
            return Err(ServiceError::EntityConflicts(
                shiftplan.id,
                persisted.version,
                shiftplan.version,
            ));
        }

        let updated = Shiftplan {
            version: self.uuid_service.new_uuid("shiftplan-version"),
            ..shiftplan.clone()
        };
        let entity = dao::shiftplan::ShiftplanEntity::from(&updated);
        self.shiftplan_dao
            .update(&entity, "shiftplan-service", tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(updated)
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let mut entity = self
            .shiftplan_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;

        entity.deleted = Some(self.clock_service.date_time_now());
        self.shiftplan_dao
            .update(&entity, "shiftplan-service", tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
