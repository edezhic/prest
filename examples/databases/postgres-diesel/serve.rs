pub mod models;
pub mod schema;

use prest::*;

use diesel::prelude::*;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection, RunQueryDsl,
};
use models::Todo;
use schema::todos::dsl::*;

state!(DB_POOL: Pool<AsyncPgConnection> = {
    let database_url = "postgres://postgres:password@localhost/prest";
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder(config).build()?
});

fn main() {
    route(
        "/",
        get(|| async { html!(@for todo in get_todos().await {(todo)}) })
            .patch(toggle_todo)
            .put(add_todo)
            .delete(delete_todo),
    )
    .wrap_non_htmx(page)
    .run()
}

async fn get_todos() -> Vec<Todo> {
    let mut con = DB_POOL.get().await.unwrap();
    todos
        .select(Todo::as_select())
        .load(&mut con)
        .await
        .expect("successful select query")
}

async fn toggle_todo(Vals(todo): Vals<Todo>) -> Markup {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::update(todos.find(todo.uuid))
        .set(done.eq(!todo.done))
        .returning(Todo::as_returning())
        .get_result(&mut con)
        .await
        .expect("successful update query")
        .render()
}

async fn add_todo(Vals(todo): Vals<Todo>) -> Markup {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::insert_into(todos)
        .values(&todo)
        .returning(Todo::as_returning())
        .get_result(&mut con)
        .await
        .expect("successful insert query")
        .render()
}

async fn delete_todo(Vals(todo): Vals<Todo>) {
    let mut con = DB_POOL.get().await.unwrap();
    diesel::delete(todos.filter(uuid.eq(todo.uuid)))
        .execute(&mut con)
        .await
        .expect("successful delete query");
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
    html! { html { (Head::with_title("With Diesel Postgres"))
        body $"max-w-screen-sm mx-auto mt-12" {
            form $"flex gap-4 justify-center" put="/" into="#list" swap-beforeend after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button type="submit" {"Add"}
            }
            div #"list" $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}
