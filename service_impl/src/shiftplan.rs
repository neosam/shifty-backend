use std::sync::Arc;

use async_trait::async_trait;
use service::{
    booking::BookingService,
    permission::Authentication,
    sales_person::SalesPersonService,
    shiftplan::{ShiftplanBooking, ShiftplanDay, ShiftplanService, ShiftplanSlot, ShiftplanWeek},
    slot::{DayOfWeek, SlotService},
    ServiceError,
};

use crate::gen_service_impl;

gen_service_impl! {
    struct ShiftplanServiceImpl: service::shiftplan::ShiftplanService = ShiftplanServiceDeps {
        SlotService: service::slot::SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: ShiftplanServiceDeps> ShiftplanService for ShiftplanServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_shiftplan_week(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<ShiftplanWeek, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Get all required data
        let slots = self
            .slot_service
            .get_slots(context.clone(), Some(tx.clone()))
            .await?;
        let bookings = self
            .booking_service
            .get_for_week(week, year, context.clone(), Some(tx.clone()))
            .await?;
        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), Some(tx.clone()))
            .await?;

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
            // Filter slots for this day
            let mut day_slots = Vec::new();
            for slot in slots.iter() {
                if slot.day_of_week != day_of_week {
                    continue;
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

                        Ok(ShiftplanBooking {
                            booking: booking.clone(),
                            sales_person,
                        })
                    })
                    .collect::<Result<Vec<_>, ServiceError>>()?;

                day_slots.push(ShiftplanSlot {
                    slot: slot.clone(),
                    bookings: slot_bookings,
                });
            }

            // Sort slots by time
            day_slots.sort_by(|a, b| a.slot.from.cmp(&b.slot.from));

            days.push(ShiftplanDay {
                day_of_week,
                slots: day_slots,
            });
        }

        self.transaction_dao.commit(tx).await?;

        Ok(ShiftplanWeek {
            year,
            calendar_week: week,
            days,
        })
    }
}
