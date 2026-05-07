use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::state::dropdown::{Dropdown, DropdownEntry};

pub static DROPDOWN: GlobalSignal<Option<Dropdown>> = Signal::global(|| None);

pub enum DropdownAction {
    CloseDropdown,
    ToggleDropdown(f64, f64, Rc<[DropdownEntry]>, Option<Rc<str>>),
}

pub async fn open_dropdown(x: f64, y: f64, entries: Rc<[DropdownEntry]>, context: Option<Rc<str>>) {
    *DROPDOWN.write() = Some(Dropdown {
        x,
        y,
        entries,
        context,
    });
}
pub async fn close_dropdown() {
    *DROPDOWN.write() = None;
}
pub async fn toggle_dropdown(
    x: f64,
    y: f64,
    entries: Rc<[DropdownEntry]>,
    context: Option<Rc<str>>,
) {
    if DROPDOWN.read().is_some() {
        close_dropdown().await;
    } else {
        open_dropdown(x, y, entries, context).await;
    }
}

pub async fn dropdown_service(mut rx: UnboundedReceiver<DropdownAction>) {
    while let Some(action) = rx.next().await {
        match action {
            DropdownAction::CloseDropdown => close_dropdown().await,
            DropdownAction::ToggleDropdown(x, y, entries, context) => {
                toggle_dropdown(x, y, entries, context).await
            }
        }
    }
}
