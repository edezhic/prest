use prest::*;
use sqlx::{migrate, query, query_as, Pool, Sqlite};

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

#[derive(Debug, serde::Deserialize, sqlx::FromRow)]
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

    let db = Lazy::force(&DB);
    
    migrate!().run(db).await.unwrap();

    let service = Router::new().route(
        "/",
        get(read_todos)
            .patch(toggle_todo)
            .put(create_todo)
            .delete(delete_todo),
    );
    
    serve(service, Default::default()).await.unwrap();
}

async fn create_todo(Form(Todo { task, .. }): Form<Todo>) -> Markup {
    use uuid::Uuid;

    let uuid = Uuid::new_v4().to_string(); 
    query("INSERT INTO todos ( uuid, task ) VALUES ( ?, ? )")
        .bind(uuid.clone())
        .bind(task)
        .execute(&*DB)
        .await
        .unwrap();
    
    read_todos().await
}

async fn toggle_todo(Form(Todo { uuid, done, .. }): Form<Todo>) -> Markup {
    let done = match done {
        true => "TRUE",
        false => "FALSE",
    };
    let rows_affected = query("UPDATE todos SET done = ? WHERE uuid = ?")
        .bind(done.to_owned())
        .bind(uuid.clone())
        .execute(&*DB)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        println!("something went wrong with toggle for {uuid:?}")
    }

    read_todos().await
}

async fn delete_todo(Form(Todo { uuid, .. }): Form<Todo>) -> Markup {
    let rows_affected = query("DELETE todos WHERE uuid = ?")
        .bind(uuid.clone())
        .execute(&*DB)
        .await
        .unwrap()
        .rows_affected();

    if rows_affected == 0 {
        println!("something went wrong with deleting {uuid:?}")
    }

    read_todos().await
}

async fn read_todos() -> Markup {
    let todos = query_as::<Sqlite, Todo>("SELECT * FROM todos")
            .fetch_all(&*DB)
            .await
            .unwrap();
    todos_page(todos)
}

fn todos_page(todos: Vec<Todo>) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("With SQLx and SQLite"))
        body."container" hx-target="body" {
            form hx-put="/" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                input type="submit" value="Create" {}
            }
            article {
                @for todo in todos {
                    div style="display: flex; justify-content: space-between; align-items: center;" {
                        label for="done" {
                            input type="checkbox" id="done" name="done" {}
                            {(todo.task)}
                        }
                        input type="hidden" name="uuid" value={(todo.uuid)} {}
                        a hx-delete="/" role="button" class="secondary outline" {"Delete"}
                    }
                }
            }
        }
    }}
}
