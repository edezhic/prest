use prest::*;

#[derive(Table, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Todo {
    #[serde(default = "generate_uuid")]
    pub id: Uuid,
    pub task: String,
    pub done: bool,
}

fn main() {
    Todo::init_table();
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
    .wrap_non_htmx(into_page)
    .run()
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
    html! { html data-theme="dark" { (Head::example("Todo app"))
        body."max-w-screen-sm mx-auto mt-12 flex flex-col items-center" {
            form hx-put="/" hx-target="article" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary ml-4" type="submit" {"Add"}
            }
            article."w-full" {(content)} (Scripts::default())
        }
    }}
}
