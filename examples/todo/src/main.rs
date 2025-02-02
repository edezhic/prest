use prest::*;

#[derive(Storage, Serialize, Deserialize)]
struct Todo {
    #[serde(default = "Uuid::now_v7")]
    pub id: Uuid,
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html! {
            $"flex justify-between items-center" swap-this vals=(json!(self)) {
                input type="checkbox" patch="/" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" delete="/" {"Delete"}
            }
        }
    }
}

async fn into_page(content: Markup) -> Markup {
    html! {(DOCTYPE) html {(Head::with_title("Todo app"))
        body $"max-w-screen-sm px-8 mx-auto mt-12 flex flex-col items-center" {
            form put="/" into-end-of="#list" after-request="this.reset()" {
                input $"border rounded-md" type="text" name="task" {}
                button $"ml-4" type="submit" {"Add"}
            }
            div #list $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}

#[init]
async fn main() -> Result {
    route(
        "/",
        get(|| async { ok(Todo::get_all().await?.render()) })
            .put(|todo: Vals<Todo>| async move { ok(todo.save().await?.render()) })
            .delete(|todo: Vals<Todo>| async move { ok(todo.remove().await?) })
            .patch(|Vals(mut todo): Vals<Todo>| async move {
                ok(todo.update_done(!todo.done).await?.render())
            }),
    )
    .wrap_non_htmx(into_page)
    .run()
    .await
}
