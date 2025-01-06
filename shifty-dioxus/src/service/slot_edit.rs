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

pub static SLOT_EDIT_STORE: GlobalSignal<SlotEdit> = Signal::global(|| SlotEdit::new_edit());
pub enum SlotEditAction {
    NewSlot(u32, u8),
    UpdateSlot(SlotEditItem),
    SaveSlot,
    Cancel,
    DeleteSlot(Uuid, u32, u8),
    LoadSlot(Uuid, u32, u8),
}

pub fn new_slot_edit(year: u32, week: u8) -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.slot_edit_type = SlotEditType::New;
    store.slot = SlotEditItem::new_valid_from(year, week).into();
    store.year = year;
    store.week = week;
    store.visible = true;
    store.has_errors = false;
    Ok(())
}

pub fn update_slot_edit(slot_edit: SlotEditItem) -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.slot = slot_edit.into();
    Ok(())
}

pub async fn save_slot_edit() -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    match store.slot_edit_type {
        SlotEditType::Edit => {
            loader::save_slot(
                CONFIG.read().clone(),
                store.slot.clone(),
                store.year,
                store.week,
            )
            .await?;
        }
        SlotEditType::New => {
            if !loader::create_slot(CONFIG.read().clone(), store.slot.clone()).await? {
                store.has_errors = true;
                return Ok(());
            }
        }
    }
    store.visible = false;
    Ok(())
}

pub async fn cancel_slot_edit() -> Result<(), ShiftyError> {
    let mut store = SLOT_EDIT_STORE.write();
    store.visible = false;
    Ok(())
}

pub async fn delete_slot_edit(id: Uuid, year: u32, week: u8) -> Result<(), ShiftyError> {
    api::delete_slot_from(CONFIG.read().clone(), id, year, week).await?;
    Ok(())
}

pub async fn load_slot_edit(slot_id: Uuid, year: u32, week: u8) -> Result<(), ShiftyError> {
    let slot = loader::load_slot(CONFIG.read().clone(), slot_id).await?;
    let mut store = SLOT_EDIT_STORE.write();
    store.slot_edit_type = SlotEditType::Edit;
    store.slot = slot.into();
    store.year = year;
    store.week = week;
    store.visible = true;
    store.has_errors = false;
    Ok(())
}

pub async fn slot_edit_service(mut rx: UnboundedReceiver<SlotEditAction>) {
    while let Some(action) = rx.next().await {
        match match action {
            SlotEditAction::NewSlot(year, week) => new_slot_edit(year, week),
            SlotEditAction::UpdateSlot(slot) => update_slot_edit(slot),
            SlotEditAction::SaveSlot => save_slot_edit().await,
            SlotEditAction::Cancel => cancel_slot_edit().await,
            SlotEditAction::DeleteSlot(id, year, week) => delete_slot_edit(id, year, week).await,
            SlotEditAction::LoadSlot(id, year, week) => load_slot_edit(id, year, week).await,
        } {
            Ok(_) => {}
            Err(err) => {
                *ERROR_STORE.write() = ErrorStore {
                    error: Some(err.into()),
                };
            }
        }
    }
}
