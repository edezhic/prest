use prest::*;

fn main() {
    init!(tables Todo);
    route(
        "/",
        get(|| async { html!(@for todo in Todo::find_all() {(todo)}) })
            .put(|Form(todo): Form<Todo>| async move { todo.save().unwrap().render() })
            .patch(|Form(mut todo): Form<Todo>| async move {
                todo.update_done(!todo.done).unwrap().render()
            })
            .delete(|Form(todo): Form<Todo>| async move {
                todo.remove().unwrap();
            }),
    )
    .route("/connect", get(|| async move {}))
    .wrap_non_htmx(into_page)
    .run();
}

#[derive(Debug, Table, Default, serde::Serialize, serde::Deserialize)]
struct Todo {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            ."flex items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/" checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto" hx-delete="/" {"Delete"}
            }
        }
    }
}

async fn into_page(content: Markup) -> Markup {
    html! {(DOCTYPE) html data-theme="dark" {(Head::with_title("Todo app"))
        body."max-w-screen-sm mx-auto mt-12 flex flex-col items-center" {
            form hx-put="/" hx-target="div" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary ml-4" type="submit" {"Add"}
            }
            ."w-full" {(content)}
            (Scripts::default())
        }
    }}
}
