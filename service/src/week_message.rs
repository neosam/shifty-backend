use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct WeekMessage {
    pub id: Uuid,
    pub year: u32,
    pub calendar_week: u8,
    pub message: Arc<str>,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&dao::week_message::WeekMessageEntity> for WeekMessage {
    fn from(entity: &dao::week_message::WeekMessageEntity) -> Self {
        Self {
            id: entity.id,
            year: entity.year,
            calendar_week: entity.calendar_week,
            message: entity.message.clone().into(),
            created: Some(entity.created),
            deleted: entity.deleted,
            version: entity.version,
        }
    }
}

impl TryFrom<&WeekMessage> for dao::week_message::WeekMessageEntity {
    type Error = ServiceError;
    fn try_from(message: &WeekMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            id: message.id,
            year: message.year,
            calendar_week: message.calendar_week,
            message: message.message.to_string(),
            created: message.created.unwrap_or_else(|| {
                time::OffsetDateTime::now_utc()
                    .date()
                    .with_time(time::Time::MIDNIGHT)
            }),
            deleted: message.deleted,
            version: message.version,
        })
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait WeekMessageService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<WeekMessage>, ServiceError>;

    async fn get_by_year_and_week(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<WeekMessage>, ServiceError>;

    async fn get_by_year(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[WeekMessage]>, ServiceError>;

    async fn create(
        &self,
        message: &WeekMessage,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekMessage, ServiceError>;

    async fn update(
        &self,
        message: &WeekMessage,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekMessage, ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
