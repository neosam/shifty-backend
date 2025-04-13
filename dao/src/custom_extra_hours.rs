use std::sync::Arc;

use crate::DaoError;
use mockall::automock;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomExtraHoursEntity {
    pub id: Uuid,
    pub name: Arc<str>,
    pub description: Option<Arc<str>>,
    pub modifies_balance: bool,

    pub assigned_sales_person_ids: Arc<[Uuid]>,

    pub created: PrimitiveDateTime,
    pub deleted: Option<PrimitiveDateTime>,
    pub version: Uuid,
}

#[automock(type Transaction = crate::MockTransaction;)]
#[async_trait::async_trait]
pub trait CustomExtraHoursDao {
    type Transaction: crate::Transaction;

    // Base CRU operations

    /// Returns everything, including deleted items.
    async fn dump(&self, tx: Self::Transaction) -> Result<Arc<[CustomExtraHoursEntity]>, DaoError>;

    /// Create a new entity
    async fn create(
        &self,
        entity: &CustomExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    /// Upates an entity
    async fn update(
        &self,
        entity: CustomExtraHoursEntity,
        process: &str,
        tx: Self::Transaction,
    ) -> Result<(), DaoError>;

    async fn find_all(
        &self,
        tx: Self::Transaction,
    ) -> Result<Arc<[CustomExtraHoursEntity]>, DaoError> {
        Ok(self
            .dump(tx.clone())
            .await?
            .iter()
            .filter(|entity| entity.deleted.is_none())
            .cloned()
            .collect())
    }

    async fn find_by_id(
        &self,
        id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Option<CustomExtraHoursEntity>, DaoError> {
        Ok(self
            .dump(tx.clone())
            .await?
            .iter()
            .find(|entity| entity.id == id)
            .cloned())
    }

    async fn find_by_sales_person_id(
        &self,
        sales_person_id: Uuid,
        tx: Self::Transaction,
    ) -> Result<Arc<[CustomExtraHoursEntity]>, DaoError> {
        Ok(self
            .dump(tx.clone())
            .await?
            .iter()
            .filter(|entity| {
                entity.deleted.is_none()
                    && entity.assigned_sales_person_ids.contains(&sales_person_id)
            })
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use time::macros::datetime;

    use uuid::{uuid, Uuid};

    use crate::DaoError;

    const ENTITY1_UUID: Uuid = uuid!("d65ed178-8d09-4863-af78-2be177ab3e20");
    const ENTITY2_UUID: Uuid = uuid!("d65ed178-8d09-4863-af78-2be177ab3e21");
    const ENTITY3_UUID: Uuid = uuid!("d65ed178-8d09-4863-af78-2be177ab3e22");
    const SALES_PERSON_1_UUID: Uuid = uuid!("67e086cb-8649-4baa-98e8-f1d9ef9546e0");
    const SALES_PERSON_2_UUID: Uuid = uuid!("67e086cb-8649-4baa-98e8-f1d9ef9546e1");
    const VERSION_UUID: Uuid = uuid!("25bf7f52-d66c-4681-a74e-e07ecf5e952e");

    pub struct CustomExtraHoursDaoTestImpl;
    use super::CustomExtraHoursDao;

    #[async_trait::async_trait]
    impl CustomExtraHoursDao for CustomExtraHoursDaoTestImpl {
        type Transaction = crate::MockTransaction;

        async fn dump(
            &self,
            _tx: Self::Transaction,
        ) -> Result<Arc<[super::CustomExtraHoursEntity]>, DaoError> {
            Ok(Arc::new([
                super::CustomExtraHoursEntity {
                    id: ENTITY1_UUID,
                    name: Arc::from("Test"),
                    description: Some(Arc::from("Test")),
                    modifies_balance: true,
                    assigned_sales_person_ids: Arc::new([SALES_PERSON_1_UUID, SALES_PERSON_2_UUID]),
                    created: datetime!(2023-10-01 12:00:00),
                    deleted: None,
                    version: VERSION_UUID,
                },
                super::CustomExtraHoursEntity {
                    id: ENTITY2_UUID,
                    name: Arc::from("Test2"),
                    description: None,
                    modifies_balance: false,
                    assigned_sales_person_ids: Arc::new([SALES_PERSON_1_UUID]),
                    created: datetime!(2023-10-01 12:00:00),
                    deleted: None,
                    version: VERSION_UUID,
                },
                super::CustomExtraHoursEntity {
                    id: ENTITY3_UUID,
                    name: Arc::from("Deleted"),
                    description: None,
                    modifies_balance: false,
                    assigned_sales_person_ids: Arc::new([SALES_PERSON_1_UUID, SALES_PERSON_2_UUID]),
                    created: datetime!(2023-10-01 12:00:00),
                    deleted: Some(datetime!(2023-10-01 12:00:00)),
                    version: VERSION_UUID,
                },
            ]))
        }

        async fn create(
            &self,
            _entity: &super::CustomExtraHoursEntity,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<(), DaoError> {
            Ok(())
        }

        async fn update(
            &self,
            _entity: super::CustomExtraHoursEntity,
            _process: &str,
            _tx: Self::Transaction,
        ) -> Result<(), DaoError> {
            Ok(())
        }
    }

    #[tokio::test]
    pub async fn test_find_all() {
        let dao = CustomExtraHoursDaoTestImpl;
        let tx = crate::MockTransaction;

        let result = dao.find_all(tx.clone()).await;
        assert!(result.is_ok());
        let mut entities = result.unwrap().iter().cloned().collect::<Vec<_>>();
        entities.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].id, ENTITY1_UUID);
        assert_eq!(entities[1].id, ENTITY2_UUID);
    }

    #[tokio::test]
    pub async fn test_find_by_id() {
        let dao = CustomExtraHoursDaoTestImpl;
        let tx = crate::MockTransaction;

        let result = dao.find_by_id(ENTITY1_UUID, tx.clone()).await;
        assert!(result.is_ok());
        let entity = result.unwrap();
        assert!(entity.is_some());
        assert_eq!(entity.unwrap().id, ENTITY1_UUID);
    }

    #[tokio::test]
    pub async fn test_find_by_sales_person_id() {
        let dao = CustomExtraHoursDaoTestImpl;
        let tx = crate::MockTransaction;

        let result = dao
            .find_by_sales_person_id(SALES_PERSON_1_UUID, tx.clone())
            .await;
        assert!(result.is_ok());
        let mut entities = result.unwrap().iter().cloned().collect::<Vec<_>>();
        entities.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(entities.len(), 2);
        assert_eq!(entities[0].id, ENTITY1_UUID);
        assert_eq!(entities[1].id, ENTITY2_UUID);

        let result = dao
            .find_by_sales_person_id(SALES_PERSON_2_UUID, tx.clone())
            .await;
        assert!(result.is_ok());
        let mut entities = result.unwrap().iter().cloned().collect::<Vec<_>>();
        entities.sort_by(|a, b| a.name.cmp(&b.name));
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].id, ENTITY1_UUID);
    }
}
