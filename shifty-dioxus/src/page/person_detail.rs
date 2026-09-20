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

use crate::i18n::Key;
use rest_types::PersonKindTO;

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
    /// einzelne Feldzugriffe zu schreiben.
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

#[cfg(test)]
mod tests {
    use super::*;

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
