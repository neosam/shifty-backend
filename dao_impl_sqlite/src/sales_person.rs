use std::collections::HashMap;
use std::sync::Arc;

use crate::{ResultDbErrorExt, TransactionImpl};
use async_trait::async_trait;
use dao::{
    sales_person::{SalesPersonDao, SalesPersonEntity},
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

/// `update_process`-Marker fuer die beim Verknuepfen geschriebene
/// `user`-Zeile. Greift nur, wenn der Aufrufer keinen eigenen Prozessnamen
/// mitgibt — siehe Methodenkommentar von `assign_to_user`.
const PERSON_LINK_PROCESS: &str = "sales-person-dao::link";

/// Erkennt eine SQLite-UNIQUE-Constraint-Verletzung im rohen `sqlx::Error`,
/// bevor er via `map_db_error` in `DatabaseQueryError` geboxt wird. Ohne diese
/// Unterscheidung wuerde aus dem Verlierer einer Race ein 500er statt eines
/// 409ers (Muster: `rebooking_batch::is_unique_violation`).
fn is_unique_violation(err: &sqlx::Error) -> bool {
    matches!(err, sqlx::Error::Database(db_err) if db_err.is_unique_violation())
}

pub struct SalesPersonDaoImpl {
    pub _pool: Arc<sqlx::SqlitePool>,
}
impl SalesPersonDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { _pool: pool }
    }
}

struct SalesPersonDb {
    id: Vec<u8>,
    name: String,
    background_color: String,
    is_paid: bool,
    inactive: bool,
    deleted: Option<String>,
    update_version: Vec<u8>,
}
impl TryFrom<&SalesPersonDb> for SalesPersonEntity {
    type Error = DaoError;
    fn try_from(sales_person: &SalesPersonDb) -> Result<Self, Self::Error> {
        Ok(Self {
            id: Uuid::from_slice(sales_person.id.as_ref()).unwrap(),
            name: sales_person.name.as_str().into(),
            background_color: sales_person.background_color.as_str().into(),
            is_paid: sales_person.is_paid,
            inactive: sales_person.inactive,
            deleted: sales_person
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&sales_person.update_version).unwrap(),
        })
    }
}

#[async_trait]
impl SalesPersonDao for SalesPersonDaoImpl {
    type Transaction = TransactionImpl;

    async fn all(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE deleted IS NULL"
        )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .iter()
            .map(SalesPersonEntity::try_from)
            .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?
        )
    }

    async fn all_paid(&self, tx: Self::Transaction) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE deleted IS NULL AND is_paid = 1"
        )
            .fetch_all(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .iter()
            .map(SalesPersonEntity::try_from)
            .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?
        )
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            SalesPersonDb,
            "SELECT id, name, background_color, is_paid, inactive, deleted, update_version FROM sales_person WHERE id = ?",
            id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonEntity::try_from)
        .transpose()?)
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive, sp.deleted, sp.update_version FROM sales_person sp JOIN user u ON u.person_id = sp.person_id WHERE u.name = ?",
            user_id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(SalesPersonEntity::try_from)
        .transpose()?)
    }

    async fn create(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let background_color = entity.background_color.as_ref();
        let is_paid = entity.is_paid;
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());

        // D-61-01: Jede neu angelegte Sales Person bekommt sofort eine eigene
        // Person-Identitaet. Ohne sie faellt die Zeile aus der
        // person-basierten Selbst-Zugriffs-Pruefung heraus, sobald der
        // Phase-60-Backfill nicht mehr greift (er lief einmalig ueber den
        // Bestand).
        //
        // Der 16-Byte-Wert entsteht hier in Rust und wird in beide Statements
        // eingesetzt. Ein Readback der von SQLite vergebenen Zeilennummer waere
        // falsch: `person.id` ist ein `blob(16) PRIMARY KEY` und damit kein
        // Alias auf die interne Zeilennummer (Research Pitfall 3).
        //
        // `created` entsteht per `strftime` in SQL — im selben ISO-8601-Format
        // wie im Phase-60-Backfill, damit die Leseseite (`Iso8601::DATE_TIME`)
        // es parst; so braucht die DAO-Ebene keinen `ClockService`.
        //
        // Beide Statements laufen auf der **uebergebenen** `tx`. Damit ist die
        // Person-Anlage atomar mit der Sales-Person-Anlage, ohne dass hier eine
        // zweite Transaktion geoeffnet werden muesste.
        let person_id = Uuid::new_v4().as_bytes().to_vec();
        query!(
            "INSERT INTO person (id, created, update_process, update_version) VALUES (?, strftime('%Y-%m-%dT%H:%M:%S.000000000Z','now'), ?, randomblob(16))",
            person_id,
            process
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        query!("INSERT INTO sales_person (id, name, background_color, is_paid, inactive, deleted, person_id, update_version, update_process) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)", id, name, background_color, is_paid, inactive, deleted, person_id, version, process)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }
    async fn update(
        &self,
        entity: &SalesPersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id = entity.id.as_bytes().to_vec();
        let version = entity.version.as_bytes().to_vec();
        let name = entity.name.as_ref();
        let background_color = entity.background_color.as_ref();
        let is_paid = entity.is_paid;
        let inactive = entity.inactive;
        let deleted = entity.deleted.as_ref().map(|deleted| deleted.to_string());
        query!("UPDATE sales_person SET name = ?, background_color = ?, is_paid = ?, inactive = ?, deleted = ?, update_version = ?, update_process = ? WHERE id = ?", name, background_color, is_paid, inactive, deleted, version, process, id)
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        Ok(())
    }

    /// Verknuepfen ist ein **Merge zweier Personen** (D-61-03): der Benutzer
    /// wird auf die Person der Sales Person umgehaengt, seine bisherige Person
    /// wird absorbiert und geloescht. Die Sales Person behaelt ihre Identitaet —
    /// sie ist die Seite mit historischem Gewicht (Buchungen, `extra_hours`,
    /// write-once-Snapshots in `billing_period_sales_person`).
    ///
    /// **Konflikt-Semantik (D-61-04):** abgelehnt wird nur echte
    /// Mehrdeutigkeit, also wenn eine *dritte* Zeile betroffen waere — der
    /// Ziel-Benutzer haengt schon an einer anderen Sales Person, oder die Sales
    /// Person haengt schon an einem anderen Benutzer. Der haeufige Bestandsfall
    /// „beide Seiten tragen je eine eigene Person" (Ergebnis des
    /// Phase-60-Backfills) ist **kein** Konflikt und geht durch.
    ///
    /// **`update_process`:** der vom Aufrufer uebergebene `process` gewinnt,
    /// weil er den fachlichen Ausloeser benennt (z. B. `sales-person-service`).
    /// Nur wenn er leer ist, greift die dateilokale Konstante
    /// [`PERSON_LINK_PROCESS`] — so bleibt die geschriebene `user`-Zeile in
    /// jedem Fall zuordenbar.
    ///
    /// Alle Statements laufen auf der uebergebenen Transaktion. Der Aufrufer
    /// entscheidet ueber Commit/Rollback; im Fehlerfall wurde nichts
    /// geschrieben.
    ///
    /// Die frueher hier gepflegte separate Verknuepfungstabelle wird von
    /// `migrations/sqlite/20260917000000_drop-sales-person-user.sql` entfernt;
    /// die Verknuepfung lebt seither ausschliesslich in `person_id`.
    async fn assign_to_user(
        &self,
        sales_person_id: Uuid,
        user_id: &str,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let sales_person_id_vec = sales_person_id.as_bytes().to_vec();

        // Schritt 1 — beide Ist-Identitaeten lesen.
        let person_of_sales_person = query!(
            "SELECT person_id FROM sales_person WHERE id = ?",
            sales_person_id_vec
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .and_then(|row| row.person_id);

        let person_of_user = query!("SELECT person_id FROM user WHERE name = ?", user_id)
            .fetch_optional(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .and_then(|row| row.person_id);

        // Schritt 2a — beide zeigen bereits auf dieselbe Person: No-Op.
        if let (Some(p_s), Some(p_u)) = (&person_of_sales_person, &person_of_user) {
            if p_s == p_u {
                return Ok(());
            }
        }

        // Fehlt der Sales Person eine Identitaet (Zeile nicht vorhanden oder
        // `person_id` leer), gibt es nichts, worauf der Benutzer zeigen
        // koennte. Fail-closed statt den Benutzer identitaetslos zu machen
        // (T-61-14) — derselbe Ausgang wie ein `rows_affected() == 0` in
        // Schritt 3.
        let Some(target_person) = person_of_sales_person.clone() else {
            return Err(DaoError::EntityAlreadyExists);
        };

        // Schritt 2b — die beiden Konflikt-Abfragen VOR jedem Schreibvorgang
        // (D-61-04, T-61-11/T-61-12).
        let foreign_user = query!(
            "SELECT name FROM user WHERE person_id = ? AND name != ?",
            target_person,
            user_id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        if foreign_user.is_some() {
            return Err(DaoError::EntityAlreadyExists);
        }

        if let Some(p_u) = &person_of_user {
            let foreign_sales_person = query!(
                "SELECT id FROM sales_person WHERE person_id = ? AND id != ?",
                p_u,
                sales_person_id_vec
            )
            .fetch_optional(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
            if foreign_sales_person.is_some() {
                return Err(DaoError::EntityAlreadyExists);
            }
        }

        // Schritt 3 — Benutzer umhaengen. Der `person_id IS ?`-Guard auf den in
        // Schritt 1 gelesenen Wert ist Defense-in-Depth gegen eine Race
        // innerhalb derselben Transaktion (Muster:
        // `rebooking_batch::update_state_conditional`). `IS` statt `=`, damit
        // auch ein leerer Ausgangswert korrekt matcht.
        let update_process = if process.is_empty() {
            PERSON_LINK_PROCESS
        } else {
            process
        };
        let update_result = query!(
            "UPDATE user SET person_id = ?, update_process = ? WHERE name = ? AND person_id IS ?",
            target_person,
            update_process,
            user_id,
            person_of_user
        )
        .execute(tx.tx.lock().await.as_mut())
        .await;
        let update_result = match update_result {
            Ok(result) => result,
            // Rueckversicherung: schlaegt trotz Vorabpruefung einer der beiden
            // vollstaendigen UNIQUE-Indizes zu, ist der Race-Verlierer ein
            // Konflikt (409) und kein Serverfehler (T-61-13, T-61-15).
            Err(err) if is_unique_violation(&err) => return Err(DaoError::EntityAlreadyExists),
            Err(err) => return Err(DaoError::DatabaseQueryError(Box::new(err))),
        };
        if update_result.rows_affected() != 1 {
            return Err(DaoError::EntityAlreadyExists);
        }

        // Schritt 4 — die absorbierte Person loeschen. Die beiden
        // `NOT EXISTS`-Guards sind Absicherung; nach Schritt 3 zeigt nichts
        // mehr auf sie.
        if let Some(p_u) = person_of_user {
            query!(
                "DELETE FROM person WHERE id = ? \
                 AND NOT EXISTS (SELECT 1 FROM sales_person WHERE person_id = person.id) \
                 AND NOT EXISTS (SELECT 1 FROM user WHERE person_id = person.id)",
                p_u
            )
            .execute(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?;
        }

        Ok(())
    }

    async fn discard_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let sales_person_id = sales_person_id.as_bytes().to_vec();
        query!(
            "DELETE FROM sales_person_user WHERE sales_person_id = ?",
            sales_person_id
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;
        Ok(())
    }

    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<Arc<str>>, DaoError> {
        let sales_person_id = sales_person_id.as_bytes().to_vec();
        Ok(query!(
            "SELECT u.name AS user_id FROM sales_person sp JOIN user u ON u.person_id = sp.person_id WHERE sp.id = ?",
            sales_person_id
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .map(|result| result.user_id.into()))
    }

    async fn get_all_user_assignments(
        &self,
        tx: Self::Transaction,
    ) -> Result<HashMap<Uuid, Arc<str>>, DaoError> {
        let rows = query!(
            "SELECT sp.id AS sales_person_id, u.name AS user_id FROM sales_person sp JOIN user u ON u.person_id = sp.person_id"
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        rows.iter()
            .map(|row| {
                let sales_person_id = Uuid::from_slice(&row.sales_person_id)?;
                let user_id: Arc<str> = row.user_id.clone().into();
                Ok((sales_person_id, user_id))
            })
            .collect::<Result<HashMap<_, _>, DaoError>>()
    }

    async fn find_sales_person_by_user_id(
        &self,
        user_id: &str,
        tx: Self::Transaction,
    ) -> Result<Option<SalesPersonEntity>, DaoError> {
        Ok(query_as!(
            SalesPersonDb,
            "SELECT sp.id, sp.name, sp.background_color, sp.is_paid, sp.inactive, sp.deleted, sp.update_version FROM sales_person sp JOIN user u ON u.person_id = sp.person_id WHERE u.name = ?",
            user_id
        )
            .fetch_optional(tx.tx.lock().await.as_mut())
            .await
            .map_db_error()?
            .as_ref()
            .map(SalesPersonEntity::try_from)
            .transpose()?
        )
    }
}
