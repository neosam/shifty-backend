use crate::permission::Authentication;
use crate::ServiceError;
use async_trait::async_trait;
use dao::MockTransaction;
use mockall::automock;
use std::fmt::Debug;

/// Domain-level KW status. Unlike the persisted `dao::week_status::WeekStatusKind`,
/// this enum carries a fourth variant `Unset`, which lives only in the service /
/// frontend layer: row absence == `Unset` (D-39-04). It is deliberately named
/// `Unset`, never `None`, to avoid Option-shadowing (D-39-03).
#[derive(Clone, Debug, PartialEq)]
pub enum WeekStatus {
    Unset,
    InPlanning,
    Planned,
    Locked,
}

impl From<dao::week_status::WeekStatusKind> for WeekStatus {
    fn from(kind: dao::week_status::WeekStatusKind) -> Self {
        match kind {
            dao::week_status::WeekStatusKind::InPlanning => WeekStatus::InPlanning,
            dao::week_status::WeekStatusKind::Planned => WeekStatus::Planned,
            dao::week_status::WeekStatusKind::Locked => WeekStatus::Locked,
        }
    }
}

#[automock(type Context=(); type Transaction=MockTransaction;)]
#[async_trait]
pub trait WeekStatusService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// Read the status of a single ISO week. Available to all roles (status is not
    /// sensitive, T-39-03). Returns `WeekStatus::Unset` when no active row exists.
    async fn get_week_status(
        &self,
        year: u32,
        calendar_week: u8,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError>;

    /// Set the status of a single ISO week. Only holders of `SHIFTPLANNER_PRIVILEGE`
    /// may mutate (D-39-01, T-39-01). `Unset` soft-deletes the active row (D-39-04);
    /// all transitions are free (D-39-02).
    async fn set_week_status(
        &self,
        year: u32,
        calendar_week: u8,
        status: WeekStatus,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<WeekStatus, ServiceError>;
}
