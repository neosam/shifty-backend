//! Phase 62 Plan 03 (PADM-01 / PADM-06 / PADM-08): Business-Logic-Tier
//! `PersonDirectoryService` — das Aggregat hinter der einen Personen-Liste.
//!
//! **Tier: Business-Logic.** Der Service fuehrt zwei Fach-Objekte zusammen
//! (Sales Person und Benutzerkonto) und pflegt mit `PersonKind` eine
//! Cross-Entity-Invariante — kein Entity-Manager im Sinne der
//! CLAUDE.md-Konvention.
//!
//! **Dependencies: genau drei — `PersonDirectoryDao`, `PermissionService`,
//! `TransactionDao`.** Kein weiterer Domain-Service. Waere einer dabei,
//! entstuende ueber `PermissionService` ein DI-Zyklus, die Gate-Reihenfolge
//! waere nicht mehr bestimmt und die Konstruktionsreihenfolge in
//! `shifty_bin/src/main.rs` nicht mehr deterministisch (CLAUDE.md
//! §Service-Tier-Konventionen, Success Criterion 5).
//!
//! **Gate: das Admin-Privileg, als erste Anweisung (D-62-08, PADM-08,
//! T-62-01/T-62-05).** Der Basic-Tier `PersonService` aus Phase 61 nutzt
//! `HR_PRIVILEGE`; sein Modulkopf haelt ausdruecklich fest, dass Phase 62/63
//! **verschaerfen** darf. Genau das passiert hier. Vorbild fuer das Literal ist
//! `PermissionServiceImpl::get_all_users`, das denselben Benutzer-Lesepfad
//! schuetzt; eine `ADMIN_PRIVILEGE`-Konstante existiert im Repo bewusst nicht.
//! Bei fehlendem Privileg wird der Fehler propagiert — keine Teil-Antwort und
//! keine Redaktion auf leere Felder, anders als bei `VoluntaryStatsService`.
//!
//! **Fat Backend (PADM-06, Success Criterion 3).** Anzeigename, Auspraegung,
//! Rollen-Zuordnung und Sortierung entstehen genau hier: nicht im SQL (62-01
//! liefert bewusst unsortierte Rohzeilen) und nicht im Frontend.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dao::person_directory::{PersonDirectoryDao, PersonDirectoryRow};
use dao::TransactionDao;
use service::permission::Authentication;
use service::person_directory::{PersonDirectoryService, PersonEntry, PersonKind};
use service::{PermissionService, ServiceError};

use crate::gen_service_impl;

gen_service_impl! {
    struct PersonDirectoryServiceImpl: PersonDirectoryService = PersonDirectoryServiceDeps {
        PersonDirectoryDao: PersonDirectoryDao<Transaction = Self::Transaction> = person_directory_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

/// Baut eine Verzeichniszeile aus der DAO-Rohzeile plus der Rollen-Map.
///
/// Bewusst eine **freie** Funktion und keine Methode: so bleiben Klassifikation
/// und Anzeigenamen-Ableitung ohne Datenbank und ohne DI-Gerüst testbar.
fn entry_from_row(
    row: &PersonDirectoryRow,
    roles_by_user: &HashMap<Arc<str>, Vec<Arc<str>>>,
) -> PersonEntry {
    let kind = match (row.sales_person_name.as_ref(), row.username.as_ref()) {
        (Some(_), Some(_)) => PersonKind::Linked,
        (Some(_), None) => PersonKind::SalesPersonOnly,
        (None, Some(_)) => PersonKind::UserOnly,
        // Traegerlose Personen sind bereits im SQL ausgeschlossen
        // (62-01: WHERE sp.id IS NOT NULL OR u.name IS NOT NULL). Der Panic ist
        // darum kein Live-Pfad, sondern ein Datenintegritaets-Alarm: er wuerde
        // bedeuten, dass die Invariante D-61-01/D-61-05 verletzt ist.
        // Stillschweigend eine Zeile zu erfinden waere hier schlechter als der
        // Abbruch (T-62-10, bewusst akzeptiert).
        (None, None) => unreachable!(
            "Verzeichniszeile ohne Traeger fuer Person {} — das SQL schliesst \
             diesen Fall aus; die person/sales_person/user-Invariante ist verletzt",
            row.person_id
        ),
    };

    // PADM-06 / Success Criterion 3: der Anzeigename wird GENAU HIER abgeleitet
    // — nicht per SQL-COALESCE und nicht im Frontend. Verkaeufername, wo
    // vorhanden, sonst Benutzername. Der match oben garantiert, dass mindestens
    // eine der beiden Seiten gefuellt ist.
    let display_name: Arc<str> = row
        .sales_person_name
        .clone()
        .or_else(|| row.username.clone())
        .expect("mindestens eine Traegerseite ist gefuellt (siehe kind-match)");

    let roles: Arc<[Arc<str>]> = row
        .username
        .as_ref()
        .and_then(|username| roles_by_user.get(username.as_ref()))
        .cloned()
        .unwrap_or_default()
        .into();

    PersonEntry {
        person_id: row.person_id,
        kind,
        display_name,
        username: row.username.clone(),
        sales_person_id: row.sales_person_id,
        background_color: row.background_color.clone(),
        is_paid: row.is_paid,
        inactive: row.inactive,
        roles,
    }
}

#[async_trait]
impl<Deps: PersonDirectoryServiceDeps> PersonDirectoryService for PersonDirectoryServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn list(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[PersonEntry]>, ServiceError> {
        // D-62-08 / PADM-08: Gate als ERSTE Anweisung, vor jedem
        // Datenbankzugriff (T-62-01, T-62-05). Das Privileg ist bewusst NICHT
        // das des Basic-Tier-Personen-Services aus Phase 61, sondern dasselbe,
        // das PermissionServiceImpl::get_all_users verlangt.
        self.permission_service
            .check_permission("admin", context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let rows = self
            .person_directory_dao
            .list_directory_entries(tx.clone())
            .await?;
        // Eine Bulk-Query statt eines Aufrufs pro Zeile (Success Criterion 5).
        let role_rows = self.person_directory_dao.all_user_roles(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;

        // D-62-06: Rollen im Service gruppieren, damit jede Zeile genau ihre
        // eigenen bekommt.
        let mut roles_by_user: HashMap<Arc<str>, Vec<Arc<str>>> = HashMap::new();
        for role_row in role_rows.iter() {
            roles_by_user
                .entry(role_row.user_name.clone())
                .or_default()
                .push(role_row.role_name.clone());
        }

        let mut entries: Vec<PersonEntry> = rows
            .iter()
            .map(|row| entry_from_row(row, &roles_by_user))
            .collect();

        // Die Sortierung gehoert in den Service, nicht ins SQL: so ist sie ohne
        // Datenbank testbar und unabhaengig von der SQLite-Collation, die auf
        // TEXT-Spalten per Voreinstellung case-SENSITIV vergleicht.
        entries.sort_by_key(|entry| entry.display_name.to_lowercase());

        Ok(entries.into())
    }
}
