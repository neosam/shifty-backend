use std::sync::Arc;

use async_trait::async_trait;
use dao::text_template::TextTemplateDao;
use dao::TransactionDao;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::text_template::{TextTemplate, TextTemplateService};
use service::{PermissionService, ServiceError};
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct TextTemplateServiceImpl: service::text_template::TextTemplateService = TextTemplateServiceDeps {
        TextTemplateDao: dao::text_template::TextTemplateDao<Transaction = Self::Transaction> = text_template_dao,
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: TextTemplateServiceDeps> TextTemplateService for TextTemplateServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_all(
        &self,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[TextTemplate]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self.text_template_dao.all(tx.clone()).await?;
        let text_templates: Arc<[TextTemplate]> = entities.iter().map(TextTemplate::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(text_templates)
    }

    async fn get_by_id(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self
            .text_template_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFoundGeneric("TextTemplate not found".into()))?;
        let text_template = TextTemplate::from(&entity);
        self.transaction_dao.commit(tx).await?;
        Ok(text_template)
    }

    async fn get_by_template_type(
        &self,
        template_type: &str,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[TextTemplate]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self
            .text_template_dao
            .find_by_template_type(template_type, tx.clone())
            .await?;
        let text_templates: Arc<[TextTemplate]> = entities.iter().map(TextTemplate::from).collect();
        self.transaction_dao.commit(tx).await?;
        Ok(text_templates)
    }

    async fn create(
        &self,
        item: &TextTemplate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let user = self
            .permission_service
            .current_user_id(context)
            .await?
            .unwrap_or("Unauthenticated".into());

        let mut entity = dao::text_template::TextTemplateEntity::from(item);
        entity.id = Uuid::new_v4();
        entity.version = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();
        entity.created_at = Some(time::PrimitiveDateTime::new(now.date(), now.time()));
        entity.created_by = Some(user.clone());

        self.text_template_dao
            .create(&entity, &user, tx.clone())
            .await?;

        let text_template = TextTemplate::from(&entity);
        self.transaction_dao.commit(tx).await?;
        Ok(text_template)
    }

    async fn update(
        &self,
        item: &TextTemplate,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<TextTemplate, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let user = self
            .permission_service
            .current_user_id(context)
            .await?
            .unwrap_or("Unauthenticated".into());

        // Verify the entity exists
        let _existing = self
            .text_template_dao
            .find_by_id(item.id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFoundGeneric("TextTemplate not found".into()))?;

        let mut entity = dao::text_template::TextTemplateEntity::from(item);
        entity.version = Uuid::new_v4();

        self.text_template_dao
            .update(&entity, &user, tx.clone())
            .await?;

        let text_template = TextTemplate::from(&entity);
        self.transaction_dao.commit(tx).await?;
        Ok(text_template)
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let user = self
            .permission_service
            .current_user_id(context)
            .await?
            .unwrap_or("Unauthenticated".into());

        // Verify the entity exists
        let _existing = self
            .text_template_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFoundGeneric("TextTemplate not found".into()))?;

        self.text_template_dao
            .delete(id, &user, tx.clone())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}