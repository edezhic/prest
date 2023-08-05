#![feature(lazy_cell)]

#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

use pwrs::*;

mod storage;
use storage::Todos;

#[tokio::main]
async fn main() {
    Todos::migrate().unwrap();
    let service = Router::new()
        .route("/", get(|| async { ([(header::CONTENT_TYPE, "text/html")],todospage().await.0) }))
        .route("/todo/add", get(add_todo))
        .route("/todo/delete", get(delete_todo))
        .layer(pwrs::host::embed(Assets));
    pwrs::host::serve(service, 80).await.unwrap();
}
#[derive(serde::Deserialize)]
struct AddTodoQuery {
    task: String
}
async fn add_todo(Query(AddTodoQuery { task }): Query<AddTodoQuery>) -> impl IntoResponse {
    if let Err(e) = Todos::add_task(task).await {
        println!("{e:?}");
    }
    Redirect::to("/")
}
#[derive(serde::Deserialize)]
struct DeleteTodoQuery {
    uuid: String
}
async fn delete_todo(Query(DeleteTodoQuery { uuid }): Query<DeleteTodoQuery>) -> impl IntoResponse {
    if let Err(e) = Todos::delete_todo_by_id(uuid).await {
        println!("{e:?}");
    }
    Redirect::to("/")
}

async fn todospage() -> Markup {
    let todos = Todos::get_todos().await;
    maud::html!(
        html {
            head {
                title {"PWRS with gluesql"}
                link rel="icon" href="/favicon.ico" {}
                link rel="manifest" href="/.webmanifest" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
                //@if cfg!(any(target_arch = "wasm32", feature = "sw")) { script src="/include_sw.js" {} }
                meta name="viewport" content="width=device-width, initial-scale=1.0";
            }
            body {
                h1{"PWRS with gluesql"}
                form action="/todo/add" {
                    label for="task" {"Task description:"}
                    input type="text" name="task" {}
                    input type="submit" value="Add" {}
                }
                ul {
                    @for todo in todos {
                        li {
                            (todo.task) 
                            form action="/todo/delete" {
                                input type="hidden" name="uuid" value={(todo.uuid)} {}
                                input type="submit" value="Remove" {}
                            }
                        }
                    }
                }
            }
        }
    )
}
