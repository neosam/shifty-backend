//! Phase 61 (PERS-05 / D-61-02): SQLite-Implementierung des Basic-Tier
//! `PersonDao`.
//!
//! Die beiden Kandidaten-Lesepfade sind bewusst LEFT-JOIN-Anti-Joins und pruefen
//! ausdruecklich **nicht** auf einen leeren `person_id`-Wert (D-61-02): seit dem
//! Backfill aus `20260916000000_create-person-backfill.sql` und der Ablösung in
//! `20260917000000_drop-sales-person-user.sql` traegt jede `sales_person`- und
//! jede `user`-Zeile eine Person. „Unverknuepft" heisst darum: es gibt keinen
//! Join-Partner auf der Gegenseite.
//!
//! `create` schreibt die vom Service erzeugten Werte und liest bewusst **keine**
//! von SQLite vergebene Rowid zurueck — `person.id` ist ein `blob(16) PRIMARY
//! KEY` und damit kein Rowid-Alias (Research Pitfall 3).

use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    person::{PersonDao, PersonEntity},
    sales_person::SalesPersonEntity,
    DaoError,
};
use sqlx::{query, query_as};
use time::{format_description::well_known::Iso8601, PrimitiveDateTime};
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

// ---------------------------------------------------------------------------
// Row-Layout
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct PersonDb {
    id: Vec<u8>,
    created: String,
    update_version: Vec<u8>,
}

impl TryFrom<&PersonDb> for PersonEntity {
    type Error = DaoError;

    fn try_from(db: &PersonDb) -> Result<Self, Self::Error> {
        Ok(PersonEntity {
            id: Uuid::from_slice(&db.id)?,
            created: PrimitiveDateTime::parse(&db.created, &Iso8601::DATE_TIME)?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

/// Dateilokale Kopie des Sales-Person-Row-Layouts. `SalesPersonDb` in
/// `dao_impl_sqlite/src/sales_person.rs` ist privat; die Sichtbarkeit dort zu
/// aendern ist kein Recht dieses Plans (die Datei gehoert den Plaenen 61-03 und
/// 61-04). Die Feldliste ist identisch — der Kandidaten-Lesepfad liefert einen
/// vollwertigen `SalesPersonEntity`.
#[derive(Debug)]
struct CandidateSalesPersonDb {
    id: Vec<u8>,
    name: String,
    background_color: String,
    is_paid: bool,
    inactive: bool,
    deleted: Option<String>,
    update_version: Vec<u8>,
}

impl TryFrom<&CandidateSalesPersonDb> for SalesPersonEntity {
    type Error = DaoError;

    fn try_from(db: &CandidateSalesPersonDb) -> Result<Self, Self::Error> {
        Ok(SalesPersonEntity {
            id: Uuid::from_slice(&db.id)?,
            name: db.name.as_str().into(),
            background_color: db.background_color.as_str().into(),
            is_paid: db.is_paid,
            inactive: db.inactive,
            deleted: db
                .deleted
                .as_ref()
                .map(|deleted| PrimitiveDateTime::parse(deleted, &Iso8601::DATE_TIME))
                .transpose()?,
            version: Uuid::from_slice(&db.update_version)?,
        })
    }
}

// ---------------------------------------------------------------------------
// DAO-Impl
// ---------------------------------------------------------------------------

pub struct PersonDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl PersonDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PersonDao for PersonDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<PersonEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            PersonDb,
            r#"SELECT id, created, update_version FROM person WHERE id = ?"#,
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(PersonEntity::try_from)
        .transpose()?)
    }

    async fn create(
        &self,
        entity: &PersonEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError> {
        let id_vec = entity.id.as_bytes().to_vec();
        let created_str = entity.created.format(&Iso8601::DATE_TIME).map_db_error()?;
        let version_vec = entity.version.as_bytes().to_vec();

        query!(
            r#"INSERT INTO person (id, created, update_process, update_version)
               VALUES (?, ?, ?, ?)"#,
            id_vec,
            created_str,
            process,
            version_vec,
        )
        .execute(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?;

        Ok(())
    }

    async fn sales_persons_without_user(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[SalesPersonEntity]>, DaoError> {
        Ok(query_as!(
            CandidateSalesPersonDb,
            r#"SELECT sp.id AS "id!: Vec<u8>", sp.name AS "name!: String",
                      sp.background_color AS "background_color!: String",
                      sp.is_paid AS "is_paid!: bool", sp.inactive AS "inactive!: bool",
                      sp.deleted AS "deleted?: String",
                      sp.update_version AS "update_version!: Vec<u8>"
               FROM sales_person sp
               LEFT JOIN user u ON u.person_id = sp.person_id
               WHERE u.name IS NULL AND sp.deleted IS NULL"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(SalesPersonEntity::try_from)
        .collect::<Result<Arc<[SalesPersonEntity]>, DaoError>>()?)
    }

    async fn users_without_sales_person(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[Arc<str>]>, DaoError> {
        Ok(query!(
            r#"SELECT u.name AS "name!: String"
               FROM user u
               LEFT JOIN sales_person sp ON sp.person_id = u.person_id
               WHERE sp.id IS NULL"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(|row| Arc::<str>::from(row.name.as_str()))
        .collect::<Arc<[Arc<str>]>>())
    }
}
