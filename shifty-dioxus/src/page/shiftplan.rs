use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use rest_types::{DayOfWeekTO, SpecialDayTO, SpecialDayTypeTO};
use time::macros::format_description;
use tracing::info;
use uuid::Uuid;

use crate::base_types::ImStr;
use crate::component::atoms::week_status_badge::should_show_badge;
use crate::component::atoms::{Btn, BtnVariant, PersonChip, WeekStatusBadge};
use crate::component::{WarningList, WarningsList, WeekStatusDropdown};
use crate::component::booking_log_table::BookingLogTable;
use crate::component::day_aggregate_view::{DayAggregateView, DayButtonBar};
use crate::component::dropdown_base::DropdownTrigger;
use crate::component::shiftplan_tab_bar::ShiftplanTabBar;
use crate::component::slot_edit::SlotEdit;
use crate::component::week_view::WeekViewButtonTypes;
use crate::component::working_hours_mini_overview::WorkingHoursMiniOverview;
use crate::component::working_hours_overview_layout_toggle::WorkingHoursOverviewLayoutToggle;
use crate::component::TopBar;
use crate::component::WeekView;
use crate::error::result_handler;
use crate::i18n::Key;
use crate::js;
use crate::loader;
use crate::service::auth::AUTH;
use crate::service::booking_conflict::BookingConflictAction;
use crate::service::booking_conflict::BOOKING_CONFLICTS_STORE;
use crate::service::booking_log::{BookingLogAction, BOOKING_LOG_STORE};
use crate::service::config::CONFIG;
use crate::service::i18n::I18N;
use crate::service::slot_edit::SlotEditAction;
use crate::service::slot_edit::SHIFTPLAN_REFRESH;
use crate::service::text_template::{
    handle_text_template_action, TextTemplateAction, TEXT_TEMPLATE_STORE,
};
use crate::service::ui_prefs;
use crate::service::ui_prefs::WorkingHoursLayout;
use crate::service::week_guard::{is_current_selection, set_selected_week, SELECTED_WEEK};
use crate::service::week_status::{WeekStatusAction, WEEK_STATUS_STORE};
use crate::state::week_status::WeekStatus;
use crate::service::weekly_summary::WeeklySummaryAction;
use crate::service::weekly_summary::WEEKLY_SUMMARY_STORE;
use crate::service::working_hours_mini::WorkingHoursMiniAction;
use crate::service::working_hours_mini::WORKING_HOURS_MINI;
use crate::state;
use crate::service::absence_marker;
use crate::state::absence_period::AbsencePeriod;
use crate::state::dropdown::DropdownEntry;
use crate::state::sales_person_available::SalesPersonUnavailable;
use crate::state::shiftplan::SalesPerson;
use crate::state::Config;
use crate::state::Weekday;

pub enum ShiftPlanAction {
    AddUserToSlot {
        slot_id: Uuid,
        sales_person_id: Uuid,
        week: u8,
        year: u32,
    },
    RemoveUserFromSlot {
        slot_id: Uuid,
        sales_person_id: Uuid,
    },
    RemoveUserFromSlotDay {
        slot_id: Uuid,
        sales_person_id: Uuid,
    },
    NextWeek,
    PreviousWeek,
    UpdateSalesPerson(Uuid),
    ToggleAvailability(Weekday),
    ToggleChangeStructureMode,
    SaveWeekMessage(String),
    LoadDayAggregate,
}

#[derive(Clone, PartialEq, Props)]
pub struct ShiftPlanProps {
    year: Option<u32>,
    week: Option<u8>,
}

#[component]
pub fn ShiftPlanDeep(props: ShiftPlanProps) -> Element {
    rsx! {
        ShiftPlan { year: props.year, week: props.week }
    }
}

#[component]
pub fn ShiftPlan(props: ShiftPlanProps) -> Element {
    let config = CONFIG.read().clone();
    let i18n = I18N.read().clone();
    let auth_info = AUTH.read().auth_info.clone();
    let booking_conflicts = BOOKING_CONFLICTS_STORE.read().clone();
    let working_hours_mini_service = use_coroutine_handle::<WorkingHoursMiniAction>();
    let booking_conflict_service = use_coroutine_handle::<BookingConflictAction>();
    let booking_log_service = use_coroutine_handle::<BookingLogAction>();
    let weekly_summary_service = use_coroutine_handle::<WeeklySummaryAction>();
    let weekly_summary = WEEKLY_SUMMARY_STORE.read().clone();
    let slot_edit_service = use_coroutine_handle::<SlotEditAction>();
    let week_status_service = use_coroutine_handle::<WeekStatusAction>();
    let week_status = WEEK_STATUS_STORE.read().status.clone();
    let is_shiftplanner = auth_info
        .as_ref()
        .map(|auth_info| auth_info.has_privilege("shiftplanner"))
        .unwrap_or(false);
    let is_shift_editor = auth_info
        .as_ref()
        .map(|auth_info| auth_info.has_privilege("shiftplan.edit"))
        .unwrap_or(false);
    let is_hr = auth_info
        .as_ref()
        .map(|auth_info| auth_info.has_privilege("hr"))
        .unwrap_or(false);

    let week = use_signal(|| props.week.unwrap_or_else(|| js::get_current_week()));
    let year = use_signal(|| props.year.unwrap_or_else(|| js::get_current_year()));
    let date =
        time::Date::from_iso_week_date(*year.read() as i32, *week.read(), time::Weekday::Monday)
            .unwrap();
    let formatter = time::format_description::parse_borrowed::<2>("[day].[month]").unwrap();
    let date_str = date.format(&formatter).unwrap().to_string();

    let backend_url = config.backend.clone();

    let calendar_week_str = i18n.t_m(
        Key::ShiftplanCalendarWeek,
        [
            ("week", week.read().to_string().as_str()),
            ("year", &year.read().to_string().as_str()),
            ("date", date_str.as_str()),
        ]
        .into(),
    );
    let edit_as_str = i18n.t(Key::ShiftplanEditAs);
    let you_are_str = i18n.t(Key::ShiftplanYouAre);
    let conflict_booking_entries_header = i18n.t(Key::ConflictBookingsHeader);
    let personal_calendar_export_str = i18n.t(Key::PersonalCalendarExport);
    let _unsufficiently_booked_calendar_export_str =
        i18n.t(Key::UnsufficientlyBookedCalendarExport);

    let mut shiftplan_catalog = {
        let config = config.clone();
        use_resource(move || loader::load_shiftplan_catalog(config.to_owned()))
    };

    let mut selected_shiftplan_id: Signal<Option<Uuid>> = use_signal(|| None);

    // Auto-select first shiftplan once the catalog finishes loading.
    // Wrapped in use_effect + peek() on the written signal so the write does not
    // re-trigger the surrounding render scope (which previously caused an
    // infinite rerender loop → WASM heap corruption).
    use_effect(move || {
        if let Some(Ok(catalog)) = &*shiftplan_catalog.read() {
            if selected_shiftplan_id.peek().is_none() && !catalog.is_empty() {
                selected_shiftplan_id.set(Some(catalog[0].id));
            }
        }
    });

    let mut shift_plan_context = {
        let config = config.clone();
        use_resource(move || {
            let config = config.clone();
            let shiftplan_id = *selected_shiftplan_id.read();
            let _refresh = *SHIFTPLAN_REFRESH.read();
            async move {
                match shiftplan_id {
                    Some(id) => {
                        loader::load_shift_plan(
                            config,
                            id,
                            *week.to_owned().read(),
                            *year.to_owned().read(),
                        )
                        .await
                    }
                    None => Ok(crate::state::Shiftplan {
                        week: *week.read(),
                        year: *year.read(),
                        slots: [].into(),
                    }),
                }
            }
        })
    };

    let sales_persons_resource = {
        let config = config.clone();
        use_resource(move || {
            let config = config.to_owned();
            let shiftplan_id = *selected_shiftplan_id.read();
            async move {
                match shiftplan_id {
                    Some(id) => loader::load_bookable_sales_persons(config, id).await,
                    None => loader::load_sales_persons(config).await,
                }
            }
        })
    };

    let current_sales_person: Signal<Option<SalesPerson>> = use_signal(|| None);
    let unavailable_days: Signal<Rc<[SalesPersonUnavailable]>> = use_signal(|| [].into());
    let person_absences: Signal<Rc<[AbsencePeriod]>> = use_signal(|| [].into());
    let mut change_structure_mode: Signal<bool> = use_signal(|| false);

    // Booking warning banner state (non-blocking, dismissible)
    let mut booking_warnings: Signal<WarningsList> = use_signal(WarningsList::empty);
    // D-24-05: slot-scoped signal holding the id of the slot whose booking was hard-blocked (409 CONFLICT).
    // None = no block active. Set to Some(slot_id) when 409 is returned; cleared on next success.
    let block_error: Signal<Option<Uuid>> = use_signal(|| None);
    let week_message = use_signal(|| String::new());
    let mut week_message_draft = use_signal(|| String::new());

    // Day view state
    let mut view_mode = use_signal(|| state::ViewMode::Week);
    let mut selected_day: Signal<Weekday> = use_signal(|| Weekday::Monday);
    let day_aggregate: Signal<Option<state::DayAggregate>> = use_signal(|| None);
    let show_sunday = use_signal(|| false);

    // Shiftplan report state
    let mut selected_template_id = use_signal(|| None::<Uuid>);
    let mut shiftplan_report_result = use_signal(|| None::<String>);
    let mut generating_report = use_signal(|| false);
    let mut copy_status = use_signal(|| None::<String>);

    // Working-hours overview layout (cards / table) — persisted per browser
    let mut working_hours_layout = use_signal(ui_prefs::get_working_hours_layout);

    // Booking log state
    let mut show_booking_log = use_signal(|| false);
    let mut booking_log_name_filter = use_signal(|| String::new());
    let mut booking_log_day_filter = use_signal(|| None::<Weekday>);
    let mut booking_log_status_filter = use_signal(|| "all".to_string());
    let mut booking_log_created_by_filter = use_signal(|| "all".to_string());

    // Plan 33-04: per-weekday special-day dropdown (shiftplanner only)
    let mut special_days_for_week = {
        let config = config.clone();
        use_resource(move || {
            let config = config.clone();
            let year_val = *year.read();
            let week_val = *week.read();
            async move {
                crate::api::get_special_days_for_week(config, year_val, week_val).await
            }
        })
    };
    let mut shortday_prompt_day: Signal<Option<Weekday>> = use_signal(|| None);
    let mut shortday_time: Signal<String> = use_signal(String::new);
    let mut special_day_error: Signal<Option<(Weekday, ImStr)>> = use_signal(|| None);
    // Clone config for weekday_sub_headers building code (coroutine later moves config)
    let config_week = config.clone();

    let button_mode = if *change_structure_mode.read() {
        WeekViewButtonTypes::Dropdown
    } else if week_status == WeekStatus::Locked && !is_shift_editor {
        // D-40-03: In a Locked week, non-shiftplan.edit users lose the +/- buttons
        // (controls disappear from the DOM). shiftplan.edit holders (is_shift_editor)
        // are the bypass (D-40-02) and keep the buttons. UX-only — the real
        // enforcement is the server gate (40-01/40-03). No banner (D-40-04).
        WeekViewButtonTypes::None
    } else if js::current_datetime().date() - date > time::Duration::weeks(2) && !is_hr {
        WeekViewButtonTypes::None
    } else {
        WeekViewButtonTypes::AddRemove
    };

    // Load shiftplan-report templates for report generation
    use_effect(move || {
        if is_shiftplanner {
            spawn(async move {
                handle_text_template_action(TextTemplateAction::LoadTemplatesByType(
                    "shiftplan-report".to_string(),
                ))
                .await;
            });
        }
    });

    // Collapse booking log and reset filters when week or year changes
    use_effect(move || {
        let _ = week.read();
        let _ = year.read();
        show_booking_log.set(false);
        booking_log_name_filter.set(String::new());
        booking_log_day_filter.set(None);
        booking_log_status_filter.set("all".to_string());
        booking_log_created_by_filter.set("all".to_string());

        // Clear booking log data to prevent showing stale data
        *BOOKING_LOG_STORE.write() = [].into();

        // Plan 33-04: clear special-day prompt and error on week/year navigation
        shortday_prompt_day.set(None);
        shortday_time.set(String::new());
        special_day_error.set(None);
    });

    let cr = use_coroutine({
        move |mut rx: UnboundedReceiver<ShiftPlanAction>| {
            to_owned![
                year,
                week,
                current_sales_person,
                unavailable_days,
                person_absences,
                config,
                week_message,
                week_message_draft,
                view_mode,
                selected_day,
                day_aggregate,
                show_sunday,
                booking_warnings,
                block_error
            ];
            async move {
                let mut update_shiftplan = {
                    to_owned![
                        config,
                        day_aggregate,
                        selected_day,
                        year,
                        week,
                        show_sunday,
                        view_mode
                    ];
                    move || {
                        shift_plan_context.restart();
                        working_hours_mini_service.send(
                            WorkingHoursMiniAction::LoadWorkingHoursMini(
                                *year.read(),
                                *week.read(),
                                is_hr,
                            ),
                        );
                        if is_shiftplanner {
                            booking_conflict_service
                                .send(BookingConflictAction::LoadWeek(*year.read(), *week.read()));
                            weekly_summary_service
                                .send(WeeklySummaryAction::LoadWeek(*year.read(), *week.read()));
                        }
                        if *view_mode.read() == state::ViewMode::Day {
                            to_owned![config, day_aggregate, selected_day, year, week, show_sunday];
                            spawn(async move {
                                if let Ok(loaded) = loader::load_day_aggregate(
                                    config.clone(),
                                    *year.read(),
                                    *week.read(),
                                    *selected_day.read(),
                                )
                                .await
                                {
                                    day_aggregate.set(Some(loaded));
                                }
                            });
                        }
                    }
                };

                // D-30-01: set the guard truth synchronously BEFORE dispatching any loader
                // so the loaders' post-await comparisons are against the correct week.
                set_selected_week(*year.read(), *week.read());

                // KW-Status: fresh status for the initially rendered week (D-39-06).
                week_status_service.send(WeekStatusAction::Load {
                    year: *year.read(),
                    week: *week.read(),
                });

                // Initial load of weekly summary
                if is_shiftplanner {
                    weekly_summary_service
                        .send(WeeklySummaryAction::LoadWeek(*year.read(), *week.read()));
                }

                // Load week message initially and when week changes
                if let Ok(Some(message)) =
                    loader::load_week_message(config.clone(), *year.read(), *week.read()).await
                {
                    week_message.set(message.clone());
                    week_message_draft.set(message);
                } else {
                    week_message.set(String::new());
                    week_message_draft.set(String::new());
                }

                let sales_person = loader::load_current_sales_person(config.to_owned())
                    .await
                    .ok()
                    .flatten();
                *current_sales_person.write() = sales_person.clone();

                let reload_unavailable_days = {
                    to_owned![current_sales_person, unavailable_days];
                    move |config: Config| async move {
                        if let Some(sales_person) = &*current_sales_person.read() {
                            // Capture year and week BEFORE the await so the guard comparison
                            // uses the week this request was made for, not the week after a
                            // possible switch (D-30-01).
                            let req_year = *year.read();
                            let req_week = *week.read();
                            let result = result_handler(
                                loader::load_unavailable_sales_person_days_for_week(
                                    config.clone(),
                                    sales_person.id,
                                    req_year,
                                    req_week,
                                )
                                .await,
                            )
                            .unwrap_or(Rc::new([]));
                            // SC3: silently drop the result if the user has already
                            // switched to a different week while this request was in flight.
                            if is_current_selection((req_year, req_week), *SELECTED_WEEK.read()) {
                                *unavailable_days.write() = result;
                            }
                        }
                    }
                };
                reload_unavailable_days(config.clone()).await;

                let reload_absence_days = {
                    to_owned![current_sales_person, person_absences];
                    move |config: Config| async move {
                        if let Some(sales_person) = &*current_sales_person.read() {
                            // Capture year and week BEFORE the await so the guard comparison
                            // uses the week this request was made for, not the week after a
                            // possible switch (D-31-06 / SC3).
                            let req_year = *year.read();
                            let req_week = *week.read();
                            let result = result_handler(
                                loader::load_absence_periods_by_sales_person(
                                    config.clone(),
                                    sales_person.id,
                                )
                                .await
                                .map(|(absences, _markers)| absences),
                            )
                            .unwrap_or_else(|| [].into());
                            // SC3 / D-31-06: silently drop the result if the user has already
                            // switched to a different week while this request was in flight.
                            // NB (review IN-01): the loader returns the person's full,
                            // week-independent absence list, so this guard is for consistency
                            // with the Phase-30 loader pattern — the actual per-week filtering
                            // happens in `absence_periods_to_discourage_days` at render time.
                            if is_current_selection((req_year, req_week), *SELECTED_WEEK.read()) {
                                *person_absences.write() = result;
                            }
                        }
                    }
                };
                reload_absence_days(config.clone()).await;
                working_hours_mini_service.send(WorkingHoursMiniAction::LoadWorkingHoursMini(
                    *year.read(),
                    *week.read(),
                    is_hr,
                ));
                if is_shiftplanner {
                    booking_conflict_service
                        .send(BookingConflictAction::LoadWeek(*year.read(), *week.read()));
                }

                while let Some(action) = rx.next().await {
                    match action {
                        ShiftPlanAction::AddUserToSlot {
                            slot_id,
                            sales_person_id,
                            week,
                            year,
                        } => {
                            info!("Registering user to slot");
                            match loader::register_user_to_slot_with_conflict_check(
                                config.to_owned(),
                                slot_id,
                                sales_person_id,
                                week,
                                year,
                            )
                            .await
                            {
                                Ok((_booking_id, warnings)) => {
                                    // Booking always kept — no rollback path
                                    block_error.set(None); // D-24-05: clear any previous hard-block on success
                                    update_shiftplan();
                                    if warnings.is_empty() {
                                        booking_warnings.set(WarningsList::empty());
                                    } else {
                                        booking_warnings
                                            .set(WarningsList(Rc::from(warnings.as_slice())));
                                    }
                                }
                                Err(crate::error::ShiftyError::Reqwest(ref e))
                                    if e.status() == Some(reqwest::StatusCode::FORBIDDEN) =>
                                {
                                    // D-13: silently ignore 403, still reload
                                    update_shiftplan();
                                }
                                Err(crate::error::ShiftyError::Reqwest(ref e))
                                    if e.status() == Some(reqwest::StatusCode::CONFLICT) =>
                                {
                                    // D-24-05: 409 = PaidLimitExceeded — surface inline at the slot, not silently
                                    block_error.set(Some(slot_id));
                                    update_shiftplan();
                                }
                                Err(e) => {
                                    crate::error::error_handler(e);
                                    update_shiftplan(); // D-13: 422 surfaced
                                }
                            }
                        }
                        ShiftPlanAction::RemoveUserFromSlot {
                            slot_id,
                            sales_person_id,
                        } => {
                            info!("Removing user from slot");
                            block_error.set(None); // D-24-05: clear hard-block error on any removal action
                            if let Some(Ok(shift_plan)) = &*shift_plan_context.read_unchecked() {
                                result_handler(
                                    loader::remove_user_from_slot(
                                        config.to_owned(),
                                        slot_id,
                                        sales_person_id,
                                        shift_plan.clone(),
                                    )
                                    .await,
                                );
                            }
                            update_shiftplan();
                        }
                        ShiftPlanAction::NextWeek => {
                            info!("Next week");
                            let current_thursday = time::Date::from_iso_week_date(
                                *year.read() as i32,
                                *week.read(),
                                time::Weekday::Thursday,
                            )
                            .unwrap();
                            let next_thursday = current_thursday + time::Duration::weeks(1);
                            let next_weeks_year = next_thursday.year() as u32;
                            let next_weeks_week = next_thursday.iso_week();
                            year.set(next_weeks_year);
                            week.set(next_weeks_week);
                            booking_warnings.set(WarningsList::empty());
                            block_error.set(None); // CR-03: clear stale 409 banner on week navigation
                            // D-30-01: update guard truth synchronously BEFORE dispatching loaders
                            set_selected_week(next_weeks_year, next_weeks_week);
                            // KW-Status: fresh status per week (D-39-06).
                            week_status_service.send(WeekStatusAction::Load {
                                year: next_weeks_year,
                                week: next_weeks_week,
                            });
                            update_shiftplan();
                            reload_unavailable_days(config.clone()).await;
                            reload_absence_days(config.clone()).await;

                            // Load week message for new week
                            if let Ok(Some(message)) = loader::load_week_message(
                                config.clone(),
                                *year.read(),
                                *week.read(),
                            )
                            .await
                            {
                                week_message.set(message.clone());
                                week_message_draft.set(message);
                            } else {
                                week_message.set(String::new());
                                week_message_draft.set(String::new());
                            }
                        }
                        ShiftPlanAction::PreviousWeek => {
                            info!("Previous week");
                            let current_thursday = time::Date::from_iso_week_date(
                                *year.read() as i32,
                                *week.read(),
                                time::Weekday::Thursday,
                            )
                            .unwrap();
                            let previous_thursday = current_thursday - time::Duration::weeks(1);
                            let previous_weeks_year = previous_thursday.year() as u32;
                            let previous_weeks_week = previous_thursday.iso_week();
                            year.set(previous_weeks_year);
                            week.set(previous_weeks_week);
                            booking_warnings.set(WarningsList::empty());
                            block_error.set(None); // CR-03: clear stale 409 banner on week navigation
                            // D-30-01: update guard truth synchronously BEFORE dispatching loaders
                            set_selected_week(previous_weeks_year, previous_weeks_week);
                            // KW-Status: fresh status per week (D-39-06).
                            week_status_service.send(WeekStatusAction::Load {
                                year: previous_weeks_year,
                                week: previous_weeks_week,
                            });
                            update_shiftplan();
                            reload_unavailable_days(config.clone()).await;
                            reload_absence_days(config.clone()).await;

                            // Load week message for new week
                            if let Ok(Some(message)) = loader::load_week_message(
                                config.clone(),
                                *year.read(),
                                *week.read(),
                            )
                            .await
                            {
                                week_message.set(message.clone());
                                week_message_draft.set(message);
                            } else {
                                week_message.set(String::new());
                                week_message_draft.set(String::new());
                            }
                        }
                        ShiftPlanAction::UpdateSalesPerson(uuid) => {
                            info!("Update sales person");
                            if let Some(Ok(sales_persons)) =
                                &*sales_persons_resource.read_unchecked()
                            {
                                let new_sales_person =
                                    sales_persons.iter().find(|sp| sp.id == uuid).cloned();
                                if let Some(new_sales_person) = new_sales_person {
                                    *current_sales_person.write() = Some(new_sales_person);
                                }
                            }
                            reload_unavailable_days(config.clone()).await;
                            reload_absence_days(config.clone()).await;
                        }
                        ShiftPlanAction::ToggleAvailability(weekday) => {
                            if let Some(available_day) = unavailable_days
                                .read()
                                .iter()
                                .find(|unavailable_day| unavailable_day.day_of_week == weekday)
                            {
                                result_handler(
                                    loader::delete_unavailable_sales_person_day(
                                        config.to_owned(),
                                        available_day.id,
                                    )
                                    .await,
                                );
                            } else if let Some(sales_person) = current_sales_person.read().as_ref()
                            {
                                result_handler(
                                    loader::create_unavailable_sales_person_day(
                                        config.to_owned(),
                                        sales_person.id,
                                        *year.read(),
                                        *week.read(),
                                        weekday,
                                    )
                                    .await,
                                );
                            }
                            update_shiftplan();
                            reload_unavailable_days(config.clone()).await;
                        }
                        ShiftPlanAction::ToggleChangeStructureMode => {
                            block_error.set(None); // WR-04: clear stale 409 banner when entering structure mode
                            let new_change_structure_mode = !*change_structure_mode.read();
                            change_structure_mode.set(new_change_structure_mode);
                        }
                        ShiftPlanAction::SaveWeekMessage(message) => {
                            if let Err(e) = loader::save_week_message(
                                config.clone(),
                                *year.read(),
                                *week.read(),
                                message.clone(),
                            )
                            .await
                            {
                                tracing::error!("Failed to save week message: {:?}", e);
                            } else {
                                week_message.set(message.clone());
                                week_message_draft.set(message);
                            }
                        }
                        ShiftPlanAction::RemoveUserFromSlotDay {
                            slot_id,
                            sales_person_id,
                        } => {
                            info!("Removing user from slot (day view)");
                            if let Some(ref agg) = *day_aggregate.read() {
                                // Find the booking across all plans
                                for plan in agg.plans.iter() {
                                    if let Some(slot) = plan.slots.iter().find(|s| s.id == slot_id)
                                    {
                                        if let Some(booking) = slot
                                            .bookings
                                            .iter()
                                            .find(|b| b.sales_person_id == sales_person_id)
                                        {
                                            if let Err(e) = crate::api::remove_booking(
                                                config.to_owned(),
                                                booking.id,
                                            )
                                            .await
                                            {
                                                tracing::error!(
                                                    "Failed to remove booking: {:?}",
                                                    e
                                                );
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                            update_shiftplan();
                        }
                        ShiftPlanAction::LoadDayAggregate => {
                            if let Ok(loaded) = loader::load_day_aggregate(
                                config.clone(),
                                *year.read(),
                                *week.read(),
                                *selected_day.read(),
                            )
                            .await
                            {
                                day_aggregate.set(Some(loaded));
                            }
                        }
                    }
                }
            }
        }
    });

    let field_dropdown_entries: Rc<[DropdownEntry]> = [
        (
            "Log slot id",
            Box::new(move |slot_id| info!("Slot id: {:?}", slot_id)),
        )
            .into(),
        (
            "Edit slot",
            Box::new(move |slot_id: Option<Rc<str>>| {
                let slot_id: Uuid = slot_id.unwrap().parse().unwrap();
                // Display-only count for the editor's non-blocking overage
                // banner (D-23-02). read_unchecked avoids a reactive subscription.
                let current_paid_count = match &*shift_plan_context.read_unchecked() {
                    Some(Ok(shift_plan)) => shift_plan
                        .slots
                        .iter()
                        .find(|s| s.id == slot_id)
                        .map(|s| s.current_paid_count)
                        .unwrap_or(0),
                    _ => 0,
                };
                slot_edit_service.send(SlotEditAction::LoadSlot(
                    slot_id,
                    *year.read(),
                    *week.read(),
                    current_paid_count,
                ))
            }),
        )
            .into(),
        (
            "Remove slot",
            Box::new(move |slot_id: Option<Rc<str>>| {
                let slot_id: Uuid = slot_id.unwrap().parse().unwrap();
                slot_edit_service.send(SlotEditAction::DeleteSlot(
                    slot_id,
                    *year.read(),
                    *week.read(),
                ))
            }),
        )
            .into(),
    ]
    .into();

    let view_week_str = i18n.t(Key::ViewModeWeek);
    let view_day_str = i18n.t(Key::ViewModeDay);

    let toggle_active_class =
        "px-3 py-1 text-body font-medium rounded-[4px] bg-surface text-ink shadow-sm";
    let toggle_inactive_class =
        "px-3 py-1 text-body font-medium rounded-[4px] text-ink-muted hover:text-ink";
    let nav_btn_class = "w-7 h-7 inline-flex items-center justify-center border border-border-strong rounded-md font-mono text-ink-soft bg-surface hover:bg-surface-alt print:hidden";

    // Plan 33-04: Build per-weekday special-day sub-header elements for the WeekView
    // grid. Only built when is_shiftplanner (D-33-01 / T-33-08 elevation-of-privilege guard).
    let loaded_special_days: Rc<[SpecialDayTO]> = match &*special_days_for_week.read_unchecked() {
        Some(Ok(days)) => days.clone(),
        _ => [].into(),
    };
    let cur_year = *year.read();
    let cur_week = *week.read();
    let weekday_sub_headers: Vec<(Weekday, Element)> = if is_shiftplanner {
        [
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
            Weekday::Sunday,
        ]
        .iter()
        .map(|&day| {
            let existing_sd = loaded_special_days
                .iter()
                .find(|sd| Weekday::from(sd.day_of_week) == day)
                .cloned();
            let has_entry = existing_sd.is_some();
            let existing_id = existing_sd.as_ref().map(|sd| sd.id);
            let is_holiday = existing_sd
                .as_ref()
                .map(|sd| sd.day_type == SpecialDayTypeTO::Holiday)
                .unwrap_or(false);
            let is_shortday = existing_sd
                .as_ref()
                .map(|sd| sd.day_type == SpecialDayTypeTO::ShortDay)
                .unwrap_or(false);

            let holiday_str: ImStr = i18n.t(Key::ShiftplanDayTypeHoliday).into();
            let shortday_str: ImStr = i18n.t(Key::ShiftplanDayTypeShortDay).into();
            let none_str: ImStr = i18n.t(Key::ShiftplanDayTypeNone).into();
            let confirm_str: ImStr = i18n.t(Key::ShiftplanDayShortDayConfirm).into();
            let cancel_str: ImStr = i18n.t(Key::CancelLabel).into();
            let error_str: ImStr = i18n.t(Key::SettingsSaveError).into();

            let trigger_label: ImStr = if is_holiday {
                holiday_str.clone()
            } else if is_shortday {
                shortday_str.clone()
            } else {
                none_str.clone()
            };

            // --- Holiday entry ---
            let holiday_entry: DropdownEntry = {
                let cfg = config_week.clone();
                let err = error_str.clone();
                let h_str = holiday_str.clone();
                (
                    h_str,
                    move |_: Option<Rc<str>>| {
                        let cfg2 = cfg.clone();
                        let err2 = err.clone();
                        let body = SpecialDayTO {
                            id: Uuid::nil(),
                            year: cur_year,
                            calendar_week: cur_week,
                            day_of_week: DayOfWeekTO::from(&day),
                            day_type: SpecialDayTypeTO::Holiday,
                            time_of_day: None,
                            created: None,
                            deleted: None,
                            version: Uuid::nil(),
                        };
                        spawn(async move {
                            match crate::api::create_special_day(cfg2, body).await {
                                Ok(_) => {
                                    special_days_for_week.restart();
                                    shift_plan_context.restart();
                                    special_day_error.set(None);
                                }
                                Err(_) => {
                                    special_day_error.set(Some((day, err2.clone())));
                                }
                            }
                        });
                    },
                    false,
                )
                    .into()
            };

            // --- ShortDay entry: opens inline time prompt ---
            // spawn() avoids FnMut (Signal::set is &mut self, which would make the closure FnMut
            // and incompatible with DropdownEntry's Fn bound).
            let shortday_entry: DropdownEntry = {
                let sd_str = shortday_str.clone();
                (
                    sd_str,
                    move |_: Option<Rc<str>>| {
                        spawn(async move {
                            shortday_prompt_day.set(Some(day));
                            shortday_time.set(String::new());
                        });
                    },
                    false,
                )
                    .into()
            };

            // --- Nichts (delete) entry: disabled when no entry exists (Pitfall 3) ---
            let none_entry: DropdownEntry = {
                let cfg = config_week.clone();
                let err = error_str.clone();
                let n_str = none_str.clone();
                (
                    n_str,
                    move |_: Option<Rc<str>>| {
                        if let Some(id) = existing_id {
                            let cfg2 = cfg.clone();
                            let err2 = err.clone();
                            spawn(async move {
                                match crate::api::delete_special_day(cfg2, id).await {
                                    Ok(_) => {
                                        special_days_for_week.restart();
                                        shift_plan_context.restart();
                                        special_day_error.set(None);
                                    }
                                    Err(_) => {
                                        special_day_error.set(Some((day, err2.clone())));
                                    }
                                }
                            });
                        }
                    },
                    !has_entry, // disabled=true → hidden by DropdownBase (Pitfall 3)
                )
                    .into()
            };

            let entries: Rc<[DropdownEntry]> =
                vec![holiday_entry, shortday_entry, none_entry].into();

            // --- Build the element for this weekday column ---
            let el: Element = if *shortday_prompt_day.read() == Some(day) {
                // ShortDay inline time prompt (Task 2)
                let cfg_sd = config_week.clone();
                let err_sd = error_str.clone();
                let confirm_label = confirm_str.clone();
                let cancel_label = cancel_str.clone();
                {
                    let sd_day_err = special_day_error
                        .read()
                        .as_ref()
                        .filter(|(d, _)| *d == day)
                        .map(|(_, msg)| msg.clone());
                    rsx! {
                        div { class: "flex flex-col gap-1 p-1",
                            input {
                                r#type: "time",
                                class: "form-input max-w-[140px] text-small",
                                value: "{shortday_time}",
                                oninput: move |e| shortday_time.set(e.value()),
                            }
                            div { class: "flex gap-1",
                                button {
                                    r#type: "button",
                                    class: "px-2 py-0.5 text-small rounded border border-accent bg-accent-soft text-accent hover:bg-accent hover:text-ink",
                                    disabled: shortday_time.read().is_empty(),
                                    onclick: move |_| {
                                        let time_str = shortday_time.read().clone();
                                        let cfg2 = cfg_sd.clone();
                                        let err2 = err_sd.clone();
                                        // WR-06: accept both HH:MM and HH:MM:SS
                                        // (some browsers emit seconds for
                                        // <input type=time>), and surface an
                                        // error instead of returning silently.
                                        let fmt_hm = format_description!("[hour]:[minute]");
                                        let fmt_hms = format_description!("[hour]:[minute]:[second]");
                                        let parsed = time::Time::parse(&time_str, fmt_hms)
                                            .or_else(|_| time::Time::parse(&time_str, fmt_hm));
                                        let Ok(parsed_time) = parsed else {
                                            special_day_error.set(Some((day, err2.clone())));
                                            return;
                                        };
                                        let body = SpecialDayTO {
                                            id: Uuid::nil(),
                                            year: cur_year,
                                            calendar_week: cur_week,
                                            day_of_week: DayOfWeekTO::from(&day),
                                            day_type: SpecialDayTypeTO::ShortDay,
                                            time_of_day: Some(parsed_time),
                                            created: None,
                                            deleted: None,
                                            version: Uuid::nil(),
                                        };
                                        spawn(async move {
                                            match crate::api::create_special_day(cfg2, body).await {
                                                Ok(_) => {
                                                    shortday_prompt_day.set(None);
                                                    shortday_time.set(String::new());
                                                    special_days_for_week.restart();
                                                    shift_plan_context.restart();
                                                    special_day_error.set(None);
                                                }
                                                Err(_) => {
                                                    special_day_error.set(Some((day, err2.clone())));
                                                }
                                            }
                                        });
                                    },
                                    "{confirm_label}"
                                }
                                button {
                                    r#type: "button",
                                    class: "px-2 py-0.5 text-small rounded border border-border text-ink-muted hover:bg-surface-alt",
                                    onclick: move |_| {
                                        shortday_prompt_day.set(None);
                                        shortday_time.set(String::new());
                                    },
                                    "{cancel_label}"
                                }
                            }
                            if let Some(err_msg) = sd_day_err {
                                span { class: "text-small text-bad", "{err_msg}" }
                            }
                        }
                    }
                }
            } else {
                // Normal state: dropdown trigger with colored dot indicator
                let dot_class: Option<&'static str> = if is_holiday {
                    Some("inline-block w-2 h-2 rounded-full bg-accent mr-1.5 flex-shrink-0")
                } else if is_shortday {
                    Some("inline-block w-2 h-2 rounded-full bg-warn mr-1.5 flex-shrink-0")
                } else {
                    None
                };
                let day_err = special_day_error
                    .read()
                    .as_ref()
                    .filter(|(d, _)| *d == day)
                    .map(|(_, msg)| msg.clone());
                let trig_label = trigger_label.clone();
                rsx! {
                    div { class: "flex flex-col gap-0.5 px-1 py-0.5",
                        DropdownTrigger {
                            entries,
                            div {
                                class: "flex items-center text-small text-ink-soft hover:text-ink cursor-pointer select-none",
                                if let Some(dot) = dot_class {
                                    span { class: dot }
                                }
                                span { "{trig_label}" }
                            }
                        }
                        if let Some(err_msg) = day_err {
                            span { class: "text-small text-bad", "{err_msg}" }
                        }
                    }
                }
            };

            (day, el)
        })
        .collect()
    } else {
        vec![]
    };

    rsx! {
        TopBar {}

        // Toolbar
        div { class: "px-4 py-3 print:hidden",
            div { class: "flex flex-wrap items-center gap-2 bg-surface border border-border rounded-lg px-[14px] py-[10px]",
                button {
                    r#type: "button",
                    class: nav_btn_class,
                    "aria-label": "Vorwoche",
                    onclick: move |_| cr.send(ShiftPlanAction::PreviousWeek),
                    "‹"
                }
                span {
                    class: "font-mono text-body font-semibold text-ink px-2",
                    style: "font-variant-numeric: tabular-nums;",
                    "{calendar_week_str}"
                }
                button {
                    r#type: "button",
                    class: nav_btn_class,
                    "aria-label": "Nächste Woche",
                    onclick: move |_| cr.send(ShiftPlanAction::NextWeek),
                    "›"
                }
                span { class: "w-px h-5 bg-border mx-1.5" }
                div { class: "inline-flex bg-surface-alt rounded-md p-0.5 gap-0.5",
                    button {
                        r#type: "button",
                        class: if *view_mode.read() == state::ViewMode::Week { toggle_active_class } else { toggle_inactive_class },
                        onclick: move |_| {
                            view_mode.set(state::ViewMode::Week);
                        },
                        "{view_week_str}"
                    }
                    button {
                        r#type: "button",
                        class: if *view_mode.read() == state::ViewMode::Day { toggle_active_class } else { toggle_inactive_class },
                        onclick: move |_| {
                            let default_day = crate::component::day_aggregate_view::default_day_for_week(
                                *year.read(),
                                *week.read(),
                            );
                            selected_day.set(default_day);
                            view_mode.set(state::ViewMode::Day);
                            cr.send(ShiftPlanAction::LoadDayAggregate);
                        },
                        "{view_day_str}"
                    }
                }
                span { class: "flex-1 min-w-0" }
                {
                    let personal_label = personal_calendar_export_str.to_string();
                    let backend_url = backend_url.clone();
                    let sales_person_id = current_sales_person.read().as_ref().map(|sp| sp.id);
                    rsx! {
                        if let Some(sp_id) = sales_person_id {
                            a {
                                class: "px-3 py-1.5 rounded-md text-body font-medium border bg-surface text-ink border-border-strong inline-flex items-center gap-1 hover:bg-surface-alt",
                                target: "_blank",
                                href: format!("{}/sales-person/{}/ical", backend_url, sp_id),
                                title: "{personal_label}",
                                span { class: "font-mono", "↓" }
                                "iCal"
                            }
                        }
                    }
                }
                if is_shiftplanner {
                    Btn {
                        variant: BtnVariant::Secondary,
                        on_click: move |_| {
                            let should_show = !*show_booking_log.read();
                            if should_show {
                                booking_log_service.send(BookingLogAction::LoadBookingLog(
                                    *year.read(),
                                    *week.read(),
                                ));
                            }
                            show_booking_log.set(should_show);
                        },
                        if *show_booking_log.read() {
                            {i18n.t(Key::BookingLogHide)}
                        } else {
                            {i18n.t(Key::BookingLogTitle)}
                        }
                    }
                }
                if is_shiftplanner {
                    DropdownTrigger {
                        entries: [
                            (
                                if *change_structure_mode.read() { "Normal mode" } else { "Edit structure" },
                                Box::new(move |_| { cr.send(ShiftPlanAction::ToggleChangeStructureMode) }),
                                !is_shift_editor,
                            )
                                .into(),
                            (
                                "New slot",
                                Box::new(move |_| {
                                    slot_edit_service
                                        .send(SlotEditAction::NewSlot(*year.read(), *week.read(), *selected_shiftplan_id.read()))
                                }),
                                !*change_structure_mode.read() || !is_shift_editor,
                            )
                                .into(),
                        ]
                            .into(),
                        button {
                            r#type: "button",
                            class: "w-7 h-7 inline-flex items-center justify-center border border-border-strong rounded-md font-mono text-ink-soft bg-surface hover:bg-surface-alt",
                            "…"
                        }
                    }
                }
                match &*sales_persons_resource.read_unchecked() {
                    Some(Ok(sales_persons)) => {
                        if is_shiftplanner {
                            rsx! {
                                div { class: "flex items-center gap-2",
                                    span { class: "text-small font-normal text-ink-muted", "{edit_as_str}" }
                                    select {
                                        class: "h-[34px] px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body form-input",
                                        onchange: move |event| {
                                            info!("Event: {:?}", event);
                                            let value = event.data.value().parse::<Uuid>().unwrap();
                                            cr.send(ShiftPlanAction::UpdateSalesPerson(value));
                                        },
                                        value: current_sales_person.read().as_ref().map(|sp| sp.id.to_string()),
                                        for sales_person in sales_persons.iter().filter(|sp| !sp.inactive) {
                                            option {
                                                value: sales_person.id.to_string(),
                                                selected: current_sales_person.read().as_ref().map_or(false, |c| c.id == sales_person.id),
                                                {sales_person.name.clone()}
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if let Some(ref current_sales_person) = *current_sales_person.read() {
                                rsx! {
                                    div { class: "flex items-center gap-2",
                                        span { class: "text-small font-normal text-ink-muted", "{you_are_str}" }
                                        PersonChip {
                                            name: ImStr::from(current_sales_person.name.as_ref()),
                                            color: Some(ImStr::from(current_sales_person.background_color.as_ref())),
                                        }
                                    }
                                }
                            } else {
                                rsx! { "" }
                            }
                        }
                    }
                    Some(Err(err)) => {
                        rsx! {
                            div { class: "text-bad text-small font-normal", "Error while loading sales persons: {err}" }
                        }
                    }
                    _ => {
                        rsx! {
                            div { class: "text-ink-muted text-small font-normal", "Loading sales persons..." }
                        }
                    }
                }
            }
        }
        if is_shiftplanner && !booking_conflicts.is_empty() {
            div { class: "mx-4 my-3 px-4 py-3 bg-bad-soft border border-bad rounded-md print:hidden",
                h2 { class: "text-h2 font-semibold pb-2 text-bad", "⚠️ {conflict_booking_entries_header}" }
                ul { class: "list-disc list-inside text-body text-ink",
                    {
                        let mut unique_booking_conflicts = Vec::new();
                        let i18n = i18n.to_owned();
                        for booking_conflict in booking_conflicts.iter() {
                            let name = booking_conflict.sales_person_name.clone();
                            let weekday = booking_conflict.day_of_week.i18n_string(&i18n);
                            let unique_booking_conflict = (
                                name,
                                weekday,
                                booking_conflict.sales_person_id,
                            );
                            if !unique_booking_conflicts
                                .iter()
                                .any(|inner| inner == &unique_booking_conflict)
                            {
                                unique_booking_conflicts.push(unique_booking_conflict);
                            }
                        }
                        rsx! {
                            for unique_booking_conflict in unique_booking_conflicts {
                                li {
                                    onclick: move |_| cr.send(ShiftPlanAction::UpdateSalesPerson(unique_booking_conflict.2)),
                                    class: "cursor-pointer hover:underline",
                                    "{unique_booking_conflict.0} - {unique_booking_conflict.1}"
                                }
                            }
                        }
                    }
                }
            }
        }

        // Booking-warning dismissible banner (non-blocking) — rendered below conflict list, above content
        if !booking_warnings.read().is_empty() {
            {
                let warnings_snap = booking_warnings.read().clone();
                let person_name: Option<ImStr> = current_sales_person
                    .read()
                    .as_ref()
                    .map(|sp| ImStr::from(sp.name.as_ref()));
                let dismiss_label = i18n.t(Key::BookingWarningDismiss).to_string();
                rsx! {
                    div { class: "mx-4 mt-3 print:hidden",
                        div { class: "relative border border-warn bg-warn-soft rounded-md px-4 py-3",
                            WarningList {
                                warnings: warnings_snap,
                                person_name,
                                suppress_header: false,
                            }
                            button {
                                r#type: "button",
                                class: "absolute top-2 right-2 w-6 h-6 inline-flex items-center justify-center rounded text-warn hover:bg-warn hover:text-ink font-mono",
                                "aria-label": "{dismiss_label}",
                                onclick: move |_| {
                                    booking_warnings.set(WarningsList::empty());
                                },
                                "×"
                            }
                        }
                    }
                }
            }
        }

        // D-24-03: Persistent overage warning section — all roles, client-side, hidden when no overage.
        // Inserted below the booking_warnings banner, above the ShiftplanTabBar.
        {
            let overage_slots: Vec<(String, u8, u8)> = if let Some(Ok(shiftplan)) =
                &*shift_plan_context.read_unchecked()
            {
                shiftplan
                    .slots
                    .iter()
                    .filter_map(|slot| {
                        if let Some(max) = slot.max_paid_employees {
                            if slot.current_paid_count > max {
                                let weekday_str = slot.day_of_week.i18n_string(&i18n);
                                let time_formatter =
                                    time::format_description::parse_borrowed::<2>("[hour]:[minute]").unwrap();
                                let from_str = slot.from.format(&time_formatter).unwrap_or_default();
                                let to_str = slot.to.format(&time_formatter).unwrap_or_default();
                                let label = format!("{weekday_str} {from_str}–{to_str}");
                                Some((label, slot.current_paid_count, max))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                Vec::new()
            };
            if !overage_slots.is_empty() {
                let overage_header = i18n.t(Key::ShiftplanPaidOverageSectionHeader);
                let overage_row_template = i18n.t(Key::ShiftplanPaidOverageRow);
                rsx! {
                    div {
                        class: "mx-4 my-3 px-4 py-3 bg-warn-soft border border-warn rounded-md print:hidden",
                        h2 { class: "text-h2 font-semibold pb-2 text-warn", "⚠️ {overage_header}" }
                        ul { class: "list-disc list-inside text-body text-ink",
                            for (slot_label , current , max) in overage_slots {
                                li {
                                    {
                                        overage_row_template
                                            .as_ref()
                                            .replace("{slot}", &slot_label)
                                            .replace("{current}", &current.to_string())
                                            .replace("{max}", &max.to_string())
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {}
            }
        }

        // D-24-05: global hard-block error banner — placed above the table (user decision,
        // not per-slot). Shown when the backend returned 409 PaidLimitExceeded; non-dismissible,
        // clears on next successful booking/removal action or week navigation.
        if block_error.read().is_some() {
            div {
                class: "mx-4 my-3 px-4 py-3 bg-bad-soft border border-bad rounded-md text-bad text-body print:hidden",
                {i18n.t(Key::BookingBlockedPaidLimit)}
            }
        }

        {
            if *view_mode.read() == state::ViewMode::Week {
                if let Some(Ok(catalog)) = &*shiftplan_catalog.read_unchecked() {
                    let catalog = catalog.clone();
                    rsx! {
                        div { class: "mx-4 mt-4",
                            ShiftplanTabBar {
                                shiftplans: catalog,
                                selected_id: *selected_shiftplan_id.read(),
                                on_select: move |id: Uuid| {
                                    selected_shiftplan_id.set(Some(id));
                                },
                                planning_mode: *change_structure_mode.read(),
                                config: CONFIG.read().clone(),
                                on_catalog_changed: move |new_selected: Option<Uuid>| {
                                    shiftplan_catalog.restart();
                                    if let Some(id) = new_selected {
                                        selected_shiftplan_id.set(Some(id));
                                    } else {
                                        selected_shiftplan_id.set(None);
                                    }
                                },
                            }
                        }
                    }
                } else {
                    rsx! {}
                }
            } else {
                rsx! {}
            }
        }

        {
            if *view_mode.read() == state::ViewMode::Day {
                // Day aggregate view
                to_owned![current_sales_person];
                rsx! {
                    div { class: "m-4",
                        DayButtonBar {
                            selected_day: *selected_day.read(),
                            show_sunday: *show_sunday.read(),
                            on_select_day: move |day: Weekday| {
                                selected_day.set(day);
                                cr.send(ShiftPlanAction::LoadDayAggregate);
                            },
                            on_prev_day: move |_| {
                                let (new_day, week_change) = crate::component::day_aggregate_view::prev_day(
                                    *selected_day.read(),
                                    *show_sunday.read(),
                                );
                                selected_day.set(new_day);
                                if week_change != 0 {
                                    cr.send(ShiftPlanAction::PreviousWeek);
                                }
                                cr.send(ShiftPlanAction::LoadDayAggregate);
                            },
                            on_next_day: move |_| {
                                let (new_day, week_change) = crate::component::day_aggregate_view::next_day(
                                    *selected_day.read(),
                                    *show_sunday.read(),
                                );
                                selected_day.set(new_day);
                                if week_change != 0 {
                                    cr.send(ShiftPlanAction::NextWeek);
                                }
                                cr.send(ShiftPlanAction::LoadDayAggregate);
                            },
                        }
                        if let Some(ref agg) = *day_aggregate.read() {
                            DayAggregateView {
                                day_aggregate: agg.clone(),
                                highlight_item_id: current_sales_person.read().as_ref().map(|sp| sp.id),
                                button_types: button_mode.clone(),
                                dropdown_entries: field_dropdown_entries.clone(),
                                add_event: {
                                    let current_sales_person = current_sales_person.clone();
                                    move |slot: state::Slot| {
                                        let sp = current_sales_person.read();
                                        if let Some(ref sp) = *sp {
                                            cr.send(ShiftPlanAction::AddUserToSlot {
                                                slot_id: slot.id,
                                                sales_person_id: sp.id,
                                                week: *week.read(),
                                                year: *year.read(),
                                            });
                                        }
                                    }
                                },
                                remove_event: {
                                    let current_sales_person = current_sales_person.clone();
                                    move |slot: state::Slot| {
                                        let sp = current_sales_person.read();
                                        if let Some(ref sp) = *sp {
                                            cr.send(ShiftPlanAction::RemoveUserFromSlotDay {
                                                slot_id: slot.id,
                                                sales_person_id: sp.id,
                                            });
                                        }
                                    }
                                },
                                item_clicked: move |sales_person_id: Uuid| {
                                    if is_shiftplanner {
                                        cr.send(ShiftPlanAction::UpdateSalesPerson(sales_person_id));
                                    }
                                },
                                is_shiftplanner,
                            }
                        } else {
                            div { class: "text-ink-muted italic", "Loading..." }
                        }
                    }
                }
            } else {
                // Week view (existing)
                match &*shift_plan_context.read_unchecked() {
                    Some(Ok(shift_plan)) => {
                        to_owned![current_sales_person, unavailable_days, person_absences];
                        rsx! {
                            div { class: "m-4",
                                SlotEdit {}
                                // KW-Status strip (D-39-05 visibility matrix; print:hidden).
                                // Shiftplaner -> dropdown; else badge only when set; else nothing.
                                div { class: "mb-3 flex items-center gap-2 print:hidden",
                                    if is_shiftplanner {
                                        WeekStatusDropdown {
                                            current_status: week_status.clone(),
                                            year: *year.read(),
                                            week: *week.read(),
                                            on_change: move |new_status: WeekStatus| {
                                                week_status_service.send(WeekStatusAction::Set {
                                                    year: *year.read(),
                                                    week: *week.read(),
                                                    status: new_status,
                                                });
                                            },
                                        }
                                    } else if should_show_badge(&week_status) {
                                        WeekStatusBadge { status: week_status.clone() }
                                    }
                                }
                                WeekView {
                                    shiftplan_data: shift_plan.clone(),
                                    date_of_monday: date,
                                    highlight_item_id: current_sales_person.read().as_ref().map(|sp| sp.id),
                                    discourage_weekdays: {
                                        // D-31-05: union-merge unavailable_days weekdays with
                                        // absence-derived weekdays (Phase 31).
                                        let mut discourage: Vec<Weekday> = unavailable_days
                                            .read()
                                            .iter()
                                            .map(|unavailable_day| unavailable_day.day_of_week)
                                            .collect();
                                        discourage.extend(
                                            absence_marker::absence_periods_to_discourage_days(
                                                person_absences.read().as_ref(),
                                                date,
                                            ),
                                        );
                                        discourage.into()
                                    },
                                    button_types: button_mode,
                                    dropdown_entries: field_dropdown_entries,
                                    // D-30-03 render-guard: show populated headers only when the
                                    // stored week-key matches the currently selected week.  On
                                    // mismatch (stale write that slipped through, or year data),
                                    // fall through to the existing empty state (`else { vec![] }`).
                                    weekday_headers: if weekly_summary.data_loaded
                                        && weekly_summary.weekly_summary.len() > 0
                                        && matches!(
                                            weekly_summary.loaded_week,
                                            Some(yw) if is_current_selection(yw, (*year.read(), *week.read()))
                                        )
                                    { vec![
                                        (
                                            Weekday::Monday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].monday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Tuesday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].tuesday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Wednesday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].wednesday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Thursday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].thursday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Friday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].friday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Saturday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].saturday_available_hours)
                                                .into(),
                                        ),
                                        (
                                            Weekday::Sunday,
                                            format!("{:.1}h", weekly_summary.weekly_summary[0].sunday_available_hours)
                                                .into(),
                                        ),
                                    ] } else { vec![] },
                                    add_event: move |slot: state::Slot| {
                                        to_owned![current_sales_person];
                                        info!("Register to slot");
                                        if let Some(ref current_sales_person) = *current_sales_person.read() {
                                            cr.send(ShiftPlanAction::AddUserToSlot {
                                                slot_id: slot.id,
                                                sales_person_id: current_sales_person.id,
                                                week: *week.read(),
                                                year: *year.read(),
                                            });
                                        }
                                        info!("Done");
                                    },
                                    remove_event: move |slot: state::Slot| {
                                        to_owned![current_sales_person];
                                        info!("Register to slot");
                                        if let Some(ref current_sales_person) = *current_sales_person.read() {
                                            cr.send(ShiftPlanAction::RemoveUserFromSlot {
                                                slot_id: slot.id,
                                                sales_person_id: current_sales_person.id,
                                            });
                                        }
                                        info!("Done");
                                    },
                                    item_clicked: move |sales_person_id: Uuid| {
                                        if is_shiftplanner {
                                            cr.send(ShiftPlanAction::UpdateSalesPerson(sales_person_id));
                                        }
                                    },
                                    title_double_clicked: move |weekday: Weekday| {
                                        if is_shiftplanner {
                                            cr.send(ShiftPlanAction::ToggleAvailability(weekday));
                                        }
                                    },
                                    is_shiftplanner,
                                    weekday_sub_headers,
                                }

                            div { class: "mt-4 print:hidden flex flex-col gap-2",
                                div { class: "flex items-center justify-end",
                                    WorkingHoursOverviewLayoutToggle {
                                        active: working_hours_layout(),
                                        on_change: move |layout: WorkingHoursLayout| {
                                            ui_prefs::set_working_hours_layout(layout);
                                            working_hours_layout.set(layout);
                                        },
                                    }
                                }
                                WorkingHoursMiniOverview {
                                    working_hours: WORKING_HOURS_MINI.read().clone(),
                                    on_dbl_click: move |employee_id: Uuid| {
                                        cr.send(ShiftPlanAction::UpdateSalesPerson(employee_id.clone()));
                                    },
                                    selected_sales_person_id: current_sales_person.read().as_ref().map(|sp| sp.id),
                                    show_balance: is_shiftplanner && is_hr,
                                    layout: working_hours_layout(),
                                }
                            }

                            // Week Message Section
                            div { class: "mt-4 mb-4 p-4 bg-surface border border-border rounded-lg",
                                h3 { class: "text-h2 font-semibold mb-2 text-ink",
                                    {i18n.t(Key::WeekMessage)}
                                }
                                if is_shiftplanner {
                                    div { class: "space-y-2",
                                        textarea {
                                            class: "w-full p-2 border border-border-strong bg-surface text-ink text-body rounded-md resize-y form-input",
                                            rows: "3",
                                            placeholder: "Enter week message...",
                                            value: "{week_message_draft}",
                                            oninput: move |event| {
                                                week_message_draft.set(event.value());
                                            }
                                        }
                                        div { class: "flex gap-2 items-center",
                                            Btn {
                                                variant: BtnVariant::Primary,
                                                disabled: week_message_draft() == week_message(),
                                                on_click: move |_| {
                                                    let message = week_message_draft();
                                                    cr.send(ShiftPlanAction::SaveWeekMessage(message));
                                                },
                                                {i18n.t(Key::Save)}
                                            }
                                            if week_message_draft() != week_message() {
                                                span { class: "text-small font-normal text-warn self-center",
                                                    {i18n.t(Key::UnsavedChanges)}
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    div { class: "p-2 bg-surface-alt rounded-md",
                                        if week_message().is_empty() {
                                            span { class: "text-ink-muted italic",
                                                "No message for this week"
                                            }
                                        } else {
                                            pre { class: "whitespace-pre-wrap text-ink text-body",
                                                {week_message()}
                                            }
                                        }
                                    }
                                }
                            }
                            // Shiftplan Report Section (only visible for shiftplanner role)
                            if is_shiftplanner {
                                div { class: "bg-surface border border-border rounded-lg p-6 mt-6 mx-4 print:hidden",
                                    h2 { class: "text-h2 font-semibold mb-4 text-ink", "{i18n.t(Key::ShiftplanReport)}" }

                                    div { class: "space-y-4",
                                        // Template Selection
                                        div { class: "mb-4",
                                            label { class: "block text-small font-medium text-ink mb-2",
                                                "{i18n.t(Key::SelectTemplate)} ({TEXT_TEMPLATE_STORE.read().filtered_templates.len()} shiftplan report templates available)"
                                            }
                                            select {
                                                class: "h-[34px] w-full px-[10px] border border-border-strong rounded-md bg-surface text-ink text-body form-input",
                                                value: selected_template_id.read().as_ref().map(|id| id.to_string()).unwrap_or_default(),
                                                onchange: move |event| {
                                                    if let Ok(uuid) = Uuid::parse_str(&event.value()) {
                                                        selected_template_id.set(Some(uuid));
                                                    } else {
                                                        selected_template_id.set(None);
                                                    }
                                                },
                                                option { value: "", "Select a template..." }
                                                for template in TEXT_TEMPLATE_STORE.read().filtered_templates.iter() {
                                                    option {
                                                        value: "{template.id}",
                                                        if let Some(ref name) = template.name {
                                                            "{name}"
                                                        } else {
                                                            "{template.template_type} - {template.template_text.chars().take(50).collect::<String>()}..."
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Generate Button
                                        Btn {
                                            variant: BtnVariant::Primary,
                                            disabled: selected_template_id.read().is_none() || *generating_report.read(),
                                            on_click: move |_| {
                                                if let Some(template_id) = *selected_template_id.read() {
                                                    let config = CONFIG.read().clone();
                                                    spawn(async move {
                                                        generating_report.set(true);
                                                        shiftplan_report_result.set(None);

                                                        match loader::generate_block_report(config, template_id).await {
                                                            Ok(report) => {
                                                                shiftplan_report_result.set(Some(report));
                                                            }
                                                            Err(e) => {
                                                                shiftplan_report_result.set(Some(format!("Error generating report: {}", e)));
                                                            }
                                                        }

                                                        generating_report.set(false);
                                                    });
                                                }
                                            },
                                            if *generating_report.read() {
                                                "{i18n.t(Key::GeneratingReport)}"
                                            } else {
                                                "{i18n.t(Key::GenerateShiftplanReport)}"
                                            }
                                        }

                                        // Report Result
                                        {
                                            let report_opt = shiftplan_report_result.read().clone();
                                            if let Some(report) = report_opt {
                                                let report_for_display = report.clone();
                                                let report_for_copy = report.clone();
                                                rsx! {
                                                    div { class: "border-t border-border pt-4",
                                                        div { class: "flex justify-between items-center mb-3",
                                                            h3 { class: "text-body font-medium text-ink", "{i18n.t(Key::ShiftplanReportGenerated)}" }
                                                            div { class: "flex items-center gap-2",
                                                                Btn {
                                                                    variant: BtnVariant::Secondary,
                                                                    on_click: move |_| {
                                                                        let report_text = report_for_copy.clone();
                                                                        let i18n_copy = I18N.read().clone();
                                                                        spawn(async move {
                                                                            copy_status.set(None);
                                                                            match crate::js::copy_to_clipboard(&report_text).await {
                                                                                Ok(_) => copy_status.set(Some(i18n_copy.t(Key::CopiedToClipboard).to_string())),
                                                                                Err(_) => copy_status.set(Some(i18n_copy.t(Key::CopyFailed).to_string())),
                                                                            }
                                                                            spawn(async move {
                                                                                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                                                                copy_status.set(None);
                                                                            });
                                                                        });
                                                                    },
                                                                    "{i18n.t(Key::CopyToClipboard)}"
                                                                }
                                                                if let Some(status) = copy_status.read().clone() {
                                                                    span { class: "text-small text-good font-medium", "{status}" }
                                                                }
                                                            }
                                                        }
                                                        div { class: "bg-surface-alt p-4 rounded-md border border-border",
                                                            pre { class: "whitespace-pre-wrap text-small font-normal font-mono overflow-x-auto text-ink", "{report_for_display}" }
                                                        }
                                                    }
                                                }
                                            } else {
                                                rsx! { div {} }
                                            }
                                        }
                                    }
                                }
                            }

                            // Booking log table — only visible for shiftplanner role AND when expanded via toolbar toggle
                            if is_shiftplanner && *show_booking_log.read() {
                                div { class: "mt-6 mx-4 print:hidden",
                                    BookingLogTable {
                                        bookings: BOOKING_LOG_STORE.read().clone(),
                                        name_filter: booking_log_name_filter.read().clone(),
                                        on_name_filter_change: move |value: String| {
                                            booking_log_name_filter.set(value);
                                        },
                                        day_filter: *booking_log_day_filter.read(),
                                        on_day_filter_change: move |value: Option<Weekday>| {
                                            booking_log_day_filter.set(value);
                                        },
                                        status_filter: booking_log_status_filter.read().clone(),
                                        on_status_filter_change: move |value: String| {
                                            booking_log_status_filter.set(value);
                                        },
                                        created_by_filter: booking_log_created_by_filter.read().clone(),
                                        on_created_by_filter_change: move |value: String| {
                                            booking_log_created_by_filter.set(value);
                                        },
                                        on_clear_filters: move |_| {
                                            booking_log_name_filter.set(String::new());
                                            booking_log_day_filter.set(None);
                                            booking_log_status_filter.set("all".to_string());
                                            booking_log_created_by_filter.set("all".to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                    Some(Err(err)) => {
                        rsx! {
                            div { class: "m-4", "Error while loading shift plan: {err}" }
                        }
                    }
                    _ => {
                        rsx! {
                            div { class: "m-4", "Loading shift plan..." }
                        }
                    }
                }
            }
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::{WarningList, WarningsList};
    use crate::i18n::{generate, Locale};
    use crate::service::i18n::I18N;
    use dioxus::prelude::*;
    use rest_types::{AbsenceCategoryTO, WarningTO};
    use std::rc::Rc;
    use uuid::Uuid;

    fn render(comp: fn() -> Element) -> String {
        let mut vdom = VirtualDom::new(comp);
        vdom.rebuild_in_place();
        dioxus_ssr::render(&vdom)
    }

    fn pin_de_locale() {
        use_hook(|| {
            *I18N.write() = generate(Locale::De);
        });
    }

    /// Banner is rendered when booking_warnings is non-empty.
    /// The WarningList header MUST be visible (suppress_header: false in banner).
    #[test]
    fn booking_banner_present_when_warnings_non_empty() {
        fn app() -> Element {
            pin_de_locale();
            let warnings = WarningsList(Rc::from(
                vec![WarningTO::BookingOnAbsenceDay {
                    booking_id: Uuid::nil(),
                    date: time::Date::from_calendar_date(2026, time::Month::January, 1).unwrap(),
                    absence_id: Uuid::nil(),
                    category: AbsenceCategoryTO::Vacation,
                }]
                .as_slice(),
            ));
            rsx! {
                WarningList {
                    warnings,
                    person_name: Some(crate::base_types::ImStr::from("Testperson")),
                    suppress_header: false,
                }
            }
        }
        let html = render(app);
        // With suppress_header: false (banner mode), the internal header MUST appear
        assert!(
            html.contains("text-micro text-warn font-semibold uppercase"),
            "WarningList with suppress_header:false must render internal header in banner, got: {html}"
        );
        // Warning content (person name and date) must be present
        assert!(
            html.contains("Testperson"),
            "banner must render person name, got: {html}"
        );
        assert!(
            html.contains("2026-01-01"),
            "banner must render booking date, got: {html}"
        );
    }

    /// Banner is absent when booking_warnings is empty.
    #[test]
    fn booking_banner_absent_when_warnings_empty() {
        fn app() -> Element {
            pin_de_locale();
            rsx! {
                WarningList { warnings: WarningsList::empty() }
            }
        }
        let html = render(app);
        // Empty warnings must produce no banner output
        assert!(
            !html.contains("border-l-[3px]"),
            "empty warnings must not render a warning box, got: {html}"
        );
        assert!(
            !html.contains("text-micro text-warn"),
            "empty warnings must not render header, got: {html}"
        );
    }

    /// Guard: Banner source must use booking_warnings signal (not the old pending_warnings).
    /// Also verifies the dismiss control exists and uses the BookingWarningDismiss i18n key.
    #[test]
    fn booking_banner_source_contract() {
        let source = include_str!("shiftplan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);

        // New signal name must be present, old ones must be gone
        assert!(
            production.contains("booking_warnings"),
            "page must use booking_warnings signal"
        );
        assert!(
            !production.contains("pending_warnings"),
            "old pending_warnings signal must be removed"
        );
        assert!(
            !production.contains("pending_rollback_id"),
            "old pending_rollback_id signal must be removed"
        );
        assert!(
            !production.contains("RollbackBooking"),
            "RollbackBooking action must be removed"
        );
        // Dismiss control must be wired
        assert!(
            production.contains("BookingWarningDismiss"),
            "dismiss button must use BookingWarningDismiss i18n key"
        );
        // Banner must use suppress_header: false so WarningList shows its own header
        assert!(
            production.contains("suppress_header: false"),
            "banner must use suppress_header: false"
        );
        // Dialog import must be gone
        assert!(
            !production.contains("DialogVariant"),
            "DialogVariant must not appear in banner-based code"
        );
    }

    #[test]
    fn shiftplan_page_no_legacy_classes_in_source() {
        let source = include_str!("shiftplan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "bg-gray-",
            "bg-white",
            "text-gray-",
            "text-blue-",
            "text-red-",
            "text-green-",
            "text-orange-",
            "bg-blue-",
            "bg-green-",
            "bg-red-",
            "bg-slate-",
            "border-gray-",
            "border-black",
        ] {
            assert!(
                !production.contains(forbidden),
                "non-test source contains legacy class `{}`",
                forbidden
            );
        }
    }

    #[test]
    fn shiftplan_page_wires_working_hours_layout_persistence() {
        // Source-level wiring check: the Shiftplan page must read the layout
        // from the prefs service to seed its signal, render the toggle, and
        // persist the choice on change. Renders the full page in SSR isn't
        // viable here (it depends on coroutine context, signals from many
        // services, and async loaders), so we assert the contract at the
        // source level instead.
        let source = include_str!("shiftplan.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            production.contains("ui_prefs::get_working_hours_layout"),
            "page must seed the layout signal from ui_prefs"
        );
        assert!(
            production.contains("ui_prefs::set_working_hours_layout"),
            "page must persist the layout choice via ui_prefs"
        );
        assert!(
            production.contains("WorkingHoursOverviewLayoutToggle"),
            "page must render the layout toggle"
        );
        assert!(
            production.contains("layout: working_hours_layout()"),
            "page must pass the current layout into the overview"
        );
    }
}
