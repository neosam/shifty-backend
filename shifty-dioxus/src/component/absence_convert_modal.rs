//! `AbsenceConvertModal` — generalisiertes Konvertierungs-Modal für HR (Phase 8.5 Plan 05).
//!
//! Extrahiert aus `cutover_admin.rs::ManualConvertModal`, aber mit vollständig
//! generalisierten Props ohne cutover-spezifische Abhängigkeiten. Verwendbar aus
//! der HR-Absence-Page (Plan 06/07).
//!
//! Unterschiede zu `ManualConvertModal`:
//! - Props: `extra_hours_id`, `initial_date`, `amount`, `category` direkt (kein Entry-Wrapper)
//! - Kein `weekday`-Label (ist in den generalisierten Props nicht vorhanden)
//! - P-7-Submit-Defense (T-8.5-05a): parse + s<=e + inline `text-bad`-Error;
//!   nur valide (Ok,Ok,s<=e) dispatcht via `on_submit`
//! - DayFractionTO-Select (Full/Half) für die Tageshälfte
//! - Custom-Backdrop: `bg-modal-veil` outer + `stop_propagation` inner
//!
//! `ManualConvertModal` in `cutover_admin.rs` bleibt unverändert bis Phase 8.6.

use dioxus::prelude::*;
use rest_types::AbsenceCategoryTO;
use uuid::Uuid;

use crate::i18n::Key;
use crate::service::i18n::I18N;

/// Generalisiertes HR-Modal zum Konvertieren einer einzelnen `extra_hours`-Row
/// (Vacation/SickLeave/UnpaidLeave) in eine `absence_period`.
///
/// Props:
/// - `extra_hours_id` — ID des zu konvertierenden Eintrags (wird via `on_submit` übergeben)
/// - `initial_date` — Vorbefüllter Wert für Von-Datumsfeld (Eintrags-Tag)
/// - `suggested_end` — Arbeitstag-basiertes Vorschlags-Bis (vom Backend berechnet, UV-01/UV-02)
/// - `amount` — Read-only Stundenanzahl (wird nur angezeigt, nicht bearbeitet)
/// - `category` — Read-only Kategorie (Vacation/SickLeave/UnpaidLeave)
/// - `on_submit` — Handler: `(extra_hours_id, start, end, DayFractionTO)`
/// - `on_cancel` — Handler: Schließt das Modal (Backdrop-Click oder Cancel-Button)
#[component]
pub fn AbsenceConvertModal(
    extra_hours_id: Uuid,
    initial_date: time::Date,
    suggested_end: time::Date,
    amount: f32,
    category: AbsenceCategoryTO,
    on_submit: EventHandler<(Uuid, time::Date, time::Date, rest_types::DayFractionTO)>,
    on_cancel: EventHandler<()>,
) -> Element {
    let i18n = I18N.read().clone();

    // Formatiere initial_date als "YYYY-MM-DD"-String für das Von-Feld.
    let fmt = time::macros::format_description!("[year]-[month]-[day]");
    let initial_str = initial_date
        .format(fmt)
        .unwrap_or_else(|_| "2026-01-01".to_string());
    // Formatiere suggested_end für das Bis-Feld (UV-01: Arbeitstag-basiert).
    let suggested_end_str = suggested_end
        .format(fmt)
        .unwrap_or_else(|_| initial_str.clone());

    // Von-Feld: Eintrags-Tag (initial_date) — unverändert.
    let mut start_str = use_signal({
        let s = initial_str.clone();
        move || s.clone()
    });
    // Bis-Feld: vom Backend vorgeschlagenes Enddatum (suggested_end), NICHT initial_date.
    let mut end_str = use_signal(move || suggested_end_str.clone());
    let mut error_msg = use_signal(|| Option::<String>::None);
    // Tageshälfte-Select: Default Full
    let mut day_fraction = use_signal(|| rest_types::DayFractionTO::Full);

    let title = i18n.t(Key::AbsenceConvertModalTitle);
    let help_text = i18n.t(Key::AbsenceConvertModalHelp);
    let amount_label = i18n.t(Key::AbsenceConvertAmountLabel);
    let start_label = i18n.t(Key::AbsenceConvertStartLabel);
    let end_label = i18n.t(Key::AbsenceConvertEndLabel);
    let submit_label = i18n.t(Key::AbsenceConvertBtnSubmit);
    let cancel_label = i18n.t(Key::AbsenceConvertBtnCancel);
    let err_start_after_end = i18n.t(Key::AbsenceConvertErrStartAfterEnd).to_string();
    let day_fraction_label = i18n.t(Key::AbsenceFieldDayFraction);
    let day_fraction_full_label = i18n.t(Key::AbsenceDayFractionFull);
    let day_fraction_half_label = i18n.t(Key::AbsenceDayFractionHalf);

    // Read-only Kategorie-Label (Pitfall 5 — statische Tailwind-Arms).
    let category_label = match category {
        AbsenceCategoryTO::Vacation => i18n.t(Key::AbsenceCategoryVacation),
        AbsenceCategoryTO::SickLeave => i18n.t(Key::AbsenceCategorySickLeave),
        AbsenceCategoryTO::UnpaidLeave => i18n.t(Key::AbsenceCategoryUnpaidLeave),
    };

    rsx! {
        div { class: "fixed inset-0 bg-modal-veil flex items-center justify-center z-50",
            onclick: move |_| { on_cancel.call(()); },
            div { class: "bg-surface rounded-lg p-6 flex flex-col gap-4 min-w-md max-w-lg border border-border",
                onclick: move |ev| { ev.stop_propagation(); },
                h3 { class: "text-lg font-semibold text-ink", "{title}" }
                p { class: "text-small text-ink-muted", "{help_text}" }
                // Read-only Kontext-Row: Stundenanzahl + Kategorie.
                div { class: "flex flex-wrap gap-4 text-small",
                    span { class: "text-ink-muted",
                        "{amount_label}: "
                        span { class: "font-mono text-ink", id: "amount-display", "{amount:.2}h" }
                    }
                    span { class: "text-ink-muted",
                        "{category_label}"
                    }
                }
                // Von-Datum (D-29).
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{start_label}" }
                    input {
                        r#type: "date",
                        class: "border border-border rounded-md p-2",
                        value: "{start_str}",
                        oninput: move |ev| { start_str.set(ev.value()); },
                    }
                }
                // Bis-Datum (D-29).
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{end_label}" }
                    input {
                        r#type: "date",
                        class: "border border-border rounded-md p-2",
                        value: "{end_str}",
                        oninput: move |ev| { end_str.set(ev.value()); },
                    }
                }
                // Tageshälfte-Select (D-08.3-FE-02).
                label { class: "flex flex-col gap-1",
                    span { class: "text-small text-ink-muted", "{day_fraction_label}" }
                    select {
                        class: "border border-border rounded-md p-2",
                        onchange: move |ev| {
                            let next = match ev.value().as_str() {
                                "Half" => rest_types::DayFractionTO::Half,
                                _ => rest_types::DayFractionTO::Full,
                            };
                            day_fraction.set(next);
                        },
                        option {
                            value: "Full",
                            selected: *day_fraction.read() == rest_types::DayFractionTO::Full,
                            "{day_fraction_full_label}"
                        }
                        option {
                            value: "Half",
                            selected: *day_fraction.read() == rest_types::DayFractionTO::Half,
                            "{day_fraction_half_label}"
                        }
                    }
                }
                // Inline-Fehler-Anzeige (T-8.5-05a Mitigation).
                if let Some(e) = error_msg.read().clone() {
                    span { class: "text-bad text-small", "{e}" }
                }
                div { class: "flex justify-end gap-2",
                    button {
                        class: "px-3 py-2 rounded-md bg-surface border border-border text-ink",
                        onclick: move |_| { on_cancel.call(()); },
                        "{cancel_label}"
                    }
                    button {
                        class: "px-3 py-2 rounded-md bg-accent text-accent-ink",
                        onclick: move |_| {
                            // P-7-Submit-Defense (T-8.5-05a): parse + s<=e + inline error.
                            // Kein unwrap_or_else-Fallback auf ein hardcoded Datum.
                            let parse_fmt = time::macros::format_description!(
                                "[year]-[month]-[day]"
                            );
                            let parsed_start = time::Date::parse(
                                start_str.read().as_str(),
                                parse_fmt,
                            );
                            let parsed_end = time::Date::parse(
                                end_str.read().as_str(),
                                parse_fmt,
                            );
                            match (parsed_start, parsed_end) {
                                (Ok(s), Ok(e)) if s <= e => {
                                    error_msg.set(None);
                                    on_submit.call((extra_hours_id, s, e, *day_fraction.read()));
                                }
                                (Ok(_), Ok(_)) => {
                                    // start > end
                                    error_msg.set(Some(err_start_after_end.clone()));
                                }
                                _ => {
                                    error_msg.set(Some(
                                        "Invalid date format".to_string(),
                                    ));
                                }
                            }
                        },
                        "{submit_label}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::{generate, Locale};
    use crate::service::i18n::I18N;
    use time::macros::date;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    /// SSR-Snapshot-Test: AbsenceConvertModal rendert zwei date-type-Inputs
    /// und einen read-only amount-Span.
    #[test]
    fn absence_convert_modal_renders_date_inputs() {
        fn comp() -> Element {
            use_hook(|| {
                *I18N.write() = generate(Locale::De);
            });
            let id = Uuid::from_u128(0xABCD);
            rsx! {
                AbsenceConvertModal {
                    extra_hours_id: id,
                    initial_date: date!(2026-06-10),
                    suggested_end: date!(2026-06-14),
                    amount: 8.0_f32,
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(comp);
        // Zwei date-Inputs vorhanden
        assert_eq!(
            html.matches(r#"type="date""#).count(),
            2,
            "Erwarte zwei type=date-Inputs, gefunden: {}",
            html.matches(r#"type="date""#).count()
        );
        // Read-only amount-Span vorhanden
        assert!(
            html.contains("id=\"amount-display\""),
            "Erwarte amount-display-Span im HTML"
        );
        // amount-Wert korrekt
        assert!(
            html.contains("8.00h"),
            "Erwarte '8.00h' im amount-Span"
        );
    }

    /// UV-01: Von wird mit initial_date befüllt, Bis mit suggested_end (darf abweichen).
    /// Prüft per SSR-HTML, dass die value-Attribute der beiden date-Inputs unterschiedlich
    /// sind, wenn suggested_end != initial_date.
    #[test]
    fn absence_convert_modal_bis_prefilled_from_suggested_end() {
        fn comp() -> Element {
            use_hook(|| {
                *I18N.write() = generate(Locale::De);
            });
            rsx! {
                AbsenceConvertModal {
                    extra_hours_id: Uuid::from_u128(0x9999),
                    initial_date: date!(2026-06-16),   // Montag (Von)
                    suggested_end: date!(2026-06-19),  // Freitag (Bis, 4 Werktage)
                    amount: 32.0_f32,
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(comp);
        // Von-Input muss den Eintrags-Tag enthalten
        assert!(
            html.contains(r#"value="2026-06-16""#),
            "Von-Input muss value=2026-06-16 enthalten. HTML: {html}"
        );
        // Bis-Input muss suggested_end enthalten (Freitag), NICHT initial_date
        assert!(
            html.contains(r#"value="2026-06-19""#),
            "Bis-Input muss value=2026-06-19 (suggested_end) enthalten. HTML: {html}"
        );
        // Sicherstellen dass Bis NICHT den Von-Wert spiegelt
        assert_eq!(
            html.matches(r#"value="2026-06-16""#).count(),
            1,
            "Nur Von darf 2026-06-16 enthalten, nicht auch Bis. HTML: {html}"
        );
    }

    /// UV-01 Halbtag-Fallback: Wenn suggested_end == initial_date, ist Von == Bis (Halbtag).
    #[test]
    fn absence_convert_modal_half_day_von_equals_bis() {
        fn comp() -> Element {
            use_hook(|| {
                *I18N.write() = generate(Locale::De);
            });
            rsx! {
                AbsenceConvertModal {
                    extra_hours_id: Uuid::from_u128(0x1111),
                    initial_date: date!(2026-06-24),   // Halbtag
                    suggested_end: date!(2026-06-24),  // Fallback: gleicher Tag
                    amount: 4.0_f32,
                    category: AbsenceCategoryTO::Vacation,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(comp);
        // Beide date-Inputs müssen denselben Wert haben
        assert_eq!(
            html.matches(r#"value="2026-06-24""#).count(),
            2,
            "Bei Halbtag müssen beide Inputs value=2026-06-24 haben. HTML: {html}"
        );
    }

    /// P-7-Submit-Defense: Das Modal dispatcht NICHT bei ungültigem Datum.
    /// (SSR-Smoke-Test — prüft Render-Fehlerfreiheit; Submit-Defense ist zur Laufzeit aktiv.)
    #[test]
    fn absence_convert_modal_renders_without_panic() {
        fn comp() -> Element {
            use_hook(|| {
                *I18N.write() = generate(Locale::De);
            });
            rsx! {
                AbsenceConvertModal {
                    extra_hours_id: Uuid::from_u128(0x1234),
                    initial_date: date!(2026-12-24),
                    suggested_end: date!(2026-12-24),
                    amount: 4.0_f32,
                    category: AbsenceCategoryTO::SickLeave,
                    on_submit: |_| {},
                    on_cancel: |_| {},
                }
            }
        }
        let html = render(comp);
        // DayFraction-Select vorhanden
        assert!(
            html.contains("Full") || html.contains("Half"),
            "Erwarte DayFraction-Optionen im Modal"
        );
    }
}
