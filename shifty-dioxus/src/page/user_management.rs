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
//! die Nav-Ausblendung faengt einen direkten URL-Aufruf nicht ab.
//!
//! **Ein Anlege-Einstieg, ein Loeschort, ein Bearbeiten-Ziel** (Phase 63,
//! D-63-03/D-63-04/PADM-02). Die Werkzeugleiste traegt genau einen
//! Primaer-Knopf; er oeffnet den Anlege-Dialog aus `page/person_detail.rs`,
//! der die Auspraegung ueber zwei Umschalter erfasst, statt sie ueber die
//! Wahl des Knopfes zu erzwingen. Der Bearbeiten-Link jeder Zeile zeigt auf
//! dieselbe Detailroute — welche Abschnitte dort erscheinen, entscheidet die
//! Auspraegung erst auf der Zielseite (Success Criterion 1). Geloescht wird
//! ausschliesslich auf der Detailseite; die Aktionsspalte fuehrt nur noch
//! Impersonation (D-62-06) und den Bearbeiten-Link.

use crate::{
    base_types::ImStr,
    component::{
        atoms::btn::build_class as btn_build_class, Btn, BtnVariant, PersonChip, TopBar,
    },
    i18n::Key,
    page::person_detail::CreatePersonEntryDialog,
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
use rest_types::{CreatePersonEntryTO, PersonEntryTO};
use std::rc::Rc;
use std::sync::Arc;

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

    let nav = navigator();
    let search = use_signal(String::new);
    let mut show_create_dialog = use_signal(|| false);
    let mut create_display_name: Signal<ImStr> = use_signal(|| "".into());
    let mut create_wants_login = use_signal(|| false);
    let mut create_wants_sales_person = use_signal(|| false);
    let mut create_username: Signal<ImStr> = use_signal(|| "".into());
    // D-63-07: das Anlegen leitet in die Detailansicht weiter. Dieses Flag
    // markiert, dass die naechste gefuellte Detailzeile aus *diesem*
    // Anlegevorgang stammt — ohne es wuerde ein Eintrag, den ein frueherer
    // Besuch im Store hinterlassen hat, eine Weiterleitung ausloesen.
    let mut create_pending = use_signal(|| false);

    use_effect(move || {
        // Success Criterion 5: ein einziger Aggregat-Aufruf fuellt die Liste.
        user_management_service.send(UserManagementAction::LoadPersonDirectory);
    });

    // D-63-07 / SC-2: Weiterleitung in die Detailansicht, sobald der
    // Anlegevorgang die neue Zeile im Store abgelegt hat. Die Kennung kommt
    // aus der Antwort des Backends, nicht aus einer Client-Ableitung
    // (PADM-06) — der Anlege-Pfad des Service laedt die zurueckgegebene
    // `person_id` nach und schreibt sie in `person_detail`.
    //
    // `create_pending` wird bewusst mit `peek` gelesen: das Setzen des Flags
    // darf den Effekt nicht ausloesen, sonst liefe er noch mit dem Stand vor
    // dem Anlegen. Ausgeloest wird er ausschliesslich von Store-Aenderungen.
    use_effect(move || {
        let entry = USER_MANAGEMENT_STORE.read().person_detail.clone();
        if !*create_pending.peek() {
            return;
        }
        if let Some(entry) = entry {
            create_pending.set(false);
            nav.push(Route::PersonDetail {
                person_id: entry.person_id.to_string(),
            });
        }
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
                on_create_entry: move |_| show_create_dialog.set(true),
                // D-32-07 / IMP-01: start impersonation from the directory row.
                on_impersonate: move |username: ImStr| {
                    impersonate_service.send(ImpersonateAction::Start(username))
                },
            }
        }

        CreatePersonEntryDialog {
            open: *show_create_dialog.read(),
            display_name: create_display_name.read().clone(),
            wants_login: *create_wants_login.read(),
            wants_sales_person: *create_wants_sales_person.read(),
            username: create_username.read().clone(),
            on_display_name_change: move |value: ImStr| create_display_name.set(value),
            on_wants_login_change: move |value: bool| create_wants_login.set(value),
            on_wants_sales_person_change: move |value: bool| create_wants_sales_person.set(value),
            on_username_change: move |value: ImStr| create_username.set(value),
            on_cancel: move |_| show_create_dialog.set(false),
            on_create: move |_| {
                let display_name = create_display_name.read().clone();
                let wants_user = *create_wants_login.read();
                let wants_sales_person = *create_wants_sales_person.read();
                let username = create_username.read().clone();
                // Dieselbe Bedingung, unter der der Dialog seinen Absende-Knopf
                // sperrt. Beides ist Bequemlichkeit — verbindlich prueft der
                // Service (ASVS V5, Plan 63-05).
                if !wants_user && !wants_sales_person {
                    return;
                }
                if wants_user && username.as_str().trim().is_empty() {
                    return;
                }
                // Den vorigen Detail-Eintrag raeumen, bevor das Flag steht:
                // sonst koennte die Weiterleitung oben eine Zeile aus einem
                // frueheren Besuch fuer das Ergebnis dieses Anlegevorgangs
                // halten.
                USER_MANAGEMENT_STORE.write().person_detail = None;
                create_pending.set(true);
                user_management_service
                    .send(
                        UserManagementAction::CreatePersonEntry(CreatePersonEntryTO {
                            display_name: Arc::from(display_name.as_str().trim()),
                            wants_user,
                            wants_sales_person,
                            username: if wants_user {
                                Some(Arc::from(username.as_str().trim()))
                            } else {
                                None
                            },
                        }),
                    );
                show_create_dialog.set(false);
                create_display_name.set("".into());
                create_username.set("".into());
                create_wants_login.set(false);
                create_wants_sales_person.set(false);
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PersonDirectoryTableProps {
    entries: Rc<[PersonEntryTO]>,
    search: Signal<String>,
    on_create_entry: EventHandler<()>,
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

    let on_create_entry = props.on_create_entry;
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
            // D-63-03: genau EIN Anlege-Einstieg. Bis Phase 62 standen hier
            // zwei Knoepfe nebeneinander — einer je Auspraegung —, und nichts
            // erklaerte den Unterschied (UAT-Befund: „Das checkt niemand").
            // Der Dialog dahinter erfasst die Auspraegung ueber zwei
            // Umschalter, statt sie ueber die Wahl des Knopfes zu erzwingen.
            Btn {
                variant: BtnVariant::Primary,
                on_click: move |_| on_create_entry.call(()),
                "{i18n.t(Key::CreatePersonEntry)}"
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
                        let roles = entry.roles.clone();
                        // Success Criterion 1 / D-63-04: EIN Ziel fuer alle
                        // drei Auspraegungen. Phase 62 musste hier noch
                        // verzweigen, weil es zwei halbe Detailseiten gab
                        // (D-62-07) — die Zeile musste also wissen, welche
                        // Haelfte der Person die richtige Seite ist. Jetzt
                        // entscheidet erst die Zielseite anhand des
                        // Backend-`kind`, welche Abschnitte sie zeigt; die
                        // Liste kennt nur noch die Identitaet des Eintrags.
                        let edit_href = format!(
                            "{}",
                            Route::PersonDetail {
                                person_id: entry.person_id.to_string(),
                            }
                        );
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
                                // Aktionsspalte: Impersonation und Bearbeiten.
                                // KEIN Loeschen. Seit Phase 63 lebt das
                                // Loeschen ausschliesslich auf der
                                // Detailseite, weil nur dort die
                                // ausprägungs-bewussten Bestaetigungstexte aus
                                // PADM-07/D-63-08 stehen — Login hart, Person
                                // weich, je mit dem Zusatz „letzte
                                // Auspraegung". Der frueher hier stehende
                                // Zeilen-Knopf trug einen generischen Satz
                                // („Soll Benutzer X wirklich geloescht
                                // werden?"), der diese Zusage nicht halten
                                // kann. Ein zweiter, schwaecherer Loeschweg
                                // waere genau die Doppeldeutigkeit, die diese
                                // Phase abbaut (63-UI-SPEC Open Question 1).
                                td { class: "py-2 px-3 w-[1%]",
                                    div { class: "flex gap-2 whitespace-nowrap",
                                        // D-62-06: Impersonation haengt an der
                                        // Benutzer-Auspraegung.
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
                    on_create_entry: |_| {},
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

    /// D-62-06: Impersonation haengt an der Benutzer-Auspraegung.
    ///
    /// Phase 63 hat den zweiten Teil dieser Zusicherung gedreht: der
    /// Zeilen-Loeschknopf existiert nicht mehr — fuer **keine** Auspraegung.
    /// Geloescht wird nur noch auf der Detailseite, wo die
    /// ausprägungs-bewussten Bestaetigungstexte stehen (PADM-07, D-63-08).
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

        for entry in [
            sample_entry(PersonKindTO::Linked, "Lena", Some("lena"), true),
            sample_entry(PersonKindTO::UserOnly, "Zoe", Some("zoe"), false),
        ] {
            let html = render_directory(vec![entry], "");
            assert!(
                html.contains("Act as"),
                "user-side row must offer impersonation: {html}"
            );
        }
    }

    /// D-63-04 / SC-5: kein Loeschweg in der Liste — fuer keine der drei
    /// Auspraegungen. Geprueft werden beide Spuren, die der entfernte Knopf
    /// hinterlassen haette: sein Piktogramm und die Danger-Tokens des
    /// `Btn`-Atoms. Der Impersonations-Knopf daneben ist sekundaer, faerbt
    /// also nicht mit.
    #[test]
    fn directory_row_has_no_delete_path() {
        for entry in [
            sample_entry(PersonKindTO::Linked, "Lena", Some("lena"), true),
            sample_entry(PersonKindTO::UserOnly, "Zoe", Some("zoe"), false),
            sample_entry(PersonKindTO::SalesPersonOnly, "Tom", None, true),
        ] {
            let html = render_directory(vec![entry], "");
            assert!(
                !html.contains("🗑"),
                "row still offers a delete button: {html}"
            );
            assert!(
                !html.contains("text-bad") && !html.contains("border-bad"),
                "row still renders danger tokens — a delete path returned: {html}"
            );
        }
    }

    /// D-63-03: **genau ein** Anlege-Einstieg in der Werkzeugleiste. Der
    /// UAT-Befund aus Phase 62 im Wortlaut des Nutzers: „Es gibt jetzt
    /// trotzdem neuen benutzer und neuen verkaeufer separat. Das checkt
    /// niemand." Gezaehlt wird die Beschriftung im gerenderten Markup — genau
    /// einmal — und die beiden abgeloesten Beschriftungen duerfen gar nicht
    /// mehr vorkommen.
    #[test]
    fn directory_toolbar_has_exactly_one_create_entry_point() {
        let html = render_directory(
            vec![sample_entry(
                PersonKindTO::Linked,
                "Lena",
                Some("lena"),
                true,
            )],
            "",
        );
        assert_eq!(
            html.matches("Add entry").count(),
            1,
            "die Werkzeugleiste muss genau einen Anlege-Einstieg rendern: {html}"
        );
        for legacy in ["Add New User", "Create New Sales Person"] {
            assert!(
                !html.contains(legacy),
                "abgeloester Anlege-Einstieg `{legacy}` ist zurueck: {html}"
            );
        }
    }

    /// Success Criterion 1: der Bearbeiten-Link zeigt fuer alle drei
    /// Auspraegungen auf dieselbe Route — ein Pfad, ein Praefix, die
    /// `person_id` als einziger veraenderlicher Teil. Bis Phase 62 verzweigte
    /// die Zeile hier noch zwischen zwei Detailseiten (D-62-07).
    #[test]
    fn directory_edit_link_targets_one_route_for_all_kinds() {
        for kind in [
            PersonKindTO::Linked,
            PersonKindTO::UserOnly,
            PersonKindTO::SalesPersonOnly,
        ] {
            let entry = sample_entry(kind, "Lena", Some("lena"), kind != PersonKindTO::UserOnly);
            let expected = format!(
                "href=\"{}\"",
                Route::PersonDetail {
                    person_id: entry.person_id.to_string(),
                }
            );
            let html = render_directory(vec![entry], "");
            assert_eq!(
                html.matches(&expected).count(),
                1,
                "{kind:?}: der Bearbeiten-Link muss genau einmal auf die eine \
                 Detailroute zeigen ({expected}): {html}"
            );
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

    // Die frueheren Dialog-Rendertests dieser Datei sind mit ihren Dialogen
    // entfallen: der Anlege-Dialog lebt seit Phase 63 in
    // `page/person_detail.rs` und wird dort geprueft (Plan 63-08), der
    // Zeilen-Loesch-Dialog existiert gar nicht mehr. Was er absicherte, deckt
    // jetzt `directory_row_has_no_delete_path` — in umgekehrter Richtung.

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
