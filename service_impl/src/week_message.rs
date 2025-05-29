use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{week_message::WeekMessageDao, TransactionDao};
use service::{
    clock::ClockService,
    permission::{Authentication, SHIFTPLANNER_PRIVILEGE},
    uuid_service::UuidService,
    week_message::{WeekMessage, WeekMessageService},
    PermissionService, ServiceError,
};
use std::sync::Arc;
use uuid::Uuid;

const WEEK_MESSAGE_SERVICE_PROCESS: &str = "week-message-service";

gen_service_impl! {
    struct WeekMessageServiceImpl: WeekMessageService = WeekMessageServiceDeps {
        WeekMessageDao: WeekMessageDao<Transaction = Self::Transaction> = week_message_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: WeekMessageServiceDeps> WeekMessageService for WeekMessageServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_by_id(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<WeekMessage>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.week_message_dao.find_by_id(id, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.map(|e| (&e).into()))
    }

    async fn get_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<WeekMessage>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self
            .week_message_dao
            .find_by_year_and_week(year, calendar_week, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.map(|e| (&e).into()))
    }

    async fn get_by_year(
        &self,
        year: u32,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[WeekMessage]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.week_message_dao.find_by_year(year, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.iter().map(|e| e.into()).collect())
    }

    async fn create(
        &self,
        message: &WeekMessage,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekMessage, ServiceError> {
        // Check permission - only shiftplanners can create week messages
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let mut message = message.clone();
        message.created = Some(self.clock_service.date_time_now());

        let mut entity: dao::week_message::WeekMessageEntity = (&message).try_into()?;

        if !entity.id.is_nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if !entity.version.is_nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        entity.id = self
            .uuid_service
            .new_uuid(&format!("{}::create id", WEEK_MESSAGE_SERVICE_PROCESS));
        entity.version = self
            .uuid_service
            .new_uuid(&format!("{}::create version", WEEK_MESSAGE_SERVICE_PROCESS));

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.week_message_dao
            .create(&entity, WEEK_MESSAGE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(WeekMessage::from(&entity))
    }

    async fn update(
        &self,
        message: &WeekMessage,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekMessage, ServiceError> {
        // Check permission - only shiftplanners can update week messages
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let mut entity: dao::week_message::WeekMessageEntity = message.try_into()?;
        entity.version = self
            .uuid_service
            .new_uuid(&format!("{}::update version", WEEK_MESSAGE_SERVICE_PROCESS));

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.week_message_dao
            .update(&entity, WEEK_MESSAGE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(WeekMessage::from(&entity))
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Check permission - only shiftplanners can delete week messages
        self.permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.week_message_dao
            .delete(id, WEEK_MESSAGE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(())
    }
}
