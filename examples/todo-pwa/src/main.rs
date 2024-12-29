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
            $"flex justify-between items-center" swap-this vals=(json!(self)) {
                input type="checkbox" patch="/todos" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" delete="/todos" {"Delete"}
            }
        }
    }
}

#[init]
async fn main() -> Result {
    shared_routes()
        .route(
            "/todos",
            get(|| async {
                ok(html!(
                    form put="/todos" into-end-of="#list" after-request="this.reset()" {
                        input $"border rounded-md" type="text" name="task" {}
                        button $"ml-4" type="submit" {"Add"}
                    }
                    div #list $"w-full" {(Todo::select_all().await?)}
                ))
            })
            .put(|todo: Vals<Todo>| async move { ok(todo.save().await?.render()) })
            .delete(|todo: Vals<Todo>| async move { ok(todo.remove().await?) })
            .patch(|Vals(mut todo): Vals<Todo>| async move {
                ok(todo.update_done(!todo.done).await?.render())
            }),
        )
        .wrap_non_htmx(into_page)
        .embed(BuiltAssets)
        .run()
        .await
}
