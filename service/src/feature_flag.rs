use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;
use std::sync::Arc;

pub const FEATURE_FLAG_ADMIN_PRIVILEGE: &str = "feature_flag_admin";

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureFlag {
    pub key: Arc<str>,
    pub enabled: bool,
    pub description: Option<Arc<str>>,
}

impl From<&dao::feature_flag::FeatureFlagEntity> for FeatureFlag {
    fn from(entity: &dao::feature_flag::FeatureFlagEntity) -> Self {
        Self {
            key: entity.key.clone().into(),
            enabled: entity.enabled,
            description: entity.description.clone().map(Into::into),
        }
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait FeatureFlagService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Read flag value. Auth-only (any authenticated user can read).
    /// Returns `false` for unknown keys (fail-safe).
    async fn is_enabled(
        &self,
        key: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError>;

    /// Set flag value. Requires `feature_flag_admin` privilege.
    /// UPDATE-only: migration must seed all known keys.
    async fn set(
        &self,
        key: &str,
        value: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
