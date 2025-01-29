use std::sync::Arc;

use async_trait::async_trait;
use service::{
    booking::BookingService,
    permission::Authentication,
    sales_person::SalesPersonService,
    ServiceError,
    shiftplan::{ShiftplanService, ShiftplanWeek, ShiftplanDay, ShiftplanSlot, ShiftplanBooking},
    slot::{DayOfWeek, SlotService},
};
use dao::TransactionDao;

pub trait ShiftplanServiceDeps {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;
    type SlotService: SlotService<Context = Self::Context>;
    type BookingService: BookingService<Context = Self::Context>;
    type SalesPersonService: SalesPersonService<Context = Self::Context>;
    type TransactionDao: TransactionDao<Transaction = Self::Transaction>;
}

pub struct ShiftplanServiceImpl<Deps: ShiftplanServiceDeps> {
    pub slot_service: Arc<Deps::SlotService>,
    pub booking_service: Arc<Deps::BookingService>,
    pub sales_person_service: Arc<Deps::SalesPersonService>,
    pub transaction_dao: Arc<Deps::TransactionDao>,
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
        let slots = self.slot_service.get_slots(context.clone(), Some(tx.clone())).await?;
        let bookings = self.booking_service.find_by_week(year, week, context.clone(), Some(tx.clone())).await?;
        let sales_persons = self.sales_person_service.get_all(context.clone(), Some(tx.clone())).await?;

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
                            .ok_or_else(|| {
                                ServiceError::EntityNotFound(booking.sales_person_id)
                            })?
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
