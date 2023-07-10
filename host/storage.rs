use crate::auth::{Role, User, UserId, Email, PwHash};
use anyhow::Result;
use axum_login::UserStore;
pub use gluesql::core::ast_builder::table;
use gluesql::{core::ast_builder::Build, prelude::*};

lazy_static! {
    static ref STORE: SharedMemoryStorage = SharedMemoryStorage::new();
}

static USER_TABLE: &str = "Users";

#[derive(Clone)]
pub struct Storage;
impl Storage {
    pub fn init() -> Result<()> {
        Self::exec_sync(
            table(USER_TABLE)
                .create_table_if_not_exists()
                .add_column("id UINT64 PRIMARY KEY")
                .add_column("email TEXT UNIQUE NOT NULL")
                .add_column("pw_hash TEXT")
                .add_column("role TEXT NOT NULL"),
        )?;
        Ok(())
    }
    pub async fn get_user_by_id(id: UserId) -> Option<User> {
        let Ok(payload) = Self::exec_inside_async(
            table(USER_TABLE)
                .select()
                .filter(format!("id = {id}")),
        ) else { return None };
        let Payload::Select {rows, ..} = payload else { return None };
        if rows.len() == 0 { return None };
        let Value::Str(email) = rows[0][1].clone() else { return None };
        let email = Email::new_unchecked(email);
        let Value::Str(role_str) = rows[0][3].clone() else { return None };
        let pw_hash = match rows[0][2].clone() {
            Value::Str(s) => PwHash(Some(s)),
            Value::Null | _ => PwHash(None),
        };
        Some(User {
            id,
            email,
            pw_hash,
            role: role_str.into(),
        })
    }
    pub async fn get_user_by_email(email: &Email) -> Option<User> {
        let Ok(payload) = Self::exec_inside_async(
            table(USER_TABLE)
                .select()
                .filter(format!("email = '{email}'")),
        ) else { return None };
        let Payload::Select {rows, ..} = payload else { return None };
        if rows.len() == 0 { return None };
        let Value::U64(id) = rows[0][0].clone() else { return None };
        let Value::Str(role_str) = rows[0][3].clone() else { return None };
        let pw_hash = match rows[0][2].clone() {
            Value::Str(s) => PwHash(Some(s)),
            Value::Null | _ => PwHash(None),
        };
        Some(User {
            id: UserId(id),
            email: email.clone(),
            pw_hash,
            role: role_str.into(),
        })
    }

    pub async fn insert_user(user: &User) -> Result<&User> {
        let hash = match &user.pw_hash.0 {
            Some(hash) => hash.as_str(),
            None => "NULL",
        };
        let values = format!(
            "{}, '{}', '{}', '{}'",
            user.id, user.email, hash, Into::<String>::into(user.role)
        );
        Self::exec_inside_async(table(USER_TABLE).insert().values(vec![values]))?;
        Ok(user)
    }

    // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1245
    pub fn exec_inside_async(stmt: impl Build) -> Result<Payload> {
        Ok(tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::exec(stmt))
        })?)
    }

    pub fn exec_sync(stmt: impl Build) -> Result<Payload> {
        Ok(futures::executor::block_on(Self::exec(stmt))?)
    }

    async fn exec(stmt: impl Build) -> Result<Payload> {
        Ok(Glue::new(STORE.clone())
            .execute_stmt(&stmt.build()?)
            .await?)
    }
}

#[async_trait::async_trait]
impl UserStore<UserId, Role> for Storage {
    type User = User;
    type Error = std::convert::Infallible;
    async fn load_user(&self, user_id: &UserId) -> Result<Option<User>, Self::Error> {
        Ok(Storage::get_user_by_id(*user_id).await)
    }
}
