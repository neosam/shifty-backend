//! Phase 63 (PADM-02 … PADM-05, PADM-07): Die Personen-Detailansicht und ihre
//! Auspraegungs-Aktionen — das Aggregat hinter der einen Personen-Seite.
//!
//! **Tier-Klassifizierung: Business-Logic-Service.** Der Service orchestriert
//! ueber zwei Fach-Objekte hinweg (Sales Person und Benutzerkonto) und pflegt
//! mit dem Auspraegungs-Lebenszyklus — Anlegen, Hinzufuegen, Entfernen einer
//! Seite, plus dem Abbau der traegerlos gewordenen Person (D-63-01) — eine
//! Cross-Entity-Invariante. Er ist damit kein Entity-Manager im Sinne der
//! CLAUDE.md-Konvention, und er darf Basic-Services konsumieren.
//!
//! **Gate-Entscheidung: `"admin"`, nicht `"hr"` (D-62-08, PADM-08).** Die
//! Detailansicht ist die Personenverwaltung des Administrators und gibt
//! Benutzernamen, Rollen und Verkaeuferdaten preis. Der Basic-Tier
//! `PersonService` aus Phase 61 ist HR-gegated; sein Modulkopf haelt
//! ausdruecklich fest, dass Phase 62/63 auf `admin` **verschaerfen** darf.
//! Genau das passiert hier — dasselbe Literal, das `PersonDirectoryService`
//! bereits fuer die Liste verlangt. Eine `ADMIN_PRIVILEGE`-Konstante existiert
//! im Repo bewusst nicht; `crate::permission` kennt nur `SALES_PRIVILEGE`,
//! `HR_PRIVILEGE` und `SHIFTPLANNER_PRIVILEGE`.
//!
//! **Zuschnitt des Detail-Aggregats: Kern-Aggregat plus gezielte
//! Nachladepfade** (63-CONTEXT.md liess die Wahl offen, 63-04 entscheidet
//! sie). `get_detail` liefert genau die Verzeichniszeile — `kind`,
//! `display_name`, `username`, `sales_person_id` und die Sales-Person-Felder.
//! Die vier Abschnitte der Seite (Stammdaten, Schichtplan-Zuordnung, Rollen,
//! Einladungen) laden darueber ihre je eigenen, bereits in Betrieb
//! befindlichen Endpoints. Das haelt die CONTEXT-Auflage „kein N+1" ein —
//! jeder Abschnitt ist ein begrenzter Aufruf fuer genau EINE Person, nie ein
//! Aufruf je Listenzeile — und vermeidet vier duplizierte Transportformen,
//! ohne dass die Seite dadurch mehr Requests braeuchte.
//!
//! **Fat Backend (PADM-06).** Auspraegung und Anzeigename entstehen genau
//! einmal: in `service_impl::person_directory::entry_from_row`, derselben
//! freien Funktion, die die Liste benutzt. Der Detail-Lesepfad leitet nichts
//! eigenes ab, und das Frontend erst recht nicht.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::person_directory::PersonEntry;
use crate::sales_person::SalesPerson;
use crate::ServiceError;

/// Welche Auspraegung eine Aktion betrifft (PADM-03, PADM-07).
///
/// Anders als `crate::person_directory::PersonKind` beschreibt dieses Enum
/// nicht den Zustand einer Zeile, sondern das **Ziel einer Aktion**: welche
/// der beiden Seiten hinzugefuegt oder entfernt werden soll. Domain-
/// Gegenstueck zu `rest_types::PersonFacetTO`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PersonFacet {
    /// Die Benutzerkonto-Seite der Person.
    User,
    /// Die Sales-Person-Seite der Person.
    SalesPerson,
}

/// Eingabe des Anlege-Dialogs „Eintrag anlegen" (PADM-02, D-63-07).
///
/// Bewusst schlank: Farbe, Bezahlt-Kennzeichen, Inaktiv-Kennzeichen und die
/// Schichtplan-Zuordnung gehoeren nicht hierher, sondern in die Detailansicht,
/// in die der Dialog nach dem Speichern weiterleitet.
///
/// Die Regel „mindestens ein Umschalter muss an sein" ist eine **Server**-
/// Pruefung. Der deaktivierte Absende-Knopf im Frontend ist UX, nicht die
/// Sicherheitsgrenze (ASVS V5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatePersonEntry {
    /// Anzeigename des neuen Eintrags. Wird bei `wants_sales_person` zum Namen
    /// der Sales Person.
    pub display_name: Arc<str>,
    /// Es soll ein Benutzerkonto entstehen.
    pub wants_user: bool,
    /// Es soll eine Sales Person entstehen.
    pub wants_sales_person: bool,
    /// Benutzername. Genau dann gefuellt, wenn `wants_user` an ist.
    pub username: Option<Arc<str>>,
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait PersonDetailService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Die Verzeichniszeile zu genau einer `person_id` (PADM-05, PADM-06).
    ///
    /// Liefert dieselben Felder wie die Listenzeile aus Phase 62 — mit
    /// gesetzter Auspraegung und gefuelltem Anzeigenamen, beide im Service
    /// abgeleitet. Das Frontend rendert sie stumpf; es leitet weder `kind`
    /// noch `display_name` selbst ab.
    ///
    /// Eine unbekannte `person_id` und eine traegerlos gewordene Person sind
    /// beide `ServiceError::EntityNotFound` — nie eine halbleere Antwort. Eine
    /// Person ohne Benutzerkonto und ohne lebende Sales Person ist im
    /// Verzeichnis unsichtbar (D-62-04), und die Detailansicht folgt dieser
    /// Sichtbarkeitsregel exakt.
    ///
    /// **Verlangt das Admin-Privileg** (D-62-08, PADM-08), als erste
    /// Anweisung, vor jedem Datenbankzugriff.
    async fn get_detail(
        &self,
        person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PersonEntry, ServiceError>;

    /// Sales Persons, die noch kein Benutzerkonto tragen — die Auswahl des
    /// Kandidaten-Pickers hinter „Login hinzufuegen" (PADM-04, SC-4).
    ///
    /// Bereits verknuepfte Verkaeufer erscheinen hier **nicht**. Das ist
    /// Bequemlichkeit, nicht die Sicherheitsgrenze: der Schreibpfad
    /// `POST /sales-person/{id}/user` lehnt eine bereits verknuepfte Seite
    /// unabhaengig davon mit 409 ab (D-61-04). Wer die Liste umgeht oder
    /// zwischen Laden und Absenden verliert, laeuft in die Ablehnung, nicht in
    /// eine stille Ueberschreibung.
    ///
    /// **Verlangt das Admin-Privileg** (D-62-08, PADM-08), als erste
    /// Anweisung, vor jedem Datenbankzugriff.
    async fn unlinked_sales_person_candidates(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError>;

    /// Benutzernamen, die noch keine Sales Person tragen — die Auswahl des
    /// Kandidaten-Pickers hinter „Person hinzufuegen" (PADM-04, SC-4).
    ///
    /// Bereits verknuepfte Benutzer erscheinen hier **nicht**; es gilt
    /// dieselbe Abgrenzung wie bei `unlinked_sales_person_candidates`: die
    /// Ausblendung ist Bequemlichkeit, die 409-Ablehnung des Schreibpfads
    /// `POST /sales-person/{id}/user` ist die Sicherheitsgrenze (D-61-04).
    ///
    /// **Verlangt das Admin-Privileg** (D-62-08, PADM-08), als erste
    /// Anweisung, vor jedem Datenbankzugriff.
    async fn unlinked_user_candidates(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Arc<str>]>, ServiceError>;

    /// Legt einen Verzeichnis-Eintrag mit genau den gewaehlten Auspraegungen
    /// an (PADM-02, Success Criterion 2).
    ///
    /// SC-2 woertlich: „Nach dem Speichern existieren **genau** die gewaehlten
    /// Objekte, und beide haengen an derselben Person." Benutzerkonto und
    /// Sales Person entstehen deshalb in **einer** Transaktion — scheitert eine
    /// der beiden Anlagen, existiert danach keine von beiden und auch keine
    /// neue `person`-Zeile.
    ///
    /// Der Dialog dahinter ist bewusst schlank (D-63-07): Farbe,
    /// Bezahlt-Kennzeichen, Inaktiv-Kennzeichen und die Schichtplan-Zuordnung
    /// werden hier **nicht** abgefragt. Sie gehoeren in die Detailansicht, in
    /// die der Aufrufer nach dem Speichern weiterleitet — und genau deren
    /// `person_id` ist die Rueckgabe dieser Methode.
    ///
    /// Die Regel „mindestens ein Umschalter muss an sein" setzt **diese**
    /// Methode durch, nicht der Dialog: ein leerer Auswahl-Satz ist
    /// `ServiceError::ValidationError`, und zwar bevor irgendetwas geschrieben
    /// wird. Der deaktivierte Absende-Knopf im Frontend ist Bequemlichkeit,
    /// nicht die Grenze (D-63-07, ASVS V5).
    ///
    /// **Verlangt das Admin-Privileg** (D-62-08, PADM-08), als erste
    /// Anweisung — vor der Validierung und vor dem Oeffnen der Transaktion.
    async fn create_combined(
        &self,
        request: &CreatePersonEntry,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Uuid, ServiceError>;
}
