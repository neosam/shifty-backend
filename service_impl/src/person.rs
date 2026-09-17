//! Phase 61 (PERS-05): Basic-Tier Person-Service.
//!
//! Konsumiert AUSSCHLIESSLICH DAO + Permission + Clock + Uuid + Transaction —
//! kein Domain-Service als Dependency (Success Criterion 5,
//! CLAUDE.md §Service-Tier-Konventionen, Roadmap-Constraint 5). Waere irgendein
//! anderer Domain-Service eine Dependency, entstuende ueber `PermissionService`
//! ein DI-Zyklus und die Gate-Reihenfolge waere nicht mehr bestimmt. Das
//! Akzeptanzkriterium dieses Plans greppt die Datei genau darauf ab.
//!
//! **Gate-Wahl: `HR_PRIVILEGE` fuer alle vier Methoden.** Die beiden
//! Kandidaten-Lesepfade geben Sales-Person- und Benutzernamen preis
//! (T-61-05, Information Disclosure), und der bestehende Lesepfad fuer die
//! Benutzer-Zuordnungen (`get_all_user_assignments` im Sales-Person-Aggregat)
//! ist heute ebenfalls HR-gated. Die Personenverwaltung in Phase 62/63 darf das
//! spaeter auf `admin` **verschaerfen** — in dieser Phase aber nicht lockern.

use crate::gen_service_impl;
use async_trait::async_trait;
use dao::{
    person::{PersonDao, PersonEntity},
    TransactionDao,
};
use service::{
    clock::ClockService,
    permission::{Authentication, HR_PRIVILEGE},
    person::{Person, PersonService},
    sales_person::SalesPerson,
    uuid_service::UuidService,
    PermissionService, ServiceError,
};
use std::sync::Arc;
use uuid::Uuid;

const PERSON_SERVICE_PROCESS: &str = "person-service";

gen_service_impl! {
    struct PersonServiceImpl: PersonService = PersonServiceDeps {
        PersonDao: PersonDao<Transaction = Self::Transaction> = person_dao,
        PermissionService: PermissionService<Context = Self::Context> = permission_service,
        ClockService: ClockService = clock_service,
        UuidService: UuidService = uuid_service,
        TransactionDao: TransactionDao<Transaction = Self::Transaction> = transaction_dao,
    }
}

#[async_trait]
impl<Deps: PersonServiceDeps> PersonService for PersonServiceImpl<Deps> {
    type Context = Deps::Context;
    type Transaction = Deps::Transaction;

    async fn create(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Person, ServiceError> {
        // HR-Gate an erster Stelle (T-61-05).
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;

        // id/version/created erzeugt der Service, nicht das DAO — dieselbe
        // defensive Form wie `RebookingBatchServiceImpl::create`.
        let entity = PersonEntity {
            id: self
                .uuid_service
                .new_uuid(&format!("{PERSON_SERVICE_PROCESS}::create person id")),
            created: self.clock_service.date_time_now(),
            version: self
                .uuid_service
                .new_uuid(&format!("{PERSON_SERVICE_PROCESS}::create person version")),
        };

        self.person_dao
            .create(&entity, PERSON_SERVICE_PROCESS, tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(Person::from(&entity))
    }

    async fn get(
        &self,
        id: Uuid,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Option<Person>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entity = self.person_dao.find_by_id(id, tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;

        Ok(entity.as_ref().map(Person::from))
    }

    async fn unlinked_sales_persons(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[SalesPerson]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let entities = self
            .person_dao
            .sales_persons_without_user(tx.clone())
            .await?;
        self.transaction_dao.commit(tx).await?;

        Ok(entities.iter().map(SalesPerson::from).collect())
    }

    async fn unlinked_users(
        &self,
        context: Authentication<Self::Context>,
        tx: Option<Self::Transaction>,
    ) -> Result<Arc<[Arc<str>]>, ServiceError> {
        self.permission_service
            .check_permission(HR_PRIVILEGE, context)
            .await?;

        let tx = self.transaction_dao.use_transaction(tx).await?;
        let names = self.person_dao.users_without_sales_person(tx.clone()).await?;
        self.transaction_dao.commit(tx).await?;

        Ok(names)
    }
}
