use std::sync::Arc;

use async_trait::async_trait;
use mockall::automock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BillingPeriodEntity {
    pub id: uuid::Uuid,
    pub start_date: time::Date,
    pub end_date: time::Date,

    pub created_at: time::PrimitiveDateTime,
    pub created_by: Arc<str>,
    pub deleted_at: Option<time::PrimitiveDateTime>,
    pub deleted_by: Option<Arc<str>>,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait]
pub trait BillingPeriodDao {
    type Transaction: crate::Transaction;

    async fn dump_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError>;
    async fn create(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, crate::DaoError>;
    async fn update(
        &self,
        entity: &BillingPeriodEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<BillingPeriodEntity, crate::DaoError>;
    async fn clear_all(
        &self,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;
    async fn delete_by_id(
        &self,
        id: uuid::Uuid,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), crate::DaoError>;

    async fn all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError> {
        Ok(self
            .dump_all(tx)
            .await?
            .iter()
            .filter(|bp| bp.deleted_at.is_none())
            .cloned()
            .collect())
    }

    async fn find_by_id(
        &self,
        id: uuid::Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<BillingPeriodEntity>, crate::DaoError> {
        self.all(tx)
            .await?
            .iter()
            .find(|bp| bp.id == id)
            .map_or(Ok(None), |bp| Ok(Some(bp.clone())))
    }

    async fn all_ordered_desc(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError> {
        let mut items: Vec<_> = self.all(tx).await?.to_vec();
        items.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        Ok(items.into())
    }

    async fn find_latest_end_date(
        &self,
        tx: Self::Transaction,
    ) -> Result<Option<time::Date>, crate::DaoError> {
        self.all(tx)
            .await?
            .iter()
            .map(|bp| bp.end_date)
            .max()
            .map_or(Ok(None), |date| Ok(Some(date)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::{date, datetime};

    struct TestBillingPeriodDao {
        entities: Arc<[BillingPeriodEntity]>,
    }

    #[async_trait]
    impl BillingPeriodDao for TestBillingPeriodDao {
        type Transaction = crate::MockTransaction;

        async fn dump_all(
            &self,
            _tx: Self::Transaction,
        ) -> Result<Arc<[BillingPeriodEntity]>, crate::DaoError> {
            Ok(self.entities.clone())
        }
        async fn create(
            &self,
            _entity: &BillingPeriodEntity,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<BillingPeriodEntity, crate::DaoError> {
            unimplemented!()
        }
        async fn update(
            &self,
            _entity: &BillingPeriodEntity,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<BillingPeriodEntity, crate::DaoError> {
            unimplemented!()
        }
        async fn clear_all(
            &self,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<(), crate::DaoError> {
            unimplemented!()
        }
        async fn delete_by_id(
            &self,
            _id: uuid::Uuid,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<(), crate::DaoError> {
            unimplemented!()
        }
    }

    fn make_entity(start: time::Date, end: time::Date) -> BillingPeriodEntity {
        BillingPeriodEntity {
            id: uuid::Uuid::new_v4(),
            start_date: start,
            end_date: end,
            created_at: datetime!(2024-01-01 0:00),
            created_by: "test".into(),
            deleted_at: None,
            deleted_by: None,
        }
    }

    #[tokio::test]
    async fn all_ordered_desc_returns_descending_by_start_date() {
        let dao = TestBillingPeriodDao {
            entities: vec![
                make_entity(date!(2025 - 01 - 01), date!(2025 - 03 - 31)),
                make_entity(date!(2025 - 07 - 01), date!(2025 - 09 - 30)),
                make_entity(date!(2025 - 04 - 01), date!(2025 - 06 - 30)),
            ]
            .into(),
        };

        let result = dao
            .all_ordered_desc(crate::MockTransaction)
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].start_date, date!(2025 - 07 - 01));
        assert_eq!(result[1].start_date, date!(2025 - 04 - 01));
        assert_eq!(result[2].start_date, date!(2025 - 01 - 01));
    }

    #[tokio::test]
    async fn all_ordered_desc_empty() {
        let dao = TestBillingPeriodDao {
            entities: vec![].into(),
        };

        let result = dao
            .all_ordered_desc(crate::MockTransaction)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn all_ordered_desc_single_element() {
        let dao = TestBillingPeriodDao {
            entities: vec![make_entity(date!(2025 - 01 - 01), date!(2025 - 03 - 31))].into(),
        };

        let result = dao
            .all_ordered_desc(crate::MockTransaction)
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].start_date, date!(2025 - 01 - 01));
    }

    #[tokio::test]
    async fn all_ordered_desc_excludes_deleted() {
        let dao = TestBillingPeriodDao {
            entities: vec![
                make_entity(date!(2025 - 01 - 01), date!(2025 - 03 - 31)),
                BillingPeriodEntity {
                    deleted_at: Some(datetime!(2025-06-01 0:00)),
                    deleted_by: Some("admin".into()),
                    ..make_entity(date!(2025 - 07 - 01), date!(2025 - 09 - 30))
                },
                make_entity(date!(2025 - 04 - 01), date!(2025 - 06 - 30)),
            ]
            .into(),
        };

        let result = dao
            .all_ordered_desc(crate::MockTransaction)
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].start_date, date!(2025 - 04 - 01));
        assert_eq!(result[1].start_date, date!(2025 - 01 - 01));
    }
}
