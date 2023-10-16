use prest::*;
use serde::Deserialize;
use sqlx::{migrate, query, query_as, FromRow, Sqlite, SqlitePool};

fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

static DB: Lazy<SqlitePool> = Lazy::new(|| {
    SqlitePool::connect_lazy("sqlite::memory:").unwrap()
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

async fn get_todos() -> Vec<Todo> {
    let q = "select * from todos";
    query_as::<Sqlite, Todo>(q)
        .fetch_all(&*DB)
        .await
        .unwrap()
}

async fn add_todo(todo: Todo) -> Todo {
    let q = "insert into todos (uuid, task) values (?, ?) returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(todo.uuid)
        .bind(todo.task)
        .fetch_one(&*DB)
        .await
        .unwrap()
}

async fn toggle_todo(todo: Todo) -> Todo {
    let q = "update todos set done = ? where uuid = ? returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(!todo.done)
        .bind(todo.uuid)
        .fetch_one(&*DB)
        .await
        .unwrap()
}
//async fn delete_todo(todo: Todo) -> Todo {}

#[tokio::main]
async fn main() {
    start_printing_traces();
    migrate!().run(&*DB).await.unwrap();
    let service = Router::new()
        .route(
            "/",
            template!(@for todo in get_todos().await {(todo)})
                .patch(|Form(todo): Form<Todo>| async move {
                    toggle_todo(todo).await.render()
                    /*
                    query("UPDATE todos SET done = ? WHERE uuid = ?")
                        .bind(!todo.done)
                        .bind(todo.uuid)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    html!(@for todo in get_todos().await {(todo)})
                    */
                })
                .put(|Form(todo): Form<Todo>| async {
                    query("INSERT INTO todos (uuid, task) VALUES (?, ?)")
                        .bind(todo.uuid)
                        .bind(todo.task)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    html!(@for todo in get_todos().await {(todo)})
                })
                .delete(|Form(todo): Form<Todo>| async {
                    query("DELETE FROM todos WHERE uuid = ?")
                        .bind(todo.uuid)
                        .execute(&*DB)
                        .await
                        .unwrap();
                    html!(@for todo in get_todos().await {(todo)})
                }),
        )
        .layer(Htmxify::wrap(page));
    serve(service, Default::default()).await.unwrap();
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
        }
        article {(content)}
    }}
}
