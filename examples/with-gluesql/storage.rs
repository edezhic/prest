use std::sync::LazyLock;

use prest::*;
pub use gluesql::core::ast_builder::table;
use gluesql::{core::ast_builder::{Build as BuildSQL}, sled_storage::SledStorage, prelude::{Payload, Value, Glue}};

static STORE: LazyLock<SledStorage> = LazyLock::new(|| { gluesql::sled_storage::SledStorage::new("sled_db").unwrap() });

pub struct Todo {
    pub uuid: String,
    pub task: String,
}

pub struct Todos;
impl Todos {
    const TABLE_NAME: &str = "Todos";

    pub fn migrate() -> Result<()> {
        Self::exec_sync(
            table(Self::TABLE_NAME)
                .create_table_if_not_exists()
                .add_column("uuid UUID PRIMARY KEY")
                .add_column("task TEXT UNIQUE NOT NULL"),
        )?;
        Ok(())
    }
    pub async fn get_todos() -> Vec<Todo> {
        let Ok(payload) = Self::exec_inside_async(table(Self::TABLE_NAME).select()) 
            else { return vec![] };
        let Payload::Select {rows, ..} = payload else { return vec![] };
        let mut todos = vec![];
        for row in rows {
            let Value::Uuid(uuid) = row[0].clone() else { continue; };
            let uuid = uuid::Uuid::from_u128(uuid).to_string();
            let Value::Str(task) = row[1].clone() else { continue; };
            todos.push(Todo {
                uuid,
                task,
            });
        }
        todos
    }
    pub async fn delete_todo_by_id(uuid: String) -> Result<()> {
        Self::exec_inside_async(
            table(Self::TABLE_NAME)
                .delete()
                .filter(format!("uuid = '{uuid}'")),
        )?;
        Ok(())
    }
    pub async fn add_task(task: String) -> Result<()> {
        let values = format!("GENERATE_UUID(), '{task}'");
        Self::exec_inside_async(table(Self::TABLE_NAME).insert().values(vec![values]))?;
        Ok(())
    }

    // temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1245
    fn exec_inside_async(stmt: impl BuildSQL) -> Result<Payload> {
        Ok(tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(Self::exec(stmt))
        })?)
    }

    fn exec_sync(stmt: impl BuildSQL) -> Result<Payload> {
        Ok(futures::executor::block_on(Self::exec(stmt))?)
    }

    async fn exec(stmt: impl BuildSQL) -> Result<Payload> {
        Ok(Glue::new(STORE.clone())
            .execute_stmt(&stmt.build()?)
            .await?)
    }
}
