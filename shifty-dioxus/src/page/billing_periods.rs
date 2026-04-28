use crate::{
    api,
    base_types::ImStr,
    component::{
        atoms::{Btn, BtnVariant},
        Dialog, DialogVariant, Field, TextInput, TopBar,
    },
    i18n::Key,
    js,
    router::Route,
    service::{
        auth::AUTH,
        billing_period::{BillingPeriodAction, BILLING_PERIOD_STORE},
        config::CONFIG,
        i18n::I18N,
    },
};
use dioxus::prelude::*;
use futures_util::StreamExt;
use time::macros::format_description;
use uuid::Uuid;

pub enum BillingPeriodsPageAction {
    ShowCreateBillingPeriodDialog,
    HideCreateBillingPeriodDialog,
    CreateBillingPeriod(String),
    DeleteBillingPeriod(Uuid),
    ConfirmDeleteBillingPeriod,
    CancelDeleteBillingPeriod,
}

#[component]
pub fn BillingPeriods() -> Element {
    let billing_period_service = use_coroutine_handle::<BillingPeriodAction>();
    let billing_periods = BILLING_PERIOD_STORE.read().clone();
    let i18n = I18N.read().clone();

    let auth_info = AUTH.read().auth_info.clone();
    let is_hr = auth_info
        .as_ref()
        .map(|a| a.has_privilege("hr"))
        .unwrap_or(false);

    let mut show_create_dialog = use_signal(|| false);
    let mut end_date = use_signal(|| {
        let date_format = format_description!("[year]-[month]-[day]");
        js::current_datetime().date().format(&date_format).unwrap()
    });

    let mut show_delete_dialog = use_signal(|| false);
    let mut delete_billing_period_id: Signal<Option<Uuid>> = use_signal(|| None);
    let mut delete_error: Signal<Option<String>> = use_signal(|| None);

    let page_action_handler = use_coroutine(
        move |mut rx: UnboundedReceiver<BillingPeriodsPageAction>| async move {
            while let Some(action) = rx.next().await {
                match action {
                    BillingPeriodsPageAction::ShowCreateBillingPeriodDialog => {
                        show_create_dialog.set(true);
                    }
                    BillingPeriodsPageAction::HideCreateBillingPeriodDialog => {
                        show_create_dialog.set(false);
                    }
                    BillingPeriodsPageAction::CreateBillingPeriod(date_string) => {
                        if let Ok(parsed_date) = time::Date::parse(
                            &date_string,
                            &format_description!("[year]-[month]-[day]"),
                        ) {
                            billing_period_service
                                .send(BillingPeriodAction::CreateBillingPeriod(parsed_date));
                            show_create_dialog.set(false);
                        }
                    }
                    BillingPeriodsPageAction::DeleteBillingPeriod(id) => {
                        delete_billing_period_id.set(Some(id));
                        delete_error.set(None);
                        show_delete_dialog.set(true);
                    }
                    BillingPeriodsPageAction::CancelDeleteBillingPeriod => {
                        show_delete_dialog.set(false);
                        delete_billing_period_id.set(None);
                        delete_error.set(None);
                    }
                    BillingPeriodsPageAction::ConfirmDeleteBillingPeriod => {
                        let bp_id = *delete_billing_period_id.read();
                        if let Some(id) = bp_id {
                            let config = CONFIG.read().clone();
                            match api::delete_billing_period(config, id).await {
                                Ok(()) => {
                                    show_delete_dialog.set(false);
                                    delete_billing_period_id.set(None);
                                    delete_error.set(None);
                                    billing_period_service
                                        .send(BillingPeriodAction::LoadBillingPeriods);
                                }
                                Err(err) => {
                                    delete_error.set(Some(err.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        },
    );

    let _billing_period_loader = use_coroutine({
        move |mut rx: UnboundedReceiver<()>| async move {
            billing_period_service.send(BillingPeriodAction::LoadBillingPeriods);
            while let Some(()) = rx.next().await {
                billing_period_service.send(BillingPeriodAction::LoadBillingPeriods);
            }
        }
    });

    let confirm_delete_text = if let Some(id) = *delete_billing_period_id.read() {
        billing_periods
            .billing_periods
            .iter()
            .find(|bp| bp.id == id)
            .map(|bp| {
                format!(
                    "{} - {}",
                    i18n.format_date(&bp.start_date),
                    i18n.format_date(&bp.end_date)
                )
            })
            .map(|period_text| {
                i18n.t(Key::ConfirmDeleteBillingPeriod)
                    .replace("{period}", &period_text)
            })
            .unwrap_or_default()
    } else {
        String::new()
    };

    rsx! {
        TopBar {}

        // Delete confirmation dialog
        Dialog {
            open: *show_delete_dialog.read(),
            on_close: move |_| page_action_handler.send(BillingPeriodsPageAction::CancelDeleteBillingPeriod),
            title: ImStr::from(i18n.t(Key::ConfirmDelete).as_ref()),
            variant: DialogVariant::Auto,
            width: 420,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| page_action_handler.send(BillingPeriodsPageAction::CancelDeleteBillingPeriod),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Danger,
                    on_click: move |_| page_action_handler.send(BillingPeriodsPageAction::ConfirmDeleteBillingPeriod),
                    "{i18n.t(Key::DeleteBillingPeriod)}"
                }
            }),
            div { class: "space-y-3 text-body text-ink",
                p { "{confirm_delete_text}" }
                if let Some(error) = delete_error.read().as_ref() {
                    p { class: "text-bad text-small font-normal",
                        {
                            i18n.t(Key::DeleteBillingPeriodError)
                                .replace("{error}", error)
                        }
                    }
                }
            }
        }

        // Create dialog
        Dialog {
            open: *show_create_dialog.read(),
            on_close: move |_| page_action_handler.send(BillingPeriodsPageAction::HideCreateBillingPeriodDialog),
            title: ImStr::from(i18n.t(Key::CreateBillingPeriod).as_ref()),
            variant: DialogVariant::Auto,
            width: 460,
            footer: Some(rsx! {
                Btn {
                    variant: BtnVariant::Secondary,
                    on_click: move |_| page_action_handler.send(BillingPeriodsPageAction::HideCreateBillingPeriodDialog),
                    "{i18n.t(Key::Cancel)}"
                }
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: move |_| page_action_handler.send(BillingPeriodsPageAction::CreateBillingPeriod(end_date.read().clone())),
                    "{i18n.t(Key::CreateBillingPeriod)}"
                }
            }),
            div { class: "flex flex-col gap-3",
                Field {
                    label: ImStr::from(i18n.t(Key::EndDate).as_ref()),
                    hint: Some(ImStr::from(i18n.t(Key::SelectEndDateForNewBillingPeriod).as_ref())),
                    TextInput {
                        value: ImStr::from(end_date.read().as_str()),
                        input_type: ImStr::from("date"),
                        on_change: move |value: ImStr| end_date.set(value.to_string()),
                    }
                }
            }
        }

        main { class: "mx-auto max-w-5xl w-full px-4 py-6 md:py-8 space-y-4",
            div { class: "flex justify-between items-center",
                h1 { class: "text-h1 text-ink", "{i18n.t(Key::BillingPeriods)}" }
                Btn {
                    variant: BtnVariant::Primary,
                    on_click: move |_| page_action_handler.send(BillingPeriodsPageAction::ShowCreateBillingPeriodDialog),
                    "{i18n.t(Key::CreateNewBillingPeriod)}"
                }
            }
            if billing_periods.billing_periods.is_empty() {
                div { class: "text-ink-muted px-4 py-3", "{i18n.t(Key::LoadingBillingPeriods)}" }
            } else {
                div { class: "grid gap-3",
                    for (index, billing_period) in billing_periods.billing_periods.iter().enumerate() {
                        Link {
                            to: Route::BillingPeriodDetails {
                                billing_period_id: billing_period.id.to_string(),
                            },
                            div { class: "rounded-md border border-border bg-surface p-4 hover:bg-surface-alt transition-colors cursor-pointer",
                                div { class: "flex justify-between items-center gap-4",
                                    div { class: "min-w-0 flex flex-col gap-1",
                                        h3 { class: "text-body font-semibold text-ink",
                                            "{i18n.t(Key::Period)}: {i18n.format_date(&billing_period.start_date)} - {i18n.format_date(&billing_period.end_date)}"
                                        }
                                        p { class: "text-small font-normal text-ink-muted",
                                            "{i18n.t(Key::CreatedAt)}: {i18n.format_date(&billing_period.created_at.date())}"
                                        }
                                        p { class: "text-small font-normal text-ink-muted",
                                            "{i18n.t(Key::CreatedBy)}: {billing_period.created_by.as_ref()}"
                                        }
                                        if !billing_period.sales_persons.is_empty() {
                                            p { class: "text-small font-normal text-ink-muted",
                                                {
                                                    i18n.t(Key::SalesPersonsIncluded)
                                                        .replace("{count}", &billing_period.sales_persons.len().to_string())
                                                }
                                            }
                                        }
                                    }
                                    div { class: "flex items-center gap-2 shrink-0",
                                        if index == 0 && is_hr {
                                            {
                                                let bp_id = billing_period.id;
                                                rsx! {
                                                    button {
                                                        class: "px-2 py-1 bg-bad-soft text-bad text-micro rounded-md hover:bg-bad-soft focus:outline-none",
                                                        onclick: move |event: Event<MouseData>| {
                                                            event.prevent_default();
                                                            event.stop_propagation();
                                                            page_action_handler
                                                                .send(BillingPeriodsPageAction::DeleteBillingPeriod(bp_id));
                                                        },
                                                        "{i18n.t(Key::DeleteBillingPeriod)}"
                                                    }
                                                }
                                            }
                                        }
                                        if billing_period.deleted_at.is_none() {
                                            span { class: "px-2 py-1 bg-accent-soft text-accent text-micro uppercase rounded-full",
                                                "{i18n.t(Key::Active)}"
                                            }
                                        } else {
                                            span { class: "px-2 py-1 bg-bad-soft text-bad text-micro uppercase rounded-full",
                                                "{i18n.t(Key::Deleted)}"
                                            }
                                        }
                                        svg {
                                            class: "w-4 h-4 text-ink-muted",
                                            fill: "none",
                                            stroke: "currentColor",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "m9 18 6-6-6-6",
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
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_legacy_classes_in_source() {
        let src = include_str!("billing_periods.rs");
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
