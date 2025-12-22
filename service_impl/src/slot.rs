use std::sync::Arc;

use async_trait::async_trait;
use service::{
    permission::{Authentication, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    slot::Slot,
    ServiceError, ValidationFailureItem,
};
use tokio::join;
use tracing::instrument;
use uuid::Uuid;

const SLOT_SERVICE_PROCESS: &str = "slot-service";

pub struct SlotServiceImpl<SlotDao, PermissionService, ClockService, UuidService, TransactionDao>
where
    SlotDao: dao::slot::SlotDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    pub slot_dao: Arc<SlotDao>,
    pub permission_service: Arc<PermissionService>,
    pub clock_service: Arc<ClockService>,
    pub uuid_service: Arc<UuidService>,
    pub transaction_dao: Arc<TransactionDao>,
}
impl<SlotDao, PermissionService, ClockService, UuidService, TransactionDao>
    SlotServiceImpl<SlotDao, PermissionService, ClockService, UuidService, TransactionDao>
where
    SlotDao: dao::slot::SlotDao + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    pub fn new(
        slot_dao: Arc<SlotDao>,
        permission_service: Arc<PermissionService>,
        clock_service: Arc<ClockService>,
        uuid_service: Arc<UuidService>,
        transaction_dao: Arc<TransactionDao>,
    ) -> Self {
        Self {
            slot_dao,
            permission_service,
            clock_service,
            uuid_service,
            transaction_dao,
        }
    }
}

pub fn test_overlapping_slots(slot_1: &Slot, slot_2: &Slot) -> bool {
    slot_1.day_of_week == slot_2.day_of_week
        && (slot_2.from < slot_1.from && slot_1.from < slot_2.to
            || slot_1.from < slot_2.from && slot_2.from < slot_1.to
            || slot_1.from == slot_2.from && slot_1.to == slot_2.to)
}

#[async_trait]
impl<SlotDao, PermissionService, ClockService, UuidService, TransactionDao>
    service::slot::SlotService
    for SlotServiceImpl<SlotDao, PermissionService, ClockService, UuidService, TransactionDao>
where
    SlotDao: dao::slot::SlotDao<Transaction = TransactionDao::Transaction> + Send + Sync,
    PermissionService: service::permission::PermissionService + Send + Sync,
    ClockService: service::clock::ClockService + Send + Sync,
    UuidService: service::uuid_service::UuidService + Send + Sync,
    TransactionDao: dao::TransactionDao + Send + Sync,
{
    type Context = PermissionService::Context;
    type Transaction = TransactionDao::Transaction;

    #[instrument(name = "SlotServiceImpl::get_slots_for_week", skip(self, tx))]
    async fn get_slots(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Slot]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, sales_permission) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context),
        );
        shiftplanner_permission.or(sales_permission)?;

        let slots = self
            .slot_dao
            .get_slots(tx.clone())
            .await?
            .iter()
            .map(Slot::from)
            .collect();

        self.transaction_dao.commit(tx).await?;
        Ok(slots)
    }

    async fn get_slot(
        &self,
        id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, sales_permission) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context),
        );
        shiftplanner_permission.or(sales_permission)?;

        let slot_entity = self.slot_dao.get_slot(id, tx.clone()).await?;
        let slot = slot_entity
            .as_ref()
            .map(Slot::from)
            .ok_or_else(move || ServiceError::EntityNotFound(*id))?;

        self.transaction_dao.commit(tx).await?;
        Ok(slot)
    }

    #[instrument(name = "SlotServiceImpl::get_slots_for_week", skip(self, tx))]
    async fn get_slots_for_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Slot]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner_permission, sales_permission) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context),
        );
        shiftplanner_permission.or(sales_permission)?;
        tracing::info!("Getting slots for week {} of year {}", week, year);

        let slots = self
            .slot_dao
            .get_slots_for_week(year, week, tx.clone())
            .await?
            .iter()
            .map(Slot::from)
            .collect();

        self.transaction_dao.commit(tx).await?;
        Ok(slots)
    }

    async fn exists(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let slot = self
            .slot_dao
            .get_slot(&id, tx.clone())
            .await
            .map(|s| s.is_some())?;
        self.transaction_dao.commit(tx).await?;
        Ok(slot)
    }

    async fn create_slot(
        &self,
        slot: &Slot,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await?;

        if slot.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if slot.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }
        if slot.from > slot.to {
            return Err(ServiceError::TimeOrderWrong(slot.from, slot.to));
        }
        if slot.valid_to.is_some() && slot.valid_to.unwrap() < slot.valid_from {
            return Err(ServiceError::DateOrderWrong(
                slot.valid_from,
                slot.valid_to.unwrap(),
            ));
        }

        if self
            .get_slots(context, tx.clone().into())
            .await?
            .iter()
            .filter(|s| {
                !(s.valid_from > slot.valid_to.unwrap_or(s.valid_from)
                    || slot.valid_from > s.valid_to.unwrap_or(slot.valid_from))
            })
            .any(|s| test_overlapping_slots(slot, s))
        {
            return Err(ServiceError::OverlappingTimeRange);
        }

        let slot = Slot {
            id: self.uuid_service.new_uuid("slot-id"),
            version: self.uuid_service.new_uuid("slot-version"),
            ..slot.clone()
        };
        self.slot_dao
            .create_slot(&(&slot).into(), SLOT_SERVICE_PROCESS, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(slot)
    }

    async fn delete_slot(
        &self,
        id: &Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let mut slot = self
            .slot_dao
            .get_slot(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(*id))?;
        slot.deleted = Some(self.clock_service.date_time_now());
        self.slot_dao
            .update_slot(&slot, SLOT_SERVICE_PROCESS, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn update_slot(
        &self,
        slot: &Slot,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;
        let persisted_slot = self
            .slot_dao
            .get_slot(&slot.id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(slot.id))?;
        if persisted_slot.version != slot.version {
            return Err(ServiceError::EntityConflicts(
                slot.id,
                persisted_slot.version,
                slot.version,
            ));
        }
        if slot.valid_to.is_some() && slot.valid_to.unwrap() < slot.valid_from {
            return Err(ServiceError::DateOrderWrong(
                slot.valid_from,
                slot.valid_to.unwrap(),
            ));
        }

        let mut validation = Vec::new();
        if persisted_slot.day_of_week != slot.day_of_week {
            validation.push(ValidationFailureItem::ModificationNotAllowed(
                "day_of_week".into(),
            ));
        }
        if persisted_slot.from != slot.from {
            validation.push(ValidationFailureItem::ModificationNotAllowed("from".into()));
        }
        if persisted_slot.to != slot.to {
            validation.push(ValidationFailureItem::ModificationNotAllowed("to".into()));
        }
        if persisted_slot.valid_from != slot.valid_from {
            validation.push(ValidationFailureItem::ModificationNotAllowed(
                "valid_from".into(),
            ));
        }
        /*if persisted_slot.valid_to.is_some() && persisted_slot.valid_to != slot.valid_to {
            validation.push(ValidationFailureItem::ModificationNotAllowed(
                "valid_to".into(),
            ));
        }*/

        if !validation.is_empty() {
            return Err(ServiceError::ValidationError(validation.into()));
        }

        let slot = Slot {
            version: self.uuid_service.new_uuid("slot-version"),
            ..slot.clone()
        };
        self.slot_dao
            .update_slot(&(&slot).into(), SLOT_SERVICE_PROCESS, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
