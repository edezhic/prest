use prest::*;
use todo_pwa::{shared_routes, into_page};

embed_build_output_as!(BuiltAssets);

#[derive(Table, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
struct Todo {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub task: String,
    pub done: bool,
}

fn main() {
    Todo::migrate();
    shared_routes()
    .route(
        "/todos",
        get(todos)
            .put(|Form(todo): Form<Todo>| async move { todo.save().unwrap().render() })
            .patch(|Form(mut todo): Form<Todo>| async move {
                todo.update_done(!todo.done).unwrap().render()
            })
            .delete(|Form(todo): Form<Todo>| async move {
                todo.remove().unwrap();
            }),
    )
    .wrap_non_htmx(into_page)
    .embed(BuiltAssets)
    .run();
}

async fn todos() -> Markup {
    html!(
        form hx-put="/todos" hx-target="div" hx-swap="beforeend" hx-on--after-request="this.reset()" {
            input."input input-bordered input-primary" type="text" name="task" {}
            button."btn btn-outline btn-primary ml-4" type="submit" {"Add"}
        }
        ."w-full" {@for todo in Todo::find_all() {(todo)}}
    )
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            ."flex items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/todos" checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto" hx-delete="/todos" {"Delete"}
            }
        }
    }
}


