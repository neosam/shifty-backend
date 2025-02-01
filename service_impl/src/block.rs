use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use service::{
    block::{Block, BlockService},
    booking::{Booking, BookingService},
    clock::ClockService,
    ical::IcalService,
    permission::Authentication,
    sales_person::SalesPersonService,
    slot::{DayOfWeek, Slot, SlotService},
    ServiceError,
};
use tracing::instrument;
use uuid::Uuid;

use crate::gen_service_impl;
use dao::TransactionDao; // import your transaction trait
use time::Time;

// Automatically generate the `BlockServiceDeps` trait and the `BlockServiceImpl` struct.
//
// This macro pattern follows your existing approach. It wires up dependencies
// (e.g., `BookingService`, `SlotService`, `SalesPersonService`, etc.) for the service.
gen_service_impl! {
    struct BlockServiceImpl: BlockService = BlockServiceDeps {
        BookingService: BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SlotService: SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        IcalService: IcalService = ical_service,
        ClockService: ClockService = clock_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: BlockServiceDeps> BlockService for BlockServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    #[instrument(skip(self))]
    async fn get_blocks_for_sales_person_week(
        &self,
        sales_person_id: Uuid,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Block]>, ServiceError> {
        // Acquire (or create) a transaction
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // (Optional) Check permissions here, if needed, e.g.:
        // self.permission_service.check_permission("view_blocks", context.clone()).await?;

        // Fetch the SalesPerson to embed in the returned `Block`.
        dbg!(year, week);
        dbg!("Getting sales person");
        let sales_person = Arc::new(
            self.sales_person_service
                .get(sales_person_id, context.clone(), Some(tx.clone()))
                .await?,
        );
        dbg!(&sales_person);

        // Get all bookings for the specified year & week. Then filter by this SalesPerson.
        let all_bookings = self
            .booking_service
            .get_for_week(week, year, context.clone(), Some(tx.clone()))
            .await?;
        let bookings_for_person: Vec<_> = all_bookings
            .iter()
            .cloned()
            .filter(|b| b.sales_person_id == sales_person_id)
            .collect();

        dbg!(&bookings_for_person);

        // Collect each booking's associated slot. We'll later group by day-of-week.
        // (You could optimize this by building a single query or caching, but this
        // example keeps it straightforward.)
        let mut booking_slot_pairs = Vec::new();
        for booking in &bookings_for_person {
            if let Ok(slot) = self
                .slot_service
                .get_slot(&booking.slot_id, context.clone(), Some(tx.clone()))
                .await
            {
                booking_slot_pairs.push((booking.clone(), slot));
            }
        }

        // Commit the transaction (will only actually commit if this is the last Arc reference).
        self.transaction_dao.commit(tx).await?;

        // Group by DayOfWeek in a BTreeMap so days are sorted Monday..Sunday.
        let mut day_map: BTreeMap<DayOfWeek, Vec<(Booking, Slot)>> = BTreeMap::new();
        for (b, s) in booking_slot_pairs {
            day_map.entry(s.day_of_week).or_default().push((b, s));
        }

        // Sort each day's entries by their slot.from time in ascending order.
        for (_, items) in day_map.iter_mut() {
            items.sort_by(|(_, slot_a), (_, slot_b)| slot_a.from.cmp(&slot_b.from));
        }

        // Build blocks by combining consecutive bookings if the next slot’s start
        // equals the current slot’s end.
        let mut all_blocks = Vec::new();
        for (day_of_week, items) in day_map {
            let mut current_bookings = Vec::new();
            let mut current_slots = Vec::new();
            let mut block_from: Option<Time> = None;
            let mut block_to: Option<Time> = None;

            for (booking, slot) in items {
                match (block_from, block_to) {
                    // If we haven't started a block yet, begin one now.
                    (None, None) => {
                        block_from = Some(slot.from);
                        block_to = Some(slot.to);
                        current_bookings.push(booking);
                        current_slots.push(slot);
                    }
                    // If the new slot's 'from' == current block's 'to', extend the block.
                    (Some(_from), Some(to)) if slot.from == to => {
                        current_bookings.push(booking);
                        current_slots.push(slot.clone());
                        // Extend the 'to' time if needed
                        if slot.to > to {
                            block_to = Some(slot.to);
                        }
                    }
                    // Otherwise, finish the current block and start a new one.
                    _ => {
                        // Finish the existing block
                        let finished_block = Block {
                            year,
                            week,
                            sales_person: sales_person.clone(),
                            day_of_week,
                            from: block_from.unwrap(),
                            to: block_to.unwrap(),
                            bookings: Arc::from(current_bookings),
                            slots: Arc::from(current_slots),
                        };
                        all_blocks.push(finished_block);

                        // Start a new block with this booking/slot
                        block_from = Some(slot.from);
                        block_to = Some(slot.to);
                        current_bookings = vec![booking];
                        current_slots = vec![slot];
                    }
                }
            }

            // If there's a partially built block leftover, push it.
            if !current_bookings.is_empty() {
                let final_block = Block {
                    year,
                    week,
                    sales_person: sales_person.clone(),
                    day_of_week,
                    from: block_from.unwrap(),
                    to: block_to.unwrap(),
                    bookings: Arc::from(current_bookings),
                    slots: Arc::from(current_slots),
                };
                all_blocks.push(final_block);
            }
        }

        Ok(Arc::from(all_blocks))
    }

    async fn get_blocks_for_next_weeks_as_ical(
        &self,
        sales_person_id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<str>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let mut now = self.clock_service.date_now();
        now -= time::Duration::weeks(2);

        let mut blocks = vec![];

        for _ in 0..12 {
            let (year, week, _) = now.to_iso_week_date();
            let week_blocks = self
                .get_blocks_for_sales_person_week(
                    sales_person_id,
                    year as u32,
                    week,
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?;
            blocks.extend_from_slice(&week_blocks);
            now += time::Duration::weeks(1);
        }
        let ical = self
            .ical_service
            .convert_blocks_to_ical_string(blocks.into(), "Schicht".into())?;

        self.transaction_dao.commit(tx).await?;
        Ok(ical)
    }
}
