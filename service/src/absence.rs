//! Absence-Domain für Phase 1.
//!
//! Stellt Trait, Domain-Modell und Enum für Abwesenheits-Perioden
//! (`AbsencePeriod`) bereit. Phase 1 unterstützt genau drei Kategorien
//! (`Vacation`, `SickLeave`, `UnpaidLeave`) — andere Hour-based-Kategorien
//! verbleiben in [`crate::extra_hours::ExtraHoursCategory`] (D-02/D-03 in
//! `01-CONTEXT.md`). Die Domain-`id` ist mit `dao::absence::AbsencePeriodEntity::logical_id`
//! identisch (D-07): externe Referenzen bleiben über Updates hinweg stabil.
//!
//! Read-Sicht (D-10 Option A): HR-Privilege ∨ self (`verify_user_is_sales_person`).
//! Schichtplan-Kollegen-Sicht ist auf Phase 3 verschoben. Schreib-Methoden enforcen
//! das gleiche Pattern (D-09).

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DateRange;
use time::Date;
use uuid::Uuid;

use crate::{permission::Authentication, ServiceError};

/// Domain-Kategorien einer Absence-Periode (Phase 1: 3 Werte).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbsenceCategory {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

impl From<&dao::absence::AbsenceCategoryEntity> for AbsenceCategory {
    fn from(c: &dao::absence::AbsenceCategoryEntity) -> Self {
        match c {
            dao::absence::AbsenceCategoryEntity::Vacation => Self::Vacation,
            dao::absence::AbsenceCategoryEntity::SickLeave => Self::SickLeave,
            dao::absence::AbsenceCategoryEntity::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}
impl From<&AbsenceCategory> for dao::absence::AbsenceCategoryEntity {
    fn from(c: &AbsenceCategory) -> Self {
        match c {
            AbsenceCategory::Vacation => Self::Vacation,
            AbsenceCategory::SickLeave => Self::SickLeave,
            AbsenceCategory::UnpaidLeave => Self::UnpaidLeave,
        }
    }
}

/// Domain-Repräsentation einer Absence-Periode.
///
/// `id` entspricht der DAO-`logical_id` (D-07). Der `update`-Pfad rotiert die
/// physische Row, hält aber `id` (= logical_id) und damit externe Referenzen
/// stabil. `from_date`/`to_date` sind beide inklusive (D-05).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriod {
    /// Externally stable id == DAO `logical_id`. Equals the physical row id of the first version.
    pub id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategory,
    pub from_date: Date,
    pub to_date: Date,
    pub description: Arc<str>,
    pub created: Option<time::PrimitiveDateTime>,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

impl From<&dao::absence::AbsencePeriodEntity> for AbsencePeriod {
    fn from(e: &dao::absence::AbsencePeriodEntity) -> Self {
        Self {
            id: e.logical_id,
            sales_person_id: e.sales_person_id,
            category: (&e.category).into(),
            from_date: e.from_date,
            to_date: e.to_date,
            description: e.description.clone(),
            created: Some(e.created),
            deleted: e.deleted,
            version: e.version,
        }
    }
}
impl TryFrom<&AbsencePeriod> for dao::absence::AbsencePeriodEntity {
    type Error = ServiceError;
    fn try_from(a: &AbsencePeriod) -> Result<Self, Self::Error> {
        Ok(Self {
            id: a.id,
            logical_id: a.id,
            sales_person_id: a.sales_person_id,
            category: (&a.category).into(),
            from_date: a.from_date,
            to_date: a.to_date,
            description: a.description.clone(),
            created: a.created.ok_or(ServiceError::InternalError)?,
            deleted: a.deleted,
            version: a.version,
        })
    }
}

impl AbsencePeriod {
    /// Liefert den `DateRange` (`from..=to`); Range-Inversion → `DateOrderWrong`.
    pub fn date_range(&self) -> Result<DateRange, ServiceError> {
        DateRange::new(self.from_date, self.to_date)
            .map_err(|_| ServiceError::DateOrderWrong(self.from_date, self.to_date))
    }
}

/// Output von [`AbsenceService::derive_hours_for_range`] — pro Tag bereits
/// conflict-resolved per D-Phase2-01..03 (Prioritaet:
/// `SickLeave > Vacation > UnpaidLeave`, BUrlG §9-konform). `hours` traegt
/// die am jeweiligen Tag gueltigen Vertragsstunden; an Feiertagen oder
/// Tagen ohne Vertrag liegt KEIN Eintrag in der Map vor.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedAbsence {
    pub category: AbsenceCategory,
    pub hours: f32,
}

#[automock(type Context=(); type Transaction=dao::MockTransaction;)]
#[async_trait]
pub trait AbsenceService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
    type Transaction: dao::Transaction;

    /// HR only — full visibility (D-10 Option A).
    async fn find_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;

    /// HR ∨ verify_user_is_sales_person(sales_person_id) (D-10 Option A;
    /// Schichtplan-Kollege deferred to Phase 3).
    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[AbsencePeriod]>, ServiceError>;

    /// HR ∨ self (Self ermittelt aus active row's sales_person_id).
    async fn find_by_id(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn create(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn update(
        &self,
        entity: &AbsencePeriod,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<AbsencePeriod, ServiceError>;

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError>;

    /// Conflict-resolved per-day hours map for a sales person in `[from, to]`.
    /// Prioritaet: `SickLeave > Vacation > UnpaidLeave` (D-Phase2-03, BUrlG §9).
    /// Tage ohne Vertrag, ohne aktive Absence, oder mit
    /// `SpecialDayType::Holiday` liefern KEINEN Eintrag in der Map.
    /// Permission: HR ∨ self (gleiche Regel wie [`Self::find_by_sales_person`]).
    async fn derive_hours_for_range(
        &self,
        from: Date,
        to: Date,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<BTreeMap<Date, ResolvedAbsence>, ServiceError>;
}

#[cfg(test)]
mod tests {
    //! Smoke-Tests: Conversions zwischen DAO-Entity und Domain-Modell, sowie
    //! `date_range()`-Helper.

    use super::*;
    use std::sync::Arc;
    use time::macros::{date, datetime};

    fn dao_entity() -> dao::absence::AbsencePeriodEntity {
        dao::absence::AbsencePeriodEntity {
            id: Uuid::nil(),
            logical_id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            category: dao::absence::AbsenceCategoryEntity::Vacation,
            from_date: date!(2026 - 04 - 12),
            to_date: date!(2026 - 04 - 15),
            description: Arc::from(""),
            created: datetime!(2026 - 04 - 01 12:00:00),
            deleted: None,
            version: Uuid::nil(),
        }
    }

    #[test]
    fn category_round_trips() {
        let dao_cat = dao::absence::AbsenceCategoryEntity::SickLeave;
        let domain: AbsenceCategory = (&dao_cat).into();
        let back: dao::absence::AbsenceCategoryEntity = (&domain).into();
        assert_eq!(dao_cat, back);
        assert_eq!(domain, AbsenceCategory::SickLeave);
    }

    #[test]
    fn domain_id_equals_logical_id() {
        let mut e = dao_entity();
        let logical = uuid::uuid!("AB000000-0000-0000-0000-000000000001");
        let physical = uuid::uuid!("AB000000-0000-0000-0000-000000000002");
        e.logical_id = logical;
        e.id = physical;
        let domain = AbsencePeriod::from(&e);
        assert_eq!(domain.id, logical, "Domain-id muss auf logical_id mappen");
    }

    #[test]
    fn try_from_without_created_returns_internal_error() {
        let domain = AbsencePeriod {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            category: AbsenceCategory::Vacation,
            from_date: date!(2026 - 04 - 12),
            to_date: date!(2026 - 04 - 15),
            description: Arc::from(""),
            created: None,
            deleted: None,
            version: Uuid::nil(),
        };
        let result = dao::absence::AbsencePeriodEntity::try_from(&domain);
        assert!(matches!(result, Err(ServiceError::InternalError)));
    }

    #[test]
    fn date_range_inversion_returns_date_order_wrong() {
        let p = AbsencePeriod {
            id: Uuid::nil(),
            sales_person_id: Uuid::nil(),
            category: AbsenceCategory::Vacation,
            from_date: date!(2026 - 04 - 20),
            to_date: date!(2026 - 04 - 12),
            description: Arc::from(""),
            created: None,
            deleted: None,
            version: Uuid::nil(),
        };
        let r = p.date_range();
        assert!(matches!(r, Err(ServiceError::DateOrderWrong(_, _))));
    }
}
