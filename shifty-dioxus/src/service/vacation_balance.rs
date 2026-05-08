//! Vacation-balance read coroutine service (Phase 8 Wave 4).
//!
//! Two stores: `VACATION_BALANCE_STORE` for the Self-variant of
//! `VacationEntitlementCard` and `VACATION_TEAM_STORE` for the HR-variant +
//! `VacationPerPersonList`. Both are read-only — there is no Create/Update/
//! Delete path for vacation balances; the value is always re-computed by the
//! backend from contract + carryover + booked absences.

use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::{loader, state::vacation_balance::VacationBalance};

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

pub static VACATION_BALANCE_STORE: GlobalSignal<Option<VacationBalance>> =
    Signal::global(|| None);

pub static VACATION_TEAM_STORE: GlobalSignal<Rc<[VacationBalance]>> =
    Signal::global(|| Rc::new([]));

#[derive(Debug)]
pub enum VacationBalanceAction {
    LoadSelf(Uuid, u32),
    LoadTeam(u32),
}

pub async fn vacation_balance_service(mut rx: UnboundedReceiver<VacationBalanceAction>) {
    while let Some(action) = rx.next().await {
        info!("VacationBalanceAction: {:?}", &action);
        let config = CONFIG.read().clone();
        match action {
            VacationBalanceAction::LoadSelf(sp_id, year) => {
                match loader::load_vacation_balance(config, sp_id, year).await {
                    Ok(b) => {
                        *VACATION_BALANCE_STORE.write() = Some(b);
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore { error: Some(err) };
                    }
                }
            }
            VacationBalanceAction::LoadTeam(year) => {
                match loader::load_team_vacation(config, year).await {
                    Ok(list) => {
                        *VACATION_TEAM_STORE.write() = list;
                    }
                    Err(err) => {
                        *ERROR_STORE.write() = ErrorStore { error: Some(err) };
                    }
                }
            }
        }
    }
}
