use prest::*;

mod storage;
use storage::Todos;

#[tokio::main]
async fn main() {
    Todos::migrate().unwrap();
    let service = Router::new()
        .route("/", get(|| async { ([(header::CONTENT_TYPE, "text/html")], todospage().await.0) }))
        .route("/todo/add", get(add_todo))
        .route("/todo/delete", get(delete_todo));
    prest::host::serve(service, Default::default()).await.unwrap();
}

async fn todospage() -> Markup {
    let todos = Todos::get_todos().await;
    maud::html!(
        html {
            head {
                title {"With GlueSQL"}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
            }
            body {
                h1{"With gluesql"}
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
