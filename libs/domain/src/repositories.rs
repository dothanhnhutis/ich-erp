use crate::{entities::user::User, errors::DomainError};

pub trait UserRepository: Send + Sync {
    fn find_by_email(
        &self,
        email: &str,
    ) -> impl Future<Output = Result<Option<User>, DomainError>> + Send;
}
