# Project description Shifty Backend

**Purpose and Goals:**  
The application manages employee shift plans, working hours, and leave data to support HR and scheduling needs within a company. Key features include:

- Organizing and presenting employee shift plans.
- Tracking working hours, overtime, vacations, holidays, and sick leave.
- Calculating each employee’s balance hours for a specific period.
- Persisting and reusing carryover hours from previous years to avoid recalculations and improve efficiency.
- Providing REST endpoints so that external tools (e.g., frontend applications or reporting dashboards) can access employee reports and balance hours information.

**High-Level Architecture and Technology Stack:**

- **Architecture:** A layered system with separate REST, service, and DAO layers, ensuring a clean separation of concerns and easy maintainability.
- **Technology:** Implemented in Rust using the Axum framework for REST endpoints, SQLx for database access to an SQLite database, and Nix/NixOS for reproducible builds and deployments.
- **Modularity:** Traits define service interfaces in `service`, with concrete implementations in `service_impl`. Data access is abstracted by DAO traits in `dao` and their SQLx-based implementations in `dao_impl`. The `app` crate wires everything together into a runnable binary, and `rest-types` provides shared DTO definitions.

**Core Domain Concepts:**

- **Shift Plans:** Define which employees should work at what times.
- **Working Hours and Expectations:** Every employee has a defined expected number of hours to work, stored and calculated based on their contract details.
- **Extra Hours (Overtime, Vacation, Holidays, Sick Leave):** Additional data ensures accurate reporting of total hours worked or not worked due to absences.
- **Balance Hours Reporting:** Combines shift plans, extra hours, and expected hours to show an employee’s surplus or deficit of hours. Previous year carryover hours are now also included to provide a cumulative context.

**Carryover Mechanism:**  
At the end of a year, the computed balance hours for each employee are stored as carryover into the database. This carryover serves as a starting point for the next year’s calculations, improving performance and reducing the need to recompute past data.

**Error Handling and Permissions:**

- Services return `Result<_, ServiceError>` and handle data access errors gracefully, allowing easy integration with the REST layer.
- Authentication and authorization are enforced by passing a `context: Authentication<Self::Context>` into service methods, ensuring only authorized personnel can access or modify data.

**Transaction Management:**

- Every service method which operates with the DAO layer must have an `Option<Transaction>` object as last parameter. 
- The transaction object is a trait which comes from the DAO layer.
- The services uses `TransactionDao::use_transaction`. It receives the `Option<Transaction>` and  returns a `Transaction`.  It creates a new transaction when it doesn't already exist.
- After all DAO calls, the service must call `Transaction::commit` which consumes the transaction.  `Transaction::commit` will commit the transaction only if it is the last instance of the transaction.  `Transaction` contains an ARC and it will only actually commit the transaction if it is the last call to ARC.

**Testing and Quality Assurance:**

- Unit tests use `#[automock(type Context=();)]` to isolate services from database dependencies.
- Integration tests run against an in-memory SQLite instance to verify end-to-end functionality.
- New features include corresponding tests to maintain high quality and confidence in changes.

**REST layer code structure**:

The REST layer uses `rest-types` for transport objects and `rest` for the actual endpoints.  
`rest` defines routes for each domain area, grouping related endpoints together.  
Endpoints rely on a `RestStateDef` trait to provide references to services.  
Handlers convert transport objects (TOs) into domain entities, call service methods, and serialize results back to TOs for responses.  
A shared `error_handler` maps `ServiceError` instances into proper HTTP responses.  
This clean separation and consistent pattern make it easy to extend the REST API with new endpoints or functionalities.

**Documentation and Maintenance:**

- Comments and trait definitions provide inline documentation.
- Nix/NixOS ensures reproducible builds and controlled deployments.
- The system is designed to be extensible, allowing the addition of new services, data sources, or integrations with minimal changes to existing code. Yes, providing an example of how a service implementation and DAO implementation typically look would be helpful. It doesn’t need to be overly long, but a concise example helps clarify the coding conventions, patterns, and expectations. This ensures anyone working with the code can quickly understand the standard approach to implementing services and DAOs in this project.

**Implementation Patterns**

**Service Implementation Example:**  
Services are defined as traits in the `service` crate and implemented in the `service_impl` crate. We use the `gen_service_impl!` macro to reduce boilerplate. Here is a simplified example:

```rust
// In service/src/some_service.rs
#[automock(type Context=();)]
#[async_trait::async_trait]
pub trait SomeService {
    type Context: Clone + Debug + PartialEq + Eq + Send + Sync + 'static;
	type Transaction: dao::Transaction;

    async fn do_something(
		&self, param: Uuid,
		context: Authentication<Self::Context>,
		tx: Option<Self::Transaction>,
	) -> Result<(), ServiceError>;
}

// In service_impl/src/some_service.rs
use crate::gen_service_impl;

gen_service_impl! {
    struct SomeServiceImpl: service::some_service::SomeService = SomeServiceDeps {
        SomeDao: dao::some_dao::SomeDao = some_dao,
        PermissionService: service::permission::PermissionService<Context = Self::Context, Transaction = Self::Transaction> = permission_service,
        UuidService: service::uuid_service::UuidService = uuid_service,
		TransactionDao: dao::TransactionDao<Transaction = Self::Transaction> = transaction_dao
    }
}

#[async_trait::async_trait]
impl<Deps: SomeServiceDeps> SomeService for SomeServiceImpl<Deps> {
    type Context = Deps::Context;
	type Transaction = Deps::Transaction;

    async fn do_something(
		&self, param: Uuid,
		context: Authentication<Self::Context>,
		tx: Option<Self::Transaction>,
	) -> Result<(), ServiceError> {
	   // Begin transaction if not available
	   let tx = self.transaction_dao.use_transaction(tx).await?;
	   
        // Check permissions
        self.permission_service.check_permission("some_privilege", context).await?;

        // Perform DAO operations
        if let Some(entity) = self.some_dao.find_by_id(param).await? {
            // Business logic here
        }

        // Commit transaction.
		self.transaction_dao.commit(tx).await?;
		
        Ok(())
    }
}
```

**DAO Implementation Example:**  
DAO traits are defined in the `dao` crate and implemented in the `dao_impl` crate. The implementation uses SQLx queries, maps database rows into entity structs, and handles errors uniformly. For example:

```rust
// In dao/src/some_dao.rs
#[automock]
#[async_trait::async_trait]
pub trait SomeDao {
    type Transaction: crate::Transaction;
	
    async fn find_by_id(&self, id: Uuid, tx: Self::Transaction) -> Result<Option<SomeEntity>, DaoError>;
}

// In dao_impl/src/some_dao.rs
use std::sync::Arc;
use dao::{some_dao::SomeDao, some_dao::SomeEntity, DaoError};
use sqlx::query_as;
use time::PrimitiveDateTime;
use uuid::Uuid;

#[derive(Debug)]
struct SomeDb {
    id: Vec<u8>,
    created: String,
}

impl TryFrom<&SomeDb> for SomeEntity {
    type Error = DaoError;

    fn try_from(db: &SomeDb) -> Result<Self, Self::Error> {
        Ok(SomeEntity {
            id: Uuid::from_slice(&db.id)?,
            created: PrimitiveDateTime::parse(&db.created, &time::format_description::well_known::Iso8601::DATE_TIME)?,
        })
    }
}

pub struct SomeDaoImpl {
    pub pool: Arc<sqlx::SqlitePool>,
}

impl SomeDaoImpl {
    pub fn new(pool: Arc<sqlx::SqlitePool>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SomeDao for SomeDaoImpl {
    type Transaction = TransactionImpl;

    async fn find_by_id(&self, id: Uuid, tx: Self::Transaction) -> Result<Option<SomeEntity>, DaoError> {
        let id_vec = id.as_bytes().to_vec();
        Ok(query_as!(
            SomeDb,
            "SELECT id, created FROM some_table WHERE id = ? AND deleted IS NULL",
            id_vec,
        )
        .fetch_optional(tx.tx.lock().await.as_mut())
        .await?
        .as_ref()
        .map(SomeEntity::try_from)
        .transpose()?)
    }
}
```

**Example Test Code Snippet**

```rust
// Example test for a hypothetical "FooService" that depends on a "FooDao".

use mockall::predicate::eq;
use service::{foo::FooService, permission::Authentication, ServiceError}; 
use dao::foo::{FooDao, FooEntity, MockFooDao};
use uuid::Uuid;
use time::{PrimitiveDateTime, Date, Month, Time};

// Define dummy dependencies for FooService
pub struct FooServiceDependencies {
    pub foo_dao: MockFooDao,
}

impl service::foo::FooServiceDeps for FooServiceDependencies {
    type Context = ();
    type FooDao = MockFooDao;
}

impl FooServiceDependencies {
    pub fn build_service(self) -> service_impl::foo::FooServiceImpl<FooServiceDependencies> {
        service_impl::foo::FooServiceImpl {
            foo_dao: self.foo_dao.into(),
        }
    }
}

// A helper function to create a default FooEntity.
fn default_foo_entity() -> FooEntity {
    FooEntity {
        id: Uuid::new_v4(),
        name: "Test Foo".into(),
        created: PrimitiveDateTime::new(
            Date::from_calendar_date(2025, Month::January, 1).unwrap(),
            Time::from_hms(12, 0, 0).unwrap(),
        ),
        deleted: None,
        version: Uuid::new_v4(),
    }
}

// Authentication helper since most tests just use `Authentication::Context(())`
trait NoneTypeExt {
    fn auth(&self) -> Authentication<()>;
}
impl NoneTypeExt for () {
    fn auth(&self) -> Authentication<()> {
        Authentication::Context(())
    }
}

#[tokio::test]
async fn test_get_foo_found() {
    let mut foo_dao = MockFooDao::new();
    let foo_entity = default_foo_entity();
    let foo_id = foo_entity.id;

    // Set expectation on the mock DAO to return the foo_entity
    foo_dao
        .expect_find_by_id()
        .with(eq(foo_id))
        .returning(move |_| Ok(Some(foo_entity.clone())));

    // Build the service with our mock DAO
    let deps = FooServiceDependencies { foo_dao };
    let service = deps.build_service();

    // Call the service method
    let result = service.get_foo(foo_id, ().auth()).await;

    // Verify result
    assert!(result.is_ok(), "Expected Ok result");
    let returned_foo = result.unwrap();
    assert_eq!(returned_foo.id, foo_id);
    assert_eq!(returned_foo.name, "Test Foo");
}

#[tokio::test]
async fn test_get_foo_not_found() {
    let mut foo_dao = MockFooDao::new();
    let foo_id = Uuid::new_v4();

    // Set expectation to return None
    foo_dao
        .expect_find_by_id()
        .with(eq(foo_id))
        .returning(|_| Ok(None));

    let deps = FooServiceDependencies { foo_dao };
    let service = deps.build_service();

    let result = service.get_foo(foo_id, ().auth()).await;
    assert!(result.is_ok(), "Expected Ok result even if not found");
    assert!(result.unwrap().is_none(), "Expected None when not found");
}

#[tokio::test]
async fn test_get_foo_dao_error() {
    let mut foo_dao = MockFooDao::new();
    let foo_id = Uuid::new_v4();

    // Set expectation to return an error
    foo_dao
        .expect_find_by_id()
        .with(eq(foo_id))
        .returning(|_| Err(dao::DaoError::DatabaseError("DB issue".into())));

    let deps = FooServiceDependencies { foo_dao };
    let service = deps.build_service();

    let result = service.get_foo(foo_id, ().auth()).await;

    // Verify we correctly translate DAO error to a ServiceError
    match result {
        Err(ServiceError::DataAccess(_)) => { /* expected */ }
        _ => panic!("Expected a data access error"),
    }
}
```

**REST dummy code**:

```rust
// Pseudocode demonstrating the pattern

// Define Transport Objects (TOs) in rest-types
#[derive(Serialize, Deserialize)]
pub struct ExampleEntityTO {
    pub id: Uuid,
    pub name: String,
}

// Define Routes in rest crate
pub fn generate_example_routes<RestState: RestStateDef>() -> Router<RestState> {
    Router::new()
        .route("/", get(get_all_entities::<RestState>))
        .route("/:id", get(get_entity_by_id::<RestState>))
        .route("/", post(create_entity::<RestState>))
}

// Handlers convert TO <-> Domain and call services
pub async fn get_all_entities<RestState: RestStateDef>(
    rest_state: State<RestState>,
    Extension(context): Extension<Context>,
) -> Response {
    error_handler(async {
        let entities = rest_state.example_service().get_all(context.into(), None).await?;
        let to_list: Vec<ExampleEntityTO> = entities.iter().map(Into::into).collect();
        Ok(json_response(&to_list))
    }.await)
}

// Common helper for JSON responses
fn json_response<T: Serialize>(val: &T) -> Response {
    Response::builder()
        .status(200)
        .body(Body::from(serde_json::to_string(val).unwrap()))
        .unwrap()
}
```