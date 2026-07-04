use std::{collections::BTreeMap, sync::Arc};

use async_trait::async_trait;
use service::{
    block::{Block, BlockService},
    booking::{Booking, BookingService},
    clock::ClockService,
    config::ConfigService,
    ical::IcalService,
    permission::Authentication,
    sales_person::SalesPersonService,
    shiftplan::ShiftplanViewService,
    slot::{Slot, SlotService},
    special_days::{SpecialDay, SpecialDayService, SpecialDayType},
    toggle::ToggleService,
    ServiceError,
};
use shifty_utils::{DayOfWeek, ShiftyWeek};
use tracing::instrument;
use uuid::Uuid;

use crate::gen_service_impl;
use crate::shortday_gate;
use dao::TransactionDao; // import your transaction trait
use time::Time;

// Automatically generate the `BlockServiceDeps` trait and the `BlockServiceImpl` struct.
//
// This macro pattern follows your existing approach. It wires up dependencies
// (e.g., `BookingService`, `SlotService`, `SalesPersonService`, etc.) for the service.
/// Ergebnis des pro-Slot-Clips für Chain A' (Block-Service).
///
/// - `Keep(slot)` — Slot bleibt (roh oder geclippt).
/// - `Drop` — Slot fällt ganz weg (Cutoff ≤ `slot.from`; D-04 Zeile 3).
enum ClipOutcome {
    Keep(Slot),
    Drop,
}

/// Wendet den ShortDay-Cutoff pro Wochentag + Stichtag-Gate auf einen Slot an.
///
/// Wird von beiden Aggregat-Methoden (`get_blocks_for_sales_person_week` und
/// `get_unsufficiently_booked_blocks`) genutzt, damit die Merge-Loops jeweils
/// mit geclippten Slots arbeiten. Keine DB-Zugriffe — reine In-Memory-Kombi.
fn clip_slot_for_week(
    slot: &Slot,
    special_days: &[SpecialDay],
    year: u32,
    week: u8,
    active_from: Option<time::Date>,
) -> ClipOutcome {
    // Stichtag-Gate: greift das Gate für diesen Wochentag überhaupt?
    let gate_active = shortday_gate::resolve_active_from_for_week(
        year,
        week,
        slot.day_of_week,
        active_from,
    );
    if !gate_active {
        return ClipOutcome::Keep(slot.clone());
    }

    // Cutoff aus SpecialDay (nur ShortDay mit `time_of_day`) für diesen dow.
    let cutoff = special_days.iter().find_map(|sd| {
        if sd.day_of_week == slot.day_of_week
            && sd.day_type == SpecialDayType::ShortDay
        {
            sd.time_of_day
        } else {
            None
        }
    });

    let Some(cutoff) = cutoff else {
        return ClipOutcome::Keep(slot.clone());
    };

    match slot.clip_to(cutoff) {
        Some(clipped) => ClipOutcome::Keep(clipped),
        None => ClipOutcome::Drop,
    }
}

gen_service_impl! {
    struct BlockServiceImpl: BlockService = BlockServiceDeps {
        BookingService: BookingService<Context = Self::Context, Transaction = Self::Transaction> = booking_service,
        SlotService: SlotService<Context = Self::Context, Transaction = Self::Transaction> = slot_service,
        SalesPersonService: SalesPersonService<Context = Self::Context, Transaction = Self::Transaction> = sales_person_service,
        ShiftplanViewService: ShiftplanViewService<Context = Self::Context, Transaction = Self::Transaction> = shiftplan_service,
        IcalService: IcalService = ical_service,
        ConfigService: ConfigService = config_service,
        ClockService: ClockService = clock_service,
        // Phase 51 (D-51-06 Chain A' + D-51-07 Stichtag-Gate): pro-Slot-Clip
        // vor Merge braucht ShortDay-Lookup pro Woche + Toggle-Wert.
        SpecialDayService: SpecialDayService<Context = Self::Context> = special_day_service,
        ToggleService: ToggleService<Context = Self::Context, Transaction = Self::Transaction> = toggle_service,
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
            .filter(|b| b.sales_person_id == sales_person_id)
            .cloned()
            .collect();

        dbg!(&bookings_for_person);

        // Phase 51 (D-51-06 Chain A' + D-51-07): ShortDay-Cutoff pro Wochentag +
        // Stichtag-Gate. Prefetch beides einmal pro Method-Call — der Cutoff hängt
        // nur an `day_of_week`, das Gate am ISO-Datum aus (year, week, dow).
        // `Unauthorized` mapped auf None (Legacy off), analog reporting.rs.
        let special_days = self
            .special_day_service
            .get_by_week(year, week, context.clone())
            .await?;
        let toggle_value = match self
            .toggle_service
            .get_toggle_value(shortday_gate::TOGGLE_NAME, context.clone(), None)
            .await
        {
            Ok(v) => v,
            Err(ServiceError::Unauthorized) => None,
            Err(e) => return Err(e),
        };
        let active_from = shortday_gate::parse_active_from(toggle_value.as_deref());

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
                // Chain A' — pro-Slot-Clip vor dem Merge-Loop. Reihenfolge kritisch
                // (Research §Risks 4): erst clippen, dann `slot.from == to`-Merge.
                let slot = match clip_slot_for_week(
                    &slot,
                    &special_days,
                    year,
                    week,
                    active_from,
                ) {
                    ClipOutcome::Keep(s) => s,
                    ClipOutcome::Drop => continue,
                };
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
            items.sort_by_key(|(_, slot)| slot.from);
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
                            sales_person: sales_person.clone().into(),
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
                    sales_person: sales_person.clone().into(),
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
            let week_blocks = if sales_person_id != Uuid::nil() {
                self.get_blocks_for_sales_person_week(
                    sales_person_id,
                    year as u32,
                    week,
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?
            } else {
                self.get_unsufficiently_booked_blocks(
                    year as u32,
                    week,
                    Authentication::Full,
                    tx.clone().into(),
                )
                .await?
            };
            blocks.extend_from_slice(&week_blocks);
            now += time::Duration::weeks(1);
        }
        let ical = self.ical_service.convert_blocks_to_ical_string(
            blocks.into(),
            self.config_service.get_config().await?.ical_label.clone(),
            self.config_service.get_config().await?.timezone.clone(),
        )?;

        self.transaction_dao.commit(tx).await?;
        Ok(ical)
    }

    async fn get_unsufficiently_booked_blocks(
        &self,
        year: u32,
        week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Block]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Get all non-planning slots and bookings for the week
        let all_slots = self
            .slot_service
            .get_slots_for_week_all_plans(year, week, context.clone(), Some(tx.clone()))
            .await?;
        let all_bookings = self
            .booking_service
            .get_for_week(week, year, context.clone(), Some(tx.clone()))
            .await?;

        // Phase 51 (D-51-06 Chain A' + D-51-07): identisches Prefetch-Muster wie in
        // `get_blocks_for_sales_person_week`. Zwei Method-Calls, aber Duplizierung
        // ist akzeptabel — bei einem dritten Konsumenten würde der Helper zu einer
        // gemeinsamen Method gehoben.
        let special_days = self
            .special_day_service
            .get_by_week(year, week, context.clone())
            .await?;
        let toggle_value = match self
            .toggle_service
            .get_toggle_value(shortday_gate::TOGGLE_NAME, context.clone(), None)
            .await
        {
            Ok(v) => v,
            Err(ServiceError::Unauthorized) => None,
            Err(e) => return Err(e),
        };
        let active_from = shortday_gate::parse_active_from(toggle_value.as_deref());

        // Group slots by day and sort by time. Slots werden pro-Slot geclippt
        // VOR dem Filter+Merge, damit die `slot.from == to`-Consecutive-Detection
        // im Merge-Loop mit den effektiven Zeiten arbeitet. Owned `Slot` (statt
        // `&Slot`), weil `clip_to` einen neuen Slot produziert.
        let mut day_map: BTreeMap<DayOfWeek, Vec<Slot>> = BTreeMap::new();
        for slot in all_slots.iter() {
            let clipped = match clip_slot_for_week(
                slot,
                &special_days,
                year,
                week,
                active_from,
            ) {
                ClipOutcome::Keep(s) => s,
                ClipOutcome::Drop => continue,
            };
            day_map.entry(clipped.day_of_week).or_default().push(clipped);
        }

        // For each day, sort slots by time and merge consecutive ones
        let mut insufficient_blocks = Vec::new();

        for (day_of_week, mut slots) in day_map {
            slots.sort_by_key(|a| a.from);

            // Filter for slots with insufficient bookings
            let slots: Vec<_> = slots
                .into_iter()
                .filter(|slot| {
                    let bookings_count = all_bookings
                        .iter()
                        .filter(|b| b.slot_id == slot.id)
                        .count();
                    bookings_count < slot.min_resources as usize
                })
                .collect();

            let mut current_slots = Vec::new();
            let mut block_from: Option<Time> = None;
            let mut block_to: Option<Time> = None;

            for slot in slots {
                match (block_from, block_to) {
                    // Start new block
                    (None, None) => {
                        block_from = Some(slot.from);
                        block_to = Some(slot.to);
                        current_slots.push(slot.clone());
                    }
                    // Extend current block if times match
                    (Some(_), Some(to)) if slot.from == to => {
                        block_to = Some(slot.to);
                        current_slots.push(slot.clone());
                    }
                    // Finish current block and start new one
                    _ => {
                        // Check if current block has enough bookings
                        let total_min_resources: u8 =
                            current_slots.iter().map(|s| s.min_resources).sum();
                        let block_bookings: Vec<Booking> = all_bookings
                            .iter()
                            .filter(|b| current_slots.iter().any(|cs| cs.id == b.slot_id))
                            .cloned()
                            .collect();

                        if block_bookings.len() < total_min_resources as usize {
                            insufficient_blocks.push(Block {
                                year,
                                week,
                                sales_person: None,
                                day_of_week,
                                from: block_from.unwrap(),
                                to: block_to.unwrap(),
                                bookings: block_bookings.into(),
                                slots: current_slots.clone().into(),
                            });
                        }

                        // Start new block
                        block_from = Some(slot.from);
                        block_to = Some(slot.to);
                        current_slots = vec![slot.clone()];
                    }
                }
            }

            // Handle last block
            if !current_slots.is_empty() {
                let total_min_resources: u8 = current_slots.iter().map(|s| s.min_resources).sum();
                let block_bookings: Vec<Booking> = all_bookings
                    .iter()
                    .filter(|b| current_slots.iter().any(|cs| cs.id == b.slot_id))
                    .cloned()
                    .collect();

                if block_bookings.len() < total_min_resources as usize {
                    insufficient_blocks.push(Block {
                        year,
                        week,
                        sales_person: None,
                        day_of_week,
                        from: block_from.unwrap(),
                        to: block_to.unwrap(),
                        bookings: block_bookings.into(),
                        slots: current_slots.into(),
                    });
                }
            }
        }

        self.transaction_dao.commit(tx).await?;
        Ok(insufficient_blocks.into())
    }

    #[instrument(skip(self))]
    async fn get_blocks_for_current_user(
        &self,
        from: ShiftyWeek,
        until: ShiftyWeek,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Block]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;

        // Get the sales person for the current user
        let sales_person = match self
            .sales_person_service
            .get_sales_person_current_user(context.clone(), Some(tx.clone()))
            .await?
        {
            Some(sp) => sp,
            None => {
                self.transaction_dao.commit(tx).await?;
                return Ok(Arc::new([]));
            }
        };

        // Collect blocks for all weeks in the range
        let mut all_blocks = Vec::new();
        for week in from.iter_until(&until) {
            let week_blocks = self
                .get_blocks_for_sales_person_week(
                    sales_person.id,
                    week.year,
                    week.week,
                    context.clone(),
                    Some(tx.clone()),
                )
                .await?;
            all_blocks.extend_from_slice(&week_blocks);
        }

        // Sort blocks by date (year, week, day_of_week) and then by start time
        all_blocks.sort_by(|a, b| {
            (a.year, a.week, a.day_of_week.to_number(), a.from)
                .cmp(&(b.year, b.week, b.day_of_week.to_number(), b.from))
        });

        self.transaction_dao.commit(tx).await?;
        Ok(Arc::from(all_blocks))
    }
}
