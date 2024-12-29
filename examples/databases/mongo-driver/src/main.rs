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

#[init]
async fn main() -> Result {
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
            todos.render()
        })
        .put(|Vals(Todo { task, .. }): Vals<Todo>| async move {
            let new_todo = Todo {
                uuid: Uuid::new(),
                task,
                done: false,
            };
            TODOS.insert_one(&new_todo, None).await.unwrap();
            new_todo.render()
        })
        .patch(|Vals(Todo { uuid, done, .. }): Vals<Todo>| async move {
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
        .delete(|Vals(Todo { uuid, .. }): Vals<Todo>| async move {
            TODOS.delete_one(doc! {"uuid": uuid}, None).await.unwrap();
        }),
    )
    .wrap_non_htmx(page)
    .run()
    .await
}

impl Render for Todo {
    fn render(&self) -> Markup {
        html!(
            $"flex items-center" swap-this vals=(json!(self)) {
                input type="checkbox" patch="/" checked[self.done] {}
                label $"ml-4 text-lg" {(self.task)}
                button $"ml-auto" detele="/" {"Delete"}
            }
        )
    }
}

async fn page(content: Markup) -> Markup {
    html! { html { (Head::with_title("With Mongo"))
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
