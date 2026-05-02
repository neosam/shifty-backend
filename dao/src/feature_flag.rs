use crate::DaoError;
use mockall::automock;

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureFlagEntity {
    pub key: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait FeatureFlagDao {
    type Transaction: crate::Transaction;

    /// Returns `false` for non-existent keys (fail-safe default).
    async fn is_enabled(&self, key: &str, tx: Self::Transaction) -> Result<bool, DaoError>;

    async fn get(
        &self,
        key: &str,
        tx: Self::Transaction,
    ) -> Result<Option<FeatureFlagEntity>, DaoError>;

    /// UPDATE-only: migration must seed all known keys.
    async fn set(
        &self,
        key: &str,
        enabled: bool,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
}
