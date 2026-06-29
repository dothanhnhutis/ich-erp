use crate::{
    entities::{
        session::{NewSession, Session},
        user::User,
    },
    errors::DomainError,
};

pub trait UserRepository: Send + Sync {
    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;
}

pub trait UserSessionRepository: Send + Sync {
    fn create(
        &self,
        new_session: NewSession,
    ) -> impl Future<Output = Result<Session, DomainError>> + Send;
}
