//! Phase 62 (PADM-01 / PADM-06): Lese-DAO fuer das Personen-Verzeichnis.
//!
//! Das Verzeichnis ist die eine Liste, die die bisherigen Tabs der
//! Benutzerverwaltung abloest: verknuepfte Personen, reine Benutzerkonten und
//! reine Verkaeufer stehen nebeneinander in derselben Tabelle.
//!
//! **Warum ein eigenes Trait statt einer fuenften Methode auf `PersonDao`?**
//! Der Modulkopf von `dao/src/person.rs` zurrt ausdruecklich fest, dass jenes
//! Trait auf genau vier Methoden beschraenkt bleibt und Vorratshaltung fuer
//! Phase 62/63 unerwuenscht ist (Research Pitfall 5). Das Verzeichnis ist
//! ausserdem eine andere Lesart derselben Tabellen: `PersonDao` liefert
//! Identitaet und Kandidaten-Anti-Joins, dieser DAO liefert das Aggregat ueber
//! alle drei Auspraegungen. Zwei Zustaendigkeiten, zwei Traits.
//!
//! D-62-02 (gesperrt): Die Lesequelle ist der Drei-Wege-Outer-Join
//! `person` ↔ `sales_person` ↔ `user`, nicht `person` direkt. Nur so entsteht
//! die Zeilenmenge, die die Oberflaeche braucht — eine reine `person`-Abfrage
//! haette weder Namen noch Benutzernamen und muesste pro Zeile nachladen.
//!
//! D-62-04 (gesperrt): Soft-geloeschte Sales Persons erscheinen NICHT im
//! Verzeichnis, inaktive schon. „Inaktiv" ist ein Anzeige-Zustand mit eigenem
//! Badge, „geloescht" ist Abwesenheit.

use std::sync::Arc;

use mockall::automock;
use uuid::Uuid;

use crate::DaoError;

/// Eine Verzeichniszeile: eine Person plus die Traeger, die sie hat.
///
/// Die Verkaeufer-Seite (`sales_person_id`, `sales_person_name`,
/// `background_color`, `is_paid`, `inactive`) ist genau dann leer, wenn die
/// Person keine **lebende** Sales Person traegt — eine soft-geloeschte zaehlt
/// nicht (D-62-04). Alle fuenf Felder sind gemeinsam gefuellt oder gemeinsam
/// leer; sie stammen aus derselben gejointen Zeile.
///
/// `username` ist genau dann leer, wenn die Person kein Benutzerkonto traegt.
///
/// Mindestens eine der beiden Seiten ist immer gefuellt: eine `person`-Zeile
/// ganz ohne Traeger erscheint nicht im Verzeichnis (das SQL filtert sie aus,
/// Research Pitfall 5). Nach D-61-01/D-61-05 traegt jede `sales_person`- und
/// jede `user`-Zeile eine Person, das Aggregat ist also vollstaendig.
#[derive(Clone, Debug, PartialEq)]
pub struct PersonDirectoryRow {
    /// Identitaet der Person — der stabile Schluessel der Verzeichniszeile.
    pub person_id: Uuid,
    /// id der lebenden Sales Person, oder leer bei reinem Benutzerkonto.
    pub sales_person_id: Option<Uuid>,
    /// Anzeigename des Verkaeufers, oder leer bei reinem Benutzerkonto.
    pub sales_person_name: Option<Arc<str>>,
    /// Farbe des Verkaeufers im Schichtplan, oder leer bei reinem Benutzerkonto.
    pub background_color: Option<Arc<str>>,
    /// Bezahlt-Kennzeichen des Verkaeufers, oder leer bei reinem Benutzerkonto.
    pub is_paid: Option<bool>,
    /// Inaktiv-Kennzeichen des Verkaeufers (D-62-04: inaktive erscheinen im
    /// Verzeichnis, mit Badge), oder leer bei reinem Benutzerkonto.
    pub inactive: Option<bool>,
    /// Benutzername des Logins, oder leer bei reinem Verkaeufer.
    pub username: Option<Arc<str>>,
}

/// Eine Rollen-Zuordnung aus `user_role`.
#[derive(Clone, Debug, PartialEq)]
pub struct UserRoleRow {
    pub user_name: Arc<str>,
    pub role_name: Arc<str>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait PersonDirectoryDao {
    type Transaction: crate::Transaction;

    /// Alle Verzeichniszeilen in genau einem Statement (D-62-02).
    ///
    /// Unsortiert — die Sortierung uebernimmt der Business-Logic-Service,
    /// damit sie testbar und locale-unabhaengig bleibt.
    async fn list_directory_entries(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[PersonDirectoryRow]>, DaoError>;

    /// Alle Rollen-Zuordnungen aller Benutzer als **eine** Bulk-Query.
    ///
    /// Die bestehende Einzel-Abfrage pro Benutzer auf `PermissionDao`
    /// (`roles_for_user`) bleibt unangetastet — sie hat ihren Platz im
    /// Auth-Pfad. Fuer das Verzeichnis waere ein Aufruf pro Zeile genau das
    /// N+1-Muster, das diese Phase abbaut (Success Criterion 5).
    async fn all_user_roles(&self, tx: Self::Transaction) -> Result<Arc<[UserRoleRow]>, DaoError>;
}
