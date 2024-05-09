use std::sync::Arc;
use std::fmt::Debug;

use async_trait::async_trait;
use mockall::automock;

use crate::ServiceError;

#[automock(type Context=();)]
#[async_trait]
pub trait UserService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    async fn current_user(&self, context: Self::Context) -> Result<Arc<str>, ServiceError>;
}
