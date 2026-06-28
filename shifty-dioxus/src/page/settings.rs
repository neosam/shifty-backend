//! SettingsPage — admin-gated page with the paid-limit hard/soft toggle (Card 1)
//! and the holiday auto-credit activation date field (Card 2).
//! Phase 24 D-24-06: paid-limit enforcement toggle.
//! Phase 25 D-25-06: holiday auto-credit cutoff date input.

use time::macros::format_description;

use dioxus::prelude::*;

use crate::{
    base_types::ImStr,
    component::{form::TextInput, TopBar},
    i18n::Key,
    loader,
    service::{auth::AUTH, config::CONFIG, i18n::I18N},
};

const TOGGLE_NAME: &str = "paid_limit_hard_enforcement";

#[component]
pub fn SettingsPage() -> Element {
    let i18n = I18N.read().clone();
    let config = CONFIG.read().clone();

    // WR-02: Component-level admin guard. The nav hides this route for non-admins,
    // but a direct URL access would bypass that. Reuse the same AUTH/has_privilege
    // pattern used by billing_periods, shiftplan, absences pages.
    let is_admin = AUTH
        .read()
        .auth_info
        .as_ref()
        .map(|a| a.has_privilege("admin"))
        .unwrap_or(false);
    if !is_admin {
        return rsx! {
            TopBar {}
            div { class: "p-md text-ink-muted", "Not authorized." }
        };
    }

    // ── Card 1: Paid-limit enforcement toggle (Phase 24) ──────────────────────

    let config_for_load = config.clone();
    let toggle_resource =
        use_resource(move || loader::get_toggle_enabled(config_for_load.clone(), TOGGLE_NAME));

    let mut hard_enforcement = use_signal(|| false);
    let mut save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut saving = use_signal(|| false);

    use_effect(move || {
        if let Some(Ok(enabled)) = &*toggle_resource.read_unchecked() {
            hard_enforcement.set(*enabled);
        }
    });

    let config_for_click = config.clone();
    let on_toggle = move |_| {
        if *saving.read() {
            return;
        }
        let current = *hard_enforcement.read();
        let next = !current;
        saving.set(true);
        save_result.set(None);

        let cfg = config_for_click.clone();
        spawn(async move {
            match loader::set_toggle(cfg, TOGGLE_NAME, next).await {
                Ok(()) => {
                    hard_enforcement.set(next);
                    save_result.set(Some(true));
                }
                Err(_) => {
                    save_result.set(Some(false));
                }
            }
            saving.set(false);
        });
    };

    let is_on = *hard_enforcement.read();
    let is_saving = *saving.read();

    let toggle_class = if is_on {
        "px-3 py-2 rounded-md border border-bad text-bad text-body font-semibold bg-bad-soft"
    } else {
        "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt"
    };

    let toggle_label = if is_on {
        i18n.t(Key::SettingsPaidLimitToggleOn)
    } else {
        i18n.t(Key::SettingsPaidLimitToggleOff)
    };

    let aria_pressed = if is_on { "true" } else { "false" };

    // ── Card 2: Holiday auto-credit activation date (Phase 25) ────────────────

    let mut date_str: Signal<String> = use_signal(String::new);
    let mut date_str_loaded_empty = use_signal(|| false);
    let mut cutoff_save_result: Signal<Option<bool>> = use_signal(|| None);
    let mut cutoff_saving = use_signal(|| false);

    let config_for_cutoff = config.clone();
    let cutoff_resource =
        use_resource(move || loader::get_holiday_cutoff_date(config_for_cutoff.clone()));

    use_effect(move || {
        match &*cutoff_resource.read_unchecked() {
            Some(Ok(Some(date))) => {
                date_str.set(date.clone());
                date_str_loaded_empty.set(false);
            }
            Some(Ok(None)) => {
                date_str.set(String::new());
                date_str_loaded_empty.set(true);
            }
            _ => {}
        }
    });

    let config_for_save = config.clone();
    let on_save_cutoff = move |_| {
        if *cutoff_saving.read() {
            return;
        }
        let val = date_str.read().clone();
        if val.is_empty() {
            return;
        }
        // Client-side ISO date validation (defense in depth; <input type=date> enforces this
        // in the browser, but we double-check before the PUT).
        let date_format = format_description!("[year]-[month]-[day]");
        if time::Date::parse(&val, date_format).is_err() {
            cutoff_save_result.set(Some(false));
            return;
        }
        cutoff_saving.set(true);
        cutoff_save_result.set(None);
        let cfg = config_for_save.clone();
        spawn(async move {
            match loader::set_holiday_cutoff_date(cfg, Some(&val)).await {
                Ok(()) => {
                    cutoff_save_result.set(Some(true));
                    date_str_loaded_empty.set(false);
                }
                Err(_) => {
                    cutoff_save_result.set(Some(false));
                }
            }
            cutoff_saving.set(false);
        });
    };

    let config_for_clear = config.clone();
    let on_clear_cutoff = move |_| {
        if *cutoff_saving.read() {
            return;
        }
        cutoff_saving.set(true);
        cutoff_save_result.set(None);
        let cfg = config_for_clear.clone();
        spawn(async move {
            match loader::set_holiday_cutoff_date(cfg, None).await {
                Ok(()) => {
                    date_str.set(String::new());
                    date_str_loaded_empty.set(true);
                    cutoff_save_result.set(Some(true));
                }
                Err(_) => {
                    cutoff_save_result.set(Some(false));
                }
            }
            cutoff_saving.set(false);
        });
    };

    let is_cutoff_saving = *cutoff_saving.read();
    let loaded_empty = *date_str_loaded_empty.read();
    let date_string = date_str.read().clone();
    let date_value = ImStr::from(date_string.as_str());
    let date_empty = date_string.is_empty();

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-h2 font-semibold pb-4",
                "{i18n.t(Key::Settings)}"
            }

            // Card 1 — Paid-limit enforcement (Phase 24, unchanged)
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3",

                // Toggle row
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsPaidLimitToggleLabel)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsPaidLimitToggleDescription)}"
                    }
                }

                div { class: "flex flex-row items-center gap-3",
                    button {
                        r#type: "button",
                        class: "{toggle_class}",
                        "aria-pressed": "{aria_pressed}",
                        disabled: is_saving,
                        onclick: on_toggle,
                        "{toggle_label}"
                    }

                    // Inline feedback
                    {match *save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-bad text-small font-normal",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }
            }

            // Card 2 — Holiday auto-credit activation date (Phase 25 D-25-06)
            div { class: "bg-surface border border-border rounded-md p-4 flex flex-col gap-3 mt-4",

                // Row A: Feature label + description
                div { class: "flex flex-col gap-1",
                    span { class: "text-body text-ink font-semibold",
                        "{i18n.t(Key::SettingsHolidayAutoCreditLabel)}"
                    }
                    span { class: "text-small text-ink-soft",
                        "{i18n.t(Key::SettingsHolidayAutoCreditDescription)}"
                    }
                }

                // Row B: Date input (width-constrained)
                div { class: "max-w-[200px]",
                    TextInput {
                        input_type: ImStr::from("date"),
                        value: date_value,
                        on_change: move |v: ImStr| date_str.set(v.as_str().to_string()),
                    }
                }

                // Row C: Action row (Save + Clear + inline feedback)
                div { class: "flex flex-row items-center gap-3",
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink text-body bg-surface hover:bg-surface-alt",
                        disabled: is_cutoff_saving,
                        onclick: on_save_cutoff,
                        "{i18n.t(Key::SettingsHolidayAutoCreditSave)}"
                    }
                    button {
                        r#type: "button",
                        class: "px-3 py-2 rounded-md border border-border text-ink-soft text-body bg-surface hover:bg-surface-alt",
                        disabled: is_cutoff_saving || date_empty,
                        onclick: on_clear_cutoff,
                        "{i18n.t(Key::SettingsHolidayAutoCreditClear)}"
                    }

                    // Inline feedback — reuses SettingsSaved / SettingsSaveError keys
                    {match *cutoff_save_result.read() {
                        Some(true) => rsx! {
                            span { class: "text-small text-ink-muted",
                                "{i18n.t(Key::SettingsSaved)}"
                            }
                        },
                        Some(false) => rsx! {
                            span { class: "text-bad text-small",
                                "{i18n.t(Key::SettingsSaveError)}"
                            }
                        },
                        None => rsx! { },
                    }}
                }

                // Row D: Unset hint (shown only when no date is set after load)
                if loaded_empty {
                    span { class: "text-small text-ink-muted",
                        "{i18n.t(Key::SettingsHolidayAutoCreditUnsetHint)}"
                    }
                }
            }
        }
    }
}
