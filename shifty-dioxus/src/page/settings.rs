//! SettingsPage — admin-gated page with the paid-limit hard/soft toggle.
//! Phase 24 D-24-06: one switch that flips the `paid_limit_hard_enforcement`
//! toggle via the existing Toggle REST API.

use dioxus::prelude::*;

use crate::{
    component::TopBar,
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

    // Load the initial toggle state.
    let config_for_load = config.clone();
    let toggle_resource =
        use_resource(move || loader::get_toggle_enabled(config_for_load.clone(), TOGGLE_NAME));

    // Local signal that mirrors the loaded state and is updated on click.
    let mut hard_enforcement = use_signal(|| false);

    // Feedback signals: None = nothing shown, Some(true) = saved, Some(false) = error.
    let mut save_result: Signal<Option<bool>> = use_signal(|| None);

    // Reflects whether a PUT is in flight (disables the button).
    let mut saving = use_signal(|| false);

    // Synchronise the signal when the resource resolves.
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
                    // Revert — state stays `current` (signal unchanged).
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

    rsx! {
        TopBar {}

        div { class: "px-4 py-4 md:px-6 lg:px-8 max-w-5xl mx-auto",
            h1 { class: "text-h2 font-semibold pb-4",
                "{i18n.t(Key::Settings)}"
            }

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
        }
    }
}
