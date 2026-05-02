use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    booking::{Booking, BookingService},
    permission::{Authentication, PermissionService, SHIFTPLANNER_PRIVILEGE},
    sales_person::{SalesPerson, SalesPersonService},
    shiftplan::{
        PlanDayView, ShiftplanBooking, ShiftplanDay, ShiftplanDayAggregate,
        ShiftplanViewService, ShiftplanSlot, ShiftplanWeek,
    },
    shiftplan_catalog::ShiftplanService,
    slot::{Slot, SlotService},
    special_days::{SpecialDay, SpecialDayService, SpecialDayType},
    ServiceError,
};
use shifty_utils::DayOfWeek;
use uuid::Uuid;

use crate::gen_service_impl;

pub(crate) fn build_shiftplan_day(
    day_of_week: DayOfWeek,
    slots: &[Slot],
    bookings: &[Booking],
    sales_persons: &[SalesPerson],
    special_days: &[SpecialDay],
    user_assignments: Option<&HashMap<Uuid, Arc<str>>>,
) -> Result<ShiftplanDay, ServiceError> {
    // Check if this day is a holiday
    let is_holiday = special_days.iter().any(|sd| {
        sd.day_of_week == day_of_week && sd.day_type == SpecialDayType::Holiday
    });

    // Find short day cutoff if applicable
    let short_day_cutoff = special_days.iter().find_map(|sd| {
        if sd.day_of_week == day_of_week
            && sd.day_type == SpecialDayType::ShortDay
            && sd.time_of_day.is_some()
        {
            sd.time_of_day
        } else {
            None
        }
    });

    let mut day_slots = Vec::new();

    if !is_holiday {
        for slot in slots.iter() {
            if slot.day_of_week != day_of_week {
                continue;
            }

            // Filter by short day cutoff
            if let Some(cutoff) = short_day_cutoff {
                if slot.to > cutoff {
                    continue;
                }
            }

            // Find bookings for this slot
            let slot_bookings = bookings
                .iter()
                .filter(|b| b.slot_id == slot.id)
                .map(|booking| {
                    let sales_person = sales_persons
                        .iter()
                        .find(|sp| sp.id == booking.sales_person_id)
                        .ok_or_else(|| ServiceError::EntityNotFound(booking.sales_person_id))?
                        .clone();

                    let self_added = user_assignments.and_then(|assignments| {
                        assignments
                            .get(&booking.sales_person_id)
                            .and_then(|assigned_user| {
                                booking
                                    .created_by
                                    .as_ref()
                                    .map(|created_by| created_by == assigned_user)
                            })
                    });

                    Ok(ShiftplanBooking {
                        booking: booking.clone(),
                        sales_person,
                        self_added,
                    })
                })
                .collect::<Result<Vec<_>, ServiceError>>()?;

            day_slots.push(ShiftplanSlot {
                slot: slot.clone(),
                bookings: slot_bookings,
            });
        }
    }

    // Sort slots by time
    day_slots.sort_by(|a, b| a.slot.from.cmp(&b.slot.from));

    Ok(ShiftplanDay {
        day_of_week,
        slots: day_slots,
        // Phase-3 additiv — globale Sicht setzt nie etwas; per-sales-person
        // Sicht (Wave 3, Plan 03-04) wird das Feld dann setzen.
        unavailable: None,
    })
}

gen_service_impl! {
    struct ShiftplanViewServiceImpl: service::shiftplan::ShiftplanViewService = ShiftplanViewServiceDeps {
        SlotService: service::slot::SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SpecialDayService: service::special_days::SpecialDayService<Context = Self::Context> = special_day_service,
        ShiftplanService: service::shiftplan_catalog::ShiftplanService<Context = Self::Context, Transaction = Self::Transaction> = shiftplan_service,
        PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: ShiftplanViewServiceDeps> ShiftplanViewService for ShiftplanViewServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_shiftplan_week(
        &self,
        shiftplan_id: uuid::Uuid,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Test if the date is valid
        time::Date::from_iso_week_date(year as i32, week, time::Weekday::Thursday)?;

        // Get all required data including special days
        let special_days = self
            .special_day_service
            .get_by_week(year, week, context.clone())
            .await?;

        let slots = self
            .slot_service
            .get_slots_for_week(year, week, shiftplan_id, context.clone(), Some(tx.clone()))
            .await?;

        let bookings = self
            .booking_service
            .get_for_week(week, year, context.clone(), Some(tx.clone()))
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), Some(tx.clone()))
            .await?;

        let user_assignments = if self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok()
        {
            Some(
                self.sales_person_service
                    .get_all_user_assignments(Authentication::Full, Some(tx.clone()))
                    .await?,
            )
        } else {
            None
        };

        // Build days
        let mut days = Vec::new();
        for day_of_week in [
            DayOfWeek::Monday,
            DayOfWeek::Tuesday,
            DayOfWeek::Wednesday,
            DayOfWeek::Thursday,
            DayOfWeek::Friday,
            DayOfWeek::Saturday,
            DayOfWeek::Sunday,
        ] {
            days.push(build_shiftplan_day(
                day_of_week,
                &slots,
                &bookings,
                &sales_persons,
                &special_days,
                user_assignments.as_ref(),
            )?);
        }

        self.transaction_dao.commit(tx).await?;

        Ok(ShiftplanWeek {
            year,
            calendar_week: week,
            days,
        })
    }

    async fn get_shiftplan_day(
        &self,
        year: u32,
        week: u8,
        day_of_week: DayOfWeek,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanDayAggregate, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Validate the date
        time::Date::from_iso_week_date(year as i32, week, time::Weekday::Thursday)?;

        // Load shared data once
        let special_days = self
            .special_day_service
            .get_by_week(year, week, context.clone())
            .await?;

        let bookings = self
            .booking_service
            .get_for_week(week, year, context.clone(), Some(tx.clone()))
            .await?;

        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), Some(tx.clone()))
            .await?;

        let user_assignments = if self
            .permission_service
            .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone())
            .await
            .is_ok()
        {
            Some(
                self.sales_person_service
                    .get_all_user_assignments(Authentication::Full, Some(tx.clone()))
                    .await?,
            )
        } else {
            None
        };

        // Load all shiftplans
        let shiftplans = self
            .shiftplan_service
            .get_all(context.clone(), Some(tx.clone()))
            .await?;

        // Build day view for each plan
        let mut plans = Vec::new();
        for shiftplan in shiftplans.iter() {
            let slots = self
                .slot_service
                .get_slots_for_week(year, week, shiftplan.id, context.clone(), Some(tx.clone()))
                .await?;

            let day = build_shiftplan_day(
                day_of_week,
                &slots,
                &bookings,
                &sales_persons,
                &special_days,
                user_assignments.as_ref(),
            )?;

            plans.push(PlanDayView {
                shiftplan: shiftplan.clone(),
                slots: day.slots,
            });
        }

        self.transaction_dao.commit(tx).await?;

        Ok(ShiftplanDayAggregate {
            year,
            calendar_week: week,
            day_of_week,
            plans,
        })
    }
}
