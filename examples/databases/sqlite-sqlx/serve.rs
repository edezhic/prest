use prest::*;
use sqlx::{migrate, query, query_as, FromRow, Sqlite, SqlitePool};

state!(DB: SqlitePool = async {
    let conn = SqlitePool::connect("sqlite::memory:").await?;
    migrate!().run(&conn).await?;
    conn
});

#[derive(Debug, FromRow, serde::Serialize, serde::Deserialize)]
struct Todo {
    #[serde(default = "new_uuid")]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn new_uuid() -> String {
    Uuid::new_v4().to_string()
}

fn main() {
    route(
        "/",
        get(get_todos)
            .patch(toggle_todo)
            .put(add_todo)
            .delete(delete_todo),
    )
    .wrap_non_htmx(page)
    .run()
}

async fn get_todos() -> Markup {
    let q = "select * from todos";
    let todos = query_as::<Sqlite, Todo>(q).fetch_all(&*DB).await.unwrap();
    html!(@for todo in todos {(todo)})
}

async fn add_todo(Form(todo): Form<Todo>) -> Markup {
    let q = "insert into todos (uuid, task) values (?, ?) returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(todo.uuid)
        .bind(todo.task)
        .fetch_one(&*DB)
        .await
        .unwrap()
        .render()
}

async fn toggle_todo(Form(todo): Form<Todo>) -> Markup {
    let q = "update todos set done = ? where uuid = ? returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(!todo.done)
        .bind(todo.uuid)
        .fetch_one(&*DB)
        .await
        .unwrap()
        .render()
}
async fn delete_todo(Form(todo): Form<Todo>) {
    let q = "delete from todos where uuid = ?";
    query(q).bind(todo.uuid).execute(&*DB).await.unwrap();
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            ."flex  items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/" checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto" hx-delete="/" {"Delete"}
            }
        }
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With SQLx SQLite"))
        body."max-w-screen-sm mx-auto mt-12" {
            form."flex gap-4 justify-center" hx-put="/" hx-target="div" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary" type="submit" {"Add"}
            }
            ."w-full" {(content)}
            (Scripts::default())
        }
    }}
}
