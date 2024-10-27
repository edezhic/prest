use prest::*;
use todo_pwa::{into_page, shared_routes};

embed_build_output_as!(BuiltAssets);

#[derive(Table, Default, Serialize, Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub task: String,
    pub done: bool,
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            $"flex justify-between items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input type="checkbox" hx-patch="/todos" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" hx-delete="/todos" {"Delete"}
            }
        }
    }
}

fn main() {
    init!(tables Todo);
    shared_routes()
        .route(
            "/todos",
            get(|| async {
                html!(
                    form hx-put="/todos" hx-target="#list" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                        input $"border rounded-md" type="text" name="task" {}
                        button $"ml-4" type="submit" {"Add"}
                    }
                    div id="list" $"w-full" {@for todo in Todo::find_all() {(todo)}}
                )
            })
                .put(|Form(todo): Form<Todo>| async move { ok(todo.save()?.render()) })
                .patch(|Form(mut todo): Form<Todo>| async move {
                    ok(todo.update_done(!todo.done)?.render())
                })
                .delete(|Form(todo): Form<Todo>| async move {
                    todo.remove()?;
                    OK
                }),
        )
        .wrap_non_htmx(into_page)
        .embed(BuiltAssets)
        .run();
}
