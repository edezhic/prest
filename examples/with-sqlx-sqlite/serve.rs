use prest::*;
use sqlx::{migrate, query, query_as, Pool, Sqlite};

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

#[derive(serde::Deserialize, sqlx::FromRow)]
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
    start_printing_traces();

    migrate!().run(&*DB).await.unwrap();

    let service = Router::new().route(
        "/",
        get(read_todos)
            .patch(toggle_todo)
            .put(create_todo)
            .delete(delete_todo),
    );
    serve(service, Default::default()).await.unwrap();
}

async fn create_todo(Query(Todo { task, .. }): Query<Todo>) -> Markup {
    let uuid = "uuid".to_owned();
    // Insert the task, then obtain the ID of this row
    let id = query("INSERT INTO todos ( uuid, task ) VALUES ( ?, ? )")
        .bind(uuid)
        .bind(task)
        .execute(&*DB)
        .await
        .unwrap()
        .last_insert_rowid();
    println!("{:?}", id);
    read_todos().await
}

async fn toggle_todo(Query(Todo { uuid, done, .. }): Query<Todo>) -> Markup {
    let done = match done {
        true => "TRUE",
        false => "FALSE",
    };
    let rows_affected = query_as("UPDATE todos SET done = TRUE WHERE uuid = ?")
        .bind(done.to_owned())
        .bind(uuid.clone())
        .execute(pool)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected = 0 {
        println!("something went wrong with completing {uuid:?}")
    }

    read_todos().await
}

async fn delete_todo(Query(Todo { uuid, .. }): Query<Todo>) -> Markup {
    read_todos().await
}

async fn read_todos() -> Markup {
    let mut stream = query_as::<Sqlite, Todo>("SELECT * FROM todos").fetch(&*DB);
    println!("{:?}", stream.size_hint());
    todos_page(vec![])
}

fn todos_page(todos: Vec<Todo>) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("With SQLx and SQLite"))
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
