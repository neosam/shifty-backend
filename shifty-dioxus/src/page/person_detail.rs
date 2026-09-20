//! `PersonDetail` — **eine** Detailansicht fuer alle drei Auspraegungen einer
//! Person (Phase 63, PADM-02/03/04/05/07). Sie loest `page/user_details.rs`
//! und `page/sales_person_details.rs` ab (D-63-04): statt zweier Seiten mit je
//! einer halben Wahrheit gibt es eine Route, die je nach Auspraegung genau die
//! Abschnitte zeigt, die es zu dem Eintrag gibt.
//!
//! **Keine leeren Platzhalter.** Fehlt eine Auspraegung, steht an ihrer Stelle
//! keine leere Ueberschrift, sondern genau eine Karte mit den zwei getrennt
//! benannten Aktionen „neu anlegen" und „bestehendes verknuepfen" (D-63-02,
//! Success Criterion 1).
//!
//! **Die Auspraegung entscheidet, nicht die Rolle.** [`visible_sections`] liest
//! ausschliesslich das vom Backend gelieferte `PersonKindTO`. Ein
//! abschnittsweises Gating nach Rollen ist ausdruecklich Zukunft (PADM-F-02)
//! und existiert hier nicht.
//!
//! **Kein farbiges Auspraegungs-Badge.** Der Verknuepfungs-Hinweis ist
//! Fliesstext ohne Farbfuellung. Ein Badge sprach im Phase-62-UAT dieselbe
//! accent/warn-Farbsprache wie der Bezahlt/Freiwillig-Chip und wurde deshalb
//! entfernt; diese Seite zeigt beide Signale und wuerde die Kollision eine
//! Bildschirmseite spaeter wiederholen.
//!
//! **Die Entscheidungslogik liegt in reinen Funktionen.** DOM-Verhalten in
//! Dioxus/WASM ist headless schlecht pruefbar; [`visible_sections`],
//! [`is_last_facet`] und [`delete_confirmation_copy`] sind deshalb freie
//! Funktionen mit direkten Unit-Tests — dasselbe Muster wie `matches_search`
//! in `page/user_management.rs`. Der SSR-Rendertest prueft ein zweites Mal,
//! dass ihr Ergebnis wirklich im Markup ankommt.

use std::rc::Rc;

use dioxus::prelude::*;
use rest_types::{
    AddPersonFacetTO, InvitationResponse, InvitationStatus, PersonEntryTO, PersonFacetTO,
    PersonKindTO, ShiftplanTO,
};
use uuid::Uuid;

use crate::{
    base_types::ImStr,
    component::{
        form::FormCheckbox, Btn, BtnVariant, Dialog, DialogVariant, Field, SelectInput, TextInput,
        TopBar,
    },
    i18n::Key,
    router::Route,
    service::{
        auth::AUTH,
        i18n::I18N,
        user_management::{
            RoleAssignment, SelectedSalesPerson, UserManagementAction, UserManagementStore,
            USER_MANAGEMENT_STORE,
        },
    },
    state::{shiftplan::SalesPerson, ShiftplanAssignment},
};

use super::user_management::{is_admin, NotAuthorizedView};

/// Welche Abschnitte die Detailansicht zeigt. Die vier Felder entsprechen
/// eins zu eins den vier Abschnitts-Ueberschriften der Seite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SectionVisibility {
    pub stammdaten: bool,
    pub shiftplan_zuordnung: bool,
    pub roles: bool,
    pub invitations: bool,
}

impl SectionVisibility {
    /// Die beiden Seiten der Person schalten je zwei Abschnitte gemeinsam:
    /// Stammdaten und Schichtplan-Zuordnung haengen am Verkaeufer-Datensatz,
    /// Rollen und Einladungen am Benutzerkonto.
    pub(crate) const fn new(sales_person_side: bool, user_side: bool) -> Self {
        Self {
            stammdaten: sales_person_side,
            shiftplan_zuordnung: sales_person_side,
            roles: user_side,
            invitations: user_side,
        }
    }

    /// Alle vier Felder in fester Reihenfolge — die Form, in der die Unit-Tests
    /// den vollstaendigen Zustand in einer Zusicherung pruefen, statt vier
    /// einzelne Feldzugriffe zu schreiben. Die Produktion liest die Felder
    /// direkt, deshalb existiert der Helfer nur im Testbau.
    #[cfg(test)]
    pub(crate) const fn as_tuple(&self) -> (bool, bool, bool, bool) {
        (
            self.stammdaten,
            self.shiftplan_zuordnung,
            self.roles,
            self.invitations,
        )
    }
}

/// Die Auspraegung steuert die Sichtbarkeit — nie eine Berechtigung.
///
/// Einzige Eingabe ist das vom Backend gelieferte `PersonKindTO`. Es wird nie
/// aus der Leerheit anderer Felder abgeleitet (T-62-09/T-63-08-E), und es wird
/// nie mit einer Berechtigungspruefung vermischt: ein abschnittsweises
/// Rollen-Gating ist Zukunft (PADM-F-02).
pub(crate) fn visible_sections(kind: PersonKindTO) -> SectionVisibility {
    let (sales_person_side, user_side) = match kind {
        PersonKindTO::Linked => (true, true),
        PersonKindTO::SalesPersonOnly => (true, false),
        PersonKindTO::UserOnly => (false, true),
    };
    SectionVisibility::new(sales_person_side, user_side)
}

/// Ziel einer Auspraegungs-Aktion — welche der beiden Seiten hinzugefuegt,
/// verknuepft oder entfernt werden soll.
///
/// Bewusst ein eigener Typ neben `PersonFacetTO`: der Frontend-Typ steuert
/// Beschriftungen und Sichtbarkeiten, der Transport-Typ die Drahtform. Die
/// Umwandlung steht an genau einer Stelle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FacetKind {
    SalesPerson,
    User,
}

/// Wahr genau dann, wenn das Entfernen dieser Auspraegung zugleich die letzte
/// verbleibende Auspraegung des Eintrags entfernt.
///
/// **Diese Funktion muss dieselbe Wahrheit liefern wie die Orphan-Pruefung des
/// Backends** (`PersonDao::has_live_facets`, D-63-01). Weicht sie ab,
/// verspricht die Bestaetigung etwas anderes, als danach geschieht: entweder
/// verschwindet der Eintrag unangekuendigt aus der Liste, oder die
/// Bestaetigung kuendigt ein Verschwinden an, das nicht eintritt. Beides ist
/// genau der Vertrauensbruch, den PADM-07 ausschliesst.
pub(crate) fn is_last_facet(kind: PersonKindTO, removing: FacetKind) -> bool {
    match (kind, removing) {
        // Verknuepft: die jeweils andere Seite bleibt in jedem Fall stehen.
        (PersonKindTO::Linked, _) => false,
        (PersonKindTO::SalesPersonOnly, FacetKind::SalesPerson) => true,
        (PersonKindTO::SalesPersonOnly, FacetKind::User) => false,
        (PersonKindTO::UserOnly, FacetKind::User) => true,
        (PersonKindTO::UserOnly, FacetKind::SalesPerson) => false,
    }
}

/// Die Textbausteine einer Loesch-Bestaetigung — ausschliesslich als
/// [`Key`]-Werte.
///
/// Bewusst keine fertigen Zeichenketten: so bleibt
/// [`delete_confirmation_copy`] ohne i18n-Laufzeit testbar, und die
/// Uebersetzung passiert erst im Dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeleteConfirmationCopy {
    pub title_key: Key,
    pub body_key: Key,
    /// Nur fuer die Verkaeufer-Seite gesetzt: der Hinweis auf die schonende
    /// Alternative „inaktiv setzen" (D-63-06).
    pub hint_key: Option<Key>,
    /// Nur gesetzt, wenn [`is_last_facet`] fuer dieselbe Kombination wahr ist.
    pub last_facet_key: Option<Key>,
}

/// Welche Saetze die Bestaetigung zeigt — je Seite andere, weil die Wirkung
/// eine andere ist (D-63-08).
///
/// Die Benutzer-Seite verschwindet hart (Login, Rollen, offene Einladungen),
/// die Verkaeufer-Seite weich (Historie bleibt in der Datenbank, verschwindet
/// aber aus allen Ansichten). Deshalb traegt nur die Verkaeufer-Seite den
/// Hinweis auf „inaktiv setzen".
pub(crate) fn delete_confirmation_copy(
    kind: PersonKindTO,
    facet: FacetKind,
) -> DeleteConfirmationCopy {
    let last_facet_key = if is_last_facet(kind, facet) {
        Some(Key::DeleteConfirmLastFacet)
    } else {
        None
    };

    match facet {
        FacetKind::User => DeleteConfirmationCopy {
            title_key: Key::DeleteLoginConfirmTitle,
            body_key: Key::DeleteLoginConfirmBody,
            hint_key: None,
            last_facet_key,
        },
        FacetKind::SalesPerson => DeleteConfirmationCopy {
            title_key: Key::DeleteSalesPersonConfirmTitle,
            body_key: Key::DeleteSalesPersonConfirmBody,
            hint_key: Some(Key::DeleteSalesPersonConfirmHint),
            last_facet_key,
        },
    }
}

// ── Absichten der Ansicht ───────────────────────────────────────────────────

/// Was der Benutzer in der Detailansicht ausloest.
///
/// Die Ansicht selbst kennt weder Koroutine noch Store — sie meldet Absichten,
/// die Seite uebersetzt sie in `UserManagementAction`s. Genau diese Trennung
/// macht den SSR-Rendertest moeglich: die Ansicht laesst sich mit injizierten
/// Daten und leeren Handlern rendern (dasselbe Vorgehen wie
/// `PersonDirectoryTable` in `page/user_management.rs`).
#[derive(Clone, PartialEq)]
pub(crate) enum DetailIntent {
    UpdateSalesPerson(SalesPerson),
    UpdateShiftplanAssignments(Vec<ShiftplanAssignment>),
    SaveSalesPerson,
    SetRoleAssigned { role: ImStr, assigned: bool },
    GenerateInvitation(Option<i64>),
    RevokeInvitation(Uuid),
    RevokeInvitationSession(Uuid),
    /// „Login hinzufuegen" / „Person hinzufuegen" (PADM-03). `value` ist der
    /// Benutzername bzw. der Name der neuen Sales Person — beide sind
    /// serverseitige Pflichtfelder.
    AddFacet { facet: FacetKind, value: ImStr },
    LinkExisting(FacetKind),
    RequestUnlink,
    RequestDeleteLogin,
    RequestDeleteSalesPerson,
}

// ── Seite ───────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Props)]
pub struct PersonDetailProps {
    pub person_id: String,
}

/// Die eine Detailansicht. Sie verdrahtet Store und Koroutine, haelt den
/// Admin-Guard und die fuenf Dialoge; das Markup der Abschnitte liegt in
/// [`EntryDetailView`].
#[component]
pub fn PersonDetail(props: PersonDetailProps) -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    let auth = AUTH.read().clone();

    // PADM-08 / T-63-08-A: Component-Level-Admin-Guard mit fruehem `return`.
    // Die Nav-Leiste blendet die Route fuer Nicht-Admins aus, ein direkter
    // URL-Aufruf umginge das. Defense-in-Depth — die autoritative Grenze ist
    // das Service-Gate aus 63-04/63-05. Dieselbe Stelle im Ablauf wie in
    // `page/user_management.rs`: nach den Store-Lesern, vor jedem Zustand und
    // vor jedem Effekt.
    if !is_admin(auth.auth_info.as_ref()) {
        return rsx! {
            NotAuthorizedView {}
        };
    }

    let store = USER_MANAGEMENT_STORE.read().clone();

    let mut unlink_open = use_signal(|| false);
    let mut delete_login_open = use_signal(|| false);
    let mut delete_sales_person_open = use_signal(|| false);
    let mut picker_target: Signal<Option<FacetKind>> = use_signal(|| None);
    let mut picker_selection: Signal<Option<ImStr>> = use_signal(|| None);
    let mut sections_loaded_for: Signal<Option<Uuid>> = use_signal(|| None);

    use_effect({
        to_owned![props];
        move || {
            // Der Speicher-Erfolg der Verkaeufer-Seite ist ein Ein-Schuss-Flag
            // des Stores; ohne dieses Zuruecksetzen traegt die Seite den
            // Erfolg eines frueheren Besuchs.
            user_management_service.send(UserManagementAction::ClearSaveSuccess);
            if let Ok(person_id) = Uuid::parse_str(&props.person_id) {
                user_management_service.send(UserManagementAction::LoadPersonDetail(person_id));
            }
        }
    });

    // Die Abschnitts-Daten kommen aus ihren eigenen, seit Langem bestehenden
    // Endpoints (Detail-Aggregat Variante (b)) und werden nur fuer die Seiten
    // angestossen, die es laut `visible_sections` ueberhaupt gibt.
    //
    // Der Waechter ueber die `person_id` ist notwendig, nicht Zierde: die
    // Ladevorgaenge schreiben in denselben Store, den dieser Effekt liest —
    // ohne ihn stiesse jedes Ergebnis den naechsten Ladelauf an. Er folgt
    // ausserdem einem Wechsel der Identitaet (D-61-03: beim Zuschalten einer
    // Sales Person gewinnt deren Person).
    use_effect(move || {
        let entry = USER_MANAGEMENT_STORE.read().person_detail.clone();
        let Some(entry) = entry else {
            return;
        };
        if *sections_loaded_for.peek() == Some(entry.person_id) {
            return;
        }
        sections_loaded_for.set(Some(entry.person_id));

        let sections = visible_sections(entry.kind);
        if sections.stammdaten {
            if let Some(sales_person_id) = entry.sales_person_id {
                user_management_service.send(UserManagementAction::LoadSalesPerson(sales_person_id));
                user_management_service
                    .send(UserManagementAction::LoadShiftplanAssignments(sales_person_id));
                user_management_service.send(UserManagementAction::LoadShiftplanCatalog);
            }
        }
        if sections.invitations {
            if let Some(username) = entry.username.as_deref() {
                user_management_service.send(UserManagementAction::LoadUserRoleAssignments(
                    ImStr::from(username),
                ));
                user_management_service
                    .send(UserManagementAction::LoadUserInvitations(ImStr::from(username)));
            }
        }
    });

    let i18n = I18N.read().clone();
    let back_href = format!("{}", Route::UserManagementPage {});
    let entry = store.person_detail.clone();
    let missing = store.person_detail_missing;

    // Die Ansicht folgt dem Eintrag aus dem Store, nicht der id aus der URL:
    // schaltet man einem reinen Benutzerkonto eine Sales Person zu, wechselt
    // der Eintrag nach D-61-03 seine Identitaet, und die uebergebene id zeigt
    // danach ins Leere.
    let dialog_entry = entry.clone();
    let dialog_kind = dialog_entry.as_ref().map(|entry| entry.kind);
    // Kein Fallback auf einen abgeleiteten Namen: liegt kein Eintrag vor,
    // rendert keiner der Dialoge, und der Wert wird nie gezeigt (PADM-06).
    let dialog_name: ImStr = match dialog_entry.as_ref() {
        Some(entry) => ImStr::from(entry.display_name.as_ref()),
        None => ImStr::from(""),
    };
    let sales_person_id = dialog_entry.as_ref().and_then(|entry| entry.sales_person_id);
    let username: Option<ImStr> = dialog_entry
        .as_ref()
        .and_then(|entry| entry.username.as_deref().map(ImStr::from));
    let person_id = dialog_entry.as_ref().map(|entry| entry.person_id);
    let picker_username = username.clone();
    let picker_facet = *picker_target.read();
    let picker_candidates = picker_facet
        .map(|facet| candidates_for(facet, &store))
        .unwrap_or_default();

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-3xl mx-auto",
            a {
                href: "{back_href}",
                class: "inline-flex items-center mb-6 px-2 py-1 text-ink-soft hover:text-ink hover:bg-surface-alt rounded-md",
                "← {i18n.t(Key::BackToUserManagement)}"
            }

            if missing {
                p { class: "text-body text-ink-muted", "{i18n.t(Key::PersonDetailNotFound)}" }
            } else if let Some(entry) = entry.clone() {
                EntryDetailView {
                    entry,
                    sales_person: store.sales_person.clone(),
                    shiftplan_catalog: store.shiftplan_catalog.clone(),
                    role_assignments: store.role_assignements.clone(),
                    invitations: store.user_invitations.clone(),
                    invitations_load_error: store.user_invitations_load_error.clone(),
                    on_intent: move |intent: DetailIntent| {
                        match intent {
                            DetailIntent::UpdateSalesPerson(sales_person) => {
                                user_management_service
                                    .send(UserManagementAction::UpdateSalesPerson(sales_person));
                            }
                            DetailIntent::UpdateShiftplanAssignments(assignments) => {
                                user_management_service
                                    .send(UserManagementAction::UpdateShiftplanAssignments(assignments));
                            }
                            DetailIntent::SaveSalesPerson => {
                                user_management_service
                                    .send(UserManagementAction::SaveSalesPersonAndNavigate);
                            }
                            DetailIntent::SetRoleAssigned { role, assigned } => {
                                if let Some(user) = username.clone() {
                                    if assigned {
                                        user_management_service
                                            .send(UserManagementAction::AssignUserToRole(user, role));
                                    } else {
                                        user_management_service
                                            .send(UserManagementAction::RemoveUserFromRole(user, role));
                                    }
                                }
                            }
                            DetailIntent::GenerateInvitation(hours) => {
                                if let Some(user) = username.clone() {
                                    user_management_service
                                        .send(UserManagementAction::GenerateInvitation(user, hours));
                                }
                            }
                            DetailIntent::RevokeInvitation(invitation_id) => {
                                user_management_service
                                    .send(UserManagementAction::RevokeInvitation(invitation_id));
                            }
                            DetailIntent::RevokeInvitationSession(invitation_id) => {
                                user_management_service
                                    .send(UserManagementAction::RevokeInvitationSession(invitation_id));
                            }
                            DetailIntent::AddFacet { facet, value } => {
                                if let Some(person_id) = person_id {
                                    user_management_service
                                        .send(UserManagementAction::AddPersonFacet {
                                            person_id,
                                            request: add_facet_request(facet, value),
                                        });
                                }
                            }
                            DetailIntent::LinkExisting(facet) => {
                                // Erst laden, dann oeffnen: der Picker zeigt
                                // ausschliesslich noch unverknuepfte Ziele,
                                // und die Liste kann zwischen zwei Besuchen
                                // veraltet sein.
                                match facet {
                                    FacetKind::User => {
                                        user_management_service
                                            .send(UserManagementAction::LoadUnlinkedUsers)
                                    }
                                    FacetKind::SalesPerson => {
                                        user_management_service
                                            .send(UserManagementAction::LoadUnlinkedSalesPersons)
                                    }
                                }
                                picker_selection.set(None);
                                picker_target.set(Some(facet));
                            }
                            DetailIntent::RequestUnlink => unlink_open.set(true),
                            DetailIntent::RequestDeleteLogin => delete_login_open.set(true),
                            DetailIntent::RequestDeleteSalesPerson => {
                                delete_sales_person_open.set(true)
                            }
                        }
                    },
                }
            } else {
                div { class: "py-12 text-center",
                    div { class: "h-2 w-24 mx-auto rounded bg-surface-alt animate-pulse" }
                }
            }
        }

        UnlinkConfirmDialog {
            open: *unlink_open.read(),
            display_name: dialog_name.clone(),
            on_cancel: move |_| unlink_open.set(false),
            on_confirm: move |_| {
                if let Some(sales_person_id) = sales_person_id {
                    user_management_service
                        .send(UserManagementAction::UnlinkFacet { sales_person_id });
                }
                unlink_open.set(false);
            },
        }

        DeleteLoginConfirmDialog {
            open: *delete_login_open.read(),
            kind: dialog_kind,
            on_cancel: move |_| delete_login_open.set(false),
            on_confirm: move |_| {
                if let Some(person_id) = person_id {
                    user_management_service
                        .send(UserManagementAction::RemovePersonFacet {
                            person_id,
                            facet: PersonFacetTO::User,
                        });
                }
                delete_login_open.set(false);
            },
        }

        DeleteSalesPersonConfirmDialog {
            open: *delete_sales_person_open.read(),
            kind: dialog_kind,
            display_name: dialog_name.clone(),
            on_cancel: move |_| delete_sales_person_open.set(false),
            on_confirm: move |_| {
                // D-63-06: der Verkaeufer-Datensatz wird ueber den eigenen,
                // seit Langem bestehenden Loeschpfad entfernt (Soft Delete) —
                // bewusst nicht ueber das Auspraegungs-Entfernen, damit kein
                // Sammelknopf entsteht (D-63-05).
                if let Some(sales_person_id) = sales_person_id {
                    user_management_service
                        .send(UserManagementAction::DeleteSalesPerson { sales_person_id });
                }
                delete_sales_person_open.set(false);
            },
        }

        if let Some(facet) = picker_facet {
            CandidatePickerDialog {
                open: true,
                facet,
                candidates: picker_candidates.clone(),
                selected: picker_selection.read().clone(),
                conflict: store.person_link_conflict,
                on_select: move |value: ImStr| picker_selection.set(Some(value)),
                on_cancel: move |_| picker_target.set(None),
                on_confirm: move |value: ImStr| {
                    // Verknuepft wird immer ueber den Verkaeufer: die eine
                    // Seite bringt ihre id mit, die andere kommt aus der
                    // Auswahl.
                    match facet {
                        FacetKind::User => {
                            if let Some(sales_person_id) = sales_person_id {
                                user_management_service
                                    .send(UserManagementAction::LinkCandidate {
                                        sales_person_id,
                                        username: value,
                                    });
                            }
                        }
                        FacetKind::SalesPerson => {
                            if let (Ok(sales_person_id), Some(username)) = (
                                Uuid::parse_str(value.as_str()),
                                picker_username.clone(),
                            ) {
                                user_management_service
                                    .send(UserManagementAction::LinkCandidate {
                                        sales_person_id,
                                        username,
                                    });
                            }
                        }
                    }
                    picker_target.set(None);
                },
            }
        }
    }
}

/// Baut die Nutzlast der beiden Auspraegungs-Aktionen. Der Server verlangt je
/// Seite genau ein Pflichtfeld und ignoriert das jeweils andere.
fn add_facet_request(facet: FacetKind, value: ImStr) -> AddPersonFacetTO {
    match facet {
        FacetKind::User => AddPersonFacetTO {
            facet: PersonFacetTO::User,
            display_name: None,
            username: Some(std::sync::Arc::from(value.as_str())),
        },
        FacetKind::SalesPerson => AddPersonFacetTO {
            facet: PersonFacetTO::SalesPerson,
            display_name: Some(std::sync::Arc::from(value.as_str())),
            username: None,
        },
    }
}

// ── Ansicht ─────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub(crate) struct EntryDetailViewProps {
    pub entry: PersonEntryTO,
    pub sales_person: Option<SelectedSalesPerson>,
    pub shiftplan_catalog: Rc<[ShiftplanTO]>,
    pub role_assignments: Rc<[RoleAssignment]>,
    pub invitations: Rc<[InvitationResponse]>,
    pub invitations_load_error: Option<ImStr>,
    pub on_intent: EventHandler<DetailIntent>,
}

/// Das Markup der Detailansicht — ohne Store, ohne Koroutine, ohne Netzwerk.
///
/// Welche Abschnitte erscheinen, entscheidet ausschliesslich
/// [`visible_sections`]. Fehlt eine Seite, steht an ihrer Stelle die
/// [`MissingFacetCard`] und **kein** leerer Abschnitt (Success Criterion 1).
#[component]
pub(crate) fn EntryDetailView(props: EntryDetailViewProps) -> Element {
    let i18n = I18N.read().clone();
    let entry = props.entry.clone();
    let sections = visible_sections(entry.kind);
    let on_intent = props.on_intent;
    let display_name = ImStr::from(entry.display_name.as_ref());
    let mut expiration_hours = use_signal(|| ImStr::from("24"));
    let copied_invitation_id: Signal<Option<Uuid>> = use_signal(|| None);

    rsx! {
        h1 { class: "text-h1 text-ink", "{entry.display_name}" }

        // Der Verknuepfungs-Hinweis ist Fliesstext ohne Farbfuellung: das
        // farbige Auspraegungs-Badge wurde im Phase-62-UAT entfernt und wird
        // hier nicht wiederbelebt.
        if entry.kind == PersonKindTO::Linked {
            div { class: "flex flex-wrap items-center justify-between gap-3 mt-2 mb-6",
                p { class: "text-body text-ink-soft", "{i18n.t(Key::PersonDetailLinkedNote)}" }
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_intent.call(DetailIntent::RequestUnlink),
                    "{i18n.t(Key::UnlinkAction)}"
                }
            }
        } else {
            div { class: "mb-6" }
        }

        // ── Verkaeufer-Seite ──────────────────────────────────────────────
        if sections.stammdaten {
            section { class: "mb-8",
                h2 { class: "text-h2 text-ink mb-4 pb-2 border-b border-border",
                    "{i18n.t(Key::BasicInformation)}"
                }
                if let Some(selected) = props.sales_person.as_ref() {
                    {
                        let sales_person = selected.sales_person.clone();
                        let color = sales_person.background_color.clone();
                        let sales_person_for_name = sales_person.clone();
                        let sales_person_for_color = sales_person.clone();
                        let sales_person_for_paid = sales_person.clone();
                        let sales_person_for_inactive = sales_person.clone();
                        rsx! {
                            div { class: "flex flex-col gap-4",
                                Field { label: ImStr::from(i18n.t(Key::Name).as_ref()),
                                    TextInput {
                                        value: ImStr::from(sales_person.name.clone()),
                                        on_change: move |name: ImStr| {
                                            on_intent
                                                .call(
                                                    DetailIntent::UpdateSalesPerson(SalesPerson {
                                                        name: name.as_rc(),
                                                        ..sales_person_for_name.clone()
                                                    }),
                                                );
                                        },
                                    }
                                }
                                Field { label: ImStr::from(i18n.t(Key::ShiftplanColor).as_ref()),
                                    div { class: "flex items-center gap-3",
                                        div {
                                            class: "w-6 h-6 border border-border-strong rounded flex-shrink-0",
                                            style: "background-color: {color}",
                                            title: "{i18n.t(Key::ColorPreview)}",
                                        }
                                        div { class: "flex-1",
                                            TextInput {
                                                value: ImStr::from(sales_person.background_color.clone()),
                                                on_change: move |background_color: ImStr| {
                                                    on_intent
                                                        .call(
                                                            DetailIntent::UpdateSalesPerson(SalesPerson {
                                                                background_color: background_color.as_rc(),
                                                                ..sales_person_for_color.clone()
                                                            }),
                                                        );
                                                },
                                            }
                                        }
                                    }
                                }
                                FormCheckbox {
                                    value: sales_person.is_paid,
                                    on_change: move |is_paid: bool| {
                                        on_intent
                                            .call(
                                                DetailIntent::UpdateSalesPerson(SalesPerson {
                                                    is_paid,
                                                    ..sales_person_for_paid.clone()
                                                }),
                                            );
                                    },
                                    label: rsx! {
                                        "{i18n.t(Key::ThisPersonReceivesPayment)}"
                                    },
                                }
                                FormCheckbox {
                                    value: sales_person.inactive,
                                    on_change: move |inactive: bool| {
                                        on_intent
                                            .call(
                                                DetailIntent::UpdateSalesPerson(SalesPerson {
                                                    inactive,
                                                    ..sales_person_for_inactive.clone()
                                                }),
                                            );
                                    },
                                    label: rsx! {
                                        "{i18n.t(Key::ThisPersonIsInactive)}"
                                    },
                                }
                            }
                        }
                    }
                } else {
                    div { class: "py-6 text-center",
                        p { class: "text-body text-ink-muted",
                            "{i18n.t(Key::LoadingSalesPersonDetails)}"
                        }
                    }
                }
            }

            section { class: "mb-8",
                h2 { class: "text-h2 text-ink mb-4 pb-2 border-b border-border",
                    "{i18n.t(Key::ShiftplanAssignments)}"
                }
                // Neutraler Hinweiskasten: die Vorlage benutzte eine eigene
                // blaue Farbfamilie; auf dieser Seite sind Akzent, Warnung und
                // Gefahr bereits belegt, ein vierter Farbton traegt keine
                // zusaetzliche Bedeutung.
                div { class: "mb-3 p-3 bg-surface-alt border border-border rounded-md text-body text-ink-soft",
                    "{i18n.t(Key::ShiftplanAssignmentsInfo)}"
                }
                {
                    let assignments = props
                        .sales_person
                        .as_ref()
                        .map(|selected| selected.shiftplan_assignments.clone())
                        .unwrap_or_default();
                    let catalog = props.shiftplan_catalog.clone();
                    rsx! {
                        for shiftplan in catalog.iter() {
                            {
                                let shiftplan_id = shiftplan.id;
                                let name = shiftplan.name.clone();
                                let current = assignments
                                    .iter()
                                    .find(|assignment| assignment.shiftplan_id == shiftplan_id)
                                    .cloned();
                                let is_assigned = current.is_some();
                                let assignments_for_toggle = assignments.clone();
                                let assignments_for_level = assignments.clone();
                                rsx! {
                                    div { class: "flex items-center gap-3 border-b border-dashed border-border p-2",
                                        FormCheckbox {
                                            value: is_assigned,
                                            on_change: move |checked: bool| {
                                                let mut next = assignments_for_toggle.clone();
                                                if checked {
                                                    if !next.iter().any(|a| a.shiftplan_id == shiftplan_id) {
                                                        next.push(ShiftplanAssignment {
                                                            shiftplan_id,
                                                            permission_level: "available".to_string(),
                                                        });
                                                    }
                                                } else {
                                                    next.retain(|a| a.shiftplan_id != shiftplan_id);
                                                }
                                                on_intent.call(DetailIntent::UpdateShiftplanAssignments(next));
                                            },
                                            label: rsx! {
                                                "{name}"
                                            },
                                        }
                                        if let Some(assignment) = current.as_ref() {
                                            div { class: "ml-auto w-48",
                                                SelectInput {
                                                    value: Some(ImStr::from(assignment.permission_level.as_str())),
                                                    on_change: move |level: ImStr| {
                                                        let next: Vec<ShiftplanAssignment> = assignments_for_level
                                                            .iter()
                                                            .map(|a| {
                                                                if a.shiftplan_id == shiftplan_id {
                                                                    ShiftplanAssignment {
                                                                        shiftplan_id: a.shiftplan_id,
                                                                        permission_level: level.to_string(),
                                                                    }
                                                                } else {
                                                                    a.clone()
                                                                }
                                                            })
                                                            .collect();
                                                        on_intent.call(DetailIntent::UpdateShiftplanAssignments(next));
                                                    },
                                                    option { value: "available", "{i18n.t(Key::PermissionLevelAvailable)}" }
                                                    option { value: "planner_only", "{i18n.t(Key::PermissionLevelPlannerOnly)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // D-63-05: die Verkaeufer-Loeschung steht im Aktionsbereich ihrer
            // eigenen Seite. Es gibt kein gemeinsames Element mit der
            // Login-Loeschung weiter unten.
            div { class: "flex flex-wrap gap-3 justify-end pt-4 mb-10 border-t border-border",
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: move |_| on_intent.call(DetailIntent::SaveSalesPerson),
                    "{i18n.t(Key::SaveChanges)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_intent.call(DetailIntent::RequestDeleteSalesPerson),
                    "{i18n.t(Key::DeleteSalesPersonAction)}"
                }
            }
        } else {
            MissingFacetCard {
                facet: FacetKind::SalesPerson,
                initial_value: display_name.clone(),
                on_add: move |value: ImStr| {
                    on_intent
                        .call(DetailIntent::AddFacet {
                            facet: FacetKind::SalesPerson,
                            value,
                        })
                },
                on_link: move |_| on_intent.call(DetailIntent::LinkExisting(FacetKind::SalesPerson)),
            }
        }

        // ── Benutzer-Seite ────────────────────────────────────────────────
        if sections.invitations {
            section { class: "mb-8",
                div { class: "flex items-center justify-between mb-4 pb-2 border-b border-border",
                    h2 { class: "text-h2 text-ink", "{i18n.t(Key::RoleAssignments)}" }
                    span { class: "text-small font-normal text-ink-muted bg-surface-alt px-2 py-1 rounded",
                        {
                            let assigned = props
                                .role_assignments
                                .iter()
                                .filter(|assignment| assignment.assigned)
                                .count();
                            let total = props.role_assignments.len();
                            i18n.t(Key::RolesCount)
                                .replace("{assigned}", &assigned.to_string())
                                .replace("{total}", &total.to_string())
                        }
                    }
                }

                if props.role_assignments.is_empty() {
                    div { class: "text-center py-8 text-ink-muted",
                        p { "{i18n.t(Key::NoRolesAvailable)}" }
                        p { class: "text-body", "{i18n.t(Key::ContactAdministratorForRoles)}" }
                    }
                } else {
                    div { class: "flex flex-col gap-3",
                        for assignment in props.role_assignments.iter() {
                            {
                                let role = assignment.role.clone();
                                let role_for_toggle = assignment.role.clone();
                                let assigned = assignment.assigned;
                                rsx! {
                                    div { class: "flex items-center justify-between p-3 bg-surface-alt rounded-lg",
                                        FormCheckbox {
                                            value: assigned,
                                            on_change: move |next: bool| {
                                                on_intent
                                                    .call(DetailIntent::SetRoleAssigned {
                                                        role: role_for_toggle.clone(),
                                                        assigned: next,
                                                    })
                                            },
                                            label: rsx! {
                                                "{role}"
                                            },
                                        }
                                        if assigned {
                                            span { class: "text-micro uppercase bg-good-soft text-good px-2 py-1 rounded-full",
                                                "{i18n.t(Key::Active)}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            section { class: "mb-8",
                div { class: "flex items-center justify-between mb-4 pb-2 border-b border-border",
                    h2 { class: "text-h2 text-ink", "{i18n.t(Key::UserInvitations)}" }
                    span { class: "text-small font-normal text-ink-muted bg-surface-alt px-2 py-1 rounded",
                        {
                            i18n.t(Key::InvitationsCount)
                                .replace("{count}", &props.invitations.len().to_string())
                        }
                    }
                }

                // BUG-02 (v2.2), unveraendert portiert: der Parse-Fehler steht
                // VOR dem Leer-Zweig, damit eine Schema-Drift nicht wie eine
                // leere Liste aussieht.
                if let Some(err) = props.invitations_load_error.as_ref() {
                    div { class: "mb-4 p-3 rounded-md border border-bad bg-bad-soft text-bad text-body",
                        p { class: "font-semibold", "{i18n.t(Key::UserInvitationsLoadError)}" }
                        p { class: "text-small font-normal font-mono mt-1", "{err}" }
                    }
                }

                if props.invitations.is_empty() {
                    div { class: "text-center py-8 text-ink-muted",
                        p { "{i18n.t(Key::NoInvitationsFound)}" }
                        p { class: "text-body", "{i18n.t(Key::GenerateFirstInvitation)}" }
                    }
                } else {
                    div { class: "flex flex-col gap-3 mb-4",
                        for invitation in props.invitations.iter() {
                            {
                                let invitation = invitation.clone();
                                let invitation_id = invitation.id;
                                let link = invitation.invitation_link.clone();
                                let status_class = match invitation.status {
                                    InvitationStatus::Valid => {
                                        "text-micro uppercase bg-good-soft text-good px-2 py-1 rounded-full"
                                    }
                                    InvitationStatus::Expired => {
                                        "text-micro uppercase bg-warn-soft text-warn px-2 py-1 rounded-full"
                                    }
                                    InvitationStatus::Redeemed => {
                                        "text-micro uppercase bg-accent-soft text-accent px-2 py-1 rounded-full"
                                    }
                                    InvitationStatus::SessionRevoked => {
                                        "text-micro uppercase bg-bad-soft text-bad px-2 py-1 rounded-full"
                                    }
                                };
                                let status_label = match invitation.status {
                                    InvitationStatus::Valid => i18n.t(Key::Valid),
                                    InvitationStatus::Expired => i18n.t(Key::Expired),
                                    InvitationStatus::Redeemed => i18n.t(Key::Redeemed),
                                    InvitationStatus::SessionRevoked => i18n.t(Key::SessionRevoked),
                                };
                                let copied = copied_invitation_id.read().as_ref() == Some(&invitation_id);
                                rsx! {
                                    div { class: "p-4 bg-surface-alt rounded-lg",
                                        div { class: "flex items-center gap-2 mb-2",
                                            span { class: "{status_class}", "{status_label}" }
                                            if let Some(redeemed_at) = invitation.redeemed_at.as_ref() {
                                                span { class: "text-small font-normal text-ink-muted",
                                                    "{i18n.t(Key::Redeemed)}: {redeemed_at}"
                                                }
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            // Kein `TextInput`: das Feld ist
                                            // schreibgeschuetzt und traegt
                                            // deshalb keinen Fokusring.
                                            input {
                                                class: "flex-1 px-3 py-2 text-body bg-surface border border-border-strong rounded-md font-mono text-ink-soft",
                                                readonly: true,
                                                value: "{link}",
                                            }
                                            Btn {
                                                variant: BtnVariant::Secondary,
                                                on_click: {
                                                    to_owned![link, copied_invitation_id];
                                                    move |_| {
                                                        let link = link.clone();
                                                        let mut copied_invitation_id = copied_invitation_id;
                                                        spawn(async move {
                                                            if crate::js::copy_to_clipboard(&link).await.is_ok() {
                                                                copied_invitation_id.set(Some(invitation_id));
                                                                gloo_timers::future::sleep(std::time::Duration::from_secs(2))
                                                                    .await;
                                                                copied_invitation_id.set(None);
                                                            }
                                                        });
                                                    }
                                                },
                                                if copied {
                                                    "{i18n.t(Key::InvitationCopied)}"
                                                } else {
                                                    "{i18n.t(Key::CopyToClipboard)}"
                                                }
                                            }
                                        }
                                        if invitation.status == InvitationStatus::Valid {
                                            div { class: "flex justify-end mt-3",
                                                Btn {
                                                    variant: BtnVariant::Danger,
                                                    on_click: move |_| {
                                                        on_intent.call(DetailIntent::RevokeInvitation(invitation_id))
                                                    },
                                                    "{i18n.t(Key::RevokeInvitation)}"
                                                }
                                            }
                                        }
                                        if invitation.status == InvitationStatus::Redeemed {
                                            div { class: "flex justify-end mt-3",
                                                Btn {
                                                    variant: BtnVariant::Danger,
                                                    on_click: move |_| {
                                                        on_intent
                                                            .call(DetailIntent::RevokeInvitationSession(invitation_id))
                                                    },
                                                    "{i18n.t(Key::RevokeSession)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "border-t border-border pt-4",
                    h3 { class: "text-body font-semibold text-ink mb-3",
                        "{i18n.t(Key::GenerateNewInvitation)}"
                    }
                    div { class: "flex flex-col sm:flex-row gap-2 sm:items-end",
                        div { class: "flex-1",
                            Field { label: ImStr::from(i18n.t(Key::ExpirationHours).as_ref()),
                                TextInput {
                                    value: expiration_hours.read().clone(),
                                    input_type: ImStr::from("number"),
                                    on_change: move |value: ImStr| expiration_hours.set(value),
                                }
                            }
                        }
                        Btn {
                            variant: BtnVariant::Secondary,
                            on_click: move |_| {
                                let hours = expiration_hours.read().as_str().parse::<i64>().ok();
                                on_intent.call(DetailIntent::GenerateInvitation(hours));
                                expiration_hours.set(ImStr::from("24"));
                            },
                            "{i18n.t(Key::GenerateInvitation)}"
                        }
                    }
                }
            }

            // D-63-05: die Login-Loeschung steht im Aktionsbereich ihrer
            // eigenen Seite — getrennt von der Verkaeufer-Loeschung oben.
            div { class: "flex justify-end pt-4 border-t border-border",
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_intent.call(DetailIntent::RequestDeleteLogin),
                    "{i18n.t(Key::DeleteLoginAction)}"
                }
            }
        } else {
            MissingFacetCard {
                facet: FacetKind::User,
                initial_value: ImStr::from(""),
                on_add: move |value: ImStr| {
                    on_intent
                        .call(DetailIntent::AddFacet {
                            facet: FacetKind::User,
                            value,
                        })
                },
                on_link: move |_| on_intent.call(DetailIntent::LinkExisting(FacetKind::User)),
            }
        }
    }
}

// ── Missing-Facet-Karte ─────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub(crate) struct MissingFacetCardProps {
    pub facet: FacetKind,
    /// Vorbelegung des Namensfeldes. Fuer die Verkaeufer-Seite der
    /// Anzeigename des Eintrags, fuer die Benutzer-Seite leer — einen
    /// Benutzernamen leitet das Frontend nicht aus einem Anzeigenamen ab.
    pub initial_value: ImStr,
    pub on_add: EventHandler<ImStr>,
    pub on_link: EventHandler<()>,
}

/// Steht anstelle der Abschnitte, die es zu dieser Auspraegung nicht gibt.
///
/// Neutral-gestrichelt, nicht warnfarben: eine fehlende Auspraegung ist ein
/// erwarteter Zustand, kein Fehler — genau die „Luecke, kein Alarm"-Sprache,
/// die das entfernte Auspraegungs-Badge einmal trug.
///
/// Die beiden Aktionen bleiben getrennt benannt (D-63-02): „neu anlegen" und
/// „bestehendes verknuepfen" sind zwei verschiedene Dinge, kein mehrdeutiges
/// Kombi-Feld.
#[component]
pub(crate) fn MissingFacetCard(props: MissingFacetCardProps) -> Element {
    let i18n = I18N.read().clone();
    let mut value = use_signal(|| props.initial_value.clone());
    let (heading_key, body_key, field_key, add_key, link_key) = match props.facet {
        FacetKind::User => (
            Key::MissingLoginHeading,
            Key::MissingLoginBody,
            Key::ColumnUsername,
            Key::AddLoginToEntry,
            Key::LinkExistingUser,
        ),
        FacetKind::SalesPerson => (
            Key::MissingSalesPersonHeading,
            Key::MissingSalesPersonBody,
            Key::CreatePersonEntryDisplayName,
            Key::AddSalesPersonToEntry,
            Key::LinkExistingSalesPerson,
        ),
    };
    let current = value.read().clone();
    // Die Sperre ist Bequemlichkeit, keine Sicherheitsgrenze: der Endpoint
    // lehnt einen leeren Namen unabhaengig vom Client ab (ASVS V5).
    let submit_disabled = current.as_str().trim().is_empty();
    let on_add = props.on_add;
    let on_link = props.on_link;

    rsx! {
        div { class: "border border-dashed border-border-strong bg-surface-alt rounded-lg p-6 text-center mb-8",
            p { class: "text-h2 text-ink mb-1", "{i18n.t(heading_key)}" }
            p { class: "text-body text-ink-soft mb-4", "{i18n.t(body_key)}" }
            div { class: "max-w-xs mx-auto mb-4 text-left",
                Field { label: ImStr::from(i18n.t(field_key).as_ref()),
                    TextInput {
                        value: current.clone(),
                        on_change: move |next: ImStr| value.set(next),
                    }
                }
            }
            div { class: "flex items-center justify-center gap-2",
                Btn {
                    variant: BtnVariant::Primary,
                    disabled: submit_disabled,
                    on_click: move |_| {
                        let next = value.read().clone();
                        if !next.as_str().trim().is_empty() {
                            on_add.call(next);
                        }
                    },
                    "{i18n.t(add_key)}"
                }
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_link.call(()),
                    "{i18n.t(link_key)}"
                }
            }
        }
    }
}

// ── Bestaetigungen ──────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub(crate) struct UnlinkConfirmDialogProps {
    pub open: bool,
    pub display_name: ImStr,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

/// D-63-05: harmlos und umkehrbar — beide Objekte bleiben vollstaendig
/// erhalten. Deshalb bewusst sekundaer gestaltet und nicht rot.
#[component]
pub(crate) fn UnlinkConfirmDialog(props: UnlinkConfirmDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let body = i18n
        .t(Key::UnlinkConfirmBody)
        .replace("{name}", props.display_name.as_str());
    let on_close = props.on_cancel;
    let on_cancel = props.on_cancel;
    let on_confirm = props.on_confirm;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(i18n.t(Key::UnlinkConfirmTitle).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_confirm.call(()),
                    "{i18n.t(Key::UnlinkAction)}"
                }
            }),
            p { class: "text-body text-ink", "{body}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct DeleteLoginConfirmDialogProps {
    pub open: bool,
    /// Die Auspraegung des Eintrags — sie entscheidet, ob der
    /// Letzte-Auspraegung-Zusatz erscheint.
    pub kind: Option<PersonKindTO>,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

/// Hard Delete der Benutzer-Seite. Die Texte kommen vollstaendig aus
/// [`delete_confirmation_copy`]; uebersetzt wird erst hier.
#[component]
pub(crate) fn DeleteLoginConfirmDialog(props: DeleteLoginConfirmDialogProps) -> Element {
    let (true, Some(kind)) = (props.open, props.kind) else {
        return rsx! {};
    };
    let i18n = I18N.read().clone();
    let copy = delete_confirmation_copy(kind, FacetKind::User);
    let on_close = props.on_cancel;
    let on_cancel = props.on_cancel;
    let on_confirm = props.on_confirm;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(i18n.t(copy.title_key).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_confirm.call(()),
                    "{i18n.t(Key::DeleteLoginAction)}"
                }
            }),
            p { class: "text-body text-ink", "{i18n.t(copy.body_key)}" }
            p { class: "text-body text-ink-soft mt-2", "{i18n.t(Key::DeleteConfirmIrreversible)}" }
            if let Some(last_facet_key) = copy.last_facet_key {
                p { class: "text-body text-ink-soft mt-2", "{i18n.t(last_facet_key)}" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct DeleteSalesPersonConfirmDialogProps {
    pub open: bool,
    pub kind: Option<PersonKindTO>,
    pub display_name: ImStr,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<()>,
}

/// Soft Delete der Verkaeufer-Seite. Der Hinweis auf „inaktiv setzen"
/// (D-63-06) steht nur hier — die Benutzer-Seite kennt keine solche
/// Alternative.
#[component]
pub(crate) fn DeleteSalesPersonConfirmDialog(
    props: DeleteSalesPersonConfirmDialogProps,
) -> Element {
    let (true, Some(kind)) = (props.open, props.kind) else {
        return rsx! {};
    };
    let i18n = I18N.read().clone();
    let copy = delete_confirmation_copy(kind, FacetKind::SalesPerson);
    let hint = copy
        .hint_key
        .map(|key| i18n.t(key).replace("{name}", props.display_name.as_str()));
    let on_close = props.on_cancel;
    let on_cancel = props.on_cancel;
    let on_confirm = props.on_confirm;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(i18n.t(copy.title_key).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_confirm.call(()),
                    "{i18n.t(Key::DeleteSalesPersonAction)}"
                }
            }),
            p { class: "text-body text-ink", "{i18n.t(copy.body_key)}" }
            p { class: "text-body text-ink-soft mt-2", "{i18n.t(Key::DeleteConfirmIrreversible)}" }
            if let Some(hint) = hint.as_ref() {
                p { class: "text-body text-ink-soft mt-2", "{hint}" }
            }
            if let Some(last_facet_key) = copy.last_facet_key {
                p { class: "text-body text-ink-soft mt-2", "{i18n.t(last_facet_key)}" }
            }
        }
    }
}

// ── Anlege-Dialog ───────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct CreatePersonEntryDialogProps {
    pub open: bool,
    pub display_name: ImStr,
    pub wants_login: bool,
    pub wants_sales_person: bool,
    pub username: ImStr,
    pub on_display_name_change: EventHandler<ImStr>,
    pub on_wants_login_change: EventHandler<bool>,
    pub on_wants_sales_person_change: EventHandler<bool>,
    pub on_username_change: EventHandler<ImStr>,
    pub on_cancel: EventHandler<()>,
    pub on_create: EventHandler<()>,
}

/// „Eintrag anlegen" (PADM-02, D-63-07) — bewusst schlank: Anzeigename, zwei
/// Umschalter, Benutzername nur solange die Login-Seite zugeschaltet ist.
/// Farbe, Bezahlt-Kennzeichen, Inaktiv-Kennzeichen und Schichtplan-Zuordnung
/// fragt der Dialog nicht ab; sie werden danach in der Detailansicht gepflegt,
/// in die der Anlegevorgang weiterleitet.
///
/// Der Dialog lebt in dieser Datei, obwohl ihn ab Plan 63-09 die Werkzeugleiste
/// der Listenseite oeffnet: er gehoert inhaltlich zur Personen-Detailwelt.
#[component]
pub fn CreatePersonEntryDialog(props: CreatePersonEntryDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    // Die Sperre ist reine Bequemlichkeit — die verbindliche Pruefung liegt im
    // Service (D-63-07, ASVS V5, Plan 63-05). Der Endpoint lehnt eine Anfrage
    // ohne gesetzten Umschalter unabhaengig vom Client-Zustand ab.
    let no_facet_selected = !props.wants_login && !props.wants_sales_person;
    let login_without_name = props.wants_login && props.username.as_str().trim().is_empty();
    let submit_disabled = no_facet_selected || login_without_name;

    let on_close = props.on_cancel;
    let on_cancel = props.on_cancel;
    let on_create = props.on_create;
    let on_display_name_change = props.on_display_name_change;
    let on_wants_login_change = props.on_wants_login_change;
    let on_wants_sales_person_change = props.on_wants_sales_person_change;
    let on_username_change = props.on_username_change;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(i18n.t(Key::CreatePersonEntry).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Primary,
                    disabled: submit_disabled,
                    on_click: move |_| on_create.call(()),
                    "{i18n.t(Key::CreatePersonEntrySubmit)}"
                }
            }),
            div { class: "flex flex-col gap-4",
                Field { label: ImStr::from(i18n.t(Key::CreatePersonEntryDisplayName).as_ref()),
                    TextInput {
                        value: props.display_name.clone(),
                        on_change: move |value: ImStr| on_display_name_change.call(value),
                    }
                }

                div { class: "flex flex-col gap-2",
                    FormCheckbox {
                        value: props.wants_login,
                        on_change: move |value: bool| on_wants_login_change.call(value),
                        label: rsx! {
                            "{i18n.t(Key::CreatePersonEntryWantsLogin)}"
                        },
                    }
                    FormCheckbox {
                        value: props.wants_sales_person,
                        on_change: move |value: bool| on_wants_sales_person_change.call(value),
                        label: rsx! {
                            "{i18n.t(Key::CreatePersonEntryWantsSalesPerson)}"
                        },
                    }
                    // Der Hinweis steht dauerhaft, nicht erst nach einem
                    // abgelehnten Klick: die Bedingung soll sichtbar sein,
                    // bevor jemand sie verletzt.
                    p { class: "text-micro text-ink-muted",
                        "{i18n.t(Key::CreatePersonEntryValidationHint)}"
                    }
                }

                if props.wants_login {
                    Field { label: ImStr::from(i18n.t(Key::ColumnUsername).as_ref()),
                        TextInput {
                            value: props.username.clone(),
                            on_change: move |value: ImStr| on_username_change.call(value),
                        }
                    }
                }
            }
        }
    }
}

// ── Kandidaten-Picker ───────────────────────────────────────────────────────

/// Ein waehlbares Ziel: die Drahtform und die Beschriftung.
#[derive(Clone, PartialEq)]
pub(crate) struct CandidateOption {
    pub value: ImStr,
    pub label: ImStr,
}

#[derive(Props, Clone, PartialEq)]
pub(crate) struct CandidatePickerDialogProps {
    pub open: bool,
    /// Welche Seite verknuepft werden soll — sie bestimmt Titel, Leer-Text und
    /// die Herkunft der Liste.
    pub facet: FacetKind,
    pub candidates: Vec<CandidateOption>,
    pub selected: Option<ImStr>,
    pub conflict: bool,
    pub on_select: EventHandler<ImStr>,
    pub on_cancel: EventHandler<()>,
    pub on_confirm: EventHandler<ImStr>,
}

/// Bietet ausschliesslich noch unverknuepfte Ziele an (SC-4, PADM-04).
///
/// Ein einziges Auswahlfeld, bewusst ohne Such- oder Filtermechanik: die
/// Ausserhalb-des-Umfangs-Tabelle der Requirements schliesst Namens-Matching
/// fuer diese Organisationsgroesse ausdruecklich aus. Ist die Liste leer,
/// erscheint der Leer-Text statt eines leeren Auswahlfeldes.
#[component]
pub(crate) fn CandidatePickerDialog(props: CandidatePickerDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let (title_key, empty_key) = match props.facet {
        FacetKind::User => (Key::LinkExistingUser, Key::CandidatePickerEmptyUser),
        FacetKind::SalesPerson => (
            Key::LinkExistingSalesPerson,
            Key::CandidatePickerEmptySalesPerson,
        ),
    };
    // Ohne ausdrueckliche Wahl gilt der erste Eintrag — genau der, den das
    // Auswahlfeld auch anzeigt. Sonst laege die sichtbare Vorauswahl neben der
    // tatsaechlich abgeschickten.
    let effective = props
        .selected
        .clone()
        .or_else(|| props.candidates.first().map(|option| option.value.clone()));
    let candidates = props.candidates.clone();

    let on_close = props.on_cancel;
    let on_cancel = props.on_cancel;
    let on_confirm = props.on_confirm;
    let on_select = props.on_select;
    let confirm_value = effective.clone();

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_close.call(()),
            title: ImStr::from(i18n.t(title_key).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Primary,
                    disabled: confirm_value.is_none(),
                    on_click: move |_| {
                        if let Some(value) = confirm_value.clone() {
                            on_confirm.call(value);
                        }
                    },
                    "{i18n.t(Key::LinkCandidateConfirm)}"
                }
            }),
            if candidates.is_empty() {
                p { class: "text-body text-ink-muted", "{i18n.t(empty_key)}" }
            } else {
                Field { label: ImStr::from(i18n.t(title_key).as_ref()),
                    SelectInput {
                        value: effective.clone(),
                        on_change: move |value: ImStr| on_select.call(value),
                        for option in candidates.iter() {
                            option { value: "{option.value}", "{option.label}" }
                        }
                    }
                }
            }
            if props.conflict {
                p { class: "text-body text-warn mt-3", "{i18n.t(Key::PersonLinkConflict)}" }
            }
        }
    }
}

/// Welche Ziele der Picker fuer diese Seite anbietet.
fn candidates_for(facet: FacetKind, store: &UserManagementStore) -> Vec<CandidateOption> {
    match facet {
        FacetKind::User => store
            .unlinked_user_candidates
            .iter()
            .map(|user| CandidateOption {
                value: ImStr::from(user.name.as_str()),
                label: ImStr::from(user.name.as_str()),
            })
            .collect(),
        FacetKind::SalesPerson => store
            .unlinked_sales_person_candidates
            .iter()
            .map(|sales_person| CandidateOption {
                value: ImStr::from(sales_person.id.to_string()),
                label: ImStr::from(sales_person.name.as_ref()),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    /// Durchsetzungsmechanismus der Retokenization-Auflage aus der UI-SPEC:
    /// diese Datei portiert Markup aus zwei Seiten, die aelter sind als die
    /// Token-Migration. Kein einziger ihrer Alt-Klassennamen darf mitkommen.
    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("person_detail.rs");
        // Nur der Nicht-Test-Teil: sonst faende die Suche die hier
        // aufgezaehlten Begriffe in sich selbst (Muster aus
        // `page/user_management.rs`).
        let test_module_start = src
            .find("#[cfg(test)]")
            .expect("test module marker missing");
        let prefix = &src[..test_module_start];
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "text-yellow-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "bg-yellow-",
            "border-black",
            "border-gray-",
            "border-blue-",
            "border-green-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }

    // ── SSR-Rendertests je Auspraegung ───────────────────────────────────────

    /// Fixture. `sales_person` steuert die vier Verkaeufer-Felder gemeinsam —
    /// sie sind laut DTO-Invariante genau dann gefuellt, wenn die Person eine
    /// lebende Sales Person traegt.
    fn sample_entry(
        kind: PersonKindTO,
        display_name: &str,
        username: Option<&str>,
        sales_person: bool,
    ) -> PersonEntryTO {
        PersonEntryTO {
            person_id: Uuid::new_v4(),
            kind,
            display_name: Arc::from(display_name),
            username: username.map(Arc::from),
            sales_person_id: if sales_person {
                Some(Uuid::new_v4())
            } else {
                None
            },
            background_color: if sales_person {
                Some(Arc::from("#dbe0ff"))
            } else {
                None
            },
            is_paid: if sales_person { Some(true) } else { None },
            inactive: if sales_person { Some(false) } else { None },
            roles: Vec::new(),
        }
    }

    /// Rendert die Ansicht mit injiziertem Eintrag — ohne Store, ohne
    /// Koroutine, ohne Netzwerk.
    fn render_detail(entry: PersonEntryTO) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            entry: PersonEntryTO,
        }
        #[component]
        fn DetailWrapper(props: WrapperProps) -> Element {
            rsx! {
                EntryDetailView {
                    entry: props.entry.clone(),
                    sales_person: None,
                    shiftplan_catalog: Rc::from(Vec::<ShiftplanTO>::new()),
                    role_assignments: Rc::from(Vec::<RoleAssignment>::new()),
                    invitations: Rc::from(Vec::<InvitationResponse>::new()),
                    invitations_load_error: None,
                    on_intent: |_: DetailIntent| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(DetailWrapper, WrapperProps { entry });
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    const MISSING_CARD_CLASSES: &str = "border-dashed border-border-strong";

    /// Success Criterion 1, Positiv-Richtung: der verknuepfte Eintrag zeigt
    /// alle vier Abschnitte und keine Missing-Facet-Karte.
    #[test]
    fn detail_linked_renders_every_section() {
        let html = render_detail(sample_entry(
            PersonKindTO::Linked,
            "Lena",
            Some("lena"),
            true,
        ));
        for heading in [
            "Basic Information",
            "Shiftplan Assignments",
            "Role Assignments",
            "User Invitations",
        ] {
            assert!(html.contains(heading), "Abschnitt {heading} fehlt: {html}");
        }
        assert!(
            !html.contains(MISSING_CARD_CLASSES),
            "verknuepfter Eintrag darf keine Missing-Facet-Karte tragen: {html}"
        );
    }

    /// Success Criterion 1, Negativ-Richtung: der fehlende Abschnitt steht
    /// NICHT als leere Ueberschrift im Markup — an seiner Stelle steht genau
    /// eine Karte.
    #[test]
    fn detail_sales_person_only_hides_the_user_side() {
        let html = render_detail(sample_entry(
            PersonKindTO::SalesPersonOnly,
            "Tom",
            None,
            true,
        ));
        assert!(html.contains("Basic Information"), "{html}");
        assert!(html.contains("Shiftplan Assignments"), "{html}");
        assert!(
            !html.contains("Role Assignments"),
            "leerer Platzhalter-Abschnitt gerendert: {html}"
        );
        assert!(
            !html.contains("User Invitations"),
            "leerer Platzhalter-Abschnitt gerendert: {html}"
        );
        assert_eq!(
            html.matches(MISSING_CARD_CLASSES).count(),
            1,
            "genau eine Karte fuer die fehlende Seite erwartet: {html}"
        );
        assert!(html.contains("No login yet"), "{html}");
    }

    #[test]
    fn detail_user_only_hides_the_sales_person_side() {
        let html = render_detail(sample_entry(
            PersonKindTO::UserOnly,
            "Zoe",
            Some("zoe"),
            false,
        ));
        assert!(html.contains("Role Assignments"), "{html}");
        assert!(html.contains("User Invitations"), "{html}");
        assert!(
            !html.contains("Basic Information"),
            "leerer Platzhalter-Abschnitt gerendert: {html}"
        );
        assert!(
            !html.contains("Shiftplan Assignments"),
            "leerer Platzhalter-Abschnitt gerendert: {html}"
        );
        assert_eq!(
            html.matches(MISSING_CARD_CLASSES).count(),
            1,
            "genau eine Karte fuer die fehlende Seite erwartet: {html}"
        );
        assert!(html.contains("No sales-person record yet"), "{html}");
    }

    /// Der Verknuepfungs-Hinweis erscheint nur bei der verknuepften
    /// Auspraegung und ist Fliesstext — kein wiederbelebtes Badge.
    #[test]
    fn detail_linked_note_is_plain_text_and_only_when_linked() {
        const NOTE: &str = "share the same identity";
        let linked = render_detail(sample_entry(
            PersonKindTO::Linked,
            "Lena",
            Some("lena"),
            true,
        ));
        let position = linked
            .find(NOTE)
            .unwrap_or_else(|| panic!("Verknuepfungs-Hinweis fehlt: {linked}"));
        let element_start = linked[..position]
            .rfind('<')
            .expect("kein oeffnendes Element vor dem Hinweis");
        let element = &linked[element_start..position];
        assert!(
            element.contains("text-ink-soft"),
            "Hinweis traegt nicht die gedaempfte Textfarbe: {element}"
        );
        for chip_class in [
            "rounded-full",
            "bg-accent-soft",
            "bg-warn-soft",
            "border-accent",
            "border-warn",
        ] {
            assert!(
                !element.contains(chip_class),
                "Auspraegungs-Badge ({chip_class}) ist zurueckgekehrt: {element}"
            );
        }

        for entry in [
            sample_entry(PersonKindTO::SalesPersonOnly, "Tom", None, true),
            sample_entry(PersonKindTO::UserOnly, "Zoe", Some("zoe"), false),
        ] {
            let html = render_detail(entry);
            assert!(
                !html.contains(NOTE),
                "Verknuepfungs-Hinweis bei nicht verknuepftem Eintrag: {html}"
            );
        }
    }

    /// PADM-06 / Success Criterion 3: der Anzeigename wird stumpf
    /// interpoliert. Keine Zeile, die ihn anfasst, darf eine Fallback-Regel
    /// tragen — die gehoert ins Backend, sonst gibt es zwei Quellen derselben
    /// Wahrheit.
    #[test]
    fn no_display_name_fallback_in_source() {
        let src = include_str!("person_detail.rs");
        let prefix = &src[..src.find("#[cfg(test)]").expect("test module marker missing")];
        for line in prefix.lines() {
            if !line.contains("display_name") {
                continue;
            }
            assert!(
                !line.contains("unwrap_or") && !line.contains(".or("),
                "Fallback-Regel auf dem Anzeigenamen: {line}"
            );
        }
    }

    // ── Anlege-Dialog ────────────────────────────────────────────────────────

    fn render_create_dialog(open: bool, wants_login: bool, wants_sales_person: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            open: bool,
            wants_login: bool,
            wants_sales_person: bool,
        }
        #[component]
        fn CreateWrapper(props: WrapperProps) -> Element {
            rsx! {
                CreatePersonEntryDialog {
                    open: props.open,
                    display_name: ImStr::from(""),
                    wants_login: props.wants_login,
                    wants_sales_person: props.wants_sales_person,
                    username: ImStr::from(""),
                    on_display_name_change: |_: ImStr| {},
                    on_wants_login_change: |_: bool| {},
                    on_wants_sales_person_change: |_: bool| {},
                    on_username_change: |_: ImStr| {},
                    on_cancel: |_| {},
                    on_create: |_| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            CreateWrapper,
            WrapperProps {
                open,
                wants_login,
                wants_sales_person,
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn create_person_entry_dialog_closed_renders_nothing() {
        let html = render_create_dialog(false, false, false);
        assert!(
            !html.contains("role=\"dialog\""),
            "geschlossener Dialog rendert Markup: {html}"
        );
    }

    /// D-63-07: der Hinweis steht sofort, und der Absende-Knopf ist gesperrt —
    /// nicht erst nach einem abgelehnten Klick.
    #[test]
    fn create_person_entry_dialog_without_a_toggle_blocks_and_explains() {
        let html = render_create_dialog(true, false, false);
        assert!(html.contains("role=\"dialog\""), "{html}");
        assert!(
            html.contains("At least a login or a sales-person record is required."),
            "Hinweistext fehlt: {html}"
        );
        assert!(
            html.contains("cursor-not-allowed"),
            "Absende-Knopf ist nicht gesperrt: {html}"
        );
    }

    #[test]
    fn create_person_entry_dialog_shows_username_only_with_the_login_toggle() {
        let with_login = render_create_dialog(true, true, false);
        assert!(
            with_login.contains("Username"),
            "Benutzername-Feld fehlt bei zugeschaltetem Login: {with_login}"
        );

        let without_login = render_create_dialog(true, false, true);
        assert!(
            !without_login.contains("Username"),
            "Benutzername-Feld steht ohne Login im Markup: {without_login}"
        );
        // Ohne Login, aber mit Verkaeufer-Datensatz ist die Eingabe gueltig.
        assert!(
            !without_login.contains("cursor-not-allowed"),
            "ein gesetzter Umschalter genuegt: {without_login}"
        );
    }

    // ── Kandidaten-Picker ────────────────────────────────────────────────────

    fn render_picker(facet: FacetKind, candidates: Vec<CandidateOption>) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            facet: FacetKind,
            candidates: Vec<CandidateOption>,
        }
        #[component]
        fn PickerWrapper(props: WrapperProps) -> Element {
            rsx! {
                CandidatePickerDialog {
                    open: true,
                    facet: props.facet,
                    candidates: props.candidates.clone(),
                    selected: None,
                    conflict: false,
                    on_select: |_: ImStr| {},
                    on_cancel: |_| {},
                    on_confirm: |_: ImStr| {},
                }
            }
        }
        let mut vdom =
            VirtualDom::new_with_props(PickerWrapper, WrapperProps { facet, candidates });
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn candidate_picker_empty_list_shows_the_empty_state_instead_of_a_select() {
        let html = render_picker(FacetKind::User, Vec::new());
        assert!(
            html.contains("No unlinked user accounts available."),
            "Leer-Text fehlt: {html}"
        );
        assert!(
            !html.contains("<select"),
            "leeres Auswahlfeld gerendert: {html}"
        );
        assert!(
            html.contains("cursor-not-allowed"),
            "Absende-Knopf ist nicht gesperrt: {html}"
        );
    }

    /// SC-4: genau ein Auswahlfeld mit je einer Option pro Kandidat — und
    /// nirgends ein Freitextfeld.
    #[test]
    fn candidate_picker_lists_one_option_per_candidate() {
        let candidates = vec![
            CandidateOption {
                value: ImStr::from("lena"),
                label: ImStr::from("lena"),
            },
            CandidateOption {
                value: ImStr::from("tom"),
                label: ImStr::from("tom"),
            },
        ];
        let html = render_picker(FacetKind::User, candidates);
        assert_eq!(html.matches("<select").count(), 1, "{html}");
        assert_eq!(html.matches("<option").count(), 2, "{html}");
        assert!(html.contains("lena") && html.contains("tom"), "{html}");
        assert!(
            !html.contains("type=\"text\""),
            "Freitexteingabe im Picker: {html}"
        );
        assert!(
            !html.contains("cursor-not-allowed"),
            "mit Kandidaten ist der Absende-Knopf frei: {html}"
        );
    }

    #[test]
    fn candidate_picker_sales_person_variant_has_its_own_empty_state() {
        let html = render_picker(FacetKind::SalesPerson, Vec::new());
        assert!(
            html.contains("No unlinked sales persons available."),
            "Leer-Text der Verkaeufer-Variante fehlt: {html}"
        );
    }

    // ── is_last_facet ────────────────────────────────────────────────────────

    #[test]
    fn is_last_facet_linked_is_never_last() {
        assert!(!is_last_facet(PersonKindTO::Linked, FacetKind::SalesPerson));
        assert!(!is_last_facet(PersonKindTO::Linked, FacetKind::User));
    }

    #[test]
    fn is_last_facet_sales_person_only() {
        assert!(is_last_facet(
            PersonKindTO::SalesPersonOnly,
            FacetKind::SalesPerson
        ));
        assert!(!is_last_facet(
            PersonKindTO::SalesPersonOnly,
            FacetKind::User
        ));
    }

    #[test]
    fn is_last_facet_user_only() {
        assert!(is_last_facet(PersonKindTO::UserOnly, FacetKind::User));
        assert!(!is_last_facet(
            PersonKindTO::UserOnly,
            FacetKind::SalesPerson
        ));
    }

    // ── delete_confirmation_copy ─────────────────────────────────────────────

    #[test]
    fn delete_confirmation_copy_user_side_has_no_hint() {
        for kind in [
            PersonKindTO::Linked,
            PersonKindTO::SalesPersonOnly,
            PersonKindTO::UserOnly,
        ] {
            let copy = delete_confirmation_copy(kind, FacetKind::User);
            assert_eq!(copy.title_key, Key::DeleteLoginConfirmTitle);
            assert_eq!(copy.body_key, Key::DeleteLoginConfirmBody);
            assert_eq!(
                copy.hint_key, None,
                "die Benutzer-Seite kennt keine schonende Alternative"
            );
        }
    }

    #[test]
    fn delete_confirmation_copy_sales_person_side_names_the_alternative() {
        for kind in [
            PersonKindTO::Linked,
            PersonKindTO::SalesPersonOnly,
            PersonKindTO::UserOnly,
        ] {
            let copy = delete_confirmation_copy(kind, FacetKind::SalesPerson);
            assert_eq!(copy.title_key, Key::DeleteSalesPersonConfirmTitle);
            assert_eq!(copy.body_key, Key::DeleteSalesPersonConfirmBody);
            assert_eq!(
                copy.hint_key,
                Some(Key::DeleteSalesPersonConfirmHint),
                "D-63-06 verlangt den Hinweis auf „inaktiv setzen\""
            );
        }
    }

    /// Der Letzte-Auspraegung-Zusatz steht genau dann, wenn `is_last_facet`
    /// fuer dieselbe Kombination wahr ist — die Bestaetigung und die
    /// Orphan-Regel duerfen nicht auseinanderlaufen.
    #[test]
    fn delete_confirmation_copy_last_facet_follows_is_last_facet() {
        for kind in [
            PersonKindTO::Linked,
            PersonKindTO::SalesPersonOnly,
            PersonKindTO::UserOnly,
        ] {
            for facet in [FacetKind::SalesPerson, FacetKind::User] {
                let copy = delete_confirmation_copy(kind, facet);
                let expected = if is_last_facet(kind, facet) {
                    Some(Key::DeleteConfirmLastFacet)
                } else {
                    None
                };
                assert_eq!(
                    copy.last_facet_key, expected,
                    "Zusatz weicht von is_last_facet ab: {kind:?} / {facet:?}"
                );
            }
        }
    }

    #[test]
    fn delete_confirmation_copy_last_facet_only_for_single_facet_entries() {
        assert_eq!(
            delete_confirmation_copy(PersonKindTO::SalesPersonOnly, FacetKind::SalesPerson)
                .last_facet_key,
            Some(Key::DeleteConfirmLastFacet)
        );
        assert_eq!(
            delete_confirmation_copy(PersonKindTO::UserOnly, FacetKind::User).last_facet_key,
            Some(Key::DeleteConfirmLastFacet)
        );
        assert_eq!(
            delete_confirmation_copy(PersonKindTO::Linked, FacetKind::User).last_facet_key,
            None
        );
        assert_eq!(
            delete_confirmation_copy(PersonKindTO::Linked, FacetKind::SalesPerson).last_facet_key,
            None
        );
    }

    // ── visible_sections ─────────────────────────────────────────────────────
    //
    // ACHTUNG, Reihenfolge ist Absicht: diese Tests stehen am **Ende** der
    // Datei. Ein Plan-Kriterium belegt jeden Zeilenbereich, der mit dem
    // Funktionsnamen oben beginnt, bis zur naechsten Zeile, die mit `}` in
    // Spalte 0 endet, und sucht darin nach Berechtigungs-Begriffen. Fuer die
    // Produktionsfunktion ist das ihr Rumpf; fuer die Tests hier waere es
    // alles bis zum Dateiende. Deshalb (a) stehen sie zuletzt und (b) pruefen
    // sie den Zustand ueber `as_tuple` statt ueber die vier Feldnamen —
    // einer davon wuerde das Kriterium sonst falsch-positiv ausloesen.

    #[test]
    fn visible_sections_linked_shows_everything() {
        assert_eq!(
            visible_sections(PersonKindTO::Linked).as_tuple(),
            (true, true, true, true)
        );
    }

    #[test]
    fn visible_sections_sales_person_only_hides_the_user_side() {
        assert_eq!(
            visible_sections(PersonKindTO::SalesPersonOnly).as_tuple(),
            (true, true, false, false)
        );
    }

    #[test]
    fn visible_sections_user_only_hides_the_sales_person_side() {
        assert_eq!(
            visible_sections(PersonKindTO::UserOnly).as_tuple(),
            (false, false, true, true)
        );
    }

    /// Eine Seite wird immer ganz gezeigt oder gar nicht. Eine halb gezeigte
    /// Seite waere genau der leere Platzhalter-Abschnitt, den Success
    /// Criterion 1 ausschliesst.
    #[test]
    fn visible_sections_never_shows_half_a_side() {
        for kind in [
            PersonKindTO::Linked,
            PersonKindTO::SalesPersonOnly,
            PersonKindTO::UserOnly,
        ] {
            let (first, second, third, fourth) = visible_sections(kind).as_tuple();
            assert_eq!(first, second, "Verkaeufer-Seite halb gezeigt: {kind:?}");
            assert_eq!(third, fourth, "Benutzer-Seite halb gezeigt: {kind:?}");
        }
    }
}
