use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    extra_hours::{self, ExtraHoursDao},
    TransactionDao,
};
use service::{
    clock::ClockService,
    cutover::CUTOVER_ADMIN_PRIVILEGE,
    custom_extra_hours::CustomExtraHoursService,
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    feature_flag::FeatureFlagService,
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError, ValidationFailureItem,
};
use shifty_utils::{DayOfWeek, ShiftyDate, ShiftyWeek};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct ExtraHoursServiceImpl: ExtraHoursService = ExtraHoursServiceDeps {
        ExtraHoursDao: ExtraHoursDao<Transaction = Self::Transaction> = extra_hours_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        CustomExtraHoursService: CustomExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = custom_extra_hours_service,
        FeatureFlagService: FeatureFlagService<Context = Self::Context, Transaction = Self::Transaction> = feature_flag_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: ExtraHoursServiceDeps> ExtraHoursServiceImpl<Deps> {
    /// Helper method to load custom extra hours definitions for lazy loaded entries
    async fn load_custom_extra_hours_definitions(
        &self,
        extra_hours_list: &mut [ExtraHours],
        tx: <Self as ExtraHoursService>::Transaction,
    ) -> Result<(), ServiceError> {
        for eh in extra_hours_list.iter_mut() {
            if let service::extra_hours::ExtraHoursCategory::CustomExtraHours(lazy_load) =
                &mut eh.category
            {
                if !lazy_load.is_loaded() {
                    let key = *lazy_load.key();
                    // Using Authentication::Full for internal service calls to fetch definitions
                    match self
                        .custom_extra_hours_service
                        .get_by_id(key, Authentication::Full, tx.clone().into())
                        .await
                    {
                        Ok(definition) => {
                            lazy_load.set(definition);
                        }
                        Err(ServiceError::EntityNotFound(_)) => {
                            // Log this? If a CustomExtraHour refers to a non-existent definition, it's an integrity issue.
                            // For now, it will remain unloaded, and .get() will return None.
                            tracing::warn!("CustomExtraHoursDefinition with id {} not found for ExtraHours entry {}", key, eh.id);
                        }
                        Err(e) => {
                            // For other errors, we might want to propagate them.
                            // Rolling back or failing the whole operation might be too drastic for a reporting query.
                            // Logging and continuing seems reasonable for now.
                            tracing::error!("Error loading CustomExtraHoursDefinition with id {} for ExtraHours entry {}: {:?}", key, eh.id, e);
                            // Potentially return the error: return Err(e);
                        }
                    }
                }
            }
        }
        Ok(())
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
        Ok(self
            .find_by_sales_person_id_and_year_range(
                sales_person_id,
                ShiftyDate::first_day_in_year(year),
                ShiftyWeek::new(year, until_week).as_date(DayOfWeek::Sunday),
                context,
                tx,
            )
            .await?
            .iter()
            .filter(|extra_hours| extra_hours.date_time.year() == year as i32)
            .cloned()
            .collect::<Vec<_>>()
            .into())
    }

    async fn find_by_sales_person_id_and_year_range(
        &self,
        sales_person_id: Uuid,
        from_date: ShiftyDate,
        to_date: ShiftyDate,
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
            .find_by_sales_person_id_and_years(
                sales_person_id,
                from_date.year(),
                to_date.year(),
                tx.clone(),
            )
            .await?;

        let mut extra_hours_list = extra_hours_entities
            .iter()
            .filter(|extra_hours| {
                extra_hours.as_date() >= from_date && extra_hours.as_date() <= to_date
            })
            .map(ExtraHours::from)
            .collect::<Vec<ExtraHours>>();

        self.load_custom_extra_hours_definitions(&mut extra_hours_list, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(extra_hours_list.into())
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
        let mut extra_hours_list = self
            .extra_hours_dao
            .find_by_week(week, year, tx.clone())
            .await?
            .iter()
            .map(ExtraHours::from)
            .collect::<Vec<ExtraHours>>();

        self.load_custom_extra_hours_definitions(&mut extra_hours_list, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(extra_hours_list.into())
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
                context.clone(),
                tx.clone().into()
            ),
        );
        hr_permission.or(sales_person_permission)?;

        // Phase-4 D-Phase4-09 service-layer flag-gate: once
        // `absence_range_source_active` is true (post-cutover), creating new
        // Vacation/SickLeave/UnpaidLeave entries via this surface is
        // deprecated — clients must use POST /absence-period instead.
        // ExtraWork/Holiday/Unavailable/VolunteerWork/Custom remain
        // unaffected by the gate. The check happens AFTER the permission gate
        // (so unauthorized callers still get Forbidden, not Deprecated) and
        // BEFORE the DAO insert (so a deprecated request makes no state
        // change; the Tx rolls back via Drop on the early Err).
        if matches!(
            extra_hours.category,
            ExtraHoursCategory::Vacation
                | ExtraHoursCategory::SickLeave
                | ExtraHoursCategory::UnpaidLeave
        ) {
            let flag_active = self
                .feature_flag_service
                .is_enabled(
                    "absence_range_source_active",
                    Authentication::Full,
                    Some(tx.clone()),
                )
                .await?;
            if flag_active {
                return Err(ServiceError::ExtraHoursCategoryDeprecated(Box::new(
                    extra_hours.category.clone(),
                )));
            }
        }

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
        Ok(extra_hours)
    }
    async fn update(
        &self,
        request: &ExtraHours,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ExtraHours, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let logical_id = request.id;

        let active = self
            .extra_hours_dao
            .find_by_logical_id(logical_id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(logical_id))?;

        let (hr_permission, sales_person_permission) = join!(
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone()),
            self.sales_person_service.verify_user_is_sales_person(
                active.sales_person_id,
                context,
                tx.clone().into()
            ),
        );
        hr_permission.or(sales_person_permission)?;

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

        let mut tombstone = active.clone();
        tombstone.deleted = Some(self.clock_service.date_time_now());
        self.extra_hours_dao
            .update(
                &tombstone,
                "extra_hours_service::update::soft_delete",
                tx.clone(),
            )
            .await?;

        let new_id = self
            .uuid_service
            .new_uuid("extra_hours_service::update::id");
        let new_version = self
            .uuid_service
            .new_uuid("extra_hours_service::update::version");
        let now = self.clock_service.date_time_now();

        let new_entity = extra_hours::ExtraHoursEntity {
            id: new_id,
            logical_id: active.logical_id,
            sales_person_id: active.sales_person_id,
            amount: request.amount,
            category: (&request.category).into(),
            description: request.description.clone(),
            date_time: request.date_time,
            created: now,
            deleted: None,
            version: new_version,
        };
        self.extra_hours_dao
            .create(
                &new_entity,
                "extra_hours_service::update::insert",
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(ExtraHours::from(&new_entity))
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
            .find_by_logical_id(extra_hours_id, tx.clone())
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

    /// Phase 4 / C-Phase4-04 — bulk soft-delete by id list.
    ///
    /// Permission-gates `CUTOVER_ADMIN_PRIVILEGE` BEFORE doing any DAO work.
    /// This ordering is verified by `soft_delete_bulk_forbidden_for_unprivileged_user`
    /// (T-04-04-01 Elevation-of-Privilege mitigation): the test pins
    /// `MockExtraHoursDao::expect_soft_delete_bulk().times(0)` so it fails
    /// if the impl ever calls the DAO before the permission check denies.
    ///
    /// Caller is the cutover commit phase (Plan 04-05). The Tx is provided by
    /// the caller and NOT committed here — the cutover service holds the Tx
    /// until the final atomic commit.
    async fn soft_delete_bulk(
        &self,
        ids: Arc<[Uuid]>,
        update_process: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Permission gate FIRST — strictly BEFORE any tx/DAO work.
        self.permission_service
            .check_permission(CUTOVER_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let now = self.clock_service.date_time_now();
        let new_version = self
            .uuid_service
            .new_uuid("extra_hours_service::soft_delete_bulk version");

        self.extra_hours_dao
            .soft_delete_bulk(&ids, now, update_process, new_version, tx.clone())
            .await?;

        // Tx held by caller (cutover commit phase, Plan 04-05) — DO NOT commit
        // here. `use_transaction` returns the caller-provided Tx untouched
        // when `tx` is `Some(_)` per Pattern-1 Tx-forwarding contract.
        Ok(())
    }
}
