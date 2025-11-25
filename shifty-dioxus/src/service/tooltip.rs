use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::state::tooltip::Tooltip;

pub static TOOLTIP: GlobalSignal<Option<Tooltip>> = Signal::global(|| None);

pub enum TooltipAction {
    ShowTooltip(f64, f64, Rc<str>),
    HideTooltip,
}

pub async fn show_tooltip(x: f64, y: f64, content: Rc<str>) {
    *TOOLTIP.write() = Some(Tooltip { x, y, content });
}

pub async fn hide_tooltip() {
    *TOOLTIP.write() = None;
}

pub async fn tooltip_service(mut rx: UnboundedReceiver<TooltipAction>) {
    while let Some(action) = rx.next().await {
        match action {
            TooltipAction::ShowTooltip(x, y, content) => show_tooltip(x, y, content).await,
            TooltipAction::HideTooltip => hide_tooltip().await,
        }
    }
}
