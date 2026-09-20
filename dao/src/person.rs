//! Phase 61 (PERS-05 / D-61-02): Basic-Tier Person-DAO.
//!
//! `person` ist seit der Migration `20260916000000_create-person-backfill.sql`
//! die Elterntabelle ueber `sales_person` und `user`. Dieser DAO liest und
//! schreibt die Identitaet selbst und liefert zusaetzlich die beiden
//! „unverknuepfte Kandidaten"-Lesepfade, die Phase 63 fuer den Kandidaten-Picker
//! braucht.
//!
//! D-60-04 (gesperrt): `PersonEntity` traegt ausschliesslich Identitaet plus die
//! beiden 2026er Audit-Spalten — KEINE `deleted`-Spalte und KEINE Namensspalte.
//! In v2.7 wird keine Person geloescht; personenbezogene Stammdaten sind
//! Future-Scope (PERS-F-01).
//!
//! D-61-02 (gesperrt): Die beiden Kandidaten-Lesepfade sind LEFT-JOIN-Anti-Joins
//! ueber `person_id`. Nach D-61-01 traegt jede Zeile eine Person — „unverknuepft"
//! ist die Abwesenheit des Join-Partners, nicht die Abwesenheit eines Werts.
//!
//! Das Trait bleibt bewusst auf genau vier Methoden beschraenkt. Vorratshaltung
//! fuer Phase 62/63 ist ausdruecklich unerwuenscht (Research Pitfall 5).

use std::sync::Arc;

use mockall::automock;
use uuid::Uuid;

use crate::sales_person::SalesPersonEntity;
use crate::DaoError;

/// Reine Identitaet plus Audit — siehe D-60-04 im Modulkopf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonEntity {
    pub id: Uuid,
    pub created: time::PrimitiveDateTime,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait PersonDao {
    type Transaction: crate::Transaction;

    /// Liefert die Person zur uebergebenen id, oder `None`.
    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<PersonEntity>, DaoError>;

    /// Schreibt eine neue Person. `id`, `created` und `version` erzeugt der
    /// Service (Muster `RebookingBatchServiceImpl::create`), nicht das DAO.
    async fn create(
        &self,
        entity: &PersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Sales Persons, deren Person keinen Benutzer traegt — Kandidaten fuer
    /// „Login hinzufuegen". Soft-geloeschte Sales Persons sind ausgeschlossen.
    async fn sales_persons_without_user(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonEntity]>, DaoError>;

    /// Benutzernamen, deren Person keine **lebende** Sales Person traegt —
    /// Kandidaten fuer „Person hinzufuegen".
    ///
    /// D-62-05 / WR-02: Die Sales-Person-Seite wird in der Join-Bedingung auf
    /// nicht-geloescht eingeschraenkt. Ein Benutzer, dessen einzige Sales
    /// Person soft-geloescht ist, ist damit wieder Kandidat — vorher fiel er
    /// aus beiden Kandidatenlisten und war nie wieder verknuepfbar. Die Tabelle
    /// `user` selbst hat weiterhin keine Loesch-Spalte.
    async fn users_without_sales_person(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[Arc<str>]>, DaoError>;
}
