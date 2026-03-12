use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TemplateEngine {
    Tera,
    MiniJinja,
}

impl From<&dao::text_template::TemplateEngineEntity> for TemplateEngine {
    fn from(entity: &dao::text_template::TemplateEngineEntity) -> Self {
        match entity {
            dao::text_template::TemplateEngineEntity::Tera => Self::Tera,
            dao::text_template::TemplateEngineEntity::MiniJinja => Self::MiniJinja,
        }
    }
}

impl From<&TemplateEngine> for dao::text_template::TemplateEngineEntity {
    fn from(engine: &TemplateEngine) -> Self {
        match engine {
            TemplateEngine::Tera => Self::Tera,
            TemplateEngine::MiniJinja => Self::MiniJinja,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextTemplate {
    pub id: Uuid,
    pub name: Option<Arc<str>>,
    pub template_type: Arc<str>,
    pub template_text: Arc<str>,
    pub template_engine: TemplateEngine,
    pub created_at: Option<time::PrimitiveDateTime>,
    pub created_by: Option<Arc<str>>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
    pub version: Uuid,
}

impl From<&dao::text_template::TextTemplateEntity> for TextTemplate {
    fn from(text_template: &dao::text_template::TextTemplateEntity) -> Self {
        Self {
            id: text_template.id,
            name: text_template.name.clone(),
            template_type: text_template.template_type.clone(),
            template_text: text_template.template_text.clone(),
            template_engine: TemplateEngine::from(&text_template.template_engine),
            created_at: text_template.created_at,
            created_by: text_template.created_by.clone(),
            deleted: text_template.deleted,
            deleted_by: text_template.deleted_by.clone(),
            version: text_template.version,
        }
    }
}

impl From<&TextTemplate> for dao::text_template::TextTemplateEntity {
    fn from(text_template: &TextTemplate) -> Self {
        Self {
            id: text_template.id,
            name: text_template.name.clone(),
            template_type: text_template.template_type.clone(),
            template_text: text_template.template_text.clone(),
            template_engine: dao::text_template::TemplateEngineEntity::from(&text_template.template_engine),
            created_at: text_template.created_at,
            created_by: text_template.created_by.clone(),
            deleted: text_template.deleted,
            deleted_by: text_template.deleted_by.clone(),
            version: text_template.version,
        }
    }
}

#[automock(type Context=(); type Transaction = dao::MockTransaction;)]
#[async_trait]
pub trait TextTemplateService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[TextTemplate]>, ServiceError>;

    async fn get_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError>;

    async fn get_by_template_type(
        &self,
        template_type: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[TextTemplate]>, ServiceError>;

    async fn create(
        &self,
        item: &TextTemplate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError>;

    async fn update(
        &self,
        item: &TextTemplate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}