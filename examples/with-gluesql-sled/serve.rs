use prest::*;
use gluesql::{
    core::ast_builder::{Build as BuildSQL, table}, 
    sled_storage::SledStorage, 
    prelude::{Payload, Value, Glue}
};

static DB: Lazy<SledStorage> = Lazy::new(|| { 
    SledStorage::new("sled_db").unwrap()
});
// temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1245
fn exec_inside_async(stmt: impl BuildSQL) -> Result<Payload> {
    use tokio::{task::block_in_place, runtime::Handle};
    Ok(block_in_place(|| {
        Handle::current().block_on(exec(stmt))
    })?)
}

fn exec_sync(stmt: impl BuildSQL) -> Result<Payload> {
    use futures::executor::block_on;
    Ok(block_on(exec(stmt))?)
}

async fn exec(stmt: impl BuildSQL) -> Result<Payload> {
    Ok(Glue::new(DB.clone())
        .execute_stmt(&stmt.build()?)
        .await?)
}

static TODOS: &str = "Todos";

#[derive(serde::Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

#[tokio::main]
async fn main() {
    let migration = exec_sync(
        table(TODOS)
            .create_table_if_not_exists()
            .add_column("uuid UUID PRIMARY KEY")
            .add_column("task TEXT UNIQUE NOT NULL")
            .add_column("done BOOLEAN NOT NULL"),
    );
    let service = Router::new().route("/", 
        get(read_todos)
            .put(create_todo)
            .delete(delete_todo)
    );
    
    serve(service, Default::default()).await.unwrap();
}

async fn create_todo(Query(Todo { task, .. }): Query<Todo>) -> Markup {
    let values = format!("GENERATE_UUID(), '{task}', false");
    let res = exec_inside_async(
        table(TODOS)
            .insert()
            .values(vec![values])
    );
    if let Err(e) = res {
        println!("{e:?}");
    }
    read_todos().await
}

async fn delete_todo(Query(Todo { uuid, .. }): Query<Todo>) -> Markup {
    let res = exec_inside_async(
        table(TODOS)
            .delete()
            .filter(format!("uuid = '{uuid}'")),
    );
    if let Err(e) = res {
        println!("{e:?}");
    }
    read_todos().await
}

async fn read_todos() -> Markup {
    let Ok(payload) = exec_inside_async(table(TODOS).select()) 
        else { return vec![] };
    let Payload::Select {rows, ..} = payload 
        else { return vec![] };
    // let todos = rows.iter().map(...).collect::<Vec<Todo>>();
    let mut todos = vec![];
    for row in rows {
        let Value::Uuid(uuid) = row[0].clone() else { continue; };
        let uuid = uuid::Uuid::from_u128(uuid).to_string();
        let Value::Str(task) = row[1].clone() else { continue; };
        let Value::Bool(done) = row[2].clone() else { continue; };
        todos.push(Todo {
            uuid,
            task,
            done,
        });
    }
    todos_page(todos)
}

fn todos_page(todos: Vec<Todo>) -> Markup {
    html!{ html data-theme="dark" {
        (Head::default().title("With GlueSQL and Sled"))
        body {
            form hx-put="/" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                input type="submit" value="Create" {}
            }
            ul {
                @for todo in todos {
                    li {
                        input type="checkbox" name="done" value={(todo.done)} {}
                        (todo.task) 
                        form hx-delete="/" {
                            input type="hidden" name="uuid" value={(todo.uuid)} {}
                            input type="submit" value="Remove" {}
                        }
                    }
                }
            }
        }
    }}
}