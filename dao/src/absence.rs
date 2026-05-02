use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;
use shifty_utils::DateRange;
use time::Date;
use uuid::Uuid;

/// Persistierte Absence-Kategorien.
///
/// Phase 1 unterstützt genau drei Werte (D-02, D-03 in CONTEXT.md): `Vacation`,
/// `SickLeave`, `UnpaidLeave`. Andere Hour-based-Kategorien bleiben in
/// `dao::extra_hours::ExtraHoursCategoryEntity` und sind hier bewusst nicht
/// modelliert — der Compiler garantiert dadurch, dass `AbsencePeriodEntity`
/// keine ungültige Kategorie tragen kann.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AbsenceCategoryEntity {
    Vacation,
    SickLeave,
    UnpaidLeave,
}

/// DAO-Repräsentation eines Absence-Period-Eintrags.
///
/// Felder spiegeln die `absence_period`-Tabelle (Migration `20260501162017`).
/// `from_date`/`to_date` sind beide inklusive (D-05). `description` ist als
/// `Arc<str>` modelliert (analog `ExtraHoursEntity.description`); leer ist der
/// Default, falls die DB-Spalte `NULL` ist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AbsencePeriodEntity {
    pub id: Uuid,
    pub logical_id: Uuid,
    pub sales_person_id: Uuid,
    pub category: AbsenceCategoryEntity,
    pub from_date: Date,
    pub to_date: Date,
    pub description: Arc<str>,
    pub created: time::PrimitiveDateTime,
    pub deleted: Option<time::PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait AbsenceDao {
    type Transaction: crate::Transaction;

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, crate::DaoError>;

    async fn find_by_logical_id(
        &self,
        logical_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<AbsencePeriodEntity>, crate::DaoError>;

    async fn find_by_sales_person(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    async fn find_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    /// Findet aktive Rows desselben `sales_person_id` und derselben `category`,
    /// die `range` inklusiv überlappen (Allen-Algebra: `from_date <= probe.to`
    /// UND `to_date >= probe.from`).
    ///
    /// `exclude_logical_id` wird beim Update verwendet, damit die zu
    /// modifizierende Row nicht mit sich selbst kollidiert (D-15). `None` für
    /// den Create-Pfad, `Some(id)` für den Update-Pfad.
    async fn find_overlapping(
        &self,
        sales_person_id: Uuid,
        category: AbsenceCategoryEntity,
        range: DateRange,
        exclude_logical_id: Option<Uuid>,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    /// Findet aktive Absence-Periods derselben `sales_person_id`, die `range`
    /// inklusiv überlappen — **kategorie-frei** (alle 3 AbsenceCategory-Werte
    /// werden zurückgegeben). Verwendet vom
    /// `AbsenceService::find_overlapping_for_booking`-Pfad und vom
    /// `ShiftplanEditService::book_slot_with_conflict_check`-Pfad
    /// (Phase 3, D-Phase3-05).
    ///
    /// Nutzt den bestehenden Composite-Index
    /// `idx_absence_period_sales_person_from(sales_person_id, from_date)
    ///  WHERE deleted IS NULL` (Phase-1-D-04). Single-Roundtrip auch bei
    /// copy_week-Loops; Performance-skalierbar.
    async fn find_overlapping_for_booking(
        &self,
        sales_person_id: Uuid,
        range: DateRange,
        tx: Self::Transaction,
    ) -> Result<Arc<[AbsencePeriodEntity]>, crate::DaoError>;

    async fn create(
        &self,
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;

    /// Schreibt `deleted`, `update_version`, `update_process` per `id`.
    /// Reine Body-Mutationen sind nicht unterstützt — der Service rotiert
    /// stets via `update(tombstone) + create(neu)` (D-07).
    async fn update(
        &self,
        entity: &AbsencePeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
}

#[cfg(test)]
mod tests {
    //! Smoke-Test: stellt sicher, dass `automock` einen `MockAbsenceDao`
    //! generiert, der mit der `MockTransaction`-Type kompiliert. Plan 02
    //! (Service-Tests) hängt von diesem Mock ab.

    use super::*;
    use time::macros::date;

    #[test]
    fn mock_absence_dao_is_constructible() {
        let _mock = MockAbsenceDao::new();
    }

    #[test]
    fn entity_round_trips_via_clone_and_equality() {
        let from = date!(2026 - 05 - 15);
        let to = date!(2026 - 05 - 20);
        let id = Uuid::nil();
        let entity = AbsencePeriodEntity {
            id,
            logical_id: id,
            sales_person_id: id,
            category: AbsenceCategoryEntity::Vacation,
            from_date: from,
            to_date: to,
            description: Arc::from(""),
            created: time::PrimitiveDateTime::new(from, time::Time::MIDNIGHT),
            deleted: None,
            version: id,
        };
        let cloned = entity.clone();
        assert_eq!(entity, cloned);
    }

    #[test]
    fn category_variants_are_distinct() {
        assert_ne!(
            AbsenceCategoryEntity::Vacation,
            AbsenceCategoryEntity::SickLeave
        );
        assert_ne!(
            AbsenceCategoryEntity::SickLeave,
            AbsenceCategoryEntity::UnpaidLeave
        );
        assert_ne!(
            AbsenceCategoryEntity::Vacation,
            AbsenceCategoryEntity::UnpaidLeave
        );
    }
}
