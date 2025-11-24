use rest_types::{ExtraHoursTO, GenerateInvitationRequest, InvitationResponse, SalesPersonTO, SpecialDayTypeTO, UserRole, UserTO, WeekMessageTO};
use std::rc::Rc;
use tracing::info;
use uuid::Uuid;

use crate::{
    api,
    base_types::ImStr,
    error::ShiftyError,
    state::{
        booking_log::BookingLog,
        employee::{Employee, ExtraHours},
        employee_work_details::{EmployeeWorkDetails, WorkingHoursMini},
        sales_person_available::SalesPersonUnavailable,
        shiftplan::{Booking, BookingConflict, SalesPerson},
        slot_edit::SlotEditItem,
        text_template::TextTemplate,
        week::Week,
        weekly_overview::WeeklySummary,
        Config, Shiftplan, Slot, User, Weekday,
    },
};

pub async fn load_sales_persons(config: Config) -> Result<Rc<[SalesPerson]>, ShiftyError> {
    let sales_person_tos = api::get_sales_persons(config).await?;
    let mut sales_persons: Vec<SalesPerson> =
        sales_person_tos.iter().map(SalesPerson::from).collect();
    sales_persons.sort_by_key(|sales_person| sales_person.name.clone());
    let sales_persons: Rc<[SalesPerson]> = sales_persons.into();

    Ok(sales_persons)
}

pub async fn load_user_for_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<Option<ImStr>, ShiftyError> {
    let user = api::get_user_for_sales_person(config, sales_person_id)
        .await?
        .map(|user| user.into());
    Ok(user)
}

pub async fn save_user_for_sales_person(
    config: Config,
    sales_person_id: Uuid,
    user_id: ImStr,
) -> Result<(), ShiftyError> {
    api::post_user_to_sales_person(config, sales_person_id, user_id).await?;
    Ok(())
}

pub async fn remove_user_from_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<(), ShiftyError> {
    api::delete_user_from_sales_person(config, sales_person_id).await?;
    Ok(())
}

pub async fn load_bookings(
    config: Config,
    sales_persons: Rc<[SalesPerson]>,
    week: u8,
    year: u32,
) -> Result<Rc<[Booking]>, ShiftyError> {
    let booking_tos = api::get_bookings_for_week(config, week, year).await?;
    let bookings: Rc<[Booking]> = booking_tos
        .iter()
        .map(|booking_to| booking_to.into())
        .map(|booking: Booking| {
            let sales_person = sales_persons
                .iter()
                .find(|sales_person| sales_person.id == booking.sales_person_id);
            if let Some(sales_person) = sales_person {
                Booking {
                    label: sales_person.name.clone(),
                    background_color: sales_person.background_color.clone(),
                    ..booking
                }
            } else {
                booking
            }
        })
        .collect();
    Ok(bookings)
}

pub async fn load_slots(
    config: Config,
    year: u32,
    week: u8,
    bookings: Rc<[Booking]>,
) -> Result<Rc<[Slot]>, ShiftyError> {
    let slot_tos = api::get_slots(config.clone(), year, week).await?;
    let special_days = api::get_special_days_for_week(config.clone(), year, week).await?;
    let slots: Rc<[Slot]> = slot_tos
        .iter()
        .filter(|slot_to| {
            !special_days.iter().any(|special_day| {
                special_day.day_of_week == slot_to.day_of_week
                    && (special_day.day_type == SpecialDayTypeTO::Holiday
                        || special_day.day_type == SpecialDayTypeTO::ShortDay
                            && special_day.time_of_day.is_some()
                            && slot_to.to > special_day.time_of_day.unwrap())
            })
        })
        .map(|slot_to| slot_to.into())
        .map(|slot: Slot| Slot {
            bookings: bookings
                .iter()
                .filter(|booking| booking.slot_id == slot.id)
                .map(|booking| booking.clone())
                .collect(),
            ..slot
        })
        .collect();
    Ok(slots)
}

pub async fn load_shift_plan(
    config: Config,
    week: u8,
    year: u32,
) -> Result<Shiftplan, ShiftyError> {
    let shiftplan_week = api::get_shiftplan_week(config, year, week).await?;
    let slots = shiftplan_week
        .days
        .iter()
        .flat_map(|day| day.slots.iter())
        .map(|slot| Slot {
            id: slot.slot.id,
            day_of_week: slot.slot.day_of_week.into(),
            from: slot.slot.from,
            to: slot.slot.to,
            min_resources: slot.slot.min_resources,
            bookings: slot
                .bookings
                .iter()
                .map(|booking| Booking {
                    id: booking.booking.id,
                    sales_person_id: booking.booking.sales_person_id,
                    slot_id: booking.booking.slot_id,
                    week: booking.booking.calendar_week as u8,
                    year: booking.booking.year,
                    label: booking.sales_person.name.as_ref().into(),
                    background_color: booking.sales_person.background_color.as_ref().into(),
                    self_added: booking.self_added.unwrap_or(false),
                })
                .collect(),
        })
        .collect();
    tracing::info!("Slots: {:?}", &slots);
    Ok(Shiftplan { week, year, slots })
}

pub async fn load_current_sales_person(config: Config) -> Result<Option<SalesPerson>, ShiftyError> {
    let sales_person_to = api::get_current_sales_person(config).await?;
    let sales_person = sales_person_to.as_ref().map(SalesPerson::from);
    Ok(sales_person)
}

pub async fn load_sales_person(
    config: Config,
    sales_person_id: Uuid,
) -> Result<SalesPerson, ShiftyError> {
    let sales_person_to = api::get_sales_person(config, sales_person_id).await?;
    let sales_person = SalesPerson::from(&sales_person_to);
    Ok(sales_person)
}

pub async fn save_sales_person(
    config: Config,
    sales_person: SalesPerson,
) -> Result<(), ShiftyError> {
    if sales_person.id.is_nil() {
        api::post_sales_person(config, SalesPersonTO::from(&sales_person)).await?;
    } else {
        api::put_sales_person(config, SalesPersonTO::from(&sales_person)).await?;
    }
    Ok(())
}

pub async fn register_user_to_slot(
    config: Config,
    slot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    week: u8,
    year: u32,
) -> Result<(), ShiftyError> {
    info!("Add booking");
    api::add_booking(config, user_id, slot_id, week, year).await?;
    Ok(())
}

pub async fn remove_user_from_slot(
    config: Config,
    slot_id: uuid::Uuid,
    user_id: uuid::Uuid,
    shiftplan: Shiftplan,
) -> Result<(), ShiftyError> {
    info!("Remove booking");
    let slot = shiftplan.slots.iter().find(|slot| slot.id == slot_id);
    if let Some(slot) = slot {
        let booking = slot
            .bookings
            .iter()
            .find(|booking| booking.sales_person_id == user_id);
        if let Some(booking) = booking {
            api::remove_booking(config, booking.id).await?;
        }
    }
    Ok(())
}

pub async fn copy_from_previous_week(
    config: Config,
    week: u8,
    year: u32,
) -> Result<(), ShiftyError> {
    info!("Copy from previous week");
    api::copy_week(config, week - 1, year, week, year).await?;
    Ok(())
}

pub async fn load_employees(
    config: Config,
    year: u32,
    week_until: u8,
) -> Result<Rc<[Employee]>, ShiftyError> {
    let report_tos = api::get_short_reports(config, year, week_until).await?;
    Ok(report_tos.iter().map(Employee::from).collect())
}

pub async fn load_employee_details(
    config: Config,
    year: u32,
    week_until: u8,
    employee_id: uuid::Uuid,
) -> Result<Employee, ShiftyError> {
    let report = api::get_employee_reports(config, employee_id, year, week_until).await?;
    Ok(Employee::from(report.as_ref()))
}

pub async fn load_extra_hours_per_year(
    config: Config,
    year: u32,
    employee_id: uuid::Uuid,
) -> Result<Rc<[ExtraHours]>, ShiftyError> {
    let mut extra_hours: Vec<ExtraHoursTO> =
        api::get_extra_hours_for_year(config, employee_id, year, 53)
            .await?
            .iter()
            .cloned()
            .collect();
    extra_hours.sort_by_key(|extra_hours| extra_hours.date_time.clone());
    Ok(extra_hours.iter().map(ExtraHours::from).collect())
}

pub async fn load_weeks(_config: Config, year: u32) -> Result<Rc<[Week]>, ShiftyError> {
    let weeks: Rc<[Week]> = (1..=53).map(|week| Week { year, week }).collect();
    Ok(weeks)
}

pub async fn load_unavailable_sales_person_days_for_week(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    week: u8,
) -> Result<Rc<[SalesPersonUnavailable]>, ShiftyError> {
    let unavailable_days =
        api::get_unavailable_sales_person_days_for_week(config, sales_person_id, year, week)
            .await?;
    let weeks: Rc<[SalesPersonUnavailable]> = unavailable_days
        .iter()
        .map(SalesPersonUnavailable::from)
        .collect();
    Ok(weeks)
}

pub async fn create_unavailable_sales_person_day(
    config: Config,
    sales_person_id: Uuid,
    year: u32,
    week: u8,
    day: Weekday,
) -> Result<(), ShiftyError> {
    api::create_unavailable_sales_person_day(config, sales_person_id, year, week, (&day).into())
        .await?;
    Ok(())
}

pub async fn delete_unavailable_sales_person_day(
    config: Config,
    unavailable_id: Uuid,
) -> Result<(), ShiftyError> {
    api::delete_unavailable_sales_person_day(config, unavailable_id).await?;
    Ok(())
}

pub async fn load_working_hours_minified_for_week(
    config: Config,
    year: u32,
    week: u8,
    //sales_persons: Rc<[SalesPerson]>,
) -> Result<Rc<[WorkingHoursMini]>, ShiftyError> {
    let reports = api::get_working_hours_minified_for_week(config, year, week).await?;
    Ok(reports
        .iter()
        .map(move |report| WorkingHoursMini {
            sales_person_id: report.sales_person.id,
            sales_person_name: report.sales_person.name.to_owned().as_ref().into(),
            expected_hours: report.expected_hours,
            dynamic_hours: report.dynamic_hours,
            actual_hours: report.overall_hours,
        })
        .collect())
}

pub async fn load_all_users(config: Config) -> Result<Rc<[User]>, ShiftyError> {
    let users = api::get_all_users(config).await?;
    Ok(users.iter().map(User::from).collect())
}

pub async fn load_all_roles(config: Config) -> Result<Rc<[ImStr]>, ShiftyError> {
    let roles = api::get_all_roles(config).await?;
    Ok(roles.iter().map(|role| role.name.clone().into()).collect())
}

pub async fn load_roles_from_user(
    config: Config,
    user_id: ImStr,
) -> Result<Rc<[ImStr]>, ShiftyError> {
    let roles = api::get_roles_from_user(config, user_id).await?;
    Ok(roles.iter().map(|role| role.name.clone().into()).collect())
}

pub async fn add_user_to_role(
    config: Config,
    user_id: ImStr,
    role: ImStr,
) -> Result<(), ShiftyError> {
    api::add_role_to_user(
        config,
        UserRole {
            user: user_id.to_string(),
            role: role.to_string(),
        },
    )
    .await?;
    Ok(())
}

pub async fn remove_user_from_role(
    config: Config,
    user_id: ImStr,
    role: ImStr,
) -> Result<(), ShiftyError> {
    api::remove_role_from_user(
        config,
        UserRole {
            user: user_id.to_string(),
            role: role.to_string(),
        },
    )
    .await?;
    Ok(())
}

pub async fn add_user(config: Config, user: ImStr) -> Result<(), ShiftyError> {
    api::add_user(
        config,
        UserTO {
            name: user.to_string(),
        },
    )
    .await?;
    Ok(())
}

pub async fn delete_user(config: Config, user: ImStr) -> Result<(), ShiftyError> {
    api::delete_user(config, user).await?;
    Ok(())
}

pub async fn load_bookings_conflicts_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[BookingConflict]>, ShiftyError> {
    Ok(api::get_booking_conflicts_for_week(config, year, week)
        .await?
        .iter()
        .map(|booking_conflict_to| BookingConflict::from(booking_conflict_to))
        .collect())
}

pub async fn load_booking_log(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Rc<[BookingLog]>, ShiftyError> {
    Ok(api::get_booking_log(config, year, week)
        .await?
        .iter()
        .map(|booking_log_to| BookingLog::from(booking_log_to))
        .collect())
}

pub async fn load_weekly_summary_for_year(
    config: Config,
    year: u32,
) -> Result<Rc<[WeeklySummary]>, ShiftyError> {
    let extra_hours_to = api::get_weekly_overview(config, year).await?;
    let mut extra_hours_to: Vec<WeeklySummary> =
        extra_hours_to.iter().map(WeeklySummary::from).collect();
    extra_hours_to.sort_by_key(|extra_hours| (extra_hours.year, extra_hours.week));

    Ok(extra_hours_to.into())
}

pub async fn load_summary_for_week(
    config: Config,
    year: u32,
    week: u8,
) -> Result<WeeklySummary, ShiftyError> {
    let yearly_summaries = api::get_weekly_overview(config, year).await?;
    if let Some(summary) = yearly_summaries.iter().find(|s| s.week == week) {
        Ok(WeeklySummary::from(summary))
    } else {
        // If no summary found for the specific week, create a default one
        Ok(WeeklySummary {
            year,
            week,
            available_hours: 0.0,
            required_hours: 0.0,
            paid_hours: 0.0,
            volunteer_hours: 0.0,
            monday_available_hours: 0.0,
            tuesday_available_hours: 0.0,
            wednesday_available_hours: 0.0,
            thursday_available_hours: 0.0,
            friday_available_hours: 0.0,
            saturday_available_hours: 0.0,
            sunday_available_hours: 0.0,
        })
    }
}

pub async fn load_employee_work_details(
    config: Config,
    employee_id: Uuid,
) -> Result<Rc<[EmployeeWorkDetails]>, ShiftyError> {
    let mut employee_work_details_to: Vec<EmployeeWorkDetails> =
        api::get_employee_work_details_for_sales_person(config, employee_id)
            .await?
            .iter()
            .flat_map(EmployeeWorkDetails::try_from)
            .collect();
    employee_work_details_to.sort_by_key(|details| details.from);
    Ok(employee_work_details_to.into())
}

pub async fn save_new_employee_work_details(
    config: Config,
    employee_work_details: EmployeeWorkDetails,
) -> Result<(), ShiftyError> {
    api::post_employee_work_details(config, (&employee_work_details).try_into()?).await?;
    Ok(())
}

pub async fn update_employee_work_details(
    config: Config,
    employee_work_details: EmployeeWorkDetails,
) -> Result<(), ShiftyError> {
    api::put_employee_work_details(config, (&employee_work_details).try_into()?).await?;
    Ok(())
}

pub async fn load_slot(config: Config, slot_id: Uuid) -> Result<SlotEditItem, ShiftyError> {
    let slot_to = api::get_slot(config, slot_id).await?;
    Ok((&slot_to).into())
}

pub async fn save_slot(
    config: Config,
    slot: Rc<SlotEditItem>,
    year: u32,
    week: u8,
) -> Result<(), ShiftyError> {
    api::update_slot(config, slot.as_ref().into(), year, week).await?;
    Ok(())
}

pub async fn create_slot(config: Config, slot: Rc<SlotEditItem>) -> Result<bool, ShiftyError> {
    Ok(api::post_slot(config, slot.as_ref().into()).await?)
}

pub async fn load_week_message(
    config: Config,
    year: u32,
    week: u8,
) -> Result<Option<String>, ShiftyError> {
    match api::get_week_message(config, year, week).await? {
        Some(week_message) => Ok(Some(week_message.message.to_string())),
        None => Ok(None),
    }
}

pub async fn save_week_message(
    config: Config,
    year: u32,
    week: u8,
    message: String,
) -> Result<(), ShiftyError> {
    // First check if a week message already exists
    match api::get_week_message(config.clone(), year, week).await? {
        Some(existing_message) => {
            // Update existing message using PUT
            let week_message = WeekMessageTO {
                id: existing_message.id,
                year,
                calendar_week: week,
                message: message.into(),
                created: existing_message.created,
                deleted: None,
                version: existing_message.version,
            };
            api::put_week_message(config, week_message).await?;
        }
        None => {
            // Create new message using POST
            let week_message = WeekMessageTO {
                id: uuid::Uuid::nil(),
                year,
                calendar_week: week,
                message: message.into(),
                created: None,
                deleted: None,
                version: uuid::Uuid::nil(),
            };
            api::post_week_message(config, week_message).await?;
        }
    }
    Ok(())
}

pub async fn load_sales_person_by_user(
    config: Config,
    username: ImStr,
) -> Result<Option<SalesPerson>, ShiftyError> {
    let sales_person_to = api::get_sales_person_by_user(config, username).await?;
    let sales_person = sales_person_to.as_ref().map(SalesPerson::from);
    Ok(sales_person)
}

pub async fn load_text_templates(config: Config) -> Result<Rc<[TextTemplate]>, ShiftyError> {
    let template_tos = api::get_text_templates(config).await?;
    let templates: Vec<TextTemplate> = template_tos.iter().map(TextTemplate::from).collect();
    Ok(templates.into())
}

pub async fn load_text_templates_by_type(
    config: Config,
    template_type: &str,
) -> Result<Rc<[TextTemplate]>, ShiftyError> {
    let template_tos = api::get_text_templates_by_type(config, template_type).await?;
    let templates: Vec<TextTemplate> = template_tos.iter().map(TextTemplate::from).collect();
    Ok(templates.into())
}

pub async fn load_text_template(
    config: Config,
    template_id: Uuid,
) -> Result<TextTemplate, ShiftyError> {
    let template_to = api::get_text_template(config, template_id).await?;
    Ok(TextTemplate::from(&template_to))
}

pub async fn save_text_template(
    config: Config,
    template: &TextTemplate,
) -> Result<TextTemplate, ShiftyError> {
    let request = template.to_create_request();
    let result_to = api::create_text_template(config, request).await?;
    Ok(TextTemplate::from(&result_to))
}

pub async fn update_text_template(
    config: Config,
    template_id: Uuid,
    template: &TextTemplate,
) -> Result<TextTemplate, ShiftyError> {
    let request = template.to_update_request();
    let result_to = api::update_text_template(config, template_id, request).await?;
    Ok(TextTemplate::from(&result_to))
}

pub async fn delete_text_template(config: Config, template_id: Uuid) -> Result<(), ShiftyError> {
    api::delete_text_template(config, template_id).await?;
    Ok(())
}

pub async fn generate_custom_report(
    config: Config,
    billing_period_id: Uuid,
    template_id: Uuid,
) -> Result<String, ShiftyError> {
    let report = api::generate_custom_report(config, billing_period_id, template_id).await?;
    Ok(report)
}

pub async fn generate_block_report(
    config: Config,
    template_id: Uuid,
) -> Result<String, ShiftyError> {
    let report = api::generate_block_report(config, template_id).await?;
    Ok(report)
}

fn fix_invitation_link(mut invitation: InvitationResponse, backend_url: &str) -> InvitationResponse {
    invitation.invitation_link = format!("{}/auth/invitation/{}", backend_url, invitation.token);
    invitation
}

pub async fn generate_invitation(
    config: Config,
    username: ImStr,
    expiration_hours: Option<i64>,
) -> Result<InvitationResponse, ShiftyError> {
    let request = GenerateInvitationRequest {
        username: username.to_string(),
        expiration_hours,
    };
    let invitation = api::generate_invitation(config.clone(), request).await?;
    let fixed_invitation = fix_invitation_link(invitation, &config.backend);
    Ok(fixed_invitation)
}

pub async fn load_user_invitations(
    config: Config,
    username: ImStr,
) -> Result<Rc<[InvitationResponse]>, ShiftyError> {
    let invitations = api::list_user_invitations(config.clone(), username).await?;
    let fixed_invitations: Vec<InvitationResponse> = invitations
        .iter()
        .map(|inv| fix_invitation_link(inv.clone(), &config.backend))
        .collect();
    Ok(fixed_invitations.into())
}

pub async fn revoke_invitation(
    config: Config,
    invitation_id: Uuid,
) -> Result<(), ShiftyError> {
    api::revoke_invitation(config, invitation_id).await?;
    Ok(())
}

pub async fn revoke_invitation_session(
    config: Config,
    invitation_id: Uuid,
) -> Result<(), ShiftyError> {
    api::revoke_session_for_invitation(config, invitation_id).await?;
    Ok(())
}
