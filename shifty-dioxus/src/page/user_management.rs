//! `UserManagementPage` — **eine** durchsuchbare Personenliste (Phase 62,
//! PADM-01). Die frueheren zwei Tabs (`SalesPersons` / `Benutzer`) und ihre
//! N+1-Ladepfade sind entfallen; alle Zeilen kommen aus einem einzigen
//! Aggregat-Request (`LoadPersonDirectory`, Success Criterion 5).
//!
//! Die Auspraegung wird **nicht** als eigenes Badge gerendert (UAT Phase 62,
//! bewusste Abweichung von D-62-01): sie ist bereits an zwei Zellen ablesbar —
//! der Bezahlt/Freiwillig-Chip erscheint nur bei vorhandenem Verkaeufer-
//! Datensatz, der Benutzername nur bei vorhandenem Login. Ein zusaetzliches
//! Badge sprach dieselbe accent/warn-Farbsprache wie der Bezahlt/Freiwillig-
//! Chip direkt daneben und war dadurch irrefuehrend. Anzeigename und
//! Benutzername werden stumpf interpoliert — im Frontend existiert keine
//! Namens-Fallback-Regel (PADM-06, Success Criterion 3), und die Auspraegung
//! kommt weiterhin als Backend-Enum, wird also nie aus leeren Feldern
//! abgeleitet (T-62-09); sie steuert Aktions-Sichtbarkeit (D-62-06) und das
//! Ziel des Bearbeiten-Links (D-62-07).
//!
//! Die Seite traegt einen Component-Level-Admin-Guard (PADM-08, D-62-08):
//! die Nav-Ausblendung faengt einen direkten URL-Aufruf nicht ab. Anlegen,
//! Loeschen und Impersonation leben weiter in `Dialog`s bzw. in der
//! Aktionsspalte und sind auf Zeilen mit Benutzer-Auspraegung beschraenkt
//! (D-62-06).

use crate::{
    base_types::ImStr,
    component::{
        atoms::btn::build_class as btn_build_class, Btn, BtnVariant, Dialog, DialogVariant, Field,
        PersonChip, TextInput, TopBar,
    },
    i18n::Key,
    router::Route,
    service::{
        auth::AUTH,
        i18n::I18N,
        impersonate::ImpersonateAction,
        user_management::{UserManagementAction, USER_MANAGEMENT_STORE},
    },
    state::AuthInfo,
};
use dioxus::prelude::*;
use rest_types::PersonEntryTO;
use std::rc::Rc;

const SEARCH_INPUT_CLASSES: &str =
    "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body w-full min-w-0 form-input";

const ACCENT_SOFT_HEX: &str = "#eaecfb";
const WARN_SOFT_HEX: &str = "#fef0d6";

pub(crate) fn matches_search(name: &str, term: &str) -> bool {
    if term.is_empty() {
        return true;
    }
    name.to_lowercase().contains(&term.to_lowercase())
}

/// PADM-08 / D-62-08 / T-62-02: Admin-Praedikat des Component-Level-Guards.
/// Dieselbe Pruefung wie in `page/settings.rs`, nur als freie Funktion, damit
/// sie ohne Dioxus-Laufzeit pruefbar ist. Fehlende Auth-Info gilt als *nicht*
/// berechtigt.
pub(crate) fn is_admin(auth_info: Option<&AuthInfo>) -> bool {
    auth_info.map(|a| a.has_privilege("admin")).unwrap_or(false)
}

/// Ablehnungs-Ansicht des Guards. Rendert genau das, was `settings.rs` beim
/// fruehen `return` rendert: die obere Leiste und einen gedaempften Hinweis.
///
/// Der Text bleibt bewusst hartkodiertes, unuebersetztes Englisch — eine
/// vorbestehende Luecke des kopierten Guard-Musters, die diese Phase
/// ausdruecklich nicht schliesst (62-UI-SPEC Open Question 4).
#[component]
pub fn NotAuthorizedView() -> Element {
    rsx! {
        TopBar {}
        div { class: "p-md text-ink-muted", "Not authorized." }
    }
}

#[component]
pub fn UserManagementPage() -> Element {
    let user_management_service = use_coroutine_handle::<UserManagementAction>();
    // D-32-07 / IMP-01: impersonate service handle for "Act as this person".
    let impersonate_service = use_coroutine_handle::<ImpersonateAction>();
    let user_management = USER_MANAGEMENT_STORE.read().clone();
    let auth = AUTH.read().clone();
    let i18n = I18N.read().clone();

    // PADM-08 / D-62-08 / T-62-02: Component-Level-Admin-Guard. Die Nav-Leiste
    // blendet diese Route fuer Nicht-Admins aus, ein direkter URL-Aufruf wuerde
    // das aber umgehen. Defense-in-Depth — das autoritative Gate ist der
    // Backend-Endpoint. Der fruehe `return` steht vor jedem Effekt-Hook und
    // vor jedem Datenzugriff.
    if !is_admin(auth.auth_info.as_ref()) {
        return rsx! {
            NotAuthorizedView {}
        };
    }

    let search = use_signal(String::new);
    let mut show_add_user_dialog = use_signal(|| false);
    let mut delete_user_confirm: Signal<Option<ImStr>> = use_signal(|| None);
    let mut new_user_name: Signal<ImStr> = use_signal(|| "".into());

    use_effect(move || {
        // Success Criterion 5: ein einziger Aggregat-Aufruf fuellt die Liste.
        user_management_service.send(UserManagementAction::LoadPersonDirectory);
    });

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-h1 mb-6 text-ink",
                "{i18n.t(Key::UserManagement)}"
            }

            PersonDirectoryTable {
                entries: user_management.directory.clone(),
                search,
                on_open_add_user: move |_| show_add_user_dialog.set(true),
                on_request_delete: move |username: ImStr| delete_user_confirm.set(Some(username)),
                // D-32-07 / IMP-01: start impersonation from the directory row.
                on_impersonate: move |username: ImStr| {
                    impersonate_service.send(ImpersonateAction::Start(username))
                },
            }
        }

        AddUserDialog {
            open: *show_add_user_dialog.read(),
            value: new_user_name.read().clone(),
            on_value_change: move |v: ImStr| new_user_name.set(v),
            on_cancel: move |_| show_add_user_dialog.set(false),
            on_create: move |_| {
                let name = new_user_name.read().clone();
                if !name.as_str().trim().is_empty() {
                    user_management_service.send(UserManagementAction::AddUser(name));
                    new_user_name.set("".into());
                    show_add_user_dialog.set(false);
                }
            },
        }

        DeleteUserConfirmDialog {
            target: delete_user_confirm.read().clone(),
            on_cancel: move |_| delete_user_confirm.set(None),
            on_confirm: move |username: ImStr| {
                user_management_service.send(UserManagementAction::DeleteUser(username));
                delete_user_confirm.set(None);
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct AddUserDialogProps {
    open: bool,
    value: ImStr,
    on_value_change: EventHandler<ImStr>,
    on_cancel: EventHandler<()>,
    on_create: EventHandler<()>,
}

#[component]
fn AddUserDialog(props: AddUserDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let i18n = I18N.read().clone();
    let on_cancel = props.on_cancel;
    let on_cancel_footer = props.on_cancel;
    let on_create = props.on_create;
    let on_value_change = props.on_value_change;

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: ImStr::from(i18n.t(Key::AddNewUser).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel_footer.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: move |_| on_create.call(()),
                    "{i18n.t(Key::CreateUser)}"
                }
            }),
            Field {
                label: ImStr::from(i18n.t(Key::AddNewUser).as_ref()),
                TextInput {
                    value: props.value.clone(),
                    on_change: move |v: ImStr| on_value_change.call(v),
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DeleteUserConfirmDialogProps {
    target: Option<ImStr>,
    on_cancel: EventHandler<()>,
    on_confirm: EventHandler<ImStr>,
}

#[component]
fn DeleteUserConfirmDialog(props: DeleteUserConfirmDialogProps) -> Element {
    let Some(username) = props.target.clone() else {
        return rsx! {};
    };
    let i18n = I18N.read().clone();
    let body_text = i18n
        .t(Key::DeleteUserConfirmBody)
        .as_ref()
        .replace("{username}", username.as_str());
    let on_cancel = props.on_cancel;
    let on_cancel_footer = props.on_cancel;
    let on_confirm = props.on_confirm;
    let username_for_confirm = username.clone();

    rsx! {
        Dialog {
            open: true,
            on_close: move |_| on_cancel.call(()),
            title: ImStr::from(i18n.t(Key::DeleteUserConfirmTitle).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| on_cancel_footer.call(()),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| on_confirm.call(username_for_confirm.clone()),
                    "{i18n.t(Key::DeleteUser)}"
                }
            }),
            p { class: "text-body text-ink", "{body_text}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PersonDirectoryTableProps {
    entries: Rc<[PersonEntryTO]>,
    search: Signal<String>,
    on_open_add_user: EventHandler<()>,
    on_request_delete: EventHandler<ImStr>,
    on_impersonate: EventHandler<ImStr>,
}

/// Die eine Liste des Personen-Verzeichnisses (PADM-01): Werkzeugleiste plus
/// Tabelle. Die Eintraege kommen fertig sortiert aus dem Backend — hier wird
/// nur gefiltert, nie umsortiert und nie umbenannt.
#[component]
fn PersonDirectoryTable(props: PersonDirectoryTableProps) -> Element {
    let i18n = I18N.read().clone();
    let mut search = props.search;
    let placeholder = i18n.t(Key::SearchPlaceholder);
    let term = search.read().clone();

    let on_open_add_user = props.on_open_add_user;
    let on_request_delete = props.on_request_delete;
    let on_impersonate = props.on_impersonate;

    // Success Criterion 2: ein einziger Suchbegriff, ODER-verknuepft ueber
    // Anzeigename und Benutzername.
    let filtered: Vec<PersonEntryTO> = props
        .entries
        .iter()
        .filter(|entry| {
            let hit_title = matches_search(&entry.display_name, &term);
            let hit_login = entry
                .username
                .as_deref()
                .is_some_and(|login| matches_search(login, &term));
            hit_title || hit_login
        })
        .cloned()
        .collect();
    let no_matches = filtered.is_empty() && !term.is_empty();

    rsx! {
        div { class: "flex items-center gap-2 mb-4",
            input {
                class: "{SEARCH_INPUT_CLASSES}",
                r#type: "text",
                placeholder: "{placeholder}",
                value: "{search.read()}",
                oninput: move |evt| search.set(evt.value()),
            }
            {
                // 62-UI-SPEC Open Question 1: bewusst Sekundaer — auf der Seite
                // steht genau ein Akzent-Button ("Neuen Benutzer anlegen").
                let secondary_cls = btn_build_class(BtnVariant::Secondary, false);
                let href = format!(
                    "{}",
                    Route::SalesPersonDetails { sales_person_id: String::new() }
                );
                rsx! {
                    a {
                        href: "{href}",
                        class: "{secondary_cls} inline-flex items-center whitespace-nowrap",
                        "{i18n.t(Key::CreateNewSalesPerson)}"
                    }
                }
            }
            Btn {
                variant: BtnVariant::Primary,
                on_click: move |_| on_open_add_user.call(()),
                "{i18n.t(Key::AddNewUser)}"
            }
        }

        table { class: "w-full text-left",
            thead { class: "border-b border-border text-micro font-bold uppercase text-ink-muted",
                tr {
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::Name)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnType)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnUsername)}" }
                    th { class: "py-2 px-3 font-semibold", "{i18n.t(Key::ColumnRoles)}" }
                    th { class: "py-2 px-3 font-semibold w-[1%]" }
                }
            }
            tbody {
                for entry in filtered.into_iter() {
                    {
                        let inactive = entry.inactive.unwrap_or(false);
                        let row_class = if inactive {
                            "border-b border-border hover:bg-surface-alt opacity-60"
                        } else {
                            "border-b border-border hover:bg-surface-alt"
                        };
                        let has_sales_person = entry.sales_person_id.is_some();
                        let is_paid = entry.is_paid.unwrap_or(false);
                        let pill_label = if is_paid {
                            i18n.t(Key::Paid)
                        } else {
                            i18n.t(Key::Volunteer)
                        };
                        let pill_color = if is_paid {
                            ImStr::from(ACCENT_SOFT_HEX)
                        } else {
                            ImStr::from(WARN_SOFT_HEX)
                        };
                        let bg = entry.background_color.clone();
                        let title = entry.display_name.clone();
                        let login: Option<ImStr> = entry.username.as_deref().map(ImStr::from);
                        let login_for_impersonate = login.clone();
                        let login_for_delete = login.clone();
                        let roles = entry.roles.clone();
                        // D-62-07: Verkaeufer-Seite vorhanden -> Verkaeufer-
                        // Detailroute, sonst Benutzer-Detailroute. Diese Phase
                        // legt keine neue Detail-Route an.
                        let edit_href = if let Some(sales_person_id) = entry.sales_person_id {
                            format!(
                                "{}",
                                Route::SalesPersonDetails {
                                    sales_person_id: sales_person_id.to_string(),
                                }
                            )
                        } else {
                            let user_id = match &login {
                                Some(value) => value.to_string(),
                                None => String::new(),
                            };
                            format!("{}", Route::UserDetails { user_id })
                        };
                        let secondary_cls = btn_build_class(BtnVariant::Secondary, false);
                        rsx! {
                            tr { class: "{row_class}",
                                td { class: "py-2 px-3",
                                    div { class: "flex items-center gap-2",
                                        if let Some(color) = bg.clone() {
                                            span {
                                                class: "w-2.5 h-2.5 rounded-full",
                                                style: "background-color: {color}",
                                            }
                                        }
                                        span { class: "text-ink", "{title}" }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    div { class: "flex items-center gap-2 flex-wrap",
                                        if has_sales_person {
                                            PersonChip {
                                                name: ImStr::from(pill_label.as_ref()),
                                                color: Some(pill_color.clone()),
                                            }
                                        }
                                        if inactive {
                                            span { class: "inline-flex px-2 py-0.5 rounded-sm text-micro font-medium bg-bad-soft text-bad",
                                                "{i18n.t(Key::Inactive)}"
                                            }
                                        }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    if let Some(value) = login.clone() {
                                        span { class: "font-mono text-ink", "{value}" }
                                    } else {
                                        span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                    }
                                }
                                td { class: "py-2 px-3",
                                    if roles.is_empty() {
                                        span { class: "text-ink-muted", "{i18n.t(Key::Unlinked)}" }
                                    } else {
                                        div { class: "flex flex-wrap gap-1",
                                            for role in roles.iter() {
                                                span { class: "inline-flex px-2 py-0.5 rounded-sm text-micro font-medium bg-accent-soft text-accent",
                                                    "{role}"
                                                }
                                            }
                                        }
                                    }
                                }
                                td { class: "py-2 px-3 w-[1%]",
                                    div { class: "flex gap-2 whitespace-nowrap",
                                        // D-62-06: Impersonation und Loeschen
                                        // haengen an der Benutzer-Auspraegung.
                                        if let Some(value) = login_for_impersonate.clone() {
                                            Btn {
                                                variant: BtnVariant::Secondary,
                                                on_click: move |_| on_impersonate.call(value.clone()),
                                                "{i18n.t(Key::ImpersonateActAs)}"
                                            }
                                        }
                                        a {
                                            href: "{edit_href}",
                                            class: "{secondary_cls} inline-flex items-center",
                                            "{i18n.t(Key::Edit)}"
                                        }
                                        if let Some(value) = login_for_delete.clone() {
                                            Btn {
                                                variant: BtnVariant::Danger,
                                                on_click: move |_| on_request_delete.call(value.clone()),
                                                "🗑"
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

        if no_matches {
            div { class: "py-6 text-center",
                p { class: "text-body font-semibold text-ink",
                    "{i18n.t(Key::PersonDirectorySearchEmptyHeading)}"
                }
                p { class: "text-body text-ink-muted",
                    "{i18n.t(Key::PersonDirectorySearchEmptyBody)}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rest_types::PersonKindTO;
    use std::sync::Arc;
    use uuid::Uuid;

    #[test]
    fn matches_search_empty_term_matches_all() {
        assert!(matches_search("alex", ""));
        assert!(matches_search("", ""));
    }

    #[test]
    fn matches_search_is_case_insensitive() {
        assert!(matches_search("Lena Müller", "lena"));
        assert!(matches_search("ALEX", "ale"));
        assert!(!matches_search("Tom", "Lena"));
    }

    // ── Personen-Verzeichnis (Phase 62) ──────────────────────────────────────

    /// Rendert die eine Liste mit injizierten Eintraegen — ohne Store, ohne
    /// Netzwerk. Ersetzt die beiden frueheren Tab-Harnesse.
    fn render_directory(entries: Vec<PersonEntryTO>, search_initial: &str) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            entries: Rc<[PersonEntryTO]>,
            initial: String,
        }
        #[component]
        fn DirectoryWrapper(props: WrapperProps) -> Element {
            let search = use_signal(|| props.initial.clone());
            let _ = search.read();
            rsx! {
                PersonDirectoryTable {
                    entries: props.entries.clone(),
                    search,
                    on_open_add_user: |_| {},
                    on_request_delete: |_: ImStr| {},
                    on_impersonate: |_: ImStr| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            DirectoryWrapper,
            WrapperProps {
                entries: Rc::from(entries),
                initial: search_initial.to_string(),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

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

    /// UAT Phase 62 — bewusste Abweichung von D-62-01: die Auspraegung wird
    /// **nicht** als eigenes Badge gerendert. Grund: das Badge sprach dieselbe
    /// accent/warn-Farbsprache wie der Bezahlt/Freiwillig-Chip in derselben
    /// Zelle und war dadurch irrefuehrend. Die Auspraegung bleibt ablesbar —
    /// Bezahlt/Freiwillig-Chip nur bei Verkaeufer-Datensatz, Benutzername nur
    /// bei Login. Dieser Test haelt beide Richtungen fest: kein Badge-Rest im
    /// Markup, und die beiden Ersatz-Signale sind je Auspraegung korrekt.
    #[test]
    fn directory_row_encodes_kind_without_badge() {
        // Je Auspraegung einzeln rendern, damit jedes Signal eindeutig einer
        // Zeile gehoert.
        let linked = render_directory(
            vec![sample_entry(PersonKindTO::Linked, "Lena", Some("lena"), true)],
            "",
        );
        let sales_only = render_directory(
            vec![sample_entry(PersonKindTO::SalesPersonOnly, "Tom", None, true)],
            "",
        );
        let user_only = render_directory(
            vec![sample_entry(PersonKindTO::UserOnly, "Zoe", Some("zoe"), false)],
            "",
        );

        // Signal 1: Verkaeufer-Datensatz vorhanden <=> Bezahlt/Freiwillig-Chip.
        // PersonChip interpoliert die Farbe als Inline-Style, der Hex-Wert ist
        // daher ein direkter Nachweis (Rollen-Chips nutzen Tailwind-Klassen,
        // keine Hex-Werte — keine Kollision).
        assert!(
            linked.contains(ACCENT_SOFT_HEX),
            "verknuepfter Eintrag muss den Bezahlt-Chip tragen: {linked}"
        );
        assert!(
            sales_only.contains(ACCENT_SOFT_HEX),
            "reiner Verkaeufer muss den Bezahlt-Chip tragen: {sales_only}"
        );
        assert!(
            !user_only.contains(ACCENT_SOFT_HEX) && !user_only.contains(WARN_SOFT_HEX),
            "reines Benutzerkonto darf keinen Bezahlt/Freiwillig-Chip tragen: {user_only}"
        );

        // Signal 2: Login vorhanden <=> Benutzername-Zelle (font-mono).
        assert!(
            linked.contains("font-mono") && linked.contains("lena"),
            "verknuepfter Eintrag muss den Benutzernamen zeigen: {linked}"
        );
        assert!(
            user_only.contains("font-mono") && user_only.contains("zoe"),
            "reines Benutzerkonto muss den Benutzernamen zeigen: {user_only}"
        );
        assert!(
            !sales_only.contains("font-mono"),
            "reiner Verkaeufer darf keine Benutzernamen-Zelle fuellen: {sales_only}"
        );

        // Kein Rest des entfernten Auspraegungs-Badges. Die Klassenketten sind
        // hier absichtlich wortwoertlich eingebettet: der Helfer, der sie
        // erzeugt hat, ist mit dem Badge geloescht worden.
        for cls in [
            "bg-accent-soft border-accent text-accent",
            "bg-warn-soft border-warn text-warn",
            "border-dashed border-border-strong",
        ] {
            for (name, html) in [
                ("linked", &linked),
                ("sales_only", &sales_only),
                ("user_only", &user_only),
            ] {
                assert!(
                    !html.contains(cls),
                    "Auspraegungs-Badge ({cls}) ist in {name} zurueckgekehrt: {html}"
                );
            }
        }
    }

    /// Success Criterion 2: ein Suchfeld findet ueber Anzeigename ODER
    /// Benutzername.
    #[test]
    fn directory_search_matches_both_names() {
        let entries = vec![
            sample_entry(PersonKindTO::Linked, "Lena", Some("lmueller"), true),
            sample_entry(PersonKindTO::UserOnly, "Tom", Some("tom"), false),
            sample_entry(PersonKindTO::SalesPersonOnly, "Zoe", None, true),
        ];

        // Treffer ausschliesslich im Benutzernamen.
        let html = render_directory(entries.clone(), "mueller");
        assert!(html.contains(">Lena<"), "display name row missing: {html}");
        assert!(!html.contains(">Tom<"), "third entry not filtered: {html}");
        assert!(!html.contains(">Zoe<"), "third entry not filtered: {html}");

        // Treffer ausschliesslich im Anzeigenamen.
        let html = render_directory(entries, "lena");
        assert!(html.contains("lmueller"), "username cell missing: {html}");
        assert!(!html.contains(">Tom<"), "third entry not filtered: {html}");
        assert!(!html.contains(">Zoe<"), "third entry not filtered: {html}");
    }

    /// PADM-06: ein reines Benutzerkonto rendert einen nicht-leeren
    /// Anzeigenamen — das Backend liefert ihn, das Frontend hat keine
    /// Fallback-Regel.
    #[test]
    fn directory_user_only_row_has_display_name() {
        let entries = vec![sample_entry(
            PersonKindTO::UserOnly,
            "alex",
            Some("alex"),
            false,
        )];
        let html = render_directory(entries, "");
        assert!(
            html.contains("text-ink\">alex<"),
            "user-only row must render a non-empty display name: {html}"
        );
        assert!(html.contains("font-mono"), "username cell missing: {html}");
    }

    /// D-62-06: Impersonation und Loeschen haengen an der Benutzer-Auspraegung.
    #[test]
    fn directory_actions_only_for_user_side() {
        let sp_only = vec![sample_entry(
            PersonKindTO::SalesPersonOnly,
            "Tom",
            None,
            true,
        )];
        let html = render_directory(sp_only, "");
        assert!(
            !html.contains("Act as"),
            "sales-person-only row must not offer impersonation: {html}"
        );
        assert!(
            !html.contains("🗑"),
            "sales-person-only row must not offer delete: {html}"
        );

        for entry in [
            sample_entry(PersonKindTO::Linked, "Lena", Some("lena"), true),
            sample_entry(PersonKindTO::UserOnly, "Zoe", Some("zoe"), false),
        ] {
            let html = render_directory(vec![entry], "");
            assert!(
                html.contains("Act as"),
                "user-side row must offer impersonation: {html}"
            );
            assert!(html.contains("🗑"), "user-side row must offer delete: {html}");
        }
    }

    /// D-62-04: inaktive Verkaeufer bleiben abgeblendet und tragen das
    /// Inaktiv-Badge.
    #[test]
    fn directory_inactive_row_is_dimmed() {
        let mut entry = sample_entry(PersonKindTO::Linked, "Old", Some("old"), true);
        entry.inactive = Some(true);
        let html = render_directory(vec![entry], "");
        assert!(html.contains("opacity-60"), "row not dimmed: {html}");
        assert!(html.contains("bg-bad-soft"), "inactive badge missing: {html}");
        assert!(html.contains("text-bad"), "inactive badge missing: {html}");
    }

    /// Leer-Zustand greift nur bei nicht-leerem Suchbegriff (62-UI-SPEC).
    #[test]
    fn directory_empty_search_result_shows_empty_state() {
        let entries = vec![sample_entry(
            PersonKindTO::Linked,
            "Lena",
            Some("lena"),
            true,
        )];
        let html = render_directory(entries.clone(), "zzz");
        assert!(
            html.contains("No matches"),
            "empty-state heading missing: {html}"
        );

        // Leere Liste ohne Suchbegriff ist kein Such-Leerzustand.
        let html = render_directory(entries, "");
        assert!(
            !html.contains("No matches"),
            "empty state must not appear without a search term: {html}"
        );
    }

    /// PADM-08 / D-62-08: der Component-Level-Guard faengt den direkten
    /// URL-Aufruf ab, den die Nav-Ausblendung durchlaesst.
    #[test]
    fn non_admin_sees_not_authorized() {
        assert!(!is_admin(None), "missing auth info must not pass the guard");

        let no_privileges = AuthInfo {
            user: "zoe".into(),
            privileges: Rc::new([]),
            authenticated: true,
        };
        assert!(
            !is_admin(Some(&no_privileges)),
            "auth info without admin privilege must not pass the guard"
        );

        let admin = AuthInfo {
            user: "root".into(),
            privileges: Rc::new([Rc::from("admin")]),
            authenticated: true,
        };
        assert!(is_admin(Some(&admin)), "admin must pass the guard");

        let mut vdom = VirtualDom::new(NotAuthorizedView);
        vdom.rebuild_in_place();
        let html = dioxus_ssr::render(&vdom);
        assert!(
            html.contains("Not authorized."),
            "denial view must render the denial text: {html}"
        );
    }

    fn render_add_user_dialog(open: bool) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            open: bool,
        }
        #[component]
        fn AddWrapper(props: WrapperProps) -> Element {
            rsx! {
                AddUserDialog {
                    open: props.open,
                    value: ImStr::from(""),
                    on_value_change: |_| {},
                    on_cancel: |_| {},
                    on_create: |_| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(AddWrapper, WrapperProps { open });
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn add_user_dialog_closed_renders_nothing() {
        let html = render_add_user_dialog(false);
        // No Dialog content (no `role="dialog"`).
        assert!(
            !html.contains("role=\"dialog\""),
            "expected closed dialog: {html}"
        );
    }

    #[test]
    fn add_user_dialog_open_renders_field_and_buttons() {
        let html = render_add_user_dialog(true);
        // Dialog renders.
        assert!(
            html.contains("role=\"dialog\""),
            "missing dialog role: {html}"
        );
        // Field label: the i18n key "Add New User" is used both as title and label.
        // TextInput class.
        assert!(html.contains("form-input"), "missing form-input class");
        // Two footer buttons (Cancel + Create).
        // Cancel uses Secondary tokens; Create uses Primary.
        assert!(html.contains("bg-accent"), "missing primary tokens: {html}");
    }

    fn render_delete_dialog(target: Option<&str>) -> String {
        #[derive(Props, Clone, PartialEq)]
        struct WrapperProps {
            target: Option<ImStr>,
        }
        #[component]
        fn DeleteWrapper(props: WrapperProps) -> Element {
            rsx! {
                DeleteUserConfirmDialog {
                    target: props.target.clone(),
                    on_cancel: |_| {},
                    on_confirm: |_: ImStr| {},
                }
            }
        }
        let mut vdom = VirtualDom::new_with_props(
            DeleteWrapper,
            WrapperProps {
                target: target.map(ImStr::from),
            },
        );
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn delete_user_confirm_dialog_closed_renders_nothing() {
        let html = render_delete_dialog(None);
        assert!(
            !html.contains("role=\"dialog\""),
            "expected closed dialog: {html}"
        );
    }

    #[test]
    fn delete_user_confirm_dialog_open_interpolates_username_and_has_danger_button() {
        let html = render_delete_dialog(Some("alex"));
        assert!(html.contains("role=\"dialog\""));
        // Username appears in the body text via interpolation.
        assert!(html.contains("alex"), "username not interpolated: {html}");
        // Danger button uses bad tokens.
        assert!(html.contains("text-bad"), "missing danger tokens: {html}");
        assert!(html.contains("border-bad"), "missing danger border: {html}");
    }

    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("user_management.rs");
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
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "border-black",
            "border-gray-",
        ] {
            assert!(
                !prefix.contains(forbidden),
                "legacy class `{forbidden}` found in source"
            );
        }
    }
}
