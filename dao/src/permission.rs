use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

use crate::DaoError;

#[derive(Debug, PartialEq, Eq)]
pub struct UserEntity {
    pub name: Arc<str>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct RoleEntity {
    pub name: Arc<str>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrivilegeEntity {
    pub name: Arc<str>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait PermissionDao {
    /// Phase 63 Plan 01: Das Trait ist bewusst **gemischt** — nur die beiden
    /// Schreibpfade, die Phase 63 orchestriert (`create_user`, `delete_user`),
    /// nehmen die geteilte Transaktion; alle uebrigen Methoden bleiben
    /// pool-basiert.
    ///
    /// Grund: SC-2 („nach dem Speichern existieren **genau** die gewaehlten
    /// Objekte") und D-63-01 (Orphan-Cleanup der `person`-Zeile) verlangen,
    /// dass Benutzer-Anlage bzw. -Loeschung gemeinsam mit der Sales-Person-
    /// Anlage bzw. dem Person-Cleanup in **einer** Transaktion stehen. Vorher
    /// oeffnete `create_user` eine eigene lokale `pool.begin()`-Transaktion und
    /// `delete_user` schrieb direkt auf den Pool — beides fuer den aufrufenden
    /// Service unsichtbar und nicht rueckrollbar.
    ///
    /// Der Assoziationstyp sitzt bewusst hier auf dem **DAO**-Trait und nicht
    /// auf `service::PermissionService`: letzteres fuehren rund zwanzig
    /// Services als Dependency, ein zusaetzlicher Assoziationstyp dort waere
    /// ein workspace-weiter Ripple ohne Gegenwert.
    type Transaction: crate::Transaction;

    async fn has_privilege(&self, user: &str, privilege: &str) -> Result<bool, DaoError>;

    /// Legt einen Benutzer auf der uebergebenen Transaktion an.
    ///
    /// `person_id`:
    /// - `None` — es wird eine frische `person`-Identitaet miterzeugt (D-61-01).
    /// - `Some(id)` — der Benutzer wird an die **bestehende** Person gehaengt;
    ///   es entsteht keine zweite `person`-Zeile. Das ist die Voraussetzung
    ///   dafuer, dass ein kombinierter Anlege-Pfad Benutzer und Verkaeufer auf
    ///   DIESELBE Person setzen kann.
    ///
    /// Der Aufrufer entscheidet ueber Commit bzw. Rollback.
    async fn create_user(
        &self,
        user: &UserEntity,
        person_id: Option<uuid::Uuid>,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;
    async fn all_users(&self) -> Result<Arc<[UserEntity]>, DaoError>;
    async fn find_user(&self, username: &str) -> Result<Option<UserEntity>, DaoError>;
    /// Loescht die `user`-Zeile auf der uebergebenen Transaktion. Die
    /// zugehoerige `person`-Zeile raeumt der aufrufende Service in derselben
    /// Transaktion auf (D-63-01) — deshalb darf dieser Schreibpfad keine eigene
    /// Transaktion mehr oeffnen.
    async fn delete_user(&self, username: &str, tx: Self::Transaction) -> Result<(), DaoError>;

    async fn create_role(&self, role: &RoleEntity, process: &str) -> Result<(), DaoError>;
    async fn all_roles(&self) -> Result<Arc<[RoleEntity]>, DaoError>;
    async fn delete_role(&self, rolename: &str) -> Result<(), DaoError>;

    async fn create_privilege(
        &self,
        privilege: &PrivilegeEntity,
        process: &str,
    ) -> Result<(), DaoError>;
    async fn all_privileges(&self) -> Result<Arc<[PrivilegeEntity]>, DaoError>;
    async fn delete_privilege(&self, privilege: &str) -> Result<(), DaoError>;

    async fn add_user_role(&self, user: &str, role: &str, process: &str) -> Result<(), DaoError>;
    async fn add_role_privilege(
        &self,
        role: &str,
        privilege: &str,
        process: &str,
    ) -> Result<(), DaoError>;
    async fn delete_role_privilege(&self, role: &str, privilege: &str) -> Result<(), DaoError>;
    async fn delete_user_role(&self, user: &str, role: &str) -> Result<(), DaoError>;

    async fn privileges_for_user(&self, user: &str) -> Result<Arc<[PrivilegeEntity]>, DaoError>;
    async fn roles_for_user(&self, user: &str) -> Result<Arc<[RoleEntity]>, DaoError>;
}
