use crate::gen_service_impl;
use async_trait::async_trait;
use chrono::Local;
use service::{
    permission::Authentication, scheduler::SchedulerService, shiftplan_edit::ShiftplanEditService,
    ServiceError,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron::{Job, Scheduler};
use tracing::{error, info};

gen_service_impl! {
    struct SchedulerServiceImpl: service::scheduler::SchedulerService = SchedulerServiceDeps {
        ShiftplanEditService: service::shiftplan_edit::ShiftplanEditService = shiftplan_edit_service,
    }
    ; custom_fields {
        scheduler: Arc<Mutex<Scheduler<Local>>> = scheduler
    }
}

impl<Deps: SchedulerServiceDeps> SchedulerServiceImpl<Deps> {
    pub fn new(shiftplan_edit_service: Arc<Deps::ShiftplanEditService>) -> Self {
        let scheduler = Arc::new(Mutex::new(Scheduler::local()));

        Self {
            scheduler,
            shiftplan_edit_service,
        }
    }
}

#[async_trait]
impl<Deps: SchedulerServiceDeps> SchedulerService for SchedulerServiceImpl<Deps> {
    type Context = Deps::Context;

    // Start the scheduler in a background task
    async fn start(&self) -> Result<(), ServiceError> {
        /*let scheduler = self.scheduler.clone();
        tokio::spawn(async move {
            info!("Starting the SchedulerService background loop");
            let mut s = scheduler.lock().await;
            s.start().await;
        });*/
        self.schedule_carryover_updates("0 * * * * *").await?;
        Ok(())
    }

    async fn schedule_carryover_updates(&self, cron: &'static str) -> Result<(), ServiceError> {
        let mut sched = self.scheduler.lock().await;

        let shiftplan_edit_service = self.shiftplan_edit_service.clone();

        sched.add(Job::new(cron, move || {
            let shiftplan_edit_service = shiftplan_edit_service.clone();
            async move {
                let date = time::OffsetDateTime::now_utc();
                let year = date.year() as u32;
                if let Err(e) = shiftplan_edit_service
                    .update_carryover_all_employees(year - 1, Authentication::Full, None)
                    .await
                {
                    error!("Failed to update carryover for previous year: {:?}", e);
                } else {
                    info!("Successfully updated carryover for previous year (cron job)");
                }
            }
        }));

        info!("Scheduled carryover updates with cron expression: {}", cron);
        Ok(())
    }
}
