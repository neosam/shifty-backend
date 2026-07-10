//! `VoluntaryStatsRow` — Phase 54 HR-only Freiwillig-Stunden-Konto
//! (VOL-STAT-01/02, VOL-ACCT-01/02).
//!
//! Rendert drei TupleRows (Ist ø/Woche · Soll · Delta) im Employee-Detail-Report.
//! Sichtbarkeits-Gate: Backend-Nullable-Redaktion (VAC-OFFSET-01-Praezedenz v1.8
//! — kein 403). Wenn `ist_per_contract_week: None` → Component gibt eine leere
//! Zeile zurueck. Non-HR sieht damit nichts, HR sieht die drei Zellen.
//!
//! Kein FE-Rollen-Check (Fat Backend, Thin Client).

use dioxus::prelude::*;

use crate::base_types::{format_hours, ImStr};
use crate::component::atoms::TupleRow;
use crate::i18n::Key;
use crate::service::i18n::I18N;
use crate::state::employee::VoluntaryStats;

#[derive(Props, Clone, PartialEq)]
pub struct VoluntaryStatsRowProps {
    pub stats: VoluntaryStats,
}

#[component]
pub fn VoluntaryStatsRow(props: VoluntaryStatsRowProps) -> Element {
    let stats = props.stats;

    // HR-Only-Guard via Nullable-DTO — kein FE-Rollen-Check.
    // Wenn ist_per_contract_week None ist, gibt der Backend-Service ohnehin
    // alle Felder als None zurueck (Non-HR-Redaktion).
    let (Some(ist_per_week), Some(soll), Some(delta)) =
        (stats.ist_per_contract_week, stats.soll_total, stats.delta)
    else {
        return rsx! {};
    };

    let i18n = I18N.read().clone();
    let hours_str: ImStr = ImStr::from(i18n.t(Key::Hours).as_ref());

    let delta_class = if delta < 0.0 {
        "font-mono tabular-nums text-warn"
    } else {
        "font-mono tabular-nums"
    };

    rsx! {
        TupleRow {
            label: ImStr::from(i18n.t(Key::VoluntaryHoursIstPerWeek).as_ref()),
            value: rsx! { span { class: "font-mono tabular-nums",
                {format!("{} {}", format_hours(ist_per_week, 2), hours_str)}
            } },
        }
        TupleRow {
            label: ImStr::from(i18n.t(Key::VoluntaryHoursSoll).as_ref()),
            value: rsx! { span { class: "font-mono tabular-nums",
                {format!("{} {}", format_hours(soll, 2), hours_str)}
            } },
        }
        TupleRow {
            label: ImStr::from(i18n.t(Key::VoluntaryHoursDelta).as_ref()),
            value: rsx! { span { class: "{delta_class}",
                {format!("{:+.2} {}", delta, hours_str)}
            } },
        }
        // Quick-Task 260710: Erfuellungsgrad in %. Wird ausgeblendet, wenn
        // Backend `None` liefert (soll_total ~= 0 → keine Freiwilligen-Zusage
        // im Range).
        if let Some(pct) = stats.ist_per_soll_pct {
            TupleRow {
                label: ImStr::from(i18n.t(Key::VoluntaryHoursFulfillment).as_ref()),
                value: rsx! { span { class: "font-mono tabular-nums",
                    {format!("{:.0} %", pct)}
                } },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::VirtualDom;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    #[test]
    fn renders_empty_when_ist_per_week_is_none() {
        // Simulates Non-HR: alle Felder None -> leere Zeile (kein Content).
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow { stats: VoluntaryStats::default() }
            }
        }
        let html = render(app);
        // Keine Voluntary-Labels im Output, kein TupleRow-Class.
        assert!(
            !html.contains("Voluntary") && !html.contains("Freiwillig") && !html.contains("Dobrovoln"),
            "Non-HR guard failed: html contains voluntary label: {html}"
        );
    }

    #[test]
    fn renders_empty_when_soll_is_none_even_if_ist_is_some() {
        // Defence-in-depth: wenn nur teilweise None, immer noch nichts rendern.
        // (Backend garantiert All-or-Nothing, aber Component muss robust sein.)
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow {
                    stats: VoluntaryStats {
                        ist_per_contract_week: Some(2.0),
                        ist_total: Some(8.0),
                        soll_total: None,
                        delta: None,
                        contract_weeks: Some(4),
                        ist_per_soll_pct: None,
                    }
                }
            }
        }
        let html = render(app);
        assert!(
            !html.contains("Voluntary") && !html.contains("Freiwillig"),
            "Partial-None guard failed: {html}"
        );
    }

    #[test]
    fn renders_three_rows_when_all_fields_are_some() {
        // Simulates HR: DTO liefert Some-Felder -> drei TupleRows.
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow {
                    stats: VoluntaryStats {
                        ist_per_contract_week: Some(2.0),
                        ist_total: Some(8.0),
                        soll_total: Some(8.0),
                        delta: Some(0.0),
                        contract_weeks: Some(4),
                        ist_per_soll_pct: Some(100.0),
                    }
                }
            }
        }
        let html = render(app);
        // Zwei markante Muster: format_hours(2.0, 2) -> "2.00" und delta "+0.00".
        assert!(html.contains("2.00"), "ist per week missing: {html}");
        assert!(html.contains("+0.00"), "delta missing: {html}");
        // TupleRow border-b klasse ist zumindest 3x da.
        let border_count = html.matches("border-b").count();
        assert!(border_count >= 3, "expected 3 TupleRows, got {border_count}");
    }

    #[test]
    fn renders_fulfillment_row_when_pct_is_some() {
        // Quick-Task 260710: ist_per_soll_pct = Some(80.0) => vierte Zelle
        // wird gerendert, formatiert als "80 %".
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow {
                    stats: VoluntaryStats {
                        ist_per_contract_week: Some(2.0),
                        ist_total: Some(8.0),
                        soll_total: Some(10.0),
                        delta: Some(-2.0),
                        contract_weeks: Some(4),
                        ist_per_soll_pct: Some(80.0),
                    }
                }
            }
        }
        let html = render(app);
        assert!(html.contains("80 %"), "fulfillment '80 %' missing: {html}");
        // Delta ist ebenfalls weiterhin sichtbar (Regression-Guard).
        assert!(html.contains("-2.00"), "delta row missing: {html}");
    }

    #[test]
    fn omits_fulfillment_row_when_pct_is_none() {
        // Quick-Task 260710: ist_per_soll_pct = None (soll=0-Guard) =>
        // Zeile wird ausgeblendet, aber Ist/Soll/Delta bleiben sichtbar.
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow {
                    stats: VoluntaryStats {
                        ist_per_contract_week: Some(0.0),
                        ist_total: Some(0.0),
                        soll_total: Some(0.0),
                        delta: Some(0.0),
                        contract_weeks: Some(0),
                        ist_per_soll_pct: None,
                    }
                }
            }
        }
        let html = render(app);
        // Kein Prozentzeichen im Output (das ist der eindeutige Marker,
        // dass die Fulfillment-Zeile ausgeblendet ist).
        assert!(
            !html.contains(" %"),
            "expected no '%' when pct is None: {html}"
        );
        // Delta ist immer noch drin (Regression-Guard: nur die pct-Zeile
        // faellt weg, nicht mehr).
        assert!(html.contains("+0.00"), "delta row must still render: {html}");
    }

    #[test]
    fn negative_delta_gets_warn_class() {
        fn app() -> Element {
            rsx! {
                VoluntaryStatsRow {
                    stats: VoluntaryStats {
                        ist_per_contract_week: Some(1.0),
                        ist_total: Some(4.0),
                        soll_total: Some(8.0),
                        delta: Some(-4.0),
                        contract_weeks: Some(4),
                        ist_per_soll_pct: Some(50.0),
                    }
                }
            }
        }
        let html = render(app);
        assert!(
            html.contains("text-warn"),
            "negative delta should apply text-warn class: {html}"
        );
        assert!(html.contains("-4.00"), "negative delta value missing: {html}");
    }
}
