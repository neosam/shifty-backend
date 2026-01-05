use std::sync::Arc;

use crate::DaoError;
use mockall::automock;

#[derive(Clone, Debug, PartialEq)]
pub struct ToggleEntity {
    pub name: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroupEntity {
    pub name: String,
    pub description: Option<String>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait ToggleDao {
    type Transaction: crate::Transaction;

    // Toggle CRUD
    async fn create_toggle(
        &self,
        toggle: &ToggleEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn get_toggle(
        &self,
        name: &str,
        tx: Self::Transaction,
    ) -> Result<Option<ToggleEntity>, DaoError>;

    async fn get_all_toggles(&self, tx: Self::Transaction)
        -> Result<Arc<[ToggleEntity]>, DaoError>;

    async fn update_toggle(
        &self,
        toggle: &ToggleEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn delete_toggle(
        &self,
        name: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn is_enabled(&self, name: &str, tx: Self::Transaction) -> Result<bool, DaoError>;

    // Toggle Group CRUD
    async fn create_toggle_group(
        &self,
        group: &ToggleGroupEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn get_toggle_group(
        &self,
        name: &str,
        tx: Self::Transaction,
    ) -> Result<Option<ToggleGroupEntity>, DaoError>;

    async fn get_all_toggle_groups(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleGroupEntity]>, DaoError>;

    async fn delete_toggle_group(
        &self,
        name: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    // Group-Toggle assignments
    async fn add_toggle_to_group(
        &self,
        group: &str,
        toggle: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn remove_toggle_from_group(
        &self,
        group: &str,
        toggle: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn get_toggles_in_group(
        &self,
        group: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleEntity]>, DaoError>;

    async fn get_groups_for_toggle(
        &self,
        toggle: &str,
        tx: Self::Transaction,
    ) -> Result<Arc<[ToggleGroupEntity]>, DaoError>;

    // Bulk operations
    async fn enable_group(
        &self,
        group: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn disable_group(
        &self,
        group: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
