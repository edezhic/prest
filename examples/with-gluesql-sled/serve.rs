use gluesql::{
    core::ast_builder::{table, Build as BuildSQL},
    prelude::{Glue, Payload, Value},
    sled_storage::SledStorage,
};
use prest::*;

static DB: Lazy<SledStorage> = Lazy::new(|| SledStorage::new("sled_db").unwrap());

// temporary workaround until Glue futures implement Send https://github.com/gluesql/gluesql/issues/1245
fn exec_inside_async(stmt: impl BuildSQL) -> Result<Payload> {
    use tokio::{runtime::Handle, task::block_in_place};
    Ok(block_in_place(|| Handle::current().block_on(exec(stmt)))?)
}

fn exec_sync(stmt: impl BuildSQL) -> Result<Payload> {
    use futures::executor::block_on;
    Ok(block_on(exec(stmt))?)
}

async fn exec(stmt: impl BuildSQL) -> Result<Payload> {
    Ok(Glue::new(DB.clone()).execute_stmt(&stmt.build()?).await?)
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

impl Todo {
    pub fn from_row(row: Vec<Value>) -> Self {
        let Value::Uuid(uuid) = row[0].clone() else {
            panic!("missing uuid");
        };
        let uuid = uuid::Uuid::from_u128(uuid).to_string();
        let Value::Str(task) = row[1].clone() else {
            panic!("missing task");
        };
        let Value::Bool(done) = row[2].clone() else {
            panic!("missing done");
        };
        Todo { uuid, task, done }
    }
}
async fn get_todos() -> Vec<Todo> {
    let Ok(Payload::Select { rows, .. }) = exec_inside_async(table(TODOS).select()) else {
        panic!("failed select query");
    };
    rows.into_iter().map(Todo::from_row).collect::<Vec<Todo>>()
}

#[tokio::main]
async fn main() {
    let _migration = exec_sync(
        table(TODOS)
            .create_table_if_not_exists()
            .add_column("uuid UUID PRIMARY KEY")
            .add_column("task TEXT NOT NULL")
            .add_column("done BOOLEAN NOT NULL"),
    );
    let service = Router::new()
        .route(
            "/",
            get(html!(@for todo in get_todos().await {(todo)}))
                .put(|Form(Todo { task, .. }): Form<Todo>| async move {
                    let values = format!("GENERATE_UUID(), '{task}', false");
                    exec_inside_async(table(TODOS).insert().values(vec![values])).unwrap();
                    Redirect::to("/")
                })
                .patch(|Form(Todo { uuid, done, .. }): Form<Todo>| async move {
                    exec_inside_async(
                        table(TODOS)
                            .update()
                            .set("done", !done)
                            .filter(format!("uuid = '{uuid}'")),
                    )
                    .unwrap();
                    Redirect::to("/")
                })
                .delete(|Form(Todo { uuid, .. }): Form<Todo>| async move {
                    exec_inside_async(table(TODOS).delete().filter(format!("uuid = '{uuid}'")))
                        .unwrap();
                    Redirect::to("/")
                }),
        )
        .layer(HTMXify::wrap(page));

    serve(service, Default::default()).await
}

impl Render for Todo {
    fn render(&self) -> Markup {
        let id = format!("uuid-{}", &self.uuid);
        let cb = format!("on change from .{id} trigger submit on #{id}");
        html!(
            div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
                form #(id) hx-patch="/"  style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input type="hidden" name="done" value={(self.done)} {}
                    label {
                        @match self.done {
                            false => { input .(id) type="checkbox" _={(cb)} {} },
                            true  => { input .(id) type="checkbox" _={(cb)} checked {} },
                        }
                        {(self.task)}
                    }
                }
                form hx-delete="/" style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input."secondary outline" type="submit" value="Delete" style="margin-bottom: 0px;" {}
                }
            }
        )
    }
}

pub fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("Todo"))
        body."container" hx-target="article" style="margin-top: 16px;" {
            form hx-put="/" _="on htmx:afterRequest reset() me" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            article {(content)}
        }
    }}
}
