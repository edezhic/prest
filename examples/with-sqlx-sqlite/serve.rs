use prest::*;
use sqlx::{migrate, query, query_as, Pool, Sqlite};

static DB: Lazy<Pool<Sqlite>> = Lazy::new(|| {
    sqlx::sqlite::SqlitePoolOptions::new()
        .connect_lazy("sqlite::memory:")
        .expect("successful DB connection")
});

mod todo {
    #[derive(Debug, serde::Deserialize, sqlx::FromRow)]
    pub struct Todo {
        #[serde(default)]
        pub uuid: String,
        #[serde(default)]
        pub task: String,
        #[serde(default)]
        pub done: bool,
    }
    impl Render for Todo {
        fn render(&self) -> Markup {
            let id = uuid::Uuid::parse_str(&self.uuid).unwrap();
            let id = format!("uuid:{id}");
            let done = if self.done {
                format!("set @checked then set @disabled")
            } else {
                format!("on change trigger submit on #{id}")
            };
            html!(
                div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
                    form hx-patch="/" id={(id)} style="margin-bottom: 0px;" {
                        input type="hidden" name="uuid" value={(self.uuid)} {}
                        input type="hidden" name="done" value={(!self.done)} {}
                        label {
                            input type="checkbox" _={(done)} {}
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
        html!( html data-theme="dark" {
            (Head::default().title("Todo"))
            body."container" hx-target="body" {
                form hx-put="/" {
                    label for="task" {"Task description:"}
                    input type="text" name="task" {}
                    input type="submit" value="Create" {}
            }
            article {(content)}
        })
    }
}
use todo::{Todo, page};
    
#[tokio::main]
async fn main() {
    //start_printing_traces();
    migrate!().run(&*DB).await.unwrap();
    let service = Router::new()
        .route(
            "/",
            get(todos)
                .patch(toggle_todo)
                .put(create_todo)
                .delete(delete_todo),
        )
        .layer(Htmxify::wrap(page));
    serve(service, Default::default()).await.unwrap();
}

async fn create_todo(Form(todo): Form<Todo>) -> Markup {
    query("INSERT INTO todos ( uuid, task ) VALUES ( ?, ? )")
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(todo.task)
        .execute(&*DB)
        .await
        .unwrap();
    todos().await
}

async fn toggle_todo(Form(todo): Form<Todo>) -> Markup {
    query("UPDATE todos SET done = ? WHERE uuid = ?")
        .bind(todo.done)
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
