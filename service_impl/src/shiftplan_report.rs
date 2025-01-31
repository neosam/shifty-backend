use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{shiftplan_report::ShiftplanReportDao, TransactionDao};
use service::{
    permission::Authentication,
    shiftplan_report::{ShiftplanQuickOverview, ShiftplanReportDay, ShiftplanReportService},
    ServiceError,
};
use std::sync::Arc;
use uuid::Uuid;

gen_service_impl! {
    struct ShiftplanReportServiceImpl: service::shiftplan_report::ShiftplanReportService = ShiftplanReportServiceDeps {
        ShiftplanReportDao: dao::shiftplan_report::ShiftplanReportDao<Transaction = Self::Transaction> = shiftplan_report_dao,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: ShiftplanReportServiceDeps> ShiftplanReportService for ShiftplanReportServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn extract_shiftplan_report(
        &self,
        sales_person_id: Uuid,
        from_year: u32,
        from_week: u8,
        to_year: u32,
        to_week: u8,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self
            .shiftplan_report_dao
            .extract_shiftplan_report(
                sales_person_id,
                from_year,
                from_week,
                to_year,
                to_week,
                tx.clone(),
            )
            .await?; // Directly use ? since ServiceError implements From<DaoError>

        let ret = Ok(entities
            .iter()
            .map(|entity| ShiftplanReportDay {
                sales_person_id: entity.sales_person_id,
                hours: entity.hours,
                year: entity.year,
                calendar_week: entity.calendar_week,
                day_of_week: entity.day_of_week.into(),
            })
            .collect::<Vec<_>>()
            .into());

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn extract_quick_shiftplan_report(
        &self,
        year: u32,
        until_week: u8,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanQuickOverview]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self
            .shiftplan_report_dao
            .extract_quick_shiftplan_report(year, until_week, tx.clone())
            .await?; // Directly use ?

        let ret = Ok(entities
            .iter()
            .map(|entity| ShiftplanQuickOverview {
                sales_person_id: entity.sales_person_id,
                hours: entity.hours,
                year: entity.year,
            })
            .collect::<Vec<_>>()
            .into());

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn extract_shiftplan_report_for_week(
        &self,
        year: u32,
        calendar_week: u8,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ShiftplanReportDay]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self
            .shiftplan_report_dao
            .extract_shiftplan_report_for_week(year, calendar_week, tx.clone())
            .await?; // Directly use ?

        let ret = Ok(entities
            .iter()
            .map(|entity| ShiftplanReportDay {
                sales_person_id: entity.sales_person_id,
                hours: entity.hours,
                year: entity.year,
                calendar_week: entity.calendar_week,
                day_of_week: entity.day_of_week.into(),
            })
            .collect::<Vec<_>>()
            .into());

        self.transaction_dao.commit(tx).await?;
        ret
    }
}
