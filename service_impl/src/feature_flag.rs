use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{feature_flag::FeatureFlagDao, TransactionDao};
use service::{
    feature_flag::{FeatureFlagService, FEATURE_FLAG_ADMIN_PRIVILEGE},
    permission::Authentication,
    PermissionService, ServiceError,
};

const FEATURE_FLAG_SERVICE_PROCESS: &str = "feature-flag-service";

gen_service_impl! {
    struct FeatureFlagServiceImpl: FeatureFlagService = FeatureFlagServiceDeps {
        FeatureFlagDao: FeatureFlagDao<Transaction = Self::Transaction> = feature_flag_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: FeatureFlagServiceDeps> FeatureFlagService for FeatureFlagServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn is_enabled(
        &self,
        key: &str,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError> {
        // Auth-only: any authenticated user can read flags. `Authentication::Full`
        // entspricht einem Service-zu-Service-Aufruf (Backend-internal trust)
        // und passiert immer; alle anderen Kontexte muessen einen aktiven User
        // liefern (Phase-2 Plan-04: ReportingService liest is_enabled mit
        // Authentication::Full pro `get_report_for_employee_range`-Run).
        if let Authentication::Context(_) = &context {
            let user_id = self.permission_service.current_user_id(context).await?;
            if user_id.is_none() {
                return Err(ServiceError::Unauthorized);
            }
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let result = self.feature_flag_dao.is_enabled(key, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(result)
    }

    async fn set(
        &self,
        key: &str,
        value: bool,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Admin-only via FEATURE_FLAG_ADMIN_PRIVILEGE
        self.permission_service
            .check_permission(FEATURE_FLAG_ADMIN_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.feature_flag_dao
            .set(key, value, FEATURE_FLAG_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
