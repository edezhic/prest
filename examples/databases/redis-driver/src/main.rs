use prest::*;
use redis::{Client, Commands};
use std::collections::HashMap;

state!(CLIENT: Client = { Client::open("redis://127.0.0.1")? });

#[derive(Serialize, Deserialize)]
pub struct Todo {
    #[serde(default)]
    pub task: String,
    #[serde(default)]
    pub done: bool,
}

#[derive(Deserialize)]
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
        .put(|Vals(TodoForm { task, .. }): Vals<TodoForm>| async move {
            add_todo(task);
            Redirect::to("/")
        })
        .patch(
            |Vals(TodoForm { uuid, done, .. }): Vals<TodoForm>| async move {
                toggle_todo(uuid, done);
                Redirect::to("/")
            },
        )
        .delete(|Vals(TodoForm { uuid, .. }): Vals<TodoForm>| async move {
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
            let todo = from_json_str::<Todo>(&todo).unwrap();
            (uuid, todo)
        })
        .collect()
}

fn add_todo(task: String) {
    let mut con = CLIENT.get_connection().unwrap();
    let uuid = Uuid::now_v7().to_string();
    con.hset_nx(
        "todos",
        uuid,
        to_json_string(&Todo { task, done: false }).unwrap(),
    )
    .unwrap()
}

fn toggle_todo(uuid: String, done: bool) {
    let mut con = CLIENT.get_connection().unwrap();
    let todo: String = con.hget("todos", &uuid).unwrap();
    let mut todo: Todo = from_json_str(&todo).unwrap();
    todo.done = !done;
    con.hset("todos", uuid, to_json_string(&todo).unwrap())
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
            form #(id) patch="/"  style="margin-bottom: 0px;" {
                input type="hidden" name="uuid" value={(uuid)} {}
                input type="hidden" name="done" value={(todo.done)} {}
                label {
                    input .(id) type="checkbox" onchange="this.form.submit()" checked[todo.done] {}
                    {(todo.task)}
                }
            }
            form detele="/" style="margin-bottom: 0px;" {
                input type="hidden" name="uuid" value={(uuid)} {}
                input type="submit" value="Delete" style="margin-bottom: 0px;" {}
            }
        }
    )
}

async fn page(content: Markup) -> Markup {
    html! { html { (Head::with_title("With Redis"))
        body $"container" target="div" style="margin-top: 16px;" {
            form put="/" after-request="this.reset()" {
                label for="task" {"Task description:"}
                input type="text" name="task" {}
                button type="submit" {"Add"}
            }
            $"w-full" {(content)}
            (Scripts::default())
        }
    }}
}
