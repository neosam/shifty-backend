//! Phase 62 (PADM-01 / PADM-06 / PADM-08): Das Personen-Verzeichnis — die eine
//! Liste, die die bisherigen Tabs der Benutzerverwaltung abloest.
//!
//! **Tier-Klassifizierung: Business-Logic-Service.** Das Aggregat fuehrt zwei
//! Fach-Objekte zusammen (Sales Person und Benutzerkonto) und pflegt mit der
//! Auspraegung (`PersonKind`) eine Cross-Entity-Invariante. Es ist damit kein
//! Entity-Manager im Sinne der CLAUDE.md-Konvention. Die Implementierung
//! konsumiert trotzdem nur DAO + Permission + Transaction — **kein** anderer
//! Domain-Service (siehe `service_impl::person_directory`): das haelt die
//! DI-Reihenfolge in `shifty_bin/src/main.rs` deterministisch und vermeidet
//! einen Zyklus ueber `PermissionService`.
//!
//! **Fat Backend (PADM-06, Success Criterion 3).** Anzeigename und Auspraegung
//! werden genau einmal berechnet: hier im Service. Nicht in SQL (der DAO
//! liefert bewusst unsortierte Rohzeilen, 62-01) und nicht im Frontend (das
//! rendert `display_name` stumpf und liest `kind` als Fakt).
//!
//! **Gate-Entscheidung D-62-08 (PADM-08): `"admin"`, nicht `"hr"`.** Der
//! Basic-Tier `PersonService` aus Phase 61 ist HR-gegated; sein Modulkopf haelt
//! ausdruecklich fest, dass Phase 62/63 auf `admin` **verschaerfen** darf
//! (REQUIREMENTS.md §„Bewusste Aenderung"). Genau das passiert hier. Das
//! Verzeichnis ist die Personenverwaltung des Administrators, und es gibt
//! Benutzernamen und Rollenzuordnungen preis (T-62-05). Vorbild fuer das
//! Literal ist `PermissionServiceImpl::get_all_users`, das denselben
//! Benutzer-Lesepfad mit `"admin"` schuetzt. Eine `ADMIN_PRIVILEGE`-Konstante
//! existiert im Repo bewusst nicht — `service/src/permission.rs` kennt nur
//! `SALES_PRIVILEGE` und die HR-Konstante.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

/// Auspraegung einer Verzeichniszeile (D-62-01).
///
/// Die Auspraegung wird **im Service** klassifiziert und als expliziter Wert
/// transportiert. Sie wird nie im Frontend aus der Leerheit anderer Felder
/// (`username`, `sales_person_id`) abgeleitet — eine solche clientseitige
/// Heuristik waere genau die Fallback-Regel, die das Fat-Backend-Prinzip
/// dieser Phase verbietet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonKind {
    /// Die Person traegt sowohl eine lebende Sales Person als auch ein
    /// Benutzerkonto.
    Linked,
    /// Die Person traegt nur eine Sales Person, noch kein Benutzerkonto.
    SalesPersonOnly,
    /// Die Person traegt nur ein Benutzerkonto, keine Sales Person.
    UserOnly,
}

/// Eine Zeile des Personen-Verzeichnisses (PADM-01, PADM-06).
///
/// Domain-Gegenstueck zu `rest_types::PersonEntryTO`; die Invarianten der
/// Felder sind dieselben, damit das Transport-Mapping rein feldweise bleibt und
/// nichts nachrechnet.
#[derive(Clone, Debug, PartialEq)]
pub struct PersonEntry {
    /// Identitaet der Person — der stabile Schluessel der Zeile (D-61-01).
    pub person_id: Uuid,
    /// Auspraegung der Zeile, im Service klassifiziert (D-62-01).
    pub kind: PersonKind,
    /// Anzeigename. **Nie** `Option` und nie leer: Verkaeufername, wo
    /// vorhanden, sonst Benutzername (PADM-06, D-62-03). Die Ableitung passiert
    /// im Service, nicht per SQL-`COALESCE` und nicht im Frontend.
    pub display_name: Arc<str>,
    /// Benutzername. Genau dann leer, wenn die Person kein Benutzerkonto
    /// traegt. Bei einem reinen Benutzerkonto ist er identisch mit
    /// `display_name` — die Dopplung wird bewusst nicht unterdrueckt (D-62-03).
    pub username: Option<Arc<str>>,
    /// Genau dann gefuellt, wenn die Person eine **lebende** Sales Person
    /// traegt; eine soft-geloeschte zaehlt nicht (D-62-04).
    pub sales_person_id: Option<Uuid>,
    /// Farbe der Sales Person im Schichtplan. Gefuellt genau dann, wenn die
    /// Person eine lebende Sales Person traegt.
    pub background_color: Option<Arc<str>>,
    /// Bezahlt-Kennzeichen der Sales Person. Gefuellt genau dann, wenn die
    /// Person eine lebende Sales Person traegt.
    pub is_paid: Option<bool>,
    /// Inaktiv-Kennzeichen der Sales Person (D-62-04: inaktive erscheinen im
    /// Verzeichnis, mit Badge). Gefuellt genau dann, wenn die Person eine
    /// lebende Sales Person traegt.
    pub inactive: Option<bool>,
    /// Rollen des Benutzerkontos. Bei fehlendem Benutzerkonto leer — nie
    /// `None`, damit das Frontend nicht zwei Leer-Faelle unterscheiden muss.
    pub roles: Arc<[Arc<str>]>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait PersonDirectoryService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Das vollstaendige Verzeichnis: verknuepfte Personen, reine
    /// Benutzerkonten und reine Verkaeufer nebeneinander, jede Zeile mit
    /// gesetzter Auspraegung und gefuelltem Anzeigenamen (PADM-01, PADM-06).
    ///
    /// Die Liste ist case-insensitive nach Anzeigename sortiert; die Sortierung
    /// gehoert in den Service, nicht ins SQL (62-01 liefert unsortiert).
    ///
    /// **Verlangt das Admin-Privileg** (D-62-08, PADM-08). Bei fehlendem
    /// Privileg wird der Fehler propagiert — es gibt hier **keine** Redaktion
    /// auf `None` wie bei `VoluntaryStatsService`. Von jenem Service ist nur die
    /// Tier- und DI-Form das Vorbild, ausdruecklich nicht seine Gate-Semantik:
    /// eine leere oder redigierte Liste waere fuer einen Nicht-Admin von einem
    /// leeren Verzeichnis ununterscheidbar.
    async fn list(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[PersonEntry]>, ServiceError>;
}
