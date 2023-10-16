use prest::*;
use sqlx::{FromRow, migrate, query, query_as, Pool, Sqlite};
use serde::Deserialize;
use uuid::Uuid;

fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

#[derive(FromRow, Deserialize)]
struct Todo {
    #[serde(default = "new_uuid")]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
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

#[tokio::main]
async fn main() {
    //start_printing_traces();
    migrate!().run(&*DB).await.unwrap();
    let service = Router::new()
        .route(
            "/",
            get(todos)
                .patch(|Form(todo): Form<Todo>| async {
                    query("UPDATE todos SET done = ? WHERE uuid = ?")
                        .bind(!todo.done)
                        .bind(todo.uuid)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    todos().await
                })
                .put(|Form(todo): Form<Todo>| async {
                    query("INSERT INTO todos ( uuid, task ) VALUES ( ?, ? )")
                        .bind(todo.uuid)
                        .bind(todo.task)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    todos().await
                })
                .delete(|Form(todo): Form<Todo>| async {
                    query("DELETE FROM todos WHERE uuid = ?")
                        .bind(todo.uuid)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    todos().await
                }),
        )
        .layer(Htmxify::wrap(page));
    serve(service, Default::default()).await.unwrap();
}

async fn todos() -> Markup {
    let todos = query_as::<Sqlite, Todo>("SELECT * FROM todos")
        .fetch_all(&*DB)
        .await
        .unwrap();
    html!(@for todo in todos {(todo)})
}

pub fn page(content: Markup) -> Markup {
    html!{ html data-theme="dark" {
        (Head::default().title("Todo"))
        body."container" hx-target="article" {
            form hx-put="/" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                input type="submit" value="Create" {}
            }
        }
        article {(content)}
    }}
}
