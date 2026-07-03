use dioxus::prelude::*;
use futures_util::StreamExt;
use uuid::Uuid;

use crate::{
    api,
    error::ShiftyError,
    loader,
    state::slot_edit::{SlotEdit, SlotEditItem, SlotEditType},
};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static SLOT_EDIT_STORE: GlobalSignal<SlotEdit> = Signal::global(SlotEdit::new_edit);
pub static SHIFTPLAN_REFRESH: GlobalSignal<u64> = Signal::global(|| 0);
pub enum SlotEditAction {
    NewSlot(u32, u8, Option<Uuid>),
    UpdateSlot(SlotEditItem),
    SaveSlot,
    Cancel,
    DeleteSlot(Uuid, u32, u8),
    LoadSlot(Uuid, u32, u8, u8),
    SetSingleWeek(bool),
}

pub fn trigger_shiftplan_refresh() {
    *SHIFTPLAN_REFRESH.write() += 1;
}

pub fn new_slot_edit(year: u32, week: u8, shiftplan_id: Option<Uuid>) -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.slot_edit_type = SlotEditType::New;
    let mut slot = SlotEditItem::new_valid_from(year, week);
    slot.shiftplan_id = shiftplan_id;
    store.slot = slot.into();
    store.year = year;
    store.week = week;
    store.visible = true;
    store.has_errors = false;
    store.current_paid_count = 0;
    store.single_week = false;
    Ok(())
}

pub fn update_slot_edit(slot_edit: SlotEditItem) -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.slot = slot_edit.into();
    Ok(())
}

/// Which loader `save_slot_edit` should invoke after taking a snapshot.
///
/// Derived purely from `SlotEdit.slot_edit_type` × `SlotEdit.single_week` — no
/// store access needed at effect time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveMode {
    /// Edit + `single_week=false` → `loader::save_slot` (multi-week / "ab dieser Woche").
    AllFromWeek,
    /// Edit + `single_week=true` → `loader::save_slot_single_week`.
    SingleWeek,
    /// New → `loader::create_slot`.
    Create,
}

/// Owned snapshot of every field of `SLOT_EDIT_STORE` that `save_slot_edit` needs
/// across `.await` points. Built before the first `.await`, then all loader calls
/// operate on `self` only — no Signal guard is held while the network is in-flight.
#[derive(Clone, PartialEq)]
pub(crate) struct SaveSlotEditSnapshot {
    pub(crate) slot_clone: std::rc::Rc<SlotEditItem>,
    pub(crate) year: u32,
    pub(crate) week: u8,
    pub(crate) mode: SaveMode,
}

/// Outcome of the loader call in `save_slot_edit`, applied after the `.await` returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveOutcome {
    /// Edit branch completed (either single-week or multi-week loader).
    /// Post-effect: `visible=false`, refresh shiftplan.
    EditSaved,
    /// New branch: `create_slot` returned `true`.
    /// Post-effect: `visible=false`, `has_errors=false`, refresh shiftplan.
    CreateSucceeded,
    /// New branch: `create_slot` returned `false` (validation error server-side).
    /// Post-effect: `has_errors=true`, keep modal open (no visibility change, no refresh).
    CreateFailed,
}

/// Pure snapshot extraction — reads the store into an owned struct so the caller
/// can drop the Signal guard before any `.await`.
pub(crate) fn snapshot_for_save(store: &SlotEdit) -> SaveSlotEditSnapshot {
    let mode = match store.slot_edit_type {
        SlotEditType::New => SaveMode::Create,
        SlotEditType::Edit if store.single_week => SaveMode::SingleWeek,
        SlotEditType::Edit => SaveMode::AllFromWeek,
    };
    SaveSlotEditSnapshot {
        slot_clone: store.slot.clone(),
        year: store.year,
        week: store.week,
        mode,
    }
}

/// Pure post-effect application — mutates the (freshly re-borrowed) store based on
/// the loader outcome. Kept separate so we can unit-test the visibility/error
/// transitions without spinning up loaders or Signals.
pub(crate) fn apply_save_outcome(store: &mut SlotEdit, outcome: SaveOutcome) {
    match outcome {
        SaveOutcome::EditSaved => {
            store.visible = false;
        }
        SaveOutcome::CreateSucceeded => {
            store.visible = false;
            store.has_errors = false;
        }
        SaveOutcome::CreateFailed => {
            store.has_errors = true;
            // visible untouched — modal stays open for error correction.
        }
    }
}

pub async fn save_slot_edit() -> Result<(), ShiftyError> {
    // Take a snapshot in a block scope so the read-guard is dropped BEFORE any `.await`.
    let snapshot = {
        let store = SLOT_EDIT_STORE.read();
        snapshot_for_save(&store)
    };
    let config = CONFIG.read().clone();

    // Perform the loader call purely on the owned snapshot — no store access here.
    let outcome = match snapshot.mode {
        SaveMode::AllFromWeek => {
            loader::save_slot(config, snapshot.slot_clone, snapshot.year, snapshot.week).await?;
            SaveOutcome::EditSaved
        }
        SaveMode::SingleWeek => {
            loader::save_slot_single_week(
                config,
                snapshot.slot_clone,
                snapshot.year,
                snapshot.week,
            )
            .await?;
            SaveOutcome::EditSaved
        }
        SaveMode::Create => {
            if loader::create_slot(config, snapshot.slot_clone).await? {
                SaveOutcome::CreateSucceeded
            } else {
                SaveOutcome::CreateFailed
            }
        }
    };

    // Fresh write-guard in its own block-scope — dropped before the fn returns.
    {
        let mut store = SLOT_EDIT_STORE.write();
        apply_save_outcome(&mut store, outcome);
    }

    // Refresh only on success. `CreateFailed` keeps the modal open and skips the refresh
    // (mirrors the pre-refactor `return Ok(());` behavior).
    if matches!(outcome, SaveOutcome::EditSaved | SaveOutcome::CreateSucceeded) {
        trigger_shiftplan_refresh();
    }
    Ok(())
}

pub async fn cancel_slot_edit() -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.visible = false;
    Ok(())
}

pub async fn delete_slot_edit(id: Uuid, year: u32, week: u8) -> Result<(), ShiftyError> {
    api::delete_slot_from(CONFIG.read().clone(), id, year, week).await?;
    trigger_shiftplan_refresh();
    Ok(())
}

pub async fn load_slot_edit(
    slot_id: Uuid,
    year: u32,
    week: u8,
    current_paid_count: u8,
) -> Result<(), ShiftyError> {
    let slot = loader::load_slot(CONFIG.read().clone(), slot_id).await?;
    let mut store = SLOT_EDIT_STORE.write();
    store.slot_edit_type = SlotEditType::Edit;
    store.slot = slot.into();
    store.year = year;
    store.week = week;
    store.visible = true;
    store.has_errors = false;
    store.current_paid_count = current_paid_count;
    store.single_week = false;
    Ok(())
}

pub fn set_single_week(val: bool) -> Result<(), ShiftyError> {
    SLOT_EDIT_STORE.write().single_week = val;
    Ok(())
}

pub async fn slot_edit_service(mut rx: UnboundedReceiver<SlotEditAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            SlotEditAction::NewSlot(year, week, shiftplan_id) => {
                new_slot_edit(year, week, shiftplan_id)
            }
            SlotEditAction::UpdateSlot(slot) => update_slot_edit(slot),
            SlotEditAction::SaveSlot => save_slot_edit().await,
            SlotEditAction::Cancel => cancel_slot_edit().await,
            SlotEditAction::DeleteSlot(id, year, week) => delete_slot_edit(id, year, week).await,
            SlotEditAction::LoadSlot(id, year, week, count) => {
                load_slot_edit(id, year, week, count).await
            }
            SlotEditAction::SetSingleWeek(val) => set_single_week(val),
        } {
            Ok(_) => {}
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore {
                    error: Some(err),
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::slot_edit::{SlotEdit, SlotEditItem, SlotEditType};

    fn base_store() -> SlotEdit {
        let mut store = SlotEdit::new_edit();
        store.year = 2026;
        store.week = 27;
        store.slot = SlotEditItem::new_valid_from(2026, 27).into();
        store
    }

    #[test]
    fn snapshot_for_save_edit_multi_week_maps_to_all_from_week() {
        let mut store = base_store();
        store.slot_edit_type = SlotEditType::Edit;
        store.single_week = false;

        let snap = snapshot_for_save(&store);

        assert_eq!(snap.year, 2026);
        assert_eq!(snap.week, 27);
        assert_eq!(snap.mode, SaveMode::AllFromWeek);
        assert_eq!(snap.slot_clone, store.slot);
    }

    #[test]
    fn snapshot_for_save_edit_single_week_maps_to_single_week() {
        let mut store = base_store();
        store.slot_edit_type = SlotEditType::Edit;
        store.single_week = true;

        let snap = snapshot_for_save(&store);

        assert_eq!(snap.mode, SaveMode::SingleWeek);
        assert_eq!(snap.year, 2026);
        assert_eq!(snap.week, 27);
    }

    #[test]
    fn snapshot_for_save_new_maps_to_create() {
        let mut store = base_store();
        store.slot_edit_type = SlotEditType::New;
        // single_week is meaningless for New, but should not affect mapping.
        store.single_week = true;

        let snap = snapshot_for_save(&store);

        assert_eq!(snap.mode, SaveMode::Create);
    }

    #[test]
    fn apply_save_outcome_edit_saved_closes_modal_and_leaves_has_errors_alone() {
        let mut store = base_store();
        store.visible = true;
        store.has_errors = true; // preserved on EditSaved (spec: "lässt has_errors unangetastet")

        apply_save_outcome(&mut store, SaveOutcome::EditSaved);

        assert!(!store.visible);
        assert!(
            store.has_errors,
            "EditSaved must not clear has_errors — spec requires it to stay untouched"
        );
    }

    #[test]
    fn apply_save_outcome_create_failed_sets_error_and_keeps_modal_open() {
        let mut store = base_store();
        store.visible = true;
        store.has_errors = false;

        apply_save_outcome(&mut store, SaveOutcome::CreateFailed);

        assert!(store.has_errors);
        assert!(
            store.visible,
            "CreateFailed must keep the modal open (visible unchanged)"
        );
    }

    #[test]
    fn apply_save_outcome_create_succeeded_closes_modal_and_clears_errors() {
        let mut store = base_store();
        store.visible = true;
        store.has_errors = true;

        apply_save_outcome(&mut store, SaveOutcome::CreateSucceeded);

        assert!(!store.visible);
        assert!(!store.has_errors);
    }
}
