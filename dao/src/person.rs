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
//! Die Klausel aus Phase 61, das Trait bleibe auf genau vier Methoden
//! beschraenkt, ist mit Phase 63 aufgeloest. Gemeint war immer: keine
//! Vorratshaltung fuer noch unbelegten Bedarf (Research Pitfall 5) — nicht ein
//! Zahlenlimit. Die vier in Phase 63 ergaenzten Methoden haben jeweils einen
//! konkreten, in derselben Phase gebauten Aufrufer:
//!
//! - `find_by_sales_person` / `find_by_user` → `PersonDetailService::create_combined`
//!   (D-61-03: nach dem Merge zweier Identitaeten muss die **gewinnende**
//!   Person zurueckgegeben werden, und die steht nur auf der Traeger-Seite).
//! - `has_live_facets` / `delete` → `PersonDetailService::remove_facet`
//!   (D-63-01: die traegerlos gewordene `person`-Zeile wird im selben Zug
//!   entfernt).
//!
//! Bleibt bestehen: wer hier eine fuenfte Methode „fuer spaeter" ergaenzen will,
//! ohne einen Aufrufer zu haben, soll es nicht tun.

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

    /// Entfernt die `person`-Zeile hart (PADM-07, D-63-01).
    ///
    /// Ein **harter** `DELETE`, kein Soft-Delete-Stempel: `person` traegt keine
    /// `deleted`-Spalte und bekommt in v2.7 auch keine (D-60-04, gesperrt).
    ///
    /// Idempotent — ein zweiter Aufruf mit derselben id ist ein No-Op ohne
    /// Fehler. Der DAO bewertet nicht, ob geloescht werden **darf**; diese
    /// Entscheidung faellt im aufrufenden Business-Logic-Service nach
    /// `has_live_facets` und nach dem Admin-Gate.
    async fn delete(&self, id: Uuid, tx: Self::Transaction) -> Result<(), DaoError>;

    /// `true`, wenn die Person noch mindestens einen **lebenden** Traeger hat
    /// (PADM-07, Ausloesebedingung von D-63-01).
    ///
    /// **D-63-01-Lesart (b)** — hier festgeschrieben, nachdem RESEARCH.md
    /// (Pitfall 1 / Assumption A1) sie offen gelassen hatte: „lebend" heisst
    /// eine `user`-Zeile mit diesem `person_id` (beliebig, `user` hat keine
    /// Loesch-Spalte) ODER eine `sales_person`-Zeile mit diesem `person_id`
    /// und `deleted IS NULL`. Eine soft-geloeschte Sales Person zaehlt als
    /// **weg** — sie ist durch den Verzeichnis-Join ohnehin unsichtbar
    /// (D-62-04), haelt also keine Admin-Sicht mehr am Leben.
    ///
    /// **Der akzeptierte Preis, ausdruecklich benannt:** wird die Person nach
    /// einem `false` geloescht, behaelt die soft-geloeschte `sales_person`-Zeile
    /// einen `person_id`, der auf nichts mehr aufloest. Das ist kein Defekt,
    /// sondern die dokumentierte Konsequenz dieser Lesart — es gibt bewusst
    /// keinen Foreign Key von `sales_person.person_id` auf `person(id)`
    /// (Phase-60-Migration: ADD COLUMN plus UNIQUE-Index, kein FK). Ein
    /// spaeterer Aufraeum-Pass darf diesen haengenden Zeiger NICHT als Bug
    /// „reparieren"; `billing_period_sales_person` referenziert
    /// `sales_person.id` und nie `person.id`, historische Snapshots bleiben
    /// unberuehrt.
    async fn has_live_facets(
        &self,
        person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<bool, DaoError>;

    /// Loest eine Sales Person rueckwaerts auf ihre Person auf (PADM-05,
    /// D-61-03).
    ///
    /// Der Lookup **bewertet nicht, er loest auf**: auch eine soft-geloeschte
    /// Sales Person liefert ihre Person. Die Bewertung, ob dieser Traeger noch
    /// zaehlt, ist Sache von `has_live_facets`.
    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<PersonEntity>, DaoError>;

    /// Loest einen Benutzernamen rueckwaerts auf seine Person auf (PADM-05,
    /// D-61-03) — die Gegenseite zu `find_by_sales_person`. Nach dem Merge im
    /// kombinierten Anlege-Pfad ist das der Weg, die gewinnende Identitaet zu
    /// erfahren.
    async fn find_by_user(
        &self,
        username: &str,
        tx: Self::Transaction,
    ) -> Result<Option<PersonEntity>, DaoError>;
}
