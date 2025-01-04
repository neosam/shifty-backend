use std::sync::Arc;

use async_trait::async_trait;
use dao::{
    sales_person::{SalesPersonDao, SalesPersonEntity},
    TransactionDao,
};
use service::{
    clock::ClockService,
    permission::{Authentication, HR_PRIVILEGE, SALES_PRIVILEGE, SHIFTPLANNER_PRIVILEGE},
    sales_person::{SalesPerson, SalesPersonService},
    uuid_service::UuidService,
    PermissionService, ServiceError, ValidationFailureItem,
};
use tokio::join;
use uuid::Uuid;

use crate::gen_service_impl;

gen_service_impl! {
    struct SalesPersonServiceImpl: SalesPersonService = SalesPersonServiceDeps {
        SalesPersonDao: SalesPersonDao<Transaction = Self::Transaction> = sales_person_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

const SALES_PERSON_SERVICE_PROCESS: &str = "sales-person-service";

#[async_trait]
impl<Deps: SalesPersonServiceDeps> SalesPersonService for SalesPersonServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn get_all(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[service::sales_person::SalesPerson]>, service::ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner, sales, hr) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales).or(hr)?;
        let mut sales_persons = self
            .sales_person_dao
            .all(tx.clone())
            .await?
            .iter()
            .map(SalesPerson::from)
            .collect::<Box<[SalesPerson]>>();

        // Remove sensitive information if user is not a sales user.
        if self
            .permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await
            .is_err()
        {
            sales_persons.iter_mut().for_each(|sales_person| {
                sales_person.is_paid = None;
            });
        }
        self.transaction_dao.commit(tx).await?;
        Ok(sales_persons.into())
    }

    async fn get_all_paid(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let ret = Ok(self
            .sales_person_dao
            .all_paid(tx.clone())
            .await?
            .iter()
            .map(SalesPerson::from)
            .collect());
        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<service::sales_person::SalesPerson, service::ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (shiftplanner, sales, hr) = join!(
            self.permission_service
                .check_permission(SHIFTPLANNER_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(SALES_PRIVILEGE, context.clone()),
            self.permission_service
                .check_permission(HR_PRIVILEGE, context.clone())
        );
        shiftplanner.or(sales).or(hr)?;
        let mut sales_person = self
            .sales_person_dao
            .find_by_id(id, tx.clone())
            .await?
            .as_ref()
            .map(SalesPerson::from)
            .ok_or(ServiceError::EntityNotFound(id))?;

        let remove_sensitive_data = if self
            .permission_service
            .check_permission(HR_PRIVILEGE, context.clone())
            .await
            .is_err()
        {
            if let (Some(current_user_id), Some(assigned_user)) = (
                self.permission_service
                    .current_user_id(context.clone())
                    .await?,
                self.get_assigned_user(id, Authentication::Full, tx.clone().into())
                    .await?,
            ) {
                current_user_id != assigned_user
            } else {
                true
            }
        } else {
            false
        };

        if remove_sensitive_data {
            sales_person.is_paid = None;
        }

        self.transaction_dao.commit(tx).await?;
        Ok(sales_person)
    }

    async fn exists(
        &self,
        id: Uuid,
        _context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<bool, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let ret = Ok(self
            .sales_person_dao
            .find_by_id(id, tx.clone())
            .await
            .map(|x| x.is_some())?);

        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn create(
        &self,
        sales_person: &SalesPerson,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<SalesPerson, service::ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        if sales_person.id != Uuid::nil() {
            return Err(ServiceError::IdSetOnCreate);
        }
        if sales_person.version != Uuid::nil() {
            return Err(ServiceError::VersionSetOnCreate);
        }

        let sales_person = SalesPerson {
            id: self.uuid_service.new_uuid("sales-person-id"),
            version: self.uuid_service.new_uuid("sales-person-version"),
            ..sales_person.clone()
        };
        self.sales_person_dao
            .create(
                &SalesPersonEntity::from(&sales_person),
                SALES_PERSON_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;

        self.transaction_dao.commit(tx).await?;
        Ok(sales_person)
    }

    async fn update(
        &self,
        sales_person: &SalesPerson,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<SalesPerson, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let sales_person_entity = self
            .sales_person_dao
            .find_by_id(sales_person.id, tx.clone())
            .await?
            .as_ref()
            .map(SalesPerson::from)
            .ok_or_else(move || ServiceError::EntityNotFound(sales_person.id))?;

        if sales_person.version != sales_person_entity.version {
            return Err(ServiceError::EntityConflicts(
                sales_person.id,
                sales_person_entity.version,
                sales_person.version,
            ));
        }

        if sales_person.deleted != sales_person_entity.deleted {
            return Err(ServiceError::ValidationError(
                [ValidationFailureItem::ModificationNotAllowed(
                    "deleted".into(),
                )]
                .into(),
            ));
        }

        let sales_person = SalesPerson {
            version: self.uuid_service.new_uuid("sales-person-version"),
            ..sales_person.clone()
        };

        self.sales_person_dao
            .update(
                &SalesPersonEntity::from(&sales_person),
                SALES_PERSON_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(sales_person)
    }

    async fn delete(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let mut sales_person_entity = self
            .sales_person_dao
            .find_by_id(id, tx.clone())
            .await?
            .ok_or(ServiceError::EntityNotFound(id))?;
        sales_person_entity.deleted = Some(self.clock_service.date_time_now());
        sales_person_entity.version = self.uuid_service.new_uuid("sales-person-version");
        self.sales_person_dao
            .update(
                &sales_person_entity,
                SALES_PERSON_SERVICE_PROCESS,
                tx.clone(),
            )
            .await?;
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn get_assigned_user(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Arc<str>>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let ret = Ok(self
            .sales_person_dao
            .get_assigned_user(sales_person_id, tx.clone())
            .await?);
        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn set_user(
        &self,
        sales_person_id: Uuid,
        user_id: Option<Arc<str>>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        self.sales_person_dao
            .discard_assigned_user(sales_person_id, tx.clone())
            .await?;
        if let Some(user) = user_id {
            self.sales_person_dao
                .assign_to_user(
                    sales_person_id,
                    user.as_ref(),
                    SALES_PERSON_SERVICE_PROCESS,
                    tx.clone(),
                )
                .await?;
        }
        self.transaction_dao.commit(tx).await?;
        Ok(())
    }

    async fn get_sales_person_for_user(
        &self,
        user_id: Arc<str>,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<SalesPerson>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;
        let ret = Ok(self
            .sales_person_dao
            .find_sales_person_by_user_id(&user_id, tx.clone())
            .await?
            .as_ref()
            .map(SalesPerson::from));
        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn get_sales_person_current_user(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<SalesPerson>, ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let current_user = if let Some(current_user) = self
            .permission_service
            .current_user_id(context.clone())
            .await?
        {
            current_user
        } else {
            return Ok(None);
        };
        let ret = Ok(self
            .get_sales_person_for_user(current_user, Authentication::Full, tx.clone().into())
            .await?);
        self.transaction_dao.commit(tx).await?;
        ret
    }

    async fn verify_user_is_sales_person(
        &self,
        sales_person_id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<(), ServiceError> {
        let tx = self.transaction_dao.use_transaction(tx).await?;
        let (Some(username), Some(sales_person_username)) = (
            self.permission_service.current_user_id(context).await?,
            self.get_assigned_user(sales_person_id, Authentication::Full, tx.clone().into())
                .await?,
        ) else {
            return Err(ServiceError::Forbidden);
        };
        self.transaction_dao.commit(tx).await?;
        if username == sales_person_username {
            Ok(())
        } else {
            Err(ServiceError::Forbidden)
        }
    }
}
