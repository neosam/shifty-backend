use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;

pub const TOGGLE_ADMIN_PRIVILEGE: &str = "toggle_admin";

#[derive(Clone, Debug, PartialEq)]
pub struct Toggle {
    pub name: Arc<str>,
    pub enabled: bool,
    pub description: Option<Arc<str>>,
}

impl From<&dao::toggle::ToggleEntity> for Toggle {
    fn from(entity: &dao::toggle::ToggleEntity) -> Self {
        Self {
            name: entity.name.clone().into(),
            enabled: entity.enabled,
            description: entity.description.clone().map(Into::into),
        }
    }
}

impl From<&Toggle> for dao::toggle::ToggleEntity {
    fn from(toggle: &Toggle) -> Self {
        Self {
            name: toggle.name.to_string(),
            enabled: toggle.enabled,
            description: toggle.description.as_ref().map(|s| s.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToggleGroup {
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
}

impl From<&dao::toggle::ToggleGroupEntity> for ToggleGroup {
    fn from(entity: &dao::toggle::ToggleGroupEntity) -> Self {
        Self {
            name: entity.name.clone().into(),
            description: entity.description.clone().map(Into::into),
        }
    }
}

impl From<&ToggleGroup> for dao::toggle::ToggleGroupEntity {
    fn from(group: &ToggleGroup) -> Self {
        Self {
            name: group.name.to_string(),
            description: group.description.as_ref().map(|s| s.to_string()),
        }
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait ToggleService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    // Read operations (requires authentication)
    async fn is_enabled(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;

    async fn get_all_toggles(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Toggle]>, ServiceError>;

    async fn get_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Toggle>, ServiceError>;

    // Admin: Toggle management (requires toggle_admin privilege)
    async fn create_toggle(
        &self,
        toggle: &Toggle,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn enable_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn disable_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn delete_toggle(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    // Admin: Group management
    async fn create_toggle_group(
        &self,
        group: &ToggleGroup,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn delete_toggle_group(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn get_all_toggle_groups(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[ToggleGroup]>, ServiceError>;

    async fn get_toggle_group(
        &self,
        name: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<ToggleGroup>, ServiceError>;

    // Admin: Group-Toggle assignments
    async fn add_toggle_to_group(
        &self,
        group: &str,
        toggle: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn remove_toggle_from_group(
        &self,
        group: &str,
        toggle: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn get_toggles_in_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Toggle]>, ServiceError>;

    async fn enable_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    async fn disable_group(
        &self,
        group: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
