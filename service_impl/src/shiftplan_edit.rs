use std::sync::Arc;

use async_trait::async_trait;
use dao::TransactionDao;
use service::{
    booking::BookingService,
    carryover::{Carryover, CarryoverService},
    employee_work_details::EmployeeWorkDetailsService,
    extra_hours::{ExtraHours, ExtraHoursCategory, ExtraHoursService},
    permission::Authentication,
    reporting::ReportingService,
    sales_person::SalesPersonService,
    sales_person_unavailable::{SalesPersonUnavailable, SalesPersonUnavailableService},
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
        CarryoverService: service::carryover::CarryoverService<Context = Self::Context, Transaction = Self::Transaction> = carryover_service,
        ReportingService: service::reporting::ReportingService<Context = Self::Context, Transaction = Self::Transaction> = reporting_service,
        SalesPersonService: service::sales_person::SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        SalesPersonUnavailableService: SalesPersonUnavailableService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_unavailable_service,
        EmployeeWorkDetailsService: service::employee_work_details::EmployeeWorkDetailsService<Context = Self::Context, Transaction = Self::Transaction> = employee_work_details_service,
        ExtraHoursService: ExtraHoursService<Context = Self::Context, Transaction = Self::Transaction> = extra_hours_service,
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

    async fn update_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        let until_week = time::util::weeks_in_year(year as i32);

        let employee_report = self
            .reporting_service
            .get_report_for_employee(
                &sales_person_id,
                year,
                until_week,
                Authentication::Full,
                tx.clone().into(),
            )
            .await?;

        let new_carryover_hours = employee_report.balance_hours;
        let new_vacation_entitlement =
            (employee_report.vacation_entitlement - employee_report.vacation_days).floor() as i32;

        let now = time::OffsetDateTime::now_utc();
        let created = time::PrimitiveDateTime::new(now.date(), now.time());

        let new_carryover = Carryover {
            sales_person_id,
            year,
            carryover_hours: new_carryover_hours,
            vacation: new_vacation_entitlement,
            created,
            deleted: None,
            version: uuid::Uuid::nil(),
        };

        self.carryover_service
            .set_carryover(&new_carryover, Authentication::Full, tx.clone().into())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn update_carryover_all_employees(
        &self,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        // Acquire (or reuse) a transaction
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Make sure the caller is allowed to edit shift plans
        self.permission_service
            .check_permission("shiftplan.edit", context.clone())
            .await?;

        // Retrieve all sales persons
        let sales_persons = self
            .sales_person_service
            .get_all(context.clone(), tx.clone().into())
            .await?;

        // Call update_carryover for each sales person
        for sp in sales_persons.iter() {
            // Pass the same transaction along so everything is done in a single transaction.
            // Alternatively, if you want each carryover update to be committed separately,
            // you could pass None here. But typically we want one big transaction.
            self.update_carryover(sp.id, year, context.clone(), Some(tx.clone()))
                .await?;
        }

        // Commit everything at the end
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn add_vacation(
        &self,
        sales_person_id: Uuid,
        from: time::Date,
        to: time::Date,
        description: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        // Permission check is done by the service calls

        let (from_year, from_week, from_weekday) = from.to_iso_week_date();

        let employee_work_details = self
            .employee_work_details_service
            .find_for_week(
                sales_person_id,
                from_week,
                from_year as u32,
                context.clone(),
                tx.clone().into(),
            )
            .await?;

        let mut date = from;
        let mut current_year = from_year as u32;
        let mut current_week = from_week;
        let mut current_weekday = from_weekday;
        let mut vacation_days_for_week = 0;
        while date <= to {
            if employee_work_details.has_day_of_week(date.weekday()) {
                vacation_days_for_week += 1;
            }

            let (year, week, weekday) = date.to_iso_week_date();
            if (current_year, current_week) != (year as u32, week) {
                let amount = (employee_work_details.hours_per_day()
                    * vacation_days_for_week as f32)
                    .min(employee_work_details.expected_hours);
                let date = time::Date::from_iso_week_date(
                    current_year as i32,
                    current_week,
                    current_weekday,
                )?;
                let date_time = time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT);

                let vacation = ExtraHours {
                    id: Uuid::nil(),
                    sales_person_id,
                    amount,
                    category: ExtraHoursCategory::Vacation,
                    description: description.clone(),
                    date_time,
                    created: None,
                    deleted: None,
                    version: Uuid::nil(),
                };

                let _ = self
                    .extra_hours_service
                    .create(&vacation, context.clone(), tx.clone().into())
                    .await?;

                current_year = year as u32;
                current_week = week;
                current_weekday = weekday;
                vacation_days_for_week = 0;
            }

            let employee_unavailable = SalesPersonUnavailable {
                sales_person_id,
                id: Uuid::nil(),
                year: year as u32,
                calendar_week: week,
                day_of_week: weekday.into(),
                created: None,
                deleted: None,
                version: Uuid::nil(),
            };
            match self
                .sales_person_unavailable_service
                .create(&employee_unavailable, context.clone(), tx.clone().into())
                .await
            {
                // Ignore if the day is already blocked.
                Err(ServiceError::EntityAlreadyExists(_)) => Ok(()),
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }?;
            date += time::Duration::days(1);
        }
        let amount = (employee_work_details.hours_per_day() * vacation_days_for_week as f32)
            .min(employee_work_details.expected_hours);
        let date = time::Date::from_iso_week_date(
            current_year as i32,
            current_week,
            current_weekday,
        )?;
        let date_time = time::PrimitiveDateTime::new(date, time::Time::MIDNIGHT);

        let vacation = ExtraHours {
            id: Uuid::nil(),
            sales_person_id,
            amount,
            category: ExtraHoursCategory::Vacation,
            description: description.clone(),
            date_time,
            created: None,
            deleted: None,
            version: Uuid::nil(),
        };

        let _ = self
            .extra_hours_service
            .create(&vacation, context.clone(), tx.clone().into())
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(())
    }
}
