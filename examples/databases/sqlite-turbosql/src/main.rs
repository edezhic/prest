use prest::*;
use turbosql::{execute, select, Turbosql};

#[derive(Default, Turbosql, Serialize, Deserialize)]
struct Todo {
    pub rowid: Option<i64>,
    pub task: Option<String>,
    pub done: Option<bool>,
}

fn main() {
    route(
        "/",
        get(|| async { select!(Vec<Todo>).unwrap().render() })
            .put(|Vals(mut todo): Vals<Todo>| async move {
                todo.done = Some(false);
                todo.insert().unwrap();
                select!(Vec<Todo>).unwrap().render()
            })
            .patch(|Vals(mut todo): Vals<Todo>| async move {
                todo.done = Some(!todo.done.unwrap());
                todo.update().unwrap();
                todo.render()
            })
            .delete(|Vals(Todo { rowid, .. }): Vals<Todo>| async move {
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
            $"flex items-center" swap-this vals=(json!({ "rowid": rowid, "task": task, "done": done})) {
                input type="checkbox" patch="/" checked[done] {}
                label $"ml-4 text-lg" {(task)}
                button $"ml-auto" detele="/" {"Delete"}
            }
        }
    }
}

async fn page(content: Markup) -> Markup {
    html! { html { (Head::with_title("With Turbosql SQLite"))
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
