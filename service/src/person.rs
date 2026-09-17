//! Phase 61 (PERS-05): Basic-Tier Person-Service.
//!
//! Tier-Klassifizierung: **Basic-Service (Entity-Manager)**. Der Service haengt
//! ausschliesslich von `PersonDao` + Permission + Clock + Uuid + Transaction ab
//! — KEIN Domain-Service als Dependency (Success Criterion 5, CLAUDE.md
//! §Service-Tier-Konventionen). Ein Domain-Service als Dep wuerde ueber
//! `PermissionService` einen DI-Zyklus erzeugen und die Gate-Reihenfolge
//! unbestimmt machen.
//!
//! Das Trait bleibt auf die vier Methoden beschraenkt, die Phase 61 braucht.
//! REST-Endpoints, Transport-Objekte und der Kandidaten-Picker sind Phase 62/63.
//!
//! `automock` erzeugt `MockPersonService` fuer downstream Tests.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::sales_person::SalesPerson;
use crate::ServiceError;

/// Transport-Gegenstueck zu `dao::person::PersonEntity`. Traegt wie die Tabelle
/// nur Identitaet plus Audit — keine `deleted`-Spalte, keine Namen (D-60-04).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub id: Uuid,
    pub created: time::PrimitiveDateTime,
    pub version: Uuid,
}

impl From<&dao::person::PersonEntity> for Person {
    fn from(person: &dao::person::PersonEntity) -> Self {
        Self {
            id: person.id,
            created: person.created,
            version: person.version,
        }
    }
}

impl From<&Person> for dao::person::PersonEntity {
    fn from(person: &Person) -> Self {
        Self {
            id: person.id,
            created: person.created,
            version: person.version,
        }
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait PersonService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Legt eine neue Person-Identitaet an. `id`, `version` und `created`
    /// erzeugt der Service (UuidService/ClockService); der Aufrufer liefert
    /// keine Nutzdaten — `person` traegt keine. HR-gated.
    async fn create(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Person, ServiceError>;

    /// Liefert die Person zur uebergebenen id, oder `None`. HR-gated.
    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Person>, ServiceError>;

    /// Sales Persons, deren Person keinen Benutzer traegt — Kandidaten fuer
    /// „Login hinzufuegen" (Backend-Vorarbeit fuer Phase 63). HR-gated.
    async fn unlinked_sales_persons(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError>;

    /// Benutzernamen, deren Person keine Sales Person traegt — Kandidaten fuer
    /// „Person hinzufuegen" (Backend-Vorarbeit fuer Phase 63). HR-gated.
    async fn unlinked_users(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Arc<str>]>, ServiceError>;
}
