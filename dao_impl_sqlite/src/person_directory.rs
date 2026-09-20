//! Phase 62 (PADM-01 / D-62-02 / D-62-04): SQLite-Implementierung des
//! Verzeichnis-Lesepfads `PersonDirectoryDao`.
//!
//! D-62-02: Das Verzeichnis entsteht aus genau EINEM Statement — ein
//! Drei-Wege-Outer-Join `person` ↔ `sales_person` ↔ `user`. Die Rollen kommen
//! als zweite Bulk-Query; ein Aufruf pro Zeile waere das N+1-Muster, das diese
//! Phase abbaut (Success Criterion 5).
//!
//! D-62-04: Soft-geloeschte Sales Persons erscheinen nicht, inaktive schon.
//! „Inaktiv" ist ein Anzeige-Zustand, „geloescht" ist Abwesenheit.
//!
//! Sortiert wird hier NICHT. Die Reihenfolge bestimmt der Business-Logic-Service
//! (Plan 62-03), damit sie testbar und locale-unabhaengig bleibt.

use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    person_directory::{PersonDirectoryDao, PersonDirectoryRow, UserRoleRow},
    DaoError,
};
use sqlx::query_as;
use uuid::Uuid;

use crate::{ResultDbErrorExt, TransactionImpl};

// ---------------------------------------------------------------------------
// Row-Layout
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct PersonDirectoryDb {
    person_id: Vec<u8>,
    sales_person_id: Option<Vec<u8>>,
    sales_person_name: Option<String>,
    background_color: Option<String>,
    is_paid: Option<bool>,
    inactive: Option<bool>,
    username: Option<String>,
}

impl TryFrom<&PersonDirectoryDb> for PersonDirectoryRow {
    type Error = DaoError;

    fn try_from(db: &PersonDirectoryDb) -> Result<Self, Self::Error> {
        Ok(PersonDirectoryRow {
            person_id: Uuid::from_slice(&db.person_id)?,
            sales_person_id: db
                .sales_person_id
                .as_ref()
                .map(|id| Uuid::from_slice(id))
                .transpose()?,
            sales_person_name: db.sales_person_name.as_deref().map(Arc::<str>::from),
            background_color: db.background_color.as_deref().map(Arc::<str>::from),
            is_paid: db.is_paid,
            inactive: db.inactive,
            username: db.username.as_deref().map(Arc::<str>::from),
        })
    }
}

#[derive(Debug)]
struct UserRoleDb {
    user_name: String,
    role_name: String,
}

impl TryFrom<&UserRoleDb> for UserRoleRow {
    type Error = DaoError;

    fn try_from(db: &UserRoleDb) -> Result<Self, Self::Error> {
        Ok(UserRoleRow {
            user_name: db.user_name.as_str().into(),
            role_name: db.role_name.as_str().into(),
        })
    }
}

// ---------------------------------------------------------------------------
// DAO-Impl
// ---------------------------------------------------------------------------

pub struct PersonDirectoryDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl PersonDirectoryDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PersonDirectoryDao for PersonDirectoryDaoImpl {
    type Transaction = TransactionImpl;

    async fn list_directory_entries(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[PersonDirectoryRow]>, DaoError> {
        Ok(query_as!(
            PersonDirectoryDb,
            // D-62-04 / WR-02: `sp.deleted IS NULL` steht in der ON-Klausel,
            // NIEMALS in WHERE. In WHERE degeneriert der LEFT JOIN zum INNER
            // JOIN (`sp.deleted` ist fuer jede nicht gematchte Zeile NULL) —
            // jede Person ohne Sales Person fiele dann aus dem Verzeichnis,
            // dieselbe Falle wie WR-02 im Kandidaten-Lesepfad.
            //
            // Research Pitfall 5: Die WHERE-Klausel schliesst traegerlose
            // `person`-Zeilen aus. Eine Person ohne lebende Sales Person UND
            // ohne Benutzerkonto hat im Verzeichnis nichts anzuzeigen.
            r#"SELECT p.id AS "person_id!: Vec<u8>",
                      sp.id AS "sales_person_id?: Vec<u8>",
                      sp.name AS "sales_person_name?: String",
                      sp.background_color AS "background_color?: String",
                      sp.is_paid AS "is_paid?: bool",
                      sp.inactive AS "inactive?: bool",
                      u.name AS "username?: String"
               FROM person p
               LEFT JOIN sales_person sp ON sp.person_id = p.id AND sp.deleted IS NULL
               LEFT JOIN user u ON u.person_id = p.id
               WHERE sp.id IS NOT NULL OR u.name IS NOT NULL"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(PersonDirectoryRow::try_from)
        .collect::<Result<Arc<[PersonDirectoryRow]>, DaoError>>()?)
    }

    async fn all_user_roles(&self, tx: Self::Transaction) -> Result<Arc<[UserRoleRow]>, DaoError> {
        Ok(query_as!(
            UserRoleDb,
            r#"SELECT user_role.user_name AS "user_name!: String",
                      user_role.role_name AS "role_name!: String"
               FROM user_role"#,
        )
        .fetch_all(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .iter()
        .map(UserRoleRow::try_from)
        .collect::<Result<Arc<[UserRoleRow]>, DaoError>>()?)
    }

    async fn find_directory_entry(
        &self,
        person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<PersonDirectoryRow>, DaoError> {
        let person_id_vec = person_id.as_bytes().to_vec();
        Ok(query_as!(
            PersonDirectoryDb,
            // Bewusst dieselbe Join- und Filter-Semantik wie
            // `list_directory_entries` (PADM-05): `sp.deleted IS NULL` steht in
            // der ON-Klausel, NIEMALS in WHERE — in WHERE degeneriert der LEFT
            // JOIN zum INNER JOIN (D-62-04 / WR-02), und eine Person mit reinem
            // Benutzerkonto verschwaende aus der Detailansicht.
            //
            // Wer die eine Query aufraeumt, muss die andere mitziehen: zwei
            // divergierende Definitionen von „lebende Sales Person" waeren
            // genau der Bug, den Phase 62 gefixt hat.
            //
            // Die Traeger-Bedingung bleibt ebenfalls erhalten: eine traegerlos
            // gewordene Person ist hier so unsichtbar wie in der Liste
            // (Research Pitfall 5) — und genau dieser Zustand ist es, den
            // D-63-01 anschliessend aufraeumt.
            r#"SELECT p.id AS "person_id!: Vec<u8>",
                      sp.id AS "sales_person_id?: Vec<u8>",
                      sp.name AS "sales_person_name?: String",
                      sp.background_color AS "background_color?: String",
                      sp.is_paid AS "is_paid?: bool",
                      sp.inactive AS "inactive?: bool",
                      u.name AS "username?: String"
               FROM person p
               LEFT JOIN sales_person sp ON sp.person_id = p.id AND sp.deleted IS NULL
               LEFT JOIN user u ON u.person_id = p.id
               WHERE p.id = ? AND (sp.id IS NOT NULL OR u.name IS NOT NULL)"#,
            person_id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await
        .map_db_error()?
        .as_ref()
        .map(PersonDirectoryRow::try_from)
        .transpose()?)
    }
}
