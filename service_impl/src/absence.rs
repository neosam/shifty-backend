//! Service-Impl der Absence-Domain (Phase 1).
//!
//! Wiring per `gen_service_impl!` (Option A — minimaler Dependency-Set: nur
//! `AbsenceDao`, `PermissionService`, `SalesPersonService`, `ClockService`,
//! `UuidService`, `TransactionDao`; siehe D-08 und D-10 für die ausgeschlossenen
//! Hilfs-Services). Schreib- und Read-Methoden nutzen
//! `tokio::join!(check_permission(HR), verify_user_is_sales_person(...))` mit
//! `or` (D-09). `create` und `update` validieren Range (`DateRange::new` →
//! `DateOrderWrong`, D-14) und Self-Overlap via `find_overlapping`. Der
//! `update`-Pfad folgt 1:1 dem ExtraHours-`logical_id`-Pattern (Tombstone +
//! Insert, D-07) und exkludiert die alte Row beim Self-Overlap-Check
//! (`Some(logical_id)`, D-15). `delete` ist Soft-Delete via
//! `update(tombstone)`.

use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    absence::{self, AbsenceDao},
    TransactionDao,
};
use service::{
    absence::{AbsencePeriod, AbsenceService},
    clock::ClockService,
    permission::{Authentication, HR_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::DateRange;
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct AbsenceServiceImpl: AbsenceService = AbsenceServiceDeps {
        AbsenceDao: AbsenceDao<Transaction = Self::Transaction> = absence_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: AbsenceServiceDeps> AbsenceService for AbsenceServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn find_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let entities = self.absence_dao.find_all(tx.clone()).await?;
        let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let entities = self
            .absence_dao
            .find_by_sales_person(sales_person_id, tx.clone())
            .await?;
        let result: Arc<[AbsencePeriod]> = entities.iter().map(AbsencePeriod::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let active = self
            .absence_dao
            .find_by_logical_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;
        let result = AbsencePeriod::from(&active);
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn create(
        &self,
        request: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                request.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let mut entity = request.to_owned();
        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        if entity.deleted.is_some() {
            return Err(ServiceError::DeletedSetOnCreate);
        }
        if entity.created.is_some() {
            return Err(ServiceError::CreatedSetOnCreate);
        }

        let new_range = DateRange::new(entity.from_date, entity.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(entity.from_date, entity.to_date))?;

        // exclude_logical_id: None (Create-Pfad — keine eigene Row zu exkludieren).
        let conflicts = self
            .absence_dao
            .find_overlapping(
                entity.sales_person_id,
                (&entity.category).into(),
                new_range,
                None, // exclude_logical_id: None for create — there is no own row yet.
                tx.clone(),
            )
            .await?;
        if !conflicts.is_empty() {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
            ])));
        }

        entity.id = self.uuid_service.new_uuid("absence_service::create::id");
        entity.version = self
            .uuid_service
            .new_uuid("absence_service::create::version");
        entity.created = Some(self.clock_service.date_time_now());

        let dao_entity = absence::AbsencePeriodEntity::try_from(&entity)?;
        self.absence_dao
            .create(&dao_entity, "absence_service::create", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(entity)
    }

    async fn update(
        &self,
        request: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let logical_id = request.id;

        let active = self
            .absence_dao
            .find_by_logical_id(logical_id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(logical_id))?;

        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        if request.sales_person_id != active.sales_person_id {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::ModificationNotAllowed("sales_person_id".into()),
            ])));
        }
        if request.version != active.version {
            return Err(ServiceError::EntityConflicts(
                logical_id,
                request.version,
                active.version,
            ));
        }

        let new_range = DateRange::new(request.from_date, request.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(request.from_date, request.to_date))?;

        let conflicts = self
            .absence_dao
            .find_overlapping(
                active.sales_person_id,
                (&request.category).into(),
                new_range,
                Some(logical_id),
                tx.clone(),
            )
            .await?;
        if !conflicts.is_empty() {
            return Err(ServiceError::ValidationError(Arc::from([
                ValidationFailureItem::OverlappingPeriod(conflicts[0].logical_id),
            ])));
        }

        let mut tombstone = active.clone();
        tombstone.deleted = Some(self.clock_service.date_time_now());
        self.absence_dao
            .update(
                &tombstone,
                "absence_service::update::soft_delete",
                tx.clone(),
            )
            .await?;

        let new_id = self.uuid_service.new_uuid("absence_service::update::id");
        let new_version = self
            .uuid_service
            .new_uuid("absence_service::update::version");
        let now = self.clock_service.date_time_now();

        let new_entity = absence::AbsencePeriodEntity {
            id: new_id,
            logical_id: active.logical_id,
            sales_person_id: active.sales_person_id,
            category: (&request.category).into(),
            from_date: request.from_date,
            to_date: request.to_date,
            description: request.description.clone(),
            created: now,
            deleted: None,
            version: new_version,
        };
        self.absence_dao
            .create(&new_entity, "absence_service::update::insert", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(AbsencePeriod::from(&new_entity))
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let active = self
            .absence_dao
            .find_by_logical_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        let (hr, sp) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr.or(sp)?;

        let mut tombstone = active;
        tombstone.deleted = Some(self.clock_service.date_time_now());
        self.absence_dao
            .update(&tombstone, "absence_service::delete", tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
