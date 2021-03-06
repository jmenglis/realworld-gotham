use crate::models::{NewUser, User};
use crate::schema::users;
use crate::Repo;

use diesel::prelude::*;
use diesel::result::Error as dieselError;
use futures::Future;

pub fn insert(repo: Repo, user: NewUser) -> impl Future<Item = User, Error = dieselError> {
    repo.run(move |conn| {
        // TODO: store password not in plain text, later
        diesel::insert_into(users::table)
            .values(&user)
            .get_result(&conn)
    })
}

pub fn find(repo: Repo, user_id: i32) -> impl Future<Item = User, Error = dieselError> {
    use crate::schema::users::dsl::*;
    repo.run(move |conn| users.find(user_id).first(&conn))
}

pub fn find_by_email_password(
    repo: Repo,
    user_email: String,
    user_password: String,
) -> impl Future<Item = User, Error = dieselError> {
    use crate::schema::users::dsl::*;
    repo.run(|conn| {
        users
            .filter(email.eq(user_email))
            .filter(password.eq(user_password))
            .first::<User>(&conn)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repo;
    use crate::test_helpers::generate;
    use tokio_threadpool::ThreadPool;

    #[test]
    fn test_create_user() {
        let pool = ThreadPool::new();
        let repo = repo();

        let new_user = generate::new_user();
        let future =
            insert(repo.clone(), new_user).and_then(move |user| find(repo.clone(), user.id));
        let results = wait_for(&pool, future);
        assert!(results.is_ok());
    }

    #[test]
    fn test_authenticate_user() {
        let pool = ThreadPool::new();
        let repo = repo();
        // Create a new user
        let new_user = generate::new_user();
        let future = insert(repo.clone(), new_user)
            .and_then(move |user| find_by_email_password(repo.clone(), user.email, user.password));

        // Check the user is in the database.
        let results = wait_for(&pool, future);
        assert!(results.is_ok());
    }

    fn wait_for<T>(
        pool: &ThreadPool,
        future: impl Future<Item = T, Error = dieselError> + Send + 'static,
    ) -> Result<T, dieselError>
    where
        T: Send + 'static,
    {
        pool.spawn_handle(future).wait()
    }
}
