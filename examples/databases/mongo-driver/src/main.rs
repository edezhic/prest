use mongodb::{
    bson::{doc, Uuid},
    options::ClientOptions,
    Client, Collection,
};
use prest::*;

state!(TODOS: Collection<Todo> = async {
    let opts = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(opts)?;
    let db = client.database("todosdb");
    db.collection::<Todo>("todos")
});

#[derive(Clone, Serialize, Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub uuid: Uuid,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn main() {
    route(
        "/",
        get(|| async {
            let todos: Vec<Todo> = TODOS
                .find(None, None)
                .await
                .unwrap()
                .try_collect()
                .await
                .unwrap();
            html!(@for todo in todos {(todo)})
        })
        .put(|Form(Todo { task, .. }): Form<Todo>| async move {
            let new_todo = Todo {
                uuid: Uuid::new(),
                task,
                done: false,
            };
            TODOS.insert_one(&new_todo, None).await.unwrap();
            new_todo.render()
        })
        .patch(|Form(Todo { uuid, done, .. }): Form<Todo>| async move {
            TODOS
                .update_one(doc! {"uuid": uuid}, doc! {"$set": {"done": !done}}, None)
                .await
                .unwrap();
            TODOS
                .find_one(doc! {"uuid": uuid}, None)
                .await
                .unwrap()
                .unwrap()
                .render()
        })
        .delete(|Form(Todo { uuid, .. }): Form<Todo>| async move {
            TODOS.delete_one(doc! {"uuid": uuid}, None).await.unwrap();
        }),
    )
    .wrap_non_htmx(page)
    .run()
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html!(
            ."flex  items-center" hx-target="this" hx-swap="outerHTML" hx-vals=(json!(self)) {
                input."toggle toggle-primary" type="checkbox" hx-patch="/" checked[self.done] {}
                label."ml-4 text-lg" {(self.task)}
                button."btn btn-ghost ml-auto" hx-delete="/" {"Delete"}
            }
        )
    }
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With Mongo"))
        body."max-w-screen-sm mx-auto mt-12" {
            form."flex gap-4 justify-center" hx-put="/" hx-target="div" hx-swap="beforeend" hx-on--after-request="this.reset()" {
                input."input input-bordered input-primary" type="text" name="task" {}
                button."btn btn-outline btn-primary" type="submit" {"Add"}
            }
            ."w-full" {(content)}
            (Scripts::default())
        }
    }}
}
