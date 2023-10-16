use prest::*;
use sqlx::{migrate, query, query_as, Pool, Sqlite};
use todo::Todo;

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

#[tokio::main]
async fn main() {
    //start_printing_traces();
    migrate!().run(&*DB).await.unwrap();
    let service = Router::new()
        .route(
            "/",
            get(todos)
                .patch(toggle_todo)
                .put(new_todo)
                .delete(delete_todo),
        )
        .layer(Htmxify::wrap(page));
    serve(service, Default::default()).await.unwrap();
}

async fn new_todo(Form(todo): Form<Todo>) -> Markup {
    query("INSERT INTO todos ( uuid, task ) VALUES ( ?, ? )")
        .bind(Todo::new_uuid())
        .bind(todo.task)
        .execute(&*DB)
        .await
        .unwrap();
    todos().await
}

async fn toggle_todo(Form(todo): Form<Todo>) -> Markup {
    query("UPDATE todos SET done = ? WHERE uuid = ?")
        .bind(!todo.done)
        .bind(todo.uuid.clone())
        .execute(&*DB)
        .await
        .unwrap();
    todos().await
}

async fn delete_todo(Form(todo): Form<Todo>) -> Markup {
    query("DELETE FROM todos WHERE uuid = ?")
        .bind(todo.uuid.clone())
        .execute(&*DB)
        .await
        .unwrap();
    todos().await
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

mod todo {
    use prest::*;
    use sqlx::FromRow;
    use serde::Deserialize;
    use uuid::Uuid;
    
    #[derive(FromRow, Deserialize)]
    pub struct Todo {
        #[serde(default)]
        pub uuid: String,
        #[serde(default)]
        pub task: String,
        #[serde(default)]
        pub done: bool,
    }
    impl Todo {
        pub fn new_uuid() -> String {
            uuid::Uuid::new_v4().to_string()
        }
    }
    impl Render for Todo {
        fn render(&self) -> Markup {
            let id = Uuid::parse_str(&self.uuid).unwrap();
            let id = format!("uuid-{id}");
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
}
