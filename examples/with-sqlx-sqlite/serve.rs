use prest::*;
use serde::Deserialize;
use sqlx::{migrate, query, query_as, FromRow, Sqlite, SqlitePool};

static DB: Lazy<SqlitePool> = Lazy::new(|| {
    let conn = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    block_on(migrate!().run(&*DB)).unwrap();
    conn
});

#[derive(Debug, FromRow, Deserialize)]
struct Todo {
    #[serde(default = "new_uuid")]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn main() {
    Router::new()
        .route(
            "/",
            get(|| async { html!(@for todo in get_todos().await {(todo)}) })
                .patch(|Form(todo): Form<Todo>| async move { toggle_todo(todo).await.render() })
                .put(|Form(todo): Form<Todo>| async move { add_todo(todo).await.render() })
                .delete(|Form(todo): Form<Todo>| async move {
                    delete_todo(todo).await;
                }),
        )
        .wrap_non_htmx(page)
        .serve(ServeOptions::default())
}

fn new_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

async fn get_todos() -> Vec<Todo> {
    let q = "select * from todos";
    query_as::<Sqlite, Todo>(q).fetch_all(&*DB).await.unwrap()
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
async fn delete_todo(todo: Todo) {
    let q = "delete from todos where uuid = ?";
    query(q).bind(todo.uuid).execute(&*DB).await.unwrap();
}

impl Render for Todo {
    fn render(&self) -> Markup {
        let id = format!("uuid-{}", &self.uuid);
        let cb = format!("on change from .{id} trigger submit on #{id}");
        html!(
            div hx-target="this" hx-swap="outerHTML" style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
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

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::example("With SQLx SQLite"))
        body."container" hx-target="article" style="margin-top: 16px;" {
            form hx-put="/" hx-swap="beforeend" _="on htmx:afterRequest reset() me" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            article {(content)}
            (Scripts::default())
        }
    }}
}
