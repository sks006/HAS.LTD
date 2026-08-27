use crate::repositories::UserRepository;
use crate::repositories::HasherPort;

pub struct AuthService<U, H>
where
    U: UserRepository,
    H: HasherPort,
{
    user_repo: U,
    hasher: H,
}

impl<U, H> AuthService<U, H>
where
    U: UserRepository,
    H: HasherPort,
{
    pub fn new(user_repo: U, hasher: H) -> Self {
        Self { user_repo, hasher }
    }

    pub async fn authenticate(
        &self,
        email: &str,
        password: &str,
    ) -> Result<crate::models::user::User, crate::errors::DomainError> {
        let identity = self.user_repo.get_user_identity_by_email(email).await?
            .ok_or_else(|| crate::errors::DomainError::Unauthorized)?;

        if self.hasher.verify_password(password, &identity.password_hash)? {
            Ok(identity.user)
        } else {
            Err(crate::errors::DomainError::Unauthorized)
        }
    }
}
