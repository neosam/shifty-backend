//! Phase 4 — Cycle-breaking carryover rebuild surface.
//!
//! ## Locked decision (Plan 04-03 enforces)
//!
//! Per **C-Phase4-02** + **Pitfall 1** in RESEARCH.md, this service is a
//! NEW Business-Logic-Tier service that consumes ReportingService (Read) and
//! CarryoverService (Write). The existing `CarryoverService` (basic-tier)
//! stays basic — it MUST NOT consume ReportingService, otherwise:
//! `Reporting -> Carryover -> Reporting` cycle.
//!
//! Variant rejected: extending CarryoverService with `rebuild_for_year`. That
//! would force CarryoverService into the Business-Logic tier (breaks Phase 1-3
//! contracts that treat it as basic). Rejected per CLAUDE.md Service-Tier-
//! Konvention.
//!
//! Variant rejected: inlining the rebuild in CutoverServiceImpl. That makes
//! CutoverServiceImpl 6 deps wide instead of 5 and is not reusable for any
//! future bulk-rebuild surface (currently deferred per CONTEXT.md `<deferred>`).

use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use uuid::Uuid;

use crate::permission::Authentication;
use crate::ServiceError;

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait CarryoverRebuildService {
    type Context: Clone + std::fmt::Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Re-derive employee_yearly_carryover for `(sales_person_id, year)` from
    /// ReportingService output. Caller passes the cutover Tx as `Some(tx)` so
    /// the read sees the post-flag-flip state. Permission: cutover_admin
    /// (called only inside the cutover Tx with `Authentication::Full`-bypass).
    async fn rebuild_for_year(
        &self,
        sales_person_id: Uuid,
        year: u32,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;
}
