pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use prest::*;
use schema::todos::dsl::*;
use models::Todo;
use std::env;

fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let router = Router::new()
        .route(
            "/",
            get(|| async {html!(@for todo in get_todos() {(todo)})})
                .patch(|Form(todo): Form<Todo>| async move { toggle_todo(todo).render() })
                .put(|Form(todo): Form<Todo>| async move { add_todo(todo).render() })
                .delete(|Form(todo): Form<Todo>| async move { delete_todo(todo); }),
        )
        .layer(HTMXify::wrap(page));
    serve(router, Default::default())
}

fn get_todos() -> Vec<Todo> {
    let con = &mut establish_connection();
    todos
        .select(Todo::as_select())
        .load(con)
        .expect("Error loading todos")
}

fn toggle_todo(todo: Todo) -> Todo {
    let con = &mut establish_connection();
    diesel::update(todos.find(todo.uuid))
        .set(done.eq(!todo.done))
        .returning(Todo::as_returning())
        .get_result(con)
        .unwrap()
}

fn add_todo(todo: Todo) -> Todo {
    let con = &mut establish_connection();
    diesel::insert_into(todos)
        .values(&todo)
        .returning(Todo::as_returning())
        .get_result(con)
        .expect("Error adding new todo")
}

fn delete_todo(todo: Todo) {
    let con = &mut establish_connection();
    diesel::delete(todos.filter(uuid.eq(todo.uuid)))
        .execute(con)
        .expect("Error deleting posts");
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

pub fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("Todo"))
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
