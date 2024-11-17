use async_trait::async_trait;
use dao::session::SessionDao;
use service::{
    clock::ClockService,
    session::{Session, SessionService},
    uuid_service::UuidService,
    ServiceError,
};
use time::OffsetDateTime;

use crate::gen_service_impl;

gen_service_impl! {
    struct SessionServiceImpl: service::session::SessionService = SessionServiceDeps {
        SessionDao: dao::session::SessionDao = session_dao,
        UuidService: service::uuid_service::UuidService = uuid_service,
        ClockService: service::clock::ClockService = clock_service
    }
}

#[async_trait]
impl<Deps: SessionServiceDeps> SessionService for SessionServiceImpl<Deps> {
    type Context = Deps::Context;

    async fn new_session_for_user(&self, user_id: &str) -> Result<Session, ServiceError> {
        let now = OffsetDateTime::now_utc();
        let created = now.unix_timestamp();
        let expires = created + 3600 * 24 * 365;

        let session = Session {
            id: self
                .uuid_service
                .new_uuid("session-service::new_session_for_user")
                .to_string()
                .into(),
            user_id: user_id.into(),
            expires,
            created,
        };
        self.session_dao.create(&(&session).into()).await?;
        Ok(session)
    }

    async fn invalidate_user_session(&self, id: &str) -> Result<(), ServiceError> {
        self.session_dao.delete(id).await?;
        Ok(())
    }

    async fn verify_user_session(&self, id: &str) -> Result<Option<Session>, ServiceError> {
        let session = self.session_dao.find_by_id(id).await?;
        Ok(session.map(|s| (&s).into()))
    }
}
