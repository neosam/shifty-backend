use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{toggle::ToggleDao, TransactionDao};
use service::{
    permission::Authentication,
    toggle::{Toggle, ToggleGroup, ToggleService, TOGGLE_ADMIN_PRIVILEGE},
    PermissionService, ServiceError,
};
use std::sync::Arc;

const TOGGLE_SERVICE_PROCESS: &str = "toggle-service";

gen_service_impl! {
    struct ToggleServiceImpl: ToggleService = ToggleServiceDeps {
        ToggleDao: ToggleDao<Transaction = Self::Transaction> = toggle_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: ToggleServiceDeps> ToggleService for ToggleServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn is_enabled(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError> {
        // Requires authentication (user must be logged in)
        let user_id = self.permission_service.current_user_id(context).await?;
        if user_id.is_none() {
            return Err(ServiceError::Unauthorized);
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.is_enabled(name, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn get_all_toggles(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Toggle]>, ServiceError> {
        // Requires authentication (user must be logged in)
        let user_id = self.permission_service.current_user_id(context).await?;
        if user_id.is_none() {
            return Err(ServiceError::Unauthorized);
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.get_all_toggles(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.iter().map(Toggle::from).collect())
    }

    async fn get_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Toggle>, ServiceError> {
        // Requires authentication (user must be logged in)
        let user_id = self.permission_service.current_user_id(context).await?;
        if user_id.is_none() {
            return Err(ServiceError::Unauthorized);
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.get_toggle(name, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.as_ref().map(Toggle::from))
    }

    async fn create_toggle(
        &self,
        toggle: &Toggle,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let entity = dao::toggle::ToggleEntity::from(toggle);
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .create_toggle(&entity, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn enable_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Get existing toggle
        let existing = self.toggle_dao.get_toggle(name, tx.clone()).await?;
        let Some(mut entity) = existing else {
            self.transaction_dao.commit(tx).await?;
            return Err(ServiceError::EntityNotFoundGeneric(name.into()));
        };

        entity.enabled = true;
        self.toggle_dao
            .update_toggle(&entity, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn disable_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Get existing toggle
        let existing = self.toggle_dao.get_toggle(name, tx.clone()).await?;
        let Some(mut entity) = existing else {
            self.transaction_dao.commit(tx).await?;
            return Err(ServiceError::EntityNotFoundGeneric(name.into()));
        };

        entity.enabled = false;
        self.toggle_dao
            .update_toggle(&entity, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn delete_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .delete_toggle(name, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn create_toggle_group(
        &self,
        group: &ToggleGroup,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let entity = dao::toggle::ToggleGroupEntity::from(group);
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .create_toggle_group(&entity, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn delete_toggle_group(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .delete_toggle_group(name, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn get_all_toggle_groups(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ToggleGroup]>, ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.get_all_toggle_groups(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.iter().map(ToggleGroup::from).collect())
    }

    async fn get_toggle_group(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<ToggleGroup>, ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.get_toggle_group(name, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.as_ref().map(ToggleGroup::from))
    }

    async fn add_toggle_to_group(
        &self,
        group: &str,
        toggle: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .add_toggle_to_group(group, toggle, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn remove_toggle_from_group(
        &self,
        group: &str,
        toggle: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .remove_toggle_from_group(group, toggle, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn get_toggles_in_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Toggle]>, ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.toggle_dao.get_toggles_in_group(group, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result.iter().map(Toggle::from).collect())
    }

    async fn enable_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .enable_group(group, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn disable_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Requires toggle_admin privilege
        self.permission_service
            .check_permission(TOGGLE_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.toggle_dao
            .disable_group(group, TOGGLE_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
