use prest::*;
use gluesql::{core::ast_builder::{Build as BuildSQL, table}, sled_storage::SledStorage, prelude::{Payload, Value, Glue}};

static STORE: Lazy<SledStorage> = Lazy::new(|| { gluesql::sled_storage::SledStorage::new("sled_db").unwrap() });

#[derive(serde::Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
}

#[tokio::main]
async fn main() {
    Todos::migrate().unwrap();
    let service = Router::new()
        .route("/", get(read_todos))
        .route("/create", get(create_todo))
        .route("/delete", get(delete_todo));
    serve(service, Default::default()).await.unwrap();
}

async fn read_todos() -> Markup {
    let todos = Todos::read_todos().await;
    html!{ html data-theme="dark" {
        (Head::default().title("With GlueSQL"))
        body {
            form action="/create" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                input type="submit" value="Create" {}
            }
            ul {
                @for todo in todos {
                    li {
                        (todo.task) 
                        form action="/delete" {
                            input type="hidden" name="uuid" value={(todo.uuid)} {}
                            input type="submit" value="Remove" {}
                        }
                    }
                }
            }
        }
    }}
}

async fn create_todo(Query(Todo { task }): Query<Todo>) -> Redirect {
    if let Err(e) = Todos::create_todo(task).await {
        println!("{e:?}");
    }
    Redirect::to("/")
}

async fn delete_todo(Query(Todo { uuid }): Query<Todo>) -> Redirect {
    if let Err(e) = Todos::delete_todo_by_id(uuid).await {
        println!("{e:?}");
    }
    Redirect::to("/")
}

struct Todos;
impl Todos {
    const NAME: &str = "Todos";

    pub fn migrate() -> Result<()> {
        Self::exec_sync(
            table(Self::NAME)
                .create_table_if_not_exists()
                .add_column("uuid UUID PRIMARY KEY")
                .add_column("task TEXT UNIQUE NOT NULL"),
        )?;
        Ok(())
    }
    pub async fn read_todos() -> Vec<Todo> {
        let Ok(payload) = Self::exec_inside_async(table(Self::NAME).select()) 
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
            table(Self::NAME)
                .delete()
                .filter(format!("uuid = '{uuid}'")),
        )?;
        Ok(())
    }
    pub async fn create_todo(task: String) -> Result<()> {
        let values = format!("GENERATE_UUID(), '{task}'");
        Self::exec_inside_async(table(Self::NAME).insert().values(vec![values]))?;
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
