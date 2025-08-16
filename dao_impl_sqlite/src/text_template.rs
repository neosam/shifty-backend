use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    text_template::{TextTemplateDao, TextTemplateEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, OffsetDateTime, PrimitiveDateTime};
use uuid::Uuid;

pub struct TextTemplateDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}

impl TextTemplateDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

struct TextTemplateDb {
    id: Vec<u8>,
    template_type: String,
    template_text: String,
    created_at: Option<String>,
    created_by: Option<String>,
    deleted: Option<String>,
    deleted_by: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&TextTemplateDb> for TextTemplateEntity {
    type Error = DaoError;
    fn try_from(text_template: &TextTemplateDb) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_slice(text_template.id.as_ref()).unwrap(),
            template_type: text_template.template_type.as_str().into(),
            template_text: text_template.template_text.as_str().into(),
            created_at: text_template
                .created_at
                .as_ref()
                .map(|created_at| PrimitiveDateTime::parse(created_at, &Iso8601::DATE_TIME))
                .transpose()?,
            created_by: text_template.created_by.as_ref().map(|s| s.as_str().into()),
            deleted: text_template
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            deleted_by: text_template.deleted_by.as_ref().map(|s| s.as_str().into()),
            version: Uuid::from_slice(&text_template.update_version).unwrap(),
        })
    }
}

#[async_trait]
impl TextTemplateDao for TextTemplateDaoImpl {
    type Transaction = TransactionImpl;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[TextTemplateEntity]>, DaoError> {
        Ok(query_as!(
            TextTemplateDb,
            "SELECT id, template_type, template_text, created_at, created_by, deleted, deleted_by, update_version FROM text_template WHERE deleted IS NULL"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(TextTemplateEntity::try_from)
        .collect::<Result<Arc<[TextTemplateEntity]>, DaoError>>()?)
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<TextTemplateEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            TextTemplateDb,
            "SELECT id, template_type, template_text, created_at, created_by, deleted, deleted_by, update_version FROM text_template WHERE id = ? AND deleted IS NULL",
            id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(TextTemplateEntity::try_from)
        .transpose()?)
    }

    async fn find_by_template_type(
        &self,
        template_type: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[TextTemplateEntity]>, DaoError> {
        Ok(query_as!(
            TextTemplateDb,
            "SELECT id, template_type, template_text, created_at, created_by, deleted, deleted_by, update_version FROM text_template WHERE template_type = ? AND deleted IS NULL",
            template_type
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(TextTemplateEntity::try_from)
        .collect::<Result<Arc<[TextTemplateEntity]>, DaoError>>()?)
    }

    async fn create(
        &self,
        entity: &TextTemplateEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let template_type = entity.template_type.as_ref();
        let template_text = entity.template_text.as_ref();
        let created_at = entity.created_at.as_ref().map(|created_at| created_at.to_string());
        let created_by = entity.created_by.as_ref().map(|s| s.as_ref());
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());

        query!(
            "INSERT INTO text_template (id, template_type, template_text, created_at, created_by, deleted, deleted_by, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            id, template_type, template_text, created_at, created_by, deleted, deleted_by, version, process
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn update(
        &self,
        entity: &TextTemplateEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let template_type = entity.template_type.as_ref();
        let template_text = entity.template_text.as_ref();
        let created_at = entity.created_at.as_ref().map(|created_at| created_at.to_string());
        let created_by = entity.created_by.as_ref().map(|s| s.as_ref());
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        let deleted_by = entity.deleted_by.as_ref().map(|s| s.as_ref());

        query!(
            "UPDATE text_template SET template_type = ?, template_text = ?, created_at = ?, created_by = ?, deleted = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE id = ?",
            template_type, template_text, created_at, created_by, deleted, deleted_by, version, process, id
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn delete(
        &self,
        id: Uuid,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let now = OffsetDateTime::now_utc()
            .format(&Iso8601::DATE_TIME)
            .map_db_error()?;
        let version_vec = Uuid::new_v4().as_bytes().to_vec();
        let id_vec = id.as_bytes().to_vec();

        query!(
            "UPDATE text_template SET deleted = ?, deleted_by = ?, update_version = ?, update_process = ? WHERE id = ?",
            now, process, version_vec, process, id_vec
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }
}