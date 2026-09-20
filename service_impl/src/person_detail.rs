//! Phase 63 Plan 04 (PADM-04 / PADM-05 / PADM-06 / PADM-08): Business-Logic-Tier
//! `PersonDetailService` — der Lesepfad hinter der Personen-Detailansicht und
//! ihrem Kandidaten-Picker.
//!
//! **Tier: Business-Logic.** Der Service orchestriert ueber zwei Fach-Objekte
//! hinweg und pflegt mit dem Auspraegungs-Lebenszyklus eine
//! Cross-Entity-Invariante — kein Entity-Manager im Sinne der
//! CLAUDE.md-Konvention.
//!
//! **Warum zwei Basic-Services als Dependencies zulaessig sind.** CLAUDE.md
//! §Service-Tier-Konventionen erlaubt einem Business-Logic-Service
//! ausdruecklich, Basic-Services zu konsumieren. `PersonService` und
//! `SalesPersonService` sind beide Basic-Tier (nur DAOs + Permission + Clock +
//! Uuid + Transaction), und keiner von beiden kennt diesen Service — es
//! entsteht also kein Zyklus und die Konstruktionsreihenfolge in
//! `shifty_bin/src/main.rs` bleibt deterministisch: erst der Basic-Block, dann
//! dieser Service.
//!
//! **Warum die Benutzer-Seite ueber `PermissionDao` statt ueber
//! `PermissionService` laeuft.** `service::PermissionService` fuehrt bewusst
//! **keinen** `Transaction`-Assoziationstyp — rund zwanzig Services fuehren ihn
//! als Dependency, ein Assoziationstyp dort waere ein workspace-weiter Ripple
//! ohne Gegenwert. Die transaktionsfaehigen Schreibpfade
//! (`create_user`/`delete_user`) liegen deshalb seit Plan 63-01 eine Ebene
//! tiefer, auf `dao::PermissionDao`. Der Schreibpfad in Plan 63-05 braucht sie
//! auf DER Transaktion, die auch die Sales Person schreibt (SC-2).
//!
//! **`PersonDao`, `PermissionDao`, `SalesPersonService` und `PersonService`
//! (Schreib-Anteil) werden erst vom Schreibpfad in Plan 63-05 gebraucht.** Sie
//! stehen bereits hier im Deps-Block, damit die DI-Verdrahtung in
//! `shifty_bin/src/main.rs` nur einmal angefasst wird.
//!
//! **Gate: das Admin-Privileg, als erste Anweisung (D-62-08, PADM-08,
//! T-63-04-A/T-63-04-B).** Bewusst **nicht** das HR-Privileg der
//! Basic-Tier-Services — die Detailansicht ist Teil der Personenverwaltung des
//! Administrators, genau wie die Liste, die sie aufschliesst. Die Prohibition
//! dieses Plans wird per Grep geprueft: das HR-Literal kommt in dieser Datei
//! nirgends vor, auch nicht als Konstantenname im Fliesstext.
//!
//! **Fat Backend (PADM-06).** Auspraegung und Anzeigename entstehen genau
//! einmal, in `crate::person_directory::entry_from_row`. Dieses Modul leitet
//! sie nicht noch einmal ab; es reicht die Rohzeile an dieselbe freie Funktion
//! weiter, die die Liste benutzt.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dao::person::PersonDao;
use dao::person_directory::PersonDirectoryDao;
use dao::{PermissionDao, TransactionDao};
use service::permission::Authentication;
use service::person::PersonService;
use service::person_detail::{CreatePersonEntry, PersonDetailService};
use service::person_directory::PersonEntry;
use service::sales_person::{SalesPerson, SalesPersonService};
use service::{PermissionService, ServiceError, ValidationFailureItem};
use uuid::Uuid;

use crate::gen_service_impl;
use crate::person_directory::entry_from_row;

/// Audit-Stempel fuer alle Schreibvorgaenge dieses Services, nach dem Muster
/// von `SALES_PERSON_SERVICE_PROCESS`. Wer in `user.update_process` oder
/// `person.update_process` diesen Wert findet, weiss: die Zeile stammt aus
/// einer Auspraegungs-Aktion der Personen-Detailansicht.
const PERSON_DETAIL_SERVICE_PROCESS: &str = "person-detail-service";

/// Vorgabefarbe einer im Anlege-Dialog entstandenen Sales Person.
///
/// D-63-07: Der Dialog fragt keine Farbe ab. Neutrales Weiss ist derselbe
/// Startwert, den der heutige Knopf „Neuen Verkaeufer erstellen" setzt; die
/// eigentliche Farbwahl passiert danach in der Detailansicht.
const DEFAULT_BACKGROUND_COLOR: &str = "#ffffff";

gen_service_impl! {
    struct PersonDetailServiceImpl: PersonDetailService = PersonDetailServiceDeps {
        PersonDirectoryDao: PersonDirectoryDao<Transaction = Self::Transaction> = person_directory_dao,
        PersonDao: PersonDao<Transaction = Self::Transaction> = person_dao,
        PermissionDao: PermissionDao<Transaction = Self::Transaction> = permission_dao,
        PersonService: PersonService<Context = Self::Context, Transaction = Self::Transaction> = person_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: PersonDetailServiceDeps> PersonDetailService for PersonDetailServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_detail(
        &self,
        person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<PersonEntry, ServiceError> {
        // D-62-08 / PADM-08 / T-63-04-A: Gate als ERSTE Anweisung — vor dem
        // Oeffnen der Transaktion und vor jedem Datenbankzugriff. Bewusst nicht
        // das HR-Privileg der Basic-Tier-Services.
        self.permission_service
            .check_permission("admin", context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        let row = self
            .person_directory_dao
            .find_directory_entry(person_id, tx.clone())
            .await?;

        // Kein Commit auf diesem Pfad: die Transaktion wird verworfen (P-61-B).
        // Eine unbekannte ODER traegerlos gewordene Person ist nicht auffindbar
        // — nie eine halbleere Antwort. Der DAO liefert `None` fuer beide
        // Faelle, weil die Detail-Query dieselbe Traeger-Bedingung traegt wie
        // die Liste (D-62-04).
        let Some(row) = row else {
            return Err(ServiceError::EntityNotFound(person_id));
        };

        let role_rows = self.person_directory_dao.all_user_roles(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;

        // Genau zwei Abfragen fuer genau EINE Person — kein Aufruf pro Zeile
        // einer Liste. Aus der Rollen-Bulk-Antwort wird nur der eine Benutzer
        // uebernommen, der zu dieser Zeile gehoert; `entry_from_row` schlaegt
        // ohnehin nur unter `row.username` nach.
        let mut roles_by_user: HashMap<Arc<str>, Vec<Arc<str>>> = HashMap::new();
        for role_row in role_rows
            .iter()
            .filter(|role_row| Some(&role_row.user_name) == row.username.as_ref())
        {
            roles_by_user
                .entry(role_row.user_name.clone())
                .or_default()
                .push(role_row.role_name.clone());
        }

        // PADM-06: dieselbe freie Funktion, die die Liste benutzt. Eine zweite
        // Ableitung von `kind` und `display_name` waere die Doppelwahrheit, die
        // das Fat-Backend-Prinzip dieser Phase verbietet.
        Ok(entry_from_row(&row, &roles_by_user))
    }

    async fn unlinked_sales_person_candidates(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError> {
        // T-63-04-B: das Gate zuerst. `Authentication::Full` unten ist erst
        // NACH dieser Zeile erreichbar und wird nie aus einem Request
        // abgeleitet.
        self.permission_service
            .check_permission("admin", context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        // `Authentication::Full` ist hier korrekt: das strengere Admin-Gate ist
        // bereits passiert, und `PersonService` ist HR-gegated. Den
        // Nutzerkontext durchzureichen wuerde einen Admin ohne HR-Rolle
        // grundlos abweisen. Dasselbe Muster benutzen seit v2.4 die internen
        // Aggregat-Konsumenten.
        let candidates = self
            .person_service
            .unlinked_sales_persons(Authentication::Full, Some(tx.clone()))
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(candidates)
    }

    async fn unlinked_user_candidates(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Arc<str>]>, ServiceError> {
        self.permission_service
            .check_permission("admin", context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        // Siehe `unlinked_sales_person_candidates`: `Full` erst nach dem Gate.
        let candidates = self
            .person_service
            .unlinked_users(Authentication::Full, Some(tx.clone()))
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(candidates)
    }

    async fn create_combined(
        &self,
        request: &CreatePersonEntry,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Uuid, ServiceError> {
        // T-63-05-A: Gate als ERSTE Anweisung — vor der Validierung und vor dem
        // Oeffnen der Transaktion. Das `Authentication::Full` weiter unten ist
        // erst nach dieser Zeile erreichbar und wird nie aus einem Request
        // abgeleitet.
        self.permission_service
            .check_permission("admin", context)
            .await?;

        // D-63-07 / T-63-05-B / ASVS V5: Die Auswahl-Regel steht bewusst HIER
        // und nicht nur im Dialog. Ein Client, der den deaktivierten
        // Absende-Knopf umgeht, laeuft in diese Pruefung — und sie faellt, bevor
        // auch nur eine Transaktion geoeffnet ist.
        if !request.wants_user && !request.wants_sales_person {
            return Err(ServiceError::ValidationError(
                [ValidationFailureItem::InvalidValue(
                    "wants_user|wants_sales_person".into(),
                )]
                .into(),
            ));
        }

        let username = match (request.wants_user, request.username.as_ref()) {
            (true, Some(username)) if !username.trim().is_empty() => Some(username.clone()),
            // `wants_user` ohne brauchbaren Benutzernamen ist kein halber
            // Erfolg, sondern eine ungueltige Eingabe.
            (true, _) => {
                return Err(ServiceError::ValidationError(
                    [ValidationFailureItem::InvalidValue("username".into())].into(),
                ))
            }
            (false, _) => None,
        };

        // Gegenstueck zur Benutzernamen-Pruefung: ein namenloser Verkaeufer
        // haette im Verzeichnis einen leeren Anzeigenamen und waere dort nicht
        // mehr auffindbar.
        if request.wants_sales_person && request.display_name.trim().is_empty() {
            return Err(ServiceError::ValidationError(
                [ValidationFailureItem::InvalidValue("display_name".into())].into(),
            ));
        }

        // Ab hier genau EINE Transaktion ueber beide Aggregate (SC-2,
        // T-63-05-C). Der innere Commit von `SalesPersonService` ist ein No-Op,
        // solange dieser Service seinen eigenen Arc noch haelt
        // (`TransactionDaoImpl::commit` per `Arc::into_inner`).
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let sales_person_person_id = if request.wants_sales_person {
            let sales_person = SalesPerson {
                id: Uuid::nil(),
                version: Uuid::nil(),
                name: request.display_name.clone(),
                background_color: DEFAULT_BACKGROUND_COLOR.into(),
                is_paid: Some(false),
                inactive: false,
                deleted: None,
            };
            // `Authentication::Full`: das strengere Admin-Gate ist oben bereits
            // passiert, und `SalesPersonService` ist HR-gegated. Den
            // Nutzerkontext durchzureichen wuerde einen Admin ohne HR-Rolle
            // grundlos abweisen.
            let created = self
                .sales_person_service
                .create(&sales_person, Authentication::Full, Some(tx.clone()))
                .await?;

            // Die Identitaet vergibt das DAO beim Anlegen mit (D-61-01); sie
            // steht nur auf der Traeger-Seite und wird von dort zurueckgelesen.
            let person = self
                .person_dao
                .find_by_sales_person(created.id, tx.clone())
                .await?
                .ok_or_else(|| {
                    ServiceError::EntityNotFoundGeneric(
                        format!("person of sales person {}", created.id).into(),
                    )
                })?;
            Some(person.id)
        } else {
            None
        };

        let person_id = match (sales_person_person_id, username) {
            // Beide Seiten gewaehlt: der Benutzer wird direkt an die Person der
            // eben angelegten Sales Person gehaengt. Deshalb braucht dieser
            // Pfad KEIN nachtraegliches Zusammenfuehren zweier Identitaeten —
            // es entsteht nie eine zweite Person, die anschliessend absorbiert
            // werden muesste.
            (Some(person_id), Some(username)) => {
                self.permission_dao
                    .create_user(
                        &dao::UserEntity { name: username },
                        Some(person_id),
                        PERSON_DETAIL_SERVICE_PROCESS,
                        tx.clone(),
                    )
                    .await?;
                person_id
            }
            (Some(person_id), None) => person_id,
            // Nur ein Benutzerkonto: das DAO vergibt die Identitaet selbst
            // (D-61-01), und sie wird rueckwaerts aufgeloest.
            (None, Some(username)) => {
                self.permission_dao
                    .create_user(
                        &dao::UserEntity {
                            name: username.clone(),
                        },
                        None,
                        PERSON_DETAIL_SERVICE_PROCESS,
                        tx.clone(),
                    )
                    .await?;
                self.person_dao
                    .find_by_user(username.as_ref(), tx.clone())
                    .await?
                    .ok_or_else(|| {
                        ServiceError::EntityNotFoundGeneric(
                            format!("person of user {username}").into(),
                        )
                    })?
                    .id
            }
            // Die Validierung oben schliesst den leeren Auswahl-Satz aus; ein
            // Treffer hier waere ein Integritaets-Alarm, kein Live-Pfad.
            (None, None) => unreachable!(
                "der leere Auswahl-Satz ist vor dem Oeffnen der Transaktion abgewiesen worden"
            ),
        };

        self.transaction_dao.commit(tx).await?;
        Ok(person_id)
    }
}
