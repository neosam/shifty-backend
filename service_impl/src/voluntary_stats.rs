//! Phase 54 Plan 03 — Business-Logic-Tier VoluntaryStatsService (VOL-STAT-01/02,
//! VOL-ACCT-01/02/03).
//!
//! Kombiniert die vier pure fns aus `crate::reporting`
//! (`voluntary_ist_total_for_year`, `contract_weeks_count`,
//! `committed_voluntary_prorata_for_week`, `committed_voluntary_target_for_year`)
//! mit einem HR-Gate an erster Stelle. Non-HR-Aufrufer erhalten ein
//! `VoluntaryStats` mit lauter `None`-Feldern (API-Level-Redaktion,
//! Praezedenz VAC-OFFSET-01 v1.8 — kein 403).
//!
//! Cross-Service-Calls verwenden `Authentication::Full` (Bypass, Auth wurde
//! bereits am Einstieg geprueft — Praezedenz `reference_toggle_service_full_context_bypass`).

use crate::gen_service_impl;
use crate::reporting::{
    committed_voluntary_target_for_year, contract_weeks_count, voluntary_ist_total_for_year,
};
use async_trait::async_trait;
use dao::TransactionDao;
use service::employee_work_details::EmployeeWorkDetailsService;
use service::extra_hours::ExtraHoursService;
use service::permission::{Authentication, HR_PRIVILEGE};
use service::sales_person::SalesPersonService;
use service::voluntary_stats::{VoluntaryStats, VoluntaryStatsService};
use service::{PermissionService, ServiceError};
use uuid::Uuid;

gen_service_impl! {
    struct VoluntaryStatsServiceImpl: VoluntaryStatsService = VoluntaryStatsServiceDeps {
        ExtraHoursService: ExtraHoursService<Transaction = Self::Transaction, Context = Self::Context> = extra_hours_service,
        EmployeeWorkDetailsService: EmployeeWorkDetailsService<Transaction = Self::Transaction, Context = Self::Context> = employee_work_details_service,
        SalesPersonService: SalesPersonService<Transaction = Self::Transaction, Context = Self::Context> = sales_person_service,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: VoluntaryStatsServiceDeps> VoluntaryStatsService for VoluntaryStatsServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_voluntary_stats(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<VoluntaryStats, ServiceError> {
        // HR-Gate ZUERST — kein Datenabruf vor Auth (Praezedenz Phase 41 D-AVG-05).
        let is_hr = self
            .permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await
            .is_ok();

        // Non-HR: alle Felder None (API-Level-Redaktion, keine 403).
        if !is_hr {
            return Ok(VoluntaryStats {
                ist_per_contract_week: None,
                ist_total: None,
                soll_total: None,
                delta: None,
                contract_weeks: None,
            });
        }

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // SalesPerson-Existenzpruefung mit internem Full-Context
        // (Cross-Service-Bypass, Auth wurde oben verifiziert).
        let _sp = self
            .sales_person_service
            .get(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;

        // ExtraHours des Jahres laden und auf sales_person filtern.
        let extra_hours = self
            .extra_hours_service
            .find_by_iso_year(year, Authentication::Full, Some(tx.clone()))
            .await?;
        let extra_hours_for_sp: Vec<_> = extra_hours
            .iter()
            .filter(|eh| eh.sales_person_id == sales_person_id)
            .cloned()
            .collect();

        // EmployeeWorkDetails des SalesPerson laden.
        let working_hours = self
            .employee_work_details_service
            .find_by_sales_person_id(sales_person_id, Authentication::Full, Some(tx.clone()))
            .await?;
        let working_hours_vec: Vec<_> = working_hours.iter().cloned().collect();

        self.transaction_dao.commit(tx).await?;

        let ist_total = voluntary_ist_total_for_year(&extra_hours_for_sp, year);
        let soll_total = committed_voluntary_target_for_year(&working_hours_vec, year);
        let contract_weeks = contract_weeks_count(&working_hours_vec, year);
        let ist_per_contract_week = if contract_weeks == 0 {
            0.0
        } else {
            ist_total / contract_weeks as f32
        };
        let delta = ist_total - soll_total;

        Ok(VoluntaryStats {
            ist_per_contract_week: Some(ist_per_contract_week),
            ist_total: Some(ist_total),
            soll_total: Some(soll_total),
            delta: Some(delta),
            contract_weeks: Some(contract_weeks),
        })
    }
}
