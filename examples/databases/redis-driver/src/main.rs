use prest::*;
use redis::{Client, Commands};
use std::collections::HashMap;

state!(CLIENT: Client = { Client::open("redis://127.0.0.1")? });

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(serde::Deserialize)]
pub struct TodoForm {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

fn main() {
    route(
        "/",
        get(|| async {
            let todos = get_todos();
            html!(@for todo in todos {(render_item(todo.0, todo.1))})
        })
        .put(|Form(TodoForm { task, .. }): Form<TodoForm>| async move {
            add_todo(task);
            Redirect::to("/")
        })
        .patch(
            |Form(TodoForm { uuid, done, .. }): Form<TodoForm>| async move {
                toggle_todo(uuid, done);
                Redirect::to("/")
            },
        )
        .delete(|Form(TodoForm { uuid, .. }): Form<TodoForm>| async move {
            delete_todo(uuid);
            Redirect::to("/")
        }),
    )
    .wrap_non_htmx(page)
    .run()
}

fn get_todos() -> Vec<(String, Todo)> {
    let mut con = CLIENT.get_connection().unwrap();
    let map: HashMap<String, String> = con.hgetall("todos").unwrap();
    map.into_iter()
        .map(|(uuid, todo)| {
            let todo = serde_json::from_str::<Todo>(&todo).unwrap();
            (uuid, todo)
        })
        .collect()
}

fn add_todo(task: String) {
    let mut con = CLIENT.get_connection().unwrap();
    let uuid = uuid::Uuid::new_v4().to_string();
    con.hset_nx(
        "todos",
        uuid,
        serde_json::to_string(&Todo { task, done: false }).unwrap(),
    )
    .unwrap()
}

fn toggle_todo(uuid: String, done: bool) {
    let mut con = CLIENT.get_connection().unwrap();
    let todo: String = con.hget("todos", &uuid).unwrap();
    let mut todo: Todo = serde_json::from_str(&todo).unwrap();
    todo.done = !done;
    con.hset("todos", uuid, serde_json::to_string(&todo).unwrap())
        .unwrap()
}

fn delete_todo(uuid: String) {
    let mut con = CLIENT.get_connection().unwrap();
    con.hdel("todos", uuid).unwrap()
}

fn render_item(uuid: String, todo: Todo) -> Markup {
    let id = format!("uuid-{}", uuid);
    html!(
        div style="height: 64px; display: flex; justify-content: space-between; align-items: center;" {
            form #(id) hx-patch="/"  style="margin-bottom: 0px;" {
                input type="hidden" name="uuid" value={(uuid)} {}
                input type="hidden" name="done" value={(todo.done)} {}
                label {
                    input .(id) type="checkbox" onchange="this.form.submit()" checked[todo.done] {}
                    {(todo.task)}
                }
            }
            form hx-delete="/" style="margin-bottom: 0px;" {
                input type="hidden" name="uuid" value={(uuid)} {}
                input."secondary outline" type="submit" value="Delete" style="margin-bottom: 0px;" {}
            }
        }
    )
}

async fn page(content: Markup) -> Markup {
    html! { html data-theme="dark" {
        (Head::with_title("With Redis"))
        body."container" hx-target="div" style="margin-top: 16px;" {
            form hx-put="/" hx-on--after-request="this.reset()" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            ."w-full" {(content)}
            (Scripts::default())
        }
    }}
}
