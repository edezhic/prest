use prest::*;
use turbosql::{execute, select, Turbosql};

#[derive(Default, Turbosql, serde::Serialize, serde::Deserialize)]
struct Todo {
    pub rowid: Option<i64>,
    pub task: Option<String>,
    pub done: Option<bool>,
}

fn main() {
    route(
        "/",
        get(|| async { html!(@for todo in select!(Vec<Todo>).unwrap() {(todo)}) })
            .put(|Form(mut todo): Form<Todo>| async move {
                todo.done = Some(false);
                todo.insert().unwrap();
                html!(@for todo in select!(Vec<Todo>).unwrap() {(todo)})
            })
            .patch(|Form(mut todo): Form<Todo>| async move {
                todo.done = Some(!todo.done.unwrap());
                todo.update().unwrap();
                todo.render()
            })
            .delete(|Form(Todo { rowid, .. }): Form<Todo>| async move {
                execute!("DELETE FROM todo WHERE rowid = " rowid.unwrap()).unwrap();
            }),
    )
    .wrap_non_htmx(page)
    .run()
}

impl Render for Todo {
    fn render(&self) -> Markup {
        let rowid = self.rowid.clone().unwrap();
        let task = self.task.clone().unwrap();
        let done = self.done.clone().unwrap();
        html! {
            ."flex  items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!({ "rowid": rowid, "task": task, "done": done})) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/" checked[done] {}
                label."ml-4 text-lg" {(task)}
                button."btn btn-ghost ml-auto" hx-delete="/" {"Delete"}
            }
        }
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With Turbosql SQLite"))
        body."max-w-screen-sm mx-auto mt-12" {
            form."flex gap-4 justify-center" hx-put="/" hx-target="div" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary" type="submit" {"Add"}
            }
            ."w-full" {(content)}
            (Scripts::default())
        }
    }}
}
