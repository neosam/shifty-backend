use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use crate::ServiceError;

#[automock(type Context=();)]
#[async_trait]
pub trait UserService {
    type Context: Clone + Send + Sync + 'static;

    async fn current_user(&self, context: Self::Context) -> Result<Arc<str>, ServiceError>;
}
