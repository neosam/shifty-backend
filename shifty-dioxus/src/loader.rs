use std::rc::Rc;

use crate::{
    api,
    error::ShiftyError,
    state::{
        shiftplan::{Booking, SalesPerson},
        Config, Shiftplan, Slot,
    },
};

pub async fn load_sales_persons(config: Config) -> Result<Rc<[SalesPerson]>, ShiftyError> {
    let sales_person_tos = api::get_sales_persons(config).await?;
    let sales_persons: Rc<[SalesPerson]> = sales_person_tos.iter().map(SalesPerson::from).collect();

    Ok(sales_persons)
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
        .map(|booking: Booking| Booking {
            label: sales_persons
                .iter()
                .find(|sales_person| sales_person.id == booking.sales_person_id)
                .map(|sales_person| sales_person.name.clone())
                .unwrap_or("".into()),
            ..booking
        })
        .collect();
    Ok(bookings)
}

pub async fn load_slots(
    config: Config,
    bookings: Rc<[Booking]>,
) -> Result<Rc<[Slot]>, ShiftyError> {
    let slot_tos = api::get_slots(config).await?;
    let slots: Rc<[Slot]> = slot_tos
        .iter()
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
    let sales_persons = load_sales_persons(config.clone()).await?;
    let bookings = load_bookings(config.clone(), sales_persons.clone(), week, year).await?;
    let slots = load_slots(config.clone(), bookings.clone()).await?;

    Ok(Shiftplan { week, year, slots })
}

pub async fn load_current_sales_person(config: Config) -> Result<Option<SalesPerson>, ShiftyError> {
    let sales_person_to = api::get_current_sales_person(config).await?;
    let sales_person = sales_person_to.as_ref().map(SalesPerson::from);
    Ok(sales_person)
}
