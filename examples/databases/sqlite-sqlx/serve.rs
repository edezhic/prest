use prest::*;
use sqlx::{migrate, query, query_as, FromRow, Sqlite, SqlitePool};

state!(DB: SqlitePool = async {
    let conn = SqlitePool::connect("sqlite::memory:").await?;
    migrate!().run(&conn).await?;
    conn
});

#[derive(Debug, FromRow, Serialize, Deserialize)]
struct Todo {
    #[serde(default = "new_uuid")]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn new_uuid() -> String {
    Uuid::now_v7().to_string()
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
    todos.render()
}

async fn add_todo(Vals(todo): Vals<Todo>) -> Markup {
    let q = "insert into todos (uuid, task) values (?, ?) returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(todo.uuid)
        .bind(todo.task)
        .fetch_one(&*DB)
        .await
        .unwrap()
        .render()
}

async fn toggle_todo(Vals(todo): Vals<Todo>) -> Markup {
    let q = "update todos set done = ? where uuid = ? returning *";
    query_as::<Sqlite, Todo>(q)
        .bind(!todo.done)
        .bind(todo.uuid)
        .fetch_one(&*DB)
        .await
        .unwrap()
        .render()
}
async fn delete_todo(Vals(todo): Vals<Todo>) {
    let q = "delete from todos where uuid = ?";
    query(q).bind(todo.uuid).execute(&*DB).await.unwrap();
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            $"flex items-center" swap-this vals=(json!(self)) {
                input type="checkbox" patch="/" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" detele="/" {"Delete"}
            }
        }
    }
}

async fn page(content: Markup) -> Markup {
    html! { html { (Head::with_title("With SQLx SQLite"))
        body $"max-w-screen-sm mx-auto mt-12" {
            form $"flex gap-4 justify-center" put="/" into-end-of="#list" after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button type="submit" {"Add"}
            }
            div #list $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}
