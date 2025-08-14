use std::rc::Rc;

use dioxus::prelude::*;
use futures_util::StreamExt;
use tracing::info;
use uuid::Uuid;

use crate::{
    api,
    error::ShiftyError,
};

use rest_types::BillingPeriodTO;

use super::{
    config::CONFIG,
    error::{ErrorStore, ERROR_STORE},
};

#[derive(Clone)]
pub struct BillingPeriodStore {
    pub billing_periods: Rc<[BillingPeriodTO]>,
    pub selected_billing_period: Option<BillingPeriodTO>,
}

impl Default for BillingPeriodStore {
    fn default() -> Self {
        Self {
            billing_periods: Rc::new([]),
            selected_billing_period: None,
        }
    }
}

pub static BILLING_PERIOD_STORE: GlobalSignal<BillingPeriodStore> = Signal::global(|| BillingPeriodStore::default());

#[derive(Debug)]
pub enum BillingPeriodAction {
    LoadBillingPeriods,
    LoadBillingPeriod(Uuid),
    CreateBillingPeriod(time::Date),
    ClearSelection,
}

pub async fn load_billing_periods() -> Result<(), ShiftyError> {
    info!("Loading billing periods");
    let billing_periods = api::get_billing_periods(CONFIG.read().clone()).await?;
    BILLING_PERIOD_STORE.write().billing_periods = billing_periods;
    info!("Loaded billing periods");
    Ok(())
}

pub async fn load_billing_period(billing_period_id: Uuid) -> Result<(), ShiftyError> {
    info!("Loading billing period {}", billing_period_id);
    let billing_period = api::get_billing_period(CONFIG.read().clone(), billing_period_id).await?;
    BILLING_PERIOD_STORE.write().selected_billing_period = Some(billing_period);
    info!("Loaded billing period");
    Ok(())
}

pub async fn create_billing_period(end_date: time::Date) -> Result<(), ShiftyError> {
    info!("Creating billing period with end date {}", end_date);
    api::post_billing_period(CONFIG.read().clone(), end_date).await?;
    info!("Created billing period");
    load_billing_periods().await?;
    Ok(())
}

pub fn clear_selected_billing_period() {
    BILLING_PERIOD_STORE.write().selected_billing_period = None;
}

pub async fn billing_period_service(mut rx: UnboundedReceiver<BillingPeriodAction>) {
    while let Some(action) = rx.next().await {
        info!("BillingPeriodAction: {:?}", &action);
        match match action {
            BillingPeriodAction::LoadBillingPeriods => load_billing_periods().await,
            BillingPeriodAction::LoadBillingPeriod(billing_period_id) => {
                load_billing_period(billing_period_id).await
            }
            BillingPeriodAction::CreateBillingPeriod(end_date) => {
                create_billing_period(end_date).await
            }
            BillingPeriodAction::ClearSelection => {
                clear_selected_billing_period();
                Ok(())
            }
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