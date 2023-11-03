use futures::{executor::block_on, stream::TryStreamExt};
use mongodb::{
    bson::{doc, Uuid},
    options::ClientOptions,
    Client, Collection,
};
use prest::*;

static COLLECTION: Lazy<Collection<Todo>> = Lazy::new(|| {
    block_on(async {
        let opts = ClientOptions::parse("mongodb://localhost:27017")
            .await
            .unwrap();
        let client = Client::with_options(opts).unwrap();
        let db = client.database("todosdb");
        db.collection::<Todo>("todos")
    })
});

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub uuid: Uuid,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route(
            "/",
            get(|| async {
                let cursor = COLLECTION.find(None, None).await.unwrap();
                let todos: Vec<Todo> = cursor.try_collect().await.unwrap();
                html!(@for todo in todos {(todo)})
            })
            .put(|Form(Todo { task, .. }): Form<Todo>| async move {
                let new_todo = Todo {
                    uuid: Uuid::new(),
                    task,
                    done: false,
                };
                COLLECTION.insert_one(new_todo, None).await.unwrap();
                Redirect::to("/")
            })
            .patch(|Form(Todo { uuid, done, .. }): Form<Todo>| async move {
                COLLECTION
                    .update_one(doc! {"uuid": uuid}, doc! {"$set": {"done": !done}}, None)
                    .await
                    .unwrap();
                Redirect::to("/")
            })
            .delete(|Form(Todo { uuid, .. }): Form<Todo>| async move {
                COLLECTION
                    .delete_one(doc! {"uuid": uuid}, None)
                    .await
                    .unwrap();
                Redirect::to("/")
            }),
        )
        .layer(HTMXify::wrap(page));
    serve(service, Default::default()).await
}

impl Render for Todo {
    fn render(&self) -> Markup {
        let id = format!("uuid-{}", &self.uuid);
        let cb = format!("on change from .{id} trigger submit on #{id}");
        html!(
            div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
                form #(id) hx-patch="/"  style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input type="hidden" name="done" value={(self.done)} {}
                    label {
                        @match self.done {
                            false => { input .(id) type="checkbox" _={(cb)} {} },
                            true  => { input .(id) type="checkbox" _={(cb)} checked {} },
                        }
                        {(self.task)}
                    }
                }
                form hx-delete="/" style="margin-bottom: 0px;" {
                    input type="hidden" name="uuid" value={(self.uuid)} {}
                    input."secondary outline" type="submit" value="Delete" style="margin-bottom: 0px;" {}
                }
            }
        )
    }
}

pub fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::default().title("Todo"))
        body."container" hx-target="article" style="margin-top: 16px;" {
            form hx-put="/" _="on htmx:afterRequest reset() me" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            article {(content)}
            (Scripts::default())
        }
    }}
}
