use crate::gen_service_impl;
use std::sync::Arc;

use async_trait::async_trait;
use dao::PermissionDao;
use dao::TransactionDao;
use service::permission::Authentication;
use service::user_service::UserService;
use service::{Privilege, ServiceError};

// Phase 63 Plan 01: `PermissionDao` traegt seit diesem Plan einen
// `Transaction`-Assoziationstyp fuer `create_user`/`delete_user`; der Service
// braucht deshalb einen `TransactionDao`, um sich selbst eine Transaktion zu
// holen. Nach aussen bleibt `service::PermissionService` unveraendert —
// bewusst KEIN `type Transaction` und KEIN `tx`-Parameter auf dem
// Service-Trait, das fuehren rund zwanzig Services als Dependency.
gen_service_impl! {
    struct PermissionServiceImpl: service::PermissionService = PermissionServiceDeps {
        PermissionDao: dao::PermissionDao<Transaction = Self::Transaction> = permission_dao,
        UserService: service::user_service::UserService<Context = Self::Context> = user_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

const PERMISSION_SERVICE_PROCESS: &str = "permission-service";

#[async_trait]
impl<Deps: PermissionServiceDeps> service::PermissionService for PermissionServiceImpl<Deps> {
    type Context = Deps::Context;

    async fn current_user_id(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Option<Arc<str>>, ServiceError> {
        match context {
            Authentication::Full => Ok(None),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                Ok(Some(current_user))
            }
        }
    }
    async fn check_permission(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                if self
                    .permission_dao
                    .has_privilege(current_user.as_ref(), privilege)
                    .await?
                {
                    Ok(())
                } else {
                    Err(service::ServiceError::Forbidden)
                }
            }
        }
    }

    async fn check_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                if current_user.as_ref() == user {
                    Ok(())
                } else {
                    Err(service::ServiceError::Forbidden)
                }
            }
        }
    }

    async fn check_only_full_authentication(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        match context {
            Authentication::Full => Ok(()),
            Authentication::Context(_) => Err(service::ServiceError::Forbidden),
        }
    }

    async fn get_privileges_for_current_user(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[Privilege]>, ServiceError> {
        match context {
            Authentication::Full => Ok(Arc::new([Privilege {
                name: "god-mode".into(),
            }])),
            Authentication::Context(context) => {
                let current_user = self.user_service.current_user(context).await?;
                Ok(self
                    .permission_dao
                    .privileges_for_user(current_user.as_ref())
                    .await?
                    .iter()
                    .map(service::Privilege::from)
                    .collect())
            }
        }
    }

    async fn get_roles_for_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::Role]>, ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .roles_for_user(user)
            .await?
            .iter()
            .map(service::Role::from)
            .collect())
    }

    async fn create_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        // T-63-01-B: Das Admin-Gate bleibt die ERSTE Anweisung — vor
        // `use_transaction` und vor jedem Datenbankzugriff.
        self.check_permission("admin", context).await?;
        let tx = self.transaction_dao.use_transaction(None).await?;
        self.permission_dao
            .create_user(
                &dao::UserEntity { name: user.into() },
                // Kein vorgegebener `person_id`: dieser Pfad legt weiterhin
                // eine eigene Person-Identitaet an (D-61-01).
                None,
                PERMISSION_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
    async fn delete_user(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        // T-63-01-B: Gate zuerst, siehe `create_user`.
        self.check_permission("admin", context).await?;
        let tx = self.transaction_dao.use_transaction(None).await?;
        self.permission_dao.delete_user(user, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn user_exists(
        &self,
        user: &str,
        context: Authentication<Self::Context>,
    ) -> Result<bool, ServiceError> {
        self.check_permission("hr", context).await?;
        Ok(self
            .permission_dao
            .find_user(user)
            .await
            .map(|x| x.is_some())?)
    }

    async fn get_all_users(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::User]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_users()
            .await?
            .iter()
            .map(service::User::from)
            .collect())
    }

    async fn create_role(
        &self,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .create_role(
                &dao::RoleEntity { name: role.into() },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }
    async fn delete_role(
        &self,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_role(role).await?;
        Ok(())
    }
    async fn get_all_roles(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::Role]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_roles()
            .await?
            .iter()
            .map(service::Role::from)
            .collect())
    }

    async fn create_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .create_privilege(
                &dao::PrivilegeEntity {
                    name: privilege.into(),
                },
                PERMISSION_SERVICE_PROCESS,
            )
            .await?;
        Ok(())
    }

    async fn delete_privilege(
        &self,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_privilege(privilege).await?;
        Ok(())
    }
    async fn get_all_privileges(
        &self,
        context: Authentication<Self::Context>,
    ) -> Result<Arc<[service::Privilege]>, service::ServiceError> {
        self.check_permission("admin", context).await?;
        Ok(self
            .permission_dao
            .all_privileges()
            .await?
            .iter()
            .map(service::Privilege::from)
            .collect())
    }

    async fn add_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .add_user_role(user, role, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .add_role_privilege(role, privilege, PERMISSION_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
    async fn delete_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao
            .delete_role_privilege(role, privilege)
            .await?;
        Ok(())
    }
    async fn delete_user_role(
        &self,
        user: &str,
        role: &str,
        context: Authentication<Self::Context>,
    ) -> Result<(), service::ServiceError> {
        self.check_permission("admin", context).await?;
        self.permission_dao.delete_user_role(user, role).await?;
        Ok(())
    }
}
