//! Phase 54 (D-54-DM-01): Basic-Tier Rebooking-Batch Service.
//!
//! Konsumiert AUSSCHLIESSLICH DAO + Permission + Clock + Uuid + Transaction —
//! kein Domain-Service als Dependency (Service-Tier-Konvention CLAUDE.md).
//! Ab Phase 55 wird dieser Service vom Business-Logic
//! `RebookingReconciliationService` konsumiert.

use std::sync::Arc;

use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{
    rebooking_batch::{
        RebookingBatchDao, RebookingBatchEntity, RebookingBatchEntryEntity, RebookingBatchState,
    },
    TransactionDao,
};
use service::{
    clock::ClockService,
    permission::{Authentication, HR_PRIVILEGE},
    rebooking_batch::RebookingBatchService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use uuid::Uuid;

const REBOOKING_BATCH_SERVICE_PROCESS: &str = "rebooking-batch-service";

gen_service_impl! {
    struct RebookingBatchServiceImpl: RebookingBatchService = RebookingBatchServiceDeps {
        RebookingBatchDao: RebookingBatchDao<Transaction = Self::Transaction> = rebooking_batch_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: RebookingBatchServiceDeps> RebookingBatchService for RebookingBatchServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError> {
        // HR-Gate an erster Stelle (T-54-01).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.rebooking_batch_dao.find_by_id(id, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn find_by_sales_person_year_week(
        &self,
        sales_person_id: Uuid,
        iso_year: u32,
        iso_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<RebookingBatchEntity>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self
            .rebooking_batch_dao
            .find_by_sales_person_year_week(sales_person_id, iso_year, iso_week, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    /// Schreibt Batch + Entries in EINER Transaktion.
    ///
    /// D-54-DM-01: Der UNIQUE-Partial-Index `rebooking_batch_week_unique_idx`
    /// (Migration `20260707000000`) enforced die Eindeutigkeit auf
    /// `(sales_person_id, iso_year, iso_week) WHERE deleted IS NULL`. Der
    /// Service macht innerhalb der Transaktion einen Pre-Check gegen den DAO
    /// (`find_by_sales_person_year_week`) — findet er einen aktiven Batch,
    /// gibt er `ServiceError::EntityAlreadyExists` zurueck, ohne das DAO-INSERT
    /// zu versuchen. Der DB-Index bleibt die Ultima-Ratio-Autoritaet und
    /// verhindert Race-Conditions bei parallelen Aufrufen ausserhalb einer
    /// SERIALIZABLE-Transaktion.
    async fn create(
        &self,
        batch: &RebookingBatchEntity,
        entries: &[RebookingBatchEntryEntity],
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<RebookingBatchEntity, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // D-54-DM-01: Pre-Check innerhalb derselben Transaktion.
        if let Some(existing) = self
            .rebooking_batch_dao
            .find_by_sales_person_year_week(
                batch.sales_person_id,
                batch.iso_year,
                batch.iso_week,
                tx.clone(),
            )
            .await?
        {
            return Err(ServiceError::EntityAlreadyExists(existing.id));
        }

        // Defensiv: id/version/created fuellen, falls Aufrufer Default-Werte
        // liefert. (In Phase 55/56 kommen die Werte typischerweise vom
        // Cron/HR-Suggest-Pfad und sind gesetzt; die Basic-Manager-Ebene
        // stellt aber eine minimale Konsistenz her.)
        let now = self.clock_service.date_time_now();
        let new_batch = RebookingBatchEntity {
            id: if batch.id == Uuid::nil() {
                self.uuid_service.new_uuid(&format!(
                    "{REBOOKING_BATCH_SERVICE_PROCESS}::create batch id"
                ))
            } else {
                batch.id
            },
            sales_person_id: batch.sales_person_id,
            iso_year: batch.iso_year,
            iso_week: batch.iso_week,
            kind: batch.kind,
            state: batch.state,
            created: if batch.created == time::PrimitiveDateTime::MIN {
                now
            } else {
                batch.created
            },
            approved: batch.approved,
            approved_by: batch.approved_by.clone(),
            deleted: batch.deleted,
            version: if batch.version == Uuid::nil() {
                self.uuid_service.new_uuid(&format!(
                    "{REBOOKING_BATCH_SERVICE_PROCESS}::create batch version"
                ))
            } else {
                batch.version
            },
        };

        let normalized_entries: Vec<RebookingBatchEntryEntity> = entries
            .iter()
            .map(|entry| RebookingBatchEntryEntity {
                id: if entry.id == Uuid::nil() {
                    self.uuid_service.new_uuid(&format!(
                        "{REBOOKING_BATCH_SERVICE_PROCESS}::create entry id"
                    ))
                } else {
                    entry.id
                },
                batch_id: if entry.batch_id == Uuid::nil() {
                    new_batch.id
                } else {
                    entry.batch_id
                },
                sales_person_id: entry.sales_person_id,
                hours: entry.hours,
                balance_before: entry.balance_before,
                voluntary_actual: entry.voluntary_actual,
                voluntary_committed: entry.voluntary_committed,
                extra_hours_out_id: entry.extra_hours_out_id,
                extra_hours_in_id: entry.extra_hours_in_id,
                created: if entry.created == time::PrimitiveDateTime::MIN {
                    now
                } else {
                    entry.created
                },
                deleted: entry.deleted,
                version: if entry.version == Uuid::nil() {
                    self.uuid_service.new_uuid(&format!(
                        "{REBOOKING_BATCH_SERVICE_PROCESS}::create entry version"
                    ))
                } else {
                    entry.version
                },
            })
            .collect();

        self.rebooking_batch_dao
            .create_batch_with_entries(
                &new_batch,
                &normalized_entries,
                REBOOKING_BATCH_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(new_batch)
    }

    async fn find_pending_for_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingBatchEntity]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self
            .rebooking_batch_dao
            .find_pending_for_sales_person(sales_person_id, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn list_all_pending(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[RebookingBatchEntity]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self
            .rebooking_batch_dao
            .list_all_pending(tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    /// Phase 55 (HR-ALERT-03, T-55-01): state-conditional UPDATE.
    ///
    /// Erzeugt intern eine frische `new_version` via UuidService — der DAO
    /// bekommt sie als expliziten Parameter. `rows_affected == 0` wird
    /// **nicht** als Fehler zurueckgegeben; der Aufrufer (`RebookingReconciliation
    /// Service::approve_suggestion` / `reject_suggestion`) entscheidet, wie
    /// er auf "already-transitioned" reagiert (typischerweise
    /// `ServiceError::BatchAlreadyResolved`).
    async fn update_state_conditional(
        &self,
        batch_id: Uuid,
        expected_state: RebookingBatchState,
        new_state: RebookingBatchState,
        approved: Option<time::PrimitiveDateTime>,
        approved_by: Option<Arc<str>>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<u64, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let new_version = self.uuid_service.new_uuid(&format!(
            "{REBOOKING_BATCH_SERVICE_PROCESS}::update_state_conditional version"
        ));

        let affected = self
            .rebooking_batch_dao
            .update_state_conditional(
                batch_id,
                expected_state,
                new_state,
                approved,
                approved_by,
                new_version,
                REBOOKING_BATCH_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(affected)
    }
}
