pub struct AuthService<UR>
where
    UR: UserRepository,
{
    user_repo: UR,
}

impl<UR> AuthService<UR>
where
    UR: UserRepository,
{
    pub fn new(user_repo: UR) -> Self {
        Self { user_repo }
    }
}
