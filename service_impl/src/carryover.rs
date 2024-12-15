use crate::gen_service_impl;
use async_trait::async_trait;
use dao::carryover::CarryoverDao;
use service::{
    carryover::{Carryover, CarryoverService},
    permission::Authentication,
    ServiceError,
};
use uuid::Uuid;

// If you need any particular process name constant, define here:
const CARRYOVER_SERVICE_PROCESS: &str = "carryover-service";

gen_service_impl! {
    struct CarryoverServiceImpl: service::carryover::CarryoverService = CarryoverServiceDeps {
        CarryoverDao: dao::carryover::CarryoverDao = carryover_dao
        // If you later need more dependencies like PermissionService, ClockService, etc.,
        // you can add them here in a similar way:
        // PermissionService: service::permission::PermissionService<Context = Self::Context> = permission_service,
        // ClockService: service::clock::ClockService = clock_service,
        // UuidService: service::uuid_service::UuidService = uuid_service,
    }
}

// Implement the trait methods for CarryoverService:
#[async_trait]
impl<Deps: CarryoverServiceDeps> CarryoverService for CarryoverServiceImpl<Deps> {
    type Context = Deps::Context;

    async fn get_carryover(
        &self,
        sales_person_id: Uuid,
        year: u32,
        _context: Authentication<Self::Context>,
    ) -> Result<Option<Carryover>, ServiceError> {
        let result = self
            .carryover_dao
            .find_by_sales_person_id_and_year(sales_person_id, year)
            .await?;
        Ok(result.map(|e| (&e).into()))
    }

    async fn set_carryover(
        &self,
        carryover: &Carryover,
        _context: Authentication<Self::Context>,
    ) -> Result<(), ServiceError> {
        let entity = carryover.try_into()?;
        self.carryover_dao
            .upsert(&entity, CARRYOVER_SERVICE_PROCESS)
            .await?;
        Ok(())
    }
}
