use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTemplateEntity {
    pub id: Uuid,
    pub name: Option<Arc<str>>,
    pub template_type: Arc<str>,
    pub template_text: Arc<str>,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub created_by: Option<Arc<str>>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait TextTemplateDao {
    type Transaction: crate::Transaction;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[TextTemplateEntity]>, DaoError>;
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<TextTemplateEntity>, DaoError>;
    async fn find_by_template_type(
        &self,
        template_type: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[TextTemplateEntity]>, DaoError>;
    async fn create(
        &self,
        entity: &TextTemplateEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn update(
        &self,
        entity: &TextTemplateEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn delete(
        &self,
        id: Uuid,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}