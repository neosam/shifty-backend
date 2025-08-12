use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    extra_hours::{self, ExtraHoursDao},
    TransactionDao,
};
use service::{
    clock::ClockService,
    custom_extra_hours::CustomExtraHoursService,
    extra_hours::{ExtraHours, ExtraHoursService},
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE},
    sales_person::SalesPersonService,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use shifty_utils::{ShiftyDate, ShiftyWeek};
use tokio::join;
use uuid::Uuid;

gen_service_impl! {
    struct ExtraHoursServiceImpl: ExtraHoursService = ExtraHoursServiceDeps {
        ExtraHoursDao: ExtraHoursDao<Transaction = Self::Transaction> = extra_hours_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        CustomExtraHoursService: CustomExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = custom_extra_hours_service,
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
                ShiftyDate::first_day_in_year(year).as_shifty_week(),
                ShiftyWeek::new(year, until_week),
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
        from_week: ShiftyWeek,
        to_week: ShiftyWeek,
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
                from_week.year,
                to_week.year,
                tx.clone(),
            )
            .await?;

        let mut extra_hours_list = extra_hours_entities
            .iter()
            .filter(|extra_hours| {
                extra_hours.as_date().as_shifty_week() >= from_week
                    && extra_hours.as_date().as_shifty_week() <= to_week
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
