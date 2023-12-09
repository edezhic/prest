pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use dotenvy::dotenv;
use models::Todo;
use prest::*;
use schema::todos::dsl::*;
use std::env;

static DB_POOL: Lazy<Pool<AsyncPgConnection>> = Lazy::new(|| {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config).build().unwrap()
});

fn main() {
    dotenv().ok();
    Router::new()
        .route(
            "/",
            get(|| async { html!(@for todo in get_todos().await {(todo)}) })
                .patch(toggle_todo)
                .put(add_todo)
                .delete(delete_todo),
        )
        .wrap_non_htmx(page)
        .serve(ServeOptions::default())
}

async fn get_todos() -> Vec<Todo> {
    let mut con = DB_POOL.get().await.unwrap();
    todos
        .select(Todo::as_select())
        .load(&mut con)
        .await
        .expect("successful select query")
}

async fn toggle_todo(Form(todo): Form<Todo>) -> Markup {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::update(todos.find(todo.uuid))
        .set(done.eq(!todo.done))
        .returning(Todo::as_returning())
        .get_result(&mut con)
        .await
        .expect("successful update query")
        .render()
}

async fn add_todo(Form(todo): Form<Todo>) -> Markup {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::insert_into(todos)
        .values(&todo)
        .returning(Todo::as_returning())
        .get_result(&mut con)
        .await
        .expect("successful insert query")
        .render()
}

async fn delete_todo(Form(todo): Form<Todo>) {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::delete(todos.filter(uuid.eq(todo.uuid)))
        .execute(&mut con)
        .await
        .expect("successful delete query");
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
        (Head::example("With Diesel Postgres"))
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
