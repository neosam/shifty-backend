use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use rest_types::{
    AddPersonFacetTO, CreatePersonEntryTO, InvitationResponse, PersonEntryTO, PersonFacetTO,
    SalesPersonTO, UserTO,
};
use uuid::Uuid;

use rest_types::ShiftplanTO;

use crate::{
    base_types::ImStr,
    error::ShiftyError,
    loader,
    state::{shiftplan::SalesPerson, ShiftplanAssignment},
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone, PartialEq)]
pub struct SelectedSalesPerson {
    pub sales_person: SalesPerson,
    pub user_id: Option<ImStr>,
    pub shiftplan_assignments: Vec<ShiftplanAssignment>,
}
impl SelectedSalesPerson {
    pub fn new(sales_person: SalesPerson) -> Self {
        Self {
            sales_person,
            user_id: None,
            shiftplan_assignments: Vec::new(),
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct RoleAssignment {
    pub role: ImStr,
    pub assigned: bool,
}

#[derive(Default, Clone, PartialEq)]
pub struct UserManagementStore {
    /// Phase 62 (PADM-01): das Personen-Verzeichnis als **ein** Aggregat.
    /// Gefuellt von [`UserManagementAction::LoadPersonDirectory`]; loest die
    /// frueheren Benutzer-/Verkaeufer-Listen und die drei Verknuepfungs- und
    /// Rollen-Maps samt ihrem N+1-Ladeverhalten ab.
    pub directory: Rc<[PersonEntryTO]>,

    pub sales_person: Option<SelectedSalesPerson>,
    pub loaded_sales_person: Option<SelectedSalesPerson>,
    pub role_assignements: Rc<[RoleAssignment]>,
    pub user_invitations: Rc<[InvitationResponse]>,
    /// BUG-02 (v2.2): last error observed while loading user invitations.
    /// `None` on success (and reset on every successful reload); `Some(msg)`
    /// when `list_user_invitations` failed — typically a
    /// `ShiftyError::InvitationParse` from a schema-drift response body.
    ///
    /// Rendered as a persistent inline red banner on
    /// `page/user_details.rs` so the empty-list state can no longer be
    /// confused with a parse failure. The central `ERROR_STORE` overlay
    /// is still fired in parallel — the two channels are complementary
    /// (overlay = one-shot toast, banner = page-scoped persistent).
    pub user_invitations_load_error: Option<ImStr>,
    pub save_success: bool,
    pub shiftplan_catalog: Rc<[ShiftplanTO]>,

    /// Phase 63 (PADM-05): die Verzeichniszeile des Eintrags, den die
    /// Detailansicht zeigt. Bewusst genau diese eine Zeile — die vier
    /// Abschnitte der Seite laden ueber ihre bestehenden Endpoints. Ein
    /// fettes Client-Aggregat waere die zweite Quelle derselben Wahrheit.
    pub person_detail: Option<PersonEntryTO>,

    /// Phase 63 (PADM-05 / D-63-01): der 404-Zustand des Detail-Lesepfades,
    /// getrennt von „noch nicht geladen". Ohne dieses Feld waere ein
    /// verschwundener Eintrag von einem laufenden Ladevorgang nicht zu
    /// unterscheiden, und die Seite zeigte dauerhaft einen Spinner statt
    /// „Dieser Eintrag existiert nicht mehr".
    pub person_detail_missing: bool,

    /// Phase 63 (PADM-04): Verkaeufer ohne Benutzerkonto — der Picker hinter
    /// „Login hinzufuegen".
    pub unlinked_sales_person_candidates: Rc<[SalesPersonTO]>,

    /// Phase 63 (PADM-04): Benutzerkonten ohne Verkaeufer — der Picker hinter
    /// „Person hinzufuegen".
    pub unlinked_user_candidates: Rc<[UserTO]>,

    /// Phase 63 (SC-4 / D-61-04): Ein-Schuss-Flag fuer die Konflikt-Meldung des
    /// Pickers. Gesetzt, wenn das ausgewaehlte Ziel zwischen Oeffnen und
    /// Absenden anderweitig verknuepft wurde (HTTP 409). Jedes **erfolgreiche**
    /// Laden einer Kandidatenliste setzt es zurueck — die Konflikt-Behandlung
    /// laedt die Liste deshalb zuerst neu und setzt das Flag danach.
    pub person_link_conflict: bool,
}
pub static USER_MANAGEMENT_STORE: GlobalSignal<UserManagementStore> =
    Signal::global(UserManagementStore::default);

pub async fn load_shiftplan_catalog() {
    let config = CONFIG.read().clone();
    match crate::api::get_all_shiftplans(config).await {
        Ok(catalog) => {
            USER_MANAGEMENT_STORE.write().shiftplan_catalog = catalog;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

pub async fn load_shiftplan_assignments(sales_person_id: Uuid) {
    let config = CONFIG.read().clone();
    match crate::api::get_shiftplan_assignments(config, sales_person_id).await {
        Ok(assignments) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            if let Some(sp) = store.sales_person.as_mut() {
                sp.shiftplan_assignments = assignments.clone();
            }
            if let Some(sp) = store.loaded_sales_person.as_mut() {
                sp.shiftplan_assignments = assignments;
            }
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err.into()),
            };
        }
    }
}

/// Phase 62: laedt das Personen-Verzeichnis mit einem einzigen Request und
/// meldet Fehler ueber den globalen `ERROR_STORE` — dieselbe `match Ok/Err`-Form
/// wie jeder andere Loader dieser Datei; kein seitenlokales Banner
/// (62-UI-SPEC, Error state).
pub async fn load_person_directory() {
    let directory = loader::load_person_directory(CONFIG.read().clone()).await;
    match directory {
        Ok(directory) => {
            USER_MANAGEMENT_STORE.write().directory = directory;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err),
            };
        }
    }
}

pub async fn load_sales_person(sales_person_id: Uuid) {
    let sales_person = loader::load_sales_person(CONFIG.read().clone(), sales_person_id).await;
    match sales_person {
        Ok(sales_person) => {
            USER_MANAGEMENT_STORE.write().sales_person =
                Some(SelectedSalesPerson::new(sales_person.clone()));
            USER_MANAGEMENT_STORE.write().loaded_sales_person =
                Some(SelectedSalesPerson::new(sales_person));
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err),
            };
        }
    }

    let user = loader::load_user_for_sales_person(CONFIG.read().clone(), sales_person_id).await;
    match user {
        Ok(user) => {
            if let Some(user) = user {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user.clone());
                USER_MANAGEMENT_STORE
                    .write()
                    .loaded_sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user);
            }
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err),
            };
        }
    }
}

pub async fn save_sales_person() -> Result<(), ShiftyError> {
    let selected_sales_person = USER_MANAGEMENT_STORE.read().sales_person.clone();
    let loaded_sales_person = USER_MANAGEMENT_STORE.read().loaded_sales_person.clone();
    if let (Some(selected_sales_person), Some(loaded_sales_person)) =
        (selected_sales_person, loaded_sales_person)
    {
        if selected_sales_person != loaded_sales_person {
            let saved_id = loader::save_sales_person(
                CONFIG.read().clone(),
                selected_sales_person.sales_person.clone(),
            )
            .await?;

            match (
                selected_sales_person.user_id.clone(),
                loaded_sales_person.user_id.clone(),
            ) {
                (Some(new_user_id), Some(old_user_id))
                    if new_user_id != old_user_id => {
                        loader::save_user_for_sales_person(
                            CONFIG.read().clone(),
                            saved_id,
                            new_user_id,
                        )
                        .await?;
                    }
                (Some(user_id), None) => {
                    loader::save_user_for_sales_person(CONFIG.read().clone(), saved_id, user_id)
                        .await?;
                }
                (None, Some(_)) => {
                    loader::remove_user_from_sales_person(CONFIG.read().clone(), saved_id).await?;
                }
                _ => {}
            }

            // Save shiftplan assignments
            if selected_sales_person.shiftplan_assignments
                != loaded_sales_person.shiftplan_assignments
            {
                crate::api::set_shiftplan_assignments(
                    CONFIG.read().clone(),
                    saved_id,
                    &selected_sales_person.shiftplan_assignments,
                )
                .await?;
            }
        }
    }
    Ok(())
}

pub async fn load_role_assignments(user: ImStr) -> Result<(), ShiftyError> {
    let config = CONFIG.read().clone();
    let roles = loader::load_all_roles(config.clone()).await?;
    let user_roles = loader::load_roles_from_user(config.clone(), user).await?;
    let mut role_assignments = roles
        .iter()
        .map(|role| RoleAssignment {
            role: role.clone(),
            assigned: user_roles.contains(role),
        })
        .collect::<Vec<_>>();
    role_assignments.sort_by_key(|role| role.role.clone());
    USER_MANAGEMENT_STORE.write().role_assignements = role_assignments.into();
    Ok(())
}

pub async fn assign_user_to_role(user: ImStr, role: ImStr) -> Result<(), ShiftyError> {
    loader::add_user_to_role(CONFIG.read().clone(), user, role).await?;
    Ok(())
}

pub async fn remove_user_from_role(user: ImStr, role: ImStr) -> Result<(), ShiftyError> {
    loader::remove_user_from_role(CONFIG.read().clone(), user, role).await?;
    Ok(())
}

pub async fn add_user(user: ImStr) -> Result<(), ShiftyError> {
    loader::add_user(CONFIG.read().clone(), user).await?;
    Ok(())
}

pub async fn delete_user(user: ImStr) -> Result<(), ShiftyError> {
    // First check if the user is connected to a sales person
    let sales_person =
        loader::load_sales_person_by_user(CONFIG.read().clone(), user.clone()).await?;

    // If connected to a sales person, remove the connection first
    if let Some(sales_person) = sales_person {
        loader::remove_user_from_sales_person(CONFIG.read().clone(), sales_person.id).await?;
    }

    // Now proceed with deleting the user
    loader::delete_user(CONFIG.read().clone(), user).await?;
    Ok(())
}

pub async fn load_user_invitations(username: ImStr) {
    let invitations = loader::load_user_invitations(CONFIG.read().clone(), username).await;
    match invitations {
        Ok(invitations) => {
            let mut s = USER_MANAGEMENT_STORE.write();
            s.user_invitations = invitations;
            s.user_invitations_load_error = None;
        }
        Err(err) => {
            // BUG-02 (v2.2): mirror the error into a persistent per-page
            // store field so `user_details.rs` can render an inline banner,
            // and clear the stale invitation list so the empty state does
            // NOT appear "successful" underneath the banner.
            let msg: String = err.to_string();
            {
                let mut s = USER_MANAGEMENT_STORE.write();
                s.user_invitations = Rc::new([]);
                s.user_invitations_load_error = Some(msg.into());
            }
            *ERROR_STORE.write() = ErrorStore {
                error: Some(err),
            };
        }
    }
}

pub async fn generate_user_invitation(
    username: ImStr,
    expiration_hours: Option<i64>,
) -> Result<(), ShiftyError> {
    loader::generate_invitation(CONFIG.read().clone(), username.clone(), expiration_hours).await?;
    // Reload invitations to show the new one
    load_user_invitations(username).await;
    Ok(())
}

pub async fn revoke_user_invitation(
    invitation_id: Uuid,
    username: ImStr,
) -> Result<(), ShiftyError> {
    loader::revoke_invitation(CONFIG.read().clone(), invitation_id).await?;
    // Reload invitations to update the list
    load_user_invitations(username).await;
    Ok(())
}

pub async fn revoke_user_invitation_session(
    invitation_id: Uuid,
    username: ImStr,
) -> Result<(), ShiftyError> {
    loader::revoke_invitation_session(CONFIG.read().clone(), invitation_id).await?;
    // Reload invitations to update the status
    load_user_invitations(username).await;
    Ok(())
}

// ─── Personen-Detailansicht (Phase 63, PADM-02/03/04/05/07) ──────────────────
//
// Jede Funktion folgt der Form der uebrigen Lader dieser Datei: Loader-Aufruf,
// `match Ok/Err`, Fehler in den zentralen `ERROR_STORE`. Kein seitenlokales
// Fehlerbanner (UI-SPEC, Error state). Antworten werden unveraendert
// uebernommen — `kind` und `display_name` kommen fertig aus dem Backend
// (PADM-06).

/// PADM-05: laedt die Verzeichniszeile des Eintrags.
///
/// Ein 404 kommt als `Ok(None)` an und setzt `person_detail_missing` — der
/// Eintrag hat seine letzte Auspraegung verloren (D-63-01) oder war nie
/// sichtbar. Das ist ein Zustand der Seite, kein Fehler-Overlay.
pub async fn load_person_detail(person_id: Uuid) {
    let entry = loader::load_person_detail(CONFIG.read().clone(), person_id).await;
    match entry {
        Ok(entry) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            store.person_detail_missing = entry.is_none();
            store.person_detail = entry;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore { error: Some(err) };
        }
    }
}

/// PADM-04 / SC-4: loest den Eintrag ueber die Sales Person neu auf.
///
/// Dieser Weg existiert, weil ein Eintrag beim Verknuepfen seine `person_id`
/// wechseln kann: nach D-61-03 gewinnt die Sales Person ihre Identitaet, und
/// die vorher bekannte `person_id` zeigt danach ins Leere.
pub async fn load_person_by_sales_person(sales_person_id: Uuid) {
    let entry = loader::load_person_by_sales_person(CONFIG.read().clone(), sales_person_id).await;
    match entry {
        Ok(entry) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            store.person_detail_missing = entry.is_none();
            store.person_detail = entry;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore { error: Some(err) };
        }
    }
}

/// PADM-04: laedt die Verkaeufer ohne Benutzerkonto und setzt dabei das
/// Konflikt-Flag zurueck.
pub async fn load_unlinked_sales_persons() {
    let candidates = loader::load_unlinked_sales_persons(CONFIG.read().clone()).await;
    match candidates {
        Ok(candidates) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            store.unlinked_sales_person_candidates = candidates;
            store.person_link_conflict = false;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore { error: Some(err) };
        }
    }
}

/// PADM-04: laedt die Benutzerkonten ohne Verkaeufer und setzt dabei das
/// Konflikt-Flag zurueck.
pub async fn load_unlinked_users() {
    let candidates = loader::load_unlinked_users(CONFIG.read().clone()).await;
    match candidates {
        Ok(candidates) => {
            let mut store = USER_MANAGEMENT_STORE.write();
            store.unlinked_user_candidates = candidates;
            store.person_link_conflict = false;
        }
        Err(err) => {
            *ERROR_STORE.write() = ErrorStore { error: Some(err) };
        }
    }
}

/// PADM-02 / SC-2 / D-63-07: legt den Eintrag an und laedt ihn nach.
///
/// Der Dialog leitet nach dem Speichern in die Detailansicht. Sein Signal ist
/// der Uebergang von `None` nach `Some` in `person_detail` — deshalb wird das
/// Feld hier zuerst geleert und danach mit dem frisch geladenen Eintrag der
/// zurueckgegebenen `person_id` gefuellt.
pub async fn create_person_entry(request: CreatePersonEntryTO) -> Result<(), ShiftyError> {
    {
        let mut store = USER_MANAGEMENT_STORE.write();
        store.person_detail = None;
        store.person_detail_missing = false;
    }
    let created = loader::create_person_entry(CONFIG.read().clone(), request).await?;
    load_person_detail(created.person_id).await;
    Ok(())
}

/// PADM-03 / SC-3: schaltet die fehlende Auspraegung zu.
///
/// Nachgeladen wird ueber die **zurueckgegebene** `person_id`, nicht ueber die
/// uebergebene: beim Zuschalten einer Sales Person auf einen reinen
/// Benutzer-Eintrag wechselt der Eintrag nach D-61-03 seine Identitaet. Wer die
/// alte id weiterverwendet, laedt eine geloeschte Person.
pub async fn add_person_facet(
    person_id: Uuid,
    request: AddPersonFacetTO,
) -> Result<(), ShiftyError> {
    let created = loader::add_person_facet(CONFIG.read().clone(), person_id, request).await?;
    load_person_detail(created.person_id).await;
    Ok(())
}

/// PADM-07 / SC-5: entfernt genau eine Auspraegung.
///
/// War es die letzte, raeumt das Backend den Eintrag mit ab (D-63-01) — das
/// Nachladen antwortet dann mit 404 und setzt `person_detail_missing`.
pub async fn remove_person_facet(
    person_id: Uuid,
    facet: PersonFacetTO,
) -> Result<(), ShiftyError> {
    loader::remove_person_facet(CONFIG.read().clone(), person_id, facet).await?;
    load_person_detail(person_id).await;
    Ok(())
}

/// PADM-04 / SC-4: verknuepft ein bestehendes Benutzerkonto mit einer
/// bestehenden Sales Person ueber den unveraenderten Phase-61-Pfad.
///
/// Ein 409 bedeutet: das Ziel war beim Oeffnen des Pickers frei und ist es beim
/// Absenden nicht mehr. Dann wird die Kandidatenliste neu geladen und
/// **danach** das Konflikt-Flag gesetzt — das Laden setzt es naemlich selbst
/// zurueck. Der Konflikt ist kein Fehler-Overlay, sondern eine Meldung im
/// Picker.
pub async fn link_candidate(sales_person_id: Uuid, username: ImStr) -> Result<(), ShiftyError> {
    match loader::save_user_for_sales_person(CONFIG.read().clone(), sales_person_id, username).await
    {
        Ok(()) => {
            load_person_by_sales_person(sales_person_id).await;
            Ok(())
        }
        Err(ShiftyError::Conflict(_)) => {
            load_unlinked_sales_persons().await;
            load_unlinked_users().await;
            USER_MANAGEMENT_STORE.write().person_link_conflict = true;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

/// PADM-07: loest die Verknuepfung zwischen Benutzerkonto und Sales Person,
/// ohne eines von beiden zu loeschen — der unveraenderte Phase-61-Pfad.
pub async fn unlink_facet(sales_person_id: Uuid) -> Result<(), ShiftyError> {
    loader::remove_user_from_sales_person(CONFIG.read().clone(), sales_person_id).await?;
    load_person_by_sales_person(sales_person_id).await;
    Ok(())
}

/// PADM-07 / D-63-06: loescht den Verkaeufer-Datensatz ueber den seit Langem
/// bestehenden Backend-Pfad `DELETE /sales-person/{id}` (Soft Delete, die
/// Historie bleibt).
///
/// Bewusst **nicht** Teil des Auspraegungs-Entfernens: die Aktion soll auch
/// ausserhalb des Person-Kontexts verfuegbar bleiben, und es entsteht kein
/// Sammelknopf, der mehrere Objekte in einem Zug beseitigt (D-63-05).
pub async fn delete_sales_person(sales_person_id: Uuid) -> Result<(), ShiftyError> {
    loader::delete_sales_person(CONFIG.read().clone(), sales_person_id).await?;
    load_person_directory().await;
    Ok(())
}

pub enum UserManagementAction {
    /// Phase 62 (PADM-01): fuellt das Personen-Verzeichnis mit genau einem
    /// Aggregat-Request.
    LoadPersonDirectory,
    LoadSalesPerson(Uuid),
    UpdateSalesPerson(SalesPerson),
    UpdateSalesPersonUser(ImStr),
    RemoveSalesPersonUser,
    SaveSalesPersonAndNavigate,
    ClearSaveSuccess,
    CreateNewSalesPerson,
    LoadUserRoleAssignments(ImStr),
    AssignUserToRole(ImStr, ImStr),
    RemoveUserFromRole(ImStr, ImStr),
    AddUser(ImStr),
    DeleteUser(ImStr),
    LoadUserInvitations(ImStr),
    GenerateInvitation(ImStr, Option<i64>),
    RevokeInvitation(Uuid),
    RevokeInvitationSession(Uuid),
    LoadShiftplanCatalog,
    LoadShiftplanAssignments(Uuid),
    UpdateShiftplanAssignments(Vec<ShiftplanAssignment>),

    // Die neun Varianten dieser Phase tragen bis zur Detailseite (Plan 63-08)
    // noch keinen Absender. Sie sind deshalb einzeln als bewusst noch nicht
    // konstruiert markiert — nicht die ganze Aufzaehlung, damit eine kuenftig
    // wirklich tote Variante weiterhin auffaellt. Mit der Seite faellt die
    // Markierung Variante fuer Variante wieder weg.
    /// Phase 63 (PADM-05): fuellt die Detailansicht mit genau einem Request.
    #[allow(dead_code)]
    LoadPersonDetail(Uuid),
    /// Phase 63 (PADM-02 / SC-2 / D-63-07).
    #[allow(dead_code)]
    CreatePersonEntry(CreatePersonEntryTO),
    /// Phase 63 (PADM-03 / SC-3).
    #[allow(dead_code)]
    AddPersonFacet {
        person_id: Uuid,
        request: AddPersonFacetTO,
    },
    /// Phase 63 (PADM-07 / SC-5).
    #[allow(dead_code)]
    RemovePersonFacet {
        person_id: Uuid,
        facet: PersonFacetTO,
    },
    /// Phase 63 (PADM-04).
    #[allow(dead_code)]
    LoadUnlinkedSalesPersons,
    /// Phase 63 (PADM-04).
    #[allow(dead_code)]
    LoadUnlinkedUsers,
    /// Phase 63 (PADM-04 / SC-4).
    #[allow(dead_code)]
    LinkCandidate {
        sales_person_id: Uuid,
        username: ImStr,
    },
    /// Phase 63 (PADM-07).
    #[allow(dead_code)]
    UnlinkFacet {
        sales_person_id: Uuid,
    },
    /// Phase 63 (PADM-07 / D-63-06).
    #[allow(dead_code)]
    DeleteSalesPerson {
        sales_person_id: Uuid,
    },
}

pub async fn user_management_service(mut rx: UnboundedReceiver<UserManagementAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            UserManagementAction::LoadPersonDirectory => {
                // Success Criterion 5: genau **ein** Netzwerk-Aufruf in diesem
                // Arm — ein zweiter waere ein Rueckfall in das abgeloeste
                // N+1-Ladeverhalten. Der Test
                // `tests::load_directory_ist_ein_request` haelt das per
                // Quell-Assertion fest (und zaehlt auch Kommentartext mit,
                // darum steht das Suffix hier bewusst nicht ausgeschrieben).
                load_person_directory().await;
                Ok(())
            }
            UserManagementAction::LoadSalesPerson(sales_person_id) => {
                load_sales_person(sales_person_id).await;
                Ok(())
            }
            UserManagementAction::UpdateSalesPerson(sales_person) => {
                if USER_MANAGEMENT_STORE.read().sales_person.is_none() {
                    USER_MANAGEMENT_STORE.write().sales_person =
                        Some(SelectedSalesPerson::new(sales_person));
                } else {
                    USER_MANAGEMENT_STORE
                        .write()
                        .sales_person
                        .as_mut()
                        .unwrap()
                        .sales_person = sales_person;
                }
                Ok(())
            }
            UserManagementAction::UpdateSalesPersonUser(user_id) => {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = Some(user_id);
                Ok(())
            }
            UserManagementAction::RemoveSalesPersonUser => {
                USER_MANAGEMENT_STORE
                    .write()
                    .sales_person
                    .as_mut()
                    .unwrap()
                    .user_id = None;
                Ok(())
            }
            UserManagementAction::SaveSalesPersonAndNavigate => {
                match save_sales_person().await {
                    Ok(_) => {
                        // Success Criterion 5: Auffrischen ueber das Aggregat
                        // statt ueber den abgeloesten Sammel-Loader.
                        load_person_directory().await;
                        USER_MANAGEMENT_STORE.write().save_success = true;
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            }
            UserManagementAction::ClearSaveSuccess => {
                USER_MANAGEMENT_STORE.write().save_success = false;
                Ok(())
            }
            UserManagementAction::CreateNewSalesPerson => {
                let new_sales_person = SalesPerson {
                    id: Uuid::nil(),
                    name: "Name".into(),
                    background_color: "#FFF".into(),
                    is_paid: false,
                    inactive: false,
                    version: Uuid::nil(),
                };
                USER_MANAGEMENT_STORE.write().sales_person =
                    Some(SelectedSalesPerson::new(new_sales_person.clone()));
                USER_MANAGEMENT_STORE.write().loaded_sales_person =
                    Some(SelectedSalesPerson::new(new_sales_person));
                Ok(())
            }
            UserManagementAction::LoadUserRoleAssignments(user) => {
                load_role_assignments(user).await
            }
            UserManagementAction::AssignUserToRole(user, role) => {
                assign_user_to_role(user, role).await
            }
            UserManagementAction::RemoveUserFromRole(user, role) => {
                remove_user_from_role(user, role).await
            }
            UserManagementAction::AddUser(user) => match add_user(user).await {
                Ok(()) => {
                    // Success Criterion 5: Auffrischen ueber das Aggregat.
                    load_person_directory().await;
                    Ok(())
                }
                Err(err) => Err(err),
            },
            UserManagementAction::DeleteUser(user) => match delete_user(user).await {
                Ok(()) => {
                    // Success Criterion 5: Auffrischen ueber das Aggregat.
                    load_person_directory().await;
                    Ok(())
                }
                Err(err) => Err(err),
            },
            UserManagementAction::LoadUserInvitations(username) => {
                load_user_invitations(username).await;
                Ok(())
            }
            UserManagementAction::GenerateInvitation(username, expiration_hours) => {
                generate_user_invitation(username, expiration_hours).await
            }
            UserManagementAction::RevokeInvitation(invitation_id) => {
                // We need to get the username from somewhere to reload invitations
                // For now, we'll get it from the first invitation in the store
                let username = USER_MANAGEMENT_STORE
                    .read()
                    .user_invitations
                    .first()
                    .map(|inv| inv.username.clone().into())
                    .unwrap_or_else(|| "".into());
                revoke_user_invitation(invitation_id, username).await
            }
            UserManagementAction::RevokeInvitationSession(invitation_id) => {
                // We need to get the username from somewhere to reload invitations
                // For now, we'll get it from the first invitation in the store
                let username = USER_MANAGEMENT_STORE
                    .read()
                    .user_invitations
                    .first()
                    .map(|inv| inv.username.clone().into())
                    .unwrap_or_else(|| "".into());
                revoke_user_invitation_session(invitation_id, username).await
            }
            UserManagementAction::LoadShiftplanCatalog => {
                load_shiftplan_catalog().await;
                Ok(())
            }
            UserManagementAction::LoadShiftplanAssignments(sales_person_id) => {
                load_shiftplan_assignments(sales_person_id).await;
                Ok(())
            }
            UserManagementAction::UpdateShiftplanAssignments(assignments) => {
                if let Some(sp) = USER_MANAGEMENT_STORE.write().sales_person.as_mut() {
                    sp.shiftplan_assignments = assignments;
                }
                Ok(())
            }
            UserManagementAction::LoadPersonDetail(person_id) => {
                // Variante (b) des Detail-Aggregats: genau **ein**
                // Netzwerk-Aufruf in diesem Arm. Die vier Abschnitte der Seite
                // laden ueber ihre eigenen, seit Langem bestehenden Endpoints;
                // ein zweiter Aufruf hier waere der Anfang eines fetten
                // Client-Aggregats. Der Test haelt das per Quell-Assertion
                // fest und zaehlt auch Kommentartext mit, darum steht das
                // Suffix hier bewusst nicht ausgeschrieben.
                load_person_detail(person_id).await;
                Ok(())
            }
            UserManagementAction::CreatePersonEntry(request) => {
                create_person_entry(request).await
            }
            UserManagementAction::AddPersonFacet { person_id, request } => {
                add_person_facet(person_id, request).await
            }
            UserManagementAction::RemovePersonFacet { person_id, facet } => {
                remove_person_facet(person_id, facet).await
            }
            UserManagementAction::LoadUnlinkedSalesPersons => {
                load_unlinked_sales_persons().await;
                Ok(())
            }
            UserManagementAction::LoadUnlinkedUsers => {
                load_unlinked_users().await;
                Ok(())
            }
            UserManagementAction::LinkCandidate {
                sales_person_id,
                username,
            } => link_candidate(sales_person_id, username).await,
            UserManagementAction::UnlinkFacet { sales_person_id } => {
                unlink_facet(sales_person_id).await
            }
            UserManagementAction::DeleteSalesPerson { sales_person_id } => {
                delete_sales_person(sales_person_id).await
            }
        } {
            Ok(_) => {}
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore {
                    error: Some(err),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Success Criterion 5 (Phase 62): genau **ein** Netzwerk-Aufruf fuellt das
    /// Personen-Verzeichnis. Der Nachweis ist eine Quell-Assertion ueber den
    /// Coroutine-Arm der Lade-Action — unabhaengig von seiner Zeilenzahl darf
    /// darin genau ein `.await` stehen. Ein zweiter `.await` waere ein
    /// Rueckfall in das abgeloeste N+1-Muster.
    #[test]
    fn load_directory_ist_ein_request() {
        const MARKER: &str = "UserManagementAction::LoadPersonDirectory =>";
        let src = include_str!("user_management.rs");
        // Nur der Nicht-Test-Teil der Datei — sonst faende `find` den Marker im
        // Literal dieses Tests selbst (Muster aus `no_legacy_classes_in_source`).
        let prod = &src[..src.find("#[cfg(test)]").expect("test module marker missing")];
        let start = prod
            .find(MARKER)
            .expect("LoadPersonDirectory arm missing in the coroutine match");
        let rest = &prod[start + MARKER.len()..];
        let end = rest
            .find("UserManagementAction::")
            .unwrap_or(rest.len());
        let arm = &rest[..end];
        assert_eq!(
            arm.matches(".await").count(),
            1,
            "expected exactly one `.await` in the LoadPersonDirectory arm, got: {arm}"
        );
    }

    /// Phase 63, Detail-Aggregat Variante (b): die Detailansicht laedt **eine**
    /// Verzeichniszeile. Ihre vier Abschnitte holen sich ihre Daten aus ihren
    /// eigenen, seit Langem bestehenden Endpoints. Ein zweiter `.await` in
    /// diesem Arm waere der Anfang eines fetten Client-Aggregats — genau das,
    /// was die Phase ausdruecklich nicht bauen wollte.
    #[test]
    fn load_person_detail_ist_ein_request() {
        const MARKER: &str = "UserManagementAction::LoadPersonDetail(person_id) =>";
        let src = include_str!("user_management.rs");
        let prod = &src[..src.find("#[cfg(test)]").expect("test module marker missing")];
        let start = prod
            .find(MARKER)
            .expect("LoadPersonDetail arm missing in the coroutine match");
        let rest = &prod[start + MARKER.len()..];
        let end = rest.find("UserManagementAction::").unwrap_or(rest.len());
        let arm = &rest[..end];
        assert_eq!(
            arm.matches(".await").count(),
            1,
            "expected exactly one `.await` in the LoadPersonDetail arm, got: {arm}"
        );
    }

    /// D-63-05: es gibt keinen Sammelknopf. Kein Arm der Koroutine darf den
    /// Verkaeufer-Loeschpfad und den Benutzer-Loeschpfad in einem Zug fahren —
    /// wer beide Seiten los werden will, loest zwei getrennte, jeweils
    /// bestaetigte Aktionen aus. Die Pruefung laeuft ueber den Koroutinen-Rumpf,
    /// nicht ueber die ganze Datei: die Hilfsfunktionen oben tragen beide
    /// Aufrufe naturgemaess, nur eben in getrennten Funktionen.
    #[test]
    fn kein_arm_loescht_beide_seiten_auf_einmal() {
        let src = include_str!("user_management.rs");
        let prod = &src[..src.find("#[cfg(test)]").expect("test module marker missing")];
        let coroutine_start = prod
            .find("pub async fn user_management_service")
            .expect("coroutine missing");
        let coroutine = &prod[coroutine_start..];
        for arm in coroutine.split("UserManagementAction::").skip(1) {
            let deletes_sales_person = arm.contains("delete_sales_person(");
            let deletes_user = arm.contains("delete_user(");
            assert!(
                !(deletes_sales_person && deletes_user),
                "a single arm deletes both facets at once — that is the collective \
                 button D-63-05 rules out: {arm}"
            );
        }
    }
}
