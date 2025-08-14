use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use dao::billing_period::{self, BillingPeriodDao, BillingPeriodEntity};
use dao::billing_period_sales_person::{
    BillingPeriodSalesPersonDao, BillingPeriodSalesPersonEntity,
};
use dao::TransactionDao;
use service::billing_period::{
    BillingPeriod, BillingPeriodSalesPerson, BillingPeriodService, BillingPeriodValue,
    BillingPeriodValueType,
};
use service::clock::ClockService;
use service::permission::Authentication;
use service::sales_person::SalesPersonService;
use service::uuid_service::UuidService;
use service::{PermissionService, ServiceError};
use shifty_utils::ShiftyDate;
use uuid::Uuid;

use crate::gen_service_impl;

const BILLING_PERIOD_REPORT_SERVICE: &str = "BillingPeriodReportService";

gen_service_impl! {
    struct BillingPeriodServiceImpl: BillingPeriodService = BillingPeriodServiceDeps {
        BillingPeriodDao: BillingPeriodDao<Transaction = Self::Transaction> = billing_period_dao,
        BillingPeriodSalesPersonDao: BillingPeriodSalesPersonDao<Transaction = Self::Transaction> = billing_period_sales_person_dao,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        UuidService: UuidService = uuid_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

impl<Deps: BillingPeriodServiceDeps> BillingPeriodServiceImpl<Deps> {
    pub async fn insert_billing_period_sales_person(
        &self,
        billing_period_id: Uuid,
        billing_period_sales_person: &BillingPeriodSalesPerson,
        values: BTreeMap<BillingPeriodValueType, BillingPeriodValue>,
        context: Authentication<Deps::Context>,
        tx: Option<Deps::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx: <Deps as BillingPeriodServiceDeps>::Transaction =
            self.transaction_dao.use_transaction(tx).await?;

        let user = self
            .permission_service
            .current_user_id(context.clone())
            .await?
            .unwrap_or("Unauthenticated".into());
        for value_type in values.keys() {
            if !billing_period_sales_person.values.contains_key(value_type) {
                continue;
            }
            let value = billing_period_sales_person.values.get(value_type).unwrap();
            let entity = BillingPeriodSalesPersonEntity {
                id: self
                    .uuid_service
                    .new_uuid("BillingPeriodServiceImpl::insert_billing_period_sales_person id"),
                billing_period_id,
                sales_person_id: billing_period_sales_person.sales_person_id,
                created_at: self.clock_service.date_time_now(),
                created_by: user.to_owned(),
                deleted_at: None,
                deleted_by: None,
                value_type: value_type.as_str(),
                value_delta: value.value_delta,
                value_ytd_from: value.value_ytd_from,
                value_ytd_to: value.value_ytd_to,
                value_full_year: value.value_full_year,
            };

            self.billing_period_sales_person_dao
                .create(&entity, BILLING_PERIOD_REPORT_SERVICE, tx.clone().into())
                .await?;
        }
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}

#[async_trait]
impl<Deps: BillingPeriodServiceDeps> BillingPeriodService for BillingPeriodServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    /// Returns all not deleted `BillingPeriod`s but no sales person data.
    async fn get_billing_period_overview(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[BillingPeriod]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self.billing_period_dao.all(tx.into()).await?;

        Ok(entities
            .iter()
            .filter(|bp| bp.deleted_at.is_none())
            .map(|pb| BillingPeriod::from_billing_period_entity(pb, [].into()))
            .collect())
    }

    /// Returns the `BillingPeriod` with all sales person data for the given ID.
    async fn get_billing_period_by_id(
        &self,
        id: uuid::Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self
            .billing_period_dao
            .find_by_id(id, tx.clone().into())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;

        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;
        let mut sales_person_report = Vec::new();
        for sales_person in sales_persons.iter() {
            let sales_person_id = sales_person.id;
            let sales_person_entities = self
                .billing_period_sales_person_dao
                .find_by_billing_period_and_sales_person(id, sales_person_id, tx.clone().into())
                .await?;
            sales_person_report.push(BillingPeriodSalesPerson::from_entities(
                &sales_person_entities,
            ));
        }

        let res = Ok(BillingPeriod::from_billing_period_entity(
            &entity,
            sales_person_report.into(),
        ));
        dbg!(&res);

        self.transaction_dao.commit(tx).await?;
        res
    }

    /// Creates a new `BillingPeriod` with the given sales person data.
    async fn create_billing_period(
        &self,
        entity: &BillingPeriod,
        process: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BillingPeriod, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let user = self
            .permission_service
            .current_user_id(context.clone())
            .await?
            .unwrap_or("Unauthenticated".into());
        let billing_period_entity = BillingPeriodEntity {
            id: self
                .uuid_service
                .new_uuid("BillingPeriodServiceImpl::create_billing_period id"),
            start_date: entity.start_date.to_date(),
            end_date: entity.end_date.to_date(),
            created_at: self.clock_service.date_time_now(),
            created_by: user,
            deleted_at: None,
            deleted_by: None,
        };

        let created_entity = self
            .billing_period_dao
            .create(&billing_period_entity, process, tx.clone().into())
            .await?;

        for sales_person in entity.sales_persons.iter() {
            self.insert_billing_period_sales_person(
                created_entity.id,
                &sales_person,
                sales_person.values.clone(),
                context.clone(),
                Some(tx.clone().into()),
            )
            .await?;
        }

        self.transaction_dao.commit(tx).await?;

        Ok(BillingPeriod::from_billing_period_entity(
            &created_entity,
            entity.sales_persons.clone(),
        ))
    }

    /// Get latest `BillingPeriod` end date.
    async fn get_latest_billing_period_end_date(
        &self,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<ShiftyDate>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let res = Ok(self
            .billing_period_dao
            .find_latest_end_date(tx.clone())
            .await?
            .map(|date| ShiftyDate::from_date(date)));

        self.transaction_dao.commit(tx).await?;
        res
    }
}
