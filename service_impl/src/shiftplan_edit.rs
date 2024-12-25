use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    booking::BookingService,
    permission::Authentication,
    shiftplan_edit::ShiftplanEditService,
    slot::{Slot, SlotService},
    PermissionService, ServiceError,
};
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct ShiftplanEditServiceImpl: ShiftplanEditService = ShiftplanEditServiceDeps {
        PermissionService: service::PermissionService<Context = Self::Context> = permission_service,
        SlotService: service::slot::SlotService<Transaction = Self::Transaction> = slot_service,
        BookingService: service::booking::BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
        TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait]
impl<Deps: ShiftplanEditServiceDeps> ShiftplanEditService for ShiftplanEditServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn modify_slot(
        &self,
        slot: &Slot,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Slot, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission("shiftplan.edit", context)
            .await?;

        let mut stored_slot = self
            .slot_service
            .get_slot(&slot.id, Authentication::Full, tx.clone().into())
            .await?;

        if stored_slot.version != slot.version {
            return Err(ServiceError::EntityConflicts(
                slot.id,
                stored_slot.version,
                slot.version,
            ));
        }

        let new_slot_valid_from =
            time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
        let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);
        let bookings = self
            .booking_service
            .get_for_slot_id_since(
                slot.id,
                change_year,
                change_week,
                Authentication::Full,
                Some(tx.clone()),
            )
            .await?;
        let original_valid_to = stored_slot.valid_to;

        stored_slot.valid_to = Some(old_slot_valid_to);

        if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
            self.slot_service
                .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
                .await?;
        } else {
            self.slot_service
                .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
                .await?;
        }

        let mut new_slot = stored_slot;
        new_slot.valid_from = new_slot_valid_from;
        new_slot.valid_to = original_valid_to;
        new_slot.id = Uuid::nil();
        new_slot.version = Uuid::nil();
        new_slot.min_resources = slot.min_resources;
        new_slot.from = slot.from;
        new_slot.to = slot.to;

        dbg!(&new_slot);

        let new_slot = self
            .slot_service
            .create_slot(&new_slot, Authentication::Full, tx.clone().into())
            .await?;

        dbg!(&new_slot);

        for booking in bookings.iter() {
            self.booking_service
                .delete(booking.id, Authentication::Full, tx.clone().into())
                .await?;

            let mut new_booking = booking.clone();
            new_booking.id = Uuid::nil();
            new_booking.version = Uuid::nil();
            new_booking.slot_id = new_slot.id;
            new_booking.year = booking.year;
            new_booking.calendar_week = booking.calendar_week;
            new_booking.created = None;

            self.booking_service
                .create(&new_booking, Authentication::Full, tx.clone().into())
                .await?;
        }

        self.transaction_dao.commit(tx).await?;
        Ok(new_slot)
    }

    async fn remove_slot(
        &self,
        slot_id: Uuid,
        change_year: u32,
        change_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission("shiftplan.edit", context)
            .await?;

        let mut stored_slot = self
            .slot_service
            .get_slot(&slot_id, Authentication::Full, tx.clone().into())
            .await?;

        let new_slot_valid_from =
            time::Date::from_iso_week_date(change_year as i32, change_week, time::Weekday::Monday)?;
        let old_slot_valid_to = new_slot_valid_from - time::Duration::days(1);
        let bookings = self
            .booking_service
            .get_for_slot_id_since(
                slot_id,
                change_year,
                change_week,
                Authentication::Full,
                None,
            )
            .await?;

        stored_slot.valid_to = Some(old_slot_valid_to);

        if stored_slot.valid_to.unwrap() < stored_slot.valid_from {
            self.slot_service
                .delete_slot(&stored_slot.id, Authentication::Full, tx.clone().into())
                .await?;
        } else {
            self.slot_service
                .update_slot(&stored_slot, Authentication::Full, tx.clone().into())
                .await?;
        }

        for booking in bookings.iter() {
            self.booking_service
                .delete(booking.id, Authentication::Full, Some(tx.clone()))
                .await?;
        }

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
